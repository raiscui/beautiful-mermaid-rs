// ============================================================================
// Native pathfinder (Rust) — CLI 性能加速专用
//
// 背景：
// - `beautiful-mermaid-rs` 在 CLI 场景下使用 QuickJS（无 JIT）执行 TS bundle。
// - 对于 Flowchart/State 的 ASCII/Unicode 渲染，瓶颈主要在 A* 路由：
//   - heap pop + 4 邻居扩展的热循环在解释器里非常慢（秒级甚至十几秒）。
//
// 目标：
// - 把 A* 的热循环挪到 Rust（release 编译优化）执行，
//   并通过 rquickjs 注入全局函数供 JS 侧调用：
//   - `globalThis.__bm_getPath(...)`
//   - `globalThis.__bm_getPathStrict(...)`
//   - `globalThis.__bm_getPathRelaxed(...)`
//
// 设计原则：
// - 输出必须与原 JS 实现一致（Rust 仓库已有 golden tests 覆盖）
// - 避免 unsafe：TypedArray 通过 `AsRef<[T]>` 只读访问即可
// - 复用大数组：用 stamp 技巧避免每次 search 清空整张 cost 表
// ============================================================================

/// heap（最小堆）节点：保存 idx/priority/cost 三个字段。
///
/// 说明：
/// - 这里刻意不用 `BinaryHeap<Reverse<_>>`：
///   - 需要包装类型与比较，开销更高
///   - 不利于完全复刻 JS 的“严格 < 比较” tie-break 行为
#[derive(Default)]
struct MinHeap {
    idxs: Vec<u32>,
    priorities: Vec<u32>,
    costs: Vec<u32>,
}

impl MinHeap {
    fn clear(&mut self) {
        self.idxs.clear();
        self.priorities.clear();
        self.costs.clear();
    }

    fn push(&mut self, idx: u32, priority: u32, cost: u32) {
        self.idxs.push(idx);
        self.priorities.push(priority);
        self.costs.push(cost);
        self.bubble_up(self.idxs.len() - 1);
    }

    fn pop(&mut self) -> Option<(u32, u32)> {
        if self.idxs.is_empty() {
            return None;
        }

        let out_idx = self.idxs[0];
        let out_cost = self.costs[0];

        let last_idx = self.idxs.pop().unwrap();
        let last_priority = self.priorities.pop().unwrap();
        let last_cost = self.costs.pop().unwrap();

        if !self.idxs.is_empty() {
            self.idxs[0] = last_idx;
            self.priorities[0] = last_priority;
            self.costs[0] = last_cost;
            self.sink_down(0);
        }

        Some((out_idx, out_cost))
    }

    fn bubble_up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = (i - 1) >> 1;
            // 只在严格更小（<）时交换，保持 tie-break 行为与 JS 一致
            if self.priorities[i] < self.priorities[parent] {
                self.idxs.swap(i, parent);
                self.priorities.swap(i, parent);
                self.costs.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    fn sink_down(&mut self, mut i: usize) {
        let n = self.idxs.len();
        loop {
            let mut smallest = i;
            let left = 2 * i + 1;
            let right = 2 * i + 2;

            if left < n && self.priorities[left] < self.priorities[smallest] {
                smallest = left;
            }
            if right < n && self.priorities[right] < self.priorities[smallest] {
                smallest = right;
            }

            if smallest != i {
                self.idxs.swap(i, smallest);
                self.priorities.swap(i, smallest);
                self.costs.swap(i, smallest);
                i = smallest;
            } else {
                break;
            }
        }
    }
}

// `UsedPointSet` 里使用的方向 bit（与 TS 侧保持一致）
const CONNECT_LEFT: u8 = 1 << 0;
const CONNECT_RIGHT: u8 = 1 << 1;
const CONNECT_UP: u8 = 1 << 2;
const CONNECT_DOWN: u8 = 1 << 3;
const H_MASK: u8 = CONNECT_LEFT | CONNECT_RIGHT;
const V_MASK: u8 = CONNECT_UP | CONNECT_DOWN;
/// 4-bit bitcount 查表(0..15)：
/// - usedPoints 的方向 mask 只使用 4 个 bit
/// - 热循环里避免 popcount/循环,用查表更快也更稳定
const BITCOUNT_4: [u8; 16] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4];
const RELAXED_PENALTY_CROSSING: u32 = 1;
// 点重叠规则(relaxed hard rule,与 TS 保持一致):
// - relaxed 允许 crossing(交错),并对“会形成 `┼` 的潜在交叉点”加轻量惩罚；
// - 但对 point overlap(走进已占用点)采取 hard forbid:
//   - 除“起点第一步”与“终点前一步”的受控豁免外,禁止走进任何已占用点位；
//   - 这样能避免在字符画里合成 `┬/┴/├/┤` 这类强歧义 junction,也能显著缩小 A* 搜索空间。

/// Rust 侧复用的 A* 缓存（对应 TS 的 AStarContext，但不持有 blocked/usage 输入）。
#[derive(Default)]
pub struct NativeAStar {
    stamp: u32,
    cost_stamp: Vec<u32>,
    cost_so_far: Vec<u32>,
    came_from: Vec<i32>,
    heap: MinHeap,
}

impl NativeAStar {
    pub fn new() -> Self {
        Self::default()
    }

    /// 保证内部缓冲区容量足够容纳 `stride * height` 的网格。
    fn ensure_capacity(&mut self, stride: usize, height: usize) {
        let needed = stride.saturating_mul(height);
        if needed == 0 {
            self.heap.clear();
            return;
        }

        if self.cost_stamp.len() < needed {
            self.cost_stamp.resize(needed, 0);
            self.cost_so_far.resize(needed, 0);
            self.came_from.resize(needed, -1);
        }
    }

    /// stamp 递增；溢出后把整张 stamp 表清零（保持正确性）。
    fn next_stamp(&mut self) -> u32 {
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            // 0 作为“未使用”保留；溢出后统一清零避免碰撞
            self.cost_stamp.fill(0);
            self.stamp = 1;
        }
        self.stamp
    }

    /// 非 strict：只考虑 blocked + bounds。
    pub fn get_path(
        &mut self,
        stride: usize,
        height: usize,
        from_idx: u32,
        to_idx: u32,
        max_x: u32,
        max_y: u32,
        blocked: &[u8],
    ) -> Result<Option<Vec<u32>>, String> {
        self.ensure_capacity(stride, height);
        let cell_count = stride.saturating_mul(height);

        if cell_count == 0 {
            return Ok(None);
        }
        if blocked.len() < cell_count {
            return Err(format!(
                "blocked 长度不足: blocked.len={} < cell_count={cell_count}",
                blocked.len()
            ));
        }

        let from = from_idx as usize;
        let to = to_idx as usize;
        if from >= cell_count || to >= cell_count {
            return Ok(None);
        }

        if max_x as usize >= stride || max_y as usize >= height {
            return Err(format!(
                "bounds 越界: max_x={max_x}, max_y={max_y}, stride={stride}, height={height}"
            ));
        }

        let stamp = self.next_stamp();
        self.heap.clear();

        self.cost_stamp[from] = stamp;
        self.cost_so_far[from] = 0;
        self.came_from[from] = -1;
        self.heap.push(from_idx, 0, 0);

        let to_y = to / stride;
        let to_x = to - to_y * stride;

        while let Some((current_idx_u32, current_cost_at_push)) = self.heap.pop() {
            let current = current_idx_u32 as usize;

            // 旧的堆项（被更优路径覆盖）直接跳过，避免重复扩展
            if self.cost_stamp[current] != stamp {
                continue;
            }
            if current_cost_at_push != self.cost_so_far[current] {
                continue;
            }

            if current == to {
                return Ok(Some(self.reconstruct_path(current_idx_u32)));
            }

            let current_cost = self.cost_so_far[current];
            let current_y = current / stride;
            let current_x = current - current_y * stride;

            // 右
            if current_x < max_x as usize {
                let next = current + 1;
                if blocked[next] == 0 || next == to {
                    let new_cost = current_cost + 1;
                    if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                        self.cost_stamp[next] = stamp;
                        self.cost_so_far[next] = new_cost;
                        self.came_from[next] = current_idx_u32 as i32;

                        let next_x = current_x + 1;
                        let abs_x = if next_x >= to_x {
                            next_x - to_x
                        } else {
                            to_x - next_x
                        };
                        let abs_y = if current_y >= to_y {
                            current_y - to_y
                        } else {
                            to_y - current_y
                        };
                        let h = abs_x as u32
                            + abs_y as u32
                            + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                        self.heap.push(next as u32, new_cost + h, new_cost);
                    }
                }
            }

            // 左
            if current_x > 0 {
                let next = current - 1;
                if blocked[next] == 0 || next == to {
                    let new_cost = current_cost + 1;
                    if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                        self.cost_stamp[next] = stamp;
                        self.cost_so_far[next] = new_cost;
                        self.came_from[next] = current_idx_u32 as i32;

                        let next_x = current_x - 1;
                        let abs_x = if next_x >= to_x {
                            next_x - to_x
                        } else {
                            to_x - next_x
                        };
                        let abs_y = if current_y >= to_y {
                            current_y - to_y
                        } else {
                            to_y - current_y
                        };
                        let h = abs_x as u32
                            + abs_y as u32
                            + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                        self.heap.push(next as u32, new_cost + h, new_cost);
                    }
                }
            }

            // 下
            if current_y < max_y as usize {
                let next = current + stride;
                if blocked[next] == 0 || next == to {
                    let new_cost = current_cost + 1;
                    if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                        self.cost_stamp[next] = stamp;
                        self.cost_so_far[next] = new_cost;
                        self.came_from[next] = current_idx_u32 as i32;

                        let next_y = current_y + 1;
                        let abs_x = if current_x >= to_x {
                            current_x - to_x
                        } else {
                            to_x - current_x
                        };
                        let abs_y = if next_y >= to_y {
                            next_y - to_y
                        } else {
                            to_y - next_y
                        };
                        let h = abs_x as u32
                            + abs_y as u32
                            + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                        self.heap.push(next as u32, new_cost + h, new_cost);
                    }
                }
            }

            // 上
            if current_y > 0 {
                let next = current - stride;
                if blocked[next] == 0 || next == to {
                    let new_cost = current_cost + 1;
                    if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                        self.cost_stamp[next] = stamp;
                        self.cost_so_far[next] = new_cost;
                        self.came_from[next] = current_idx_u32 as i32;

                        let next_y = current_y - 1;
                        let abs_x = if current_x >= to_x {
                            current_x - to_x
                        } else {
                            to_x - current_x
                        };
                        let abs_y = if next_y >= to_y {
                            next_y - to_y
                        } else {
                            to_y - next_y
                        };
                        let h = abs_x as u32
                            + abs_y as u32
                            + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                        self.heap.push(next as u32, new_cost + h, new_cost);
                    }
                }
            }
        }

        Ok(None)
    }

    /// strict：额外考虑 UsedPointSet（禁止 `┼` 四向交叉）+ SegmentUsage（共线共享规则）。
    #[allow(clippy::too_many_arguments)]
    pub fn get_path_strict(
        &mut self,
        stride: usize,
        height: usize,
        from_idx: u32,
        to_idx: u32,
        max_x: u32,
        max_y: u32,
        blocked: &[u8],
        segment_used: &[u8],
        used_as_middle: &[u8],
        start_source: &[u32],
        start_source_multi: &[u8],
        end_target: &[u32],
        end_target_multi: &[u8],
        used_points: Option<&[u8]>,
        route_from_idx: u32,
        route_to_idx: u32,
        edge_from_id: u32,
        edge_to_id: u32,
    ) -> Result<Option<Vec<u32>>, String> {
        self.ensure_capacity(stride, height);
        let cell_count = stride.saturating_mul(height);

        if cell_count == 0 {
            return Ok(None);
        }
        if blocked.len() < cell_count {
            return Err(format!(
                "blocked 长度不足: blocked.len={} < cell_count={cell_count}",
                blocked.len()
            ));
        }

        let seg_count = cell_count.saturating_mul(2);
        if segment_used.len() < seg_count
            || used_as_middle.len() < seg_count
            || start_source.len() < seg_count
            || start_source_multi.len() < seg_count
            || end_target.len() < seg_count
            || end_target_multi.len() < seg_count
        {
            return Err(format!(
                "segmentUsage 长度不足: seg_count={seg_count}, segment_used={}, used_as_middle={}, start_source={}, start_source_multi={}, end_target={}, end_target_multi={}",
                segment_used.len(),
                used_as_middle.len(),
                start_source.len(),
                start_source_multi.len(),
                end_target.len(),
                end_target_multi.len(),
            ));
        }

        if let Some(points) = used_points {
            if points.len() < cell_count {
                return Err(format!(
                    "usedPoints 长度不足: used_points.len={} < cell_count={cell_count}",
                    points.len()
                ));
            }
        }

        let from = from_idx as usize;
        let to = to_idx as usize;
        if from >= cell_count || to >= cell_count {
            return Ok(None);
        }

        if max_x as usize >= stride || max_y as usize >= height {
            return Err(format!(
                "bounds 越界: max_x={max_x}, max_y={max_y}, stride={stride}, height={height}"
            ));
        }

        let stamp = self.next_stamp();
        self.heap.clear();

        self.cost_stamp[from] = stamp;
        self.cost_so_far[from] = 0;
        self.came_from[from] = -1;
        self.heap.push(from_idx, 0, 0);

        let to_y = to / stride;
        let to_x = to - to_y * stride;

        while let Some((current_idx_u32, current_cost_at_push)) = self.heap.pop() {
            let current = current_idx_u32 as usize;

            if self.cost_stamp[current] != stamp {
                continue;
            }
            if current_cost_at_push != self.cost_so_far[current] {
                continue;
            }

            if current == to {
                return Ok(Some(self.reconstruct_path(current_idx_u32)));
            }

            let current_cost = self.cost_so_far[current];
            let current_y = current / stride;
            let current_x = current - current_y * stride;

            // -----------------------------------------------------------------
            // 4 邻居扩展（顺序与 JS 保持一致：右/左/下/上）
            // -----------------------------------------------------------------

            // 右
            if current_x < max_x as usize {
                let next = current + 1;
                if blocked[next] == 0 || next == to {
                    if is_step_allowed_strict(
                        current,
                        next,
                        /*seg_key=*/ current * 2,
                        CONNECT_RIGHT,
                        CONNECT_LEFT,
                        segment_used,
                        used_as_middle,
                        start_source,
                        start_source_multi,
                        end_target,
                        end_target_multi,
                        used_points,
                        route_from_idx,
                        route_to_idx,
                        edge_from_id,
                        edge_to_id,
                    ) {
                        let new_cost = current_cost + 1;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_x = current_x + 1;
                            let abs_x = if next_x >= to_x {
                                next_x - to_x
                            } else {
                                to_x - next_x
                            };
                            let abs_y = if current_y >= to_y {
                                current_y - to_y
                            } else {
                                to_y - current_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 左
            if current_x > 0 {
                let next = current - 1;
                if blocked[next] == 0 || next == to {
                    if is_step_allowed_strict(
                        current,
                        next,
                        /*seg_key=*/ next * 2,
                        CONNECT_LEFT,
                        CONNECT_RIGHT,
                        segment_used,
                        used_as_middle,
                        start_source,
                        start_source_multi,
                        end_target,
                        end_target_multi,
                        used_points,
                        route_from_idx,
                        route_to_idx,
                        edge_from_id,
                        edge_to_id,
                    ) {
                        let new_cost = current_cost + 1;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_x = current_x - 1;
                            let abs_x = if next_x >= to_x {
                                next_x - to_x
                            } else {
                                to_x - next_x
                            };
                            let abs_y = if current_y >= to_y {
                                current_y - to_y
                            } else {
                                to_y - current_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 下
            if current_y < max_y as usize {
                let next = current + stride;
                if blocked[next] == 0 || next == to {
                    if is_step_allowed_strict(
                        current,
                        next,
                        /*seg_key=*/ current * 2 + 1,
                        CONNECT_DOWN,
                        CONNECT_UP,
                        segment_used,
                        used_as_middle,
                        start_source,
                        start_source_multi,
                        end_target,
                        end_target_multi,
                        used_points,
                        route_from_idx,
                        route_to_idx,
                        edge_from_id,
                        edge_to_id,
                    ) {
                        let new_cost = current_cost + 1;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_y = current_y + 1;
                            let abs_x = if current_x >= to_x {
                                current_x - to_x
                            } else {
                                to_x - current_x
                            };
                            let abs_y = if next_y >= to_y {
                                next_y - to_y
                            } else {
                                to_y - next_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 上
            if current_y > 0 {
                let next = current - stride;
                if blocked[next] == 0 || next == to {
                    if is_step_allowed_strict(
                        current,
                        next,
                        /*seg_key=*/ next * 2 + 1,
                        CONNECT_UP,
                        CONNECT_DOWN,
                        segment_used,
                        used_as_middle,
                        start_source,
                        start_source_multi,
                        end_target,
                        end_target_multi,
                        used_points,
                        route_from_idx,
                        route_to_idx,
                        edge_from_id,
                        edge_to_id,
                    ) {
                        let new_cost = current_cost + 1;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_y = current_y - 1;
                            let abs_x = if current_x >= to_x {
                                current_x - to_x
                            } else {
                                to_x - current_x
                            };
                            let abs_y = if next_y >= to_y {
                                next_y - to_y
                            } else {
                                to_y - next_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// relaxed：允许 crossing（交错），但会对“形成 `┼` 的潜在交叉点”加惩罚；
    /// 同时遵守“禁止共线重叠”的 segment reuse hard rule（仅允许受控的起点/终点段共享）。
    ///
    /// 返回：
    /// - Some((path, cost))：path 包含 fromIdx/toIdx；cost 是 A* 的累计代价（步长 + 惩罚项）
    /// - None：不可达
    #[allow(clippy::too_many_arguments)]
    pub fn get_path_relaxed(
        &mut self,
        stride: usize,
        height: usize,
        from_idx: u32,
        to_idx: u32,
        max_x: u32,
        max_y: u32,
        blocked: &[u8],
        segment_used: &[u8],
        used_as_middle: &[u8],
        segment_pair: &[u32],
        segment_pair_multi: &[u8],
        start_source: &[u32],
        start_source_multi: &[u8],
        end_target: &[u32],
        end_target_multi: &[u8],
        used_points: Option<&[u8]>,
        route_from_idx: u32,
        route_to_idx: u32,
        edge_from_id: u32,
        edge_to_id: u32,
        allow_end_segment_reuse: bool,
    ) -> Result<Option<(Vec<u32>, u32)>, String> {
        self.ensure_capacity(stride, height);
        let cell_count = stride.saturating_mul(height);

        if cell_count == 0 {
            return Ok(None);
        }
        if blocked.len() < cell_count {
            return Err(format!(
                "blocked 长度不足: blocked.len={} < cell_count={cell_count}",
                blocked.len()
            ));
        }

        let seg_count = cell_count.saturating_mul(2);
        if segment_used.len() < seg_count
            || used_as_middle.len() < seg_count
            || segment_pair.len() < seg_count
            || segment_pair_multi.len() < seg_count
            || start_source.len() < seg_count
            || start_source_multi.len() < seg_count
            || end_target.len() < seg_count
            || end_target_multi.len() < seg_count
        {
            return Err(format!(
                "segmentUsage 长度不足: seg_count={seg_count}, segment_used={}, used_as_middle={}, segment_pair={}, segment_pair_multi={}, start_source={}, start_source_multi={}, end_target={}, end_target_multi={}",
                segment_used.len(),
                used_as_middle.len(),
                segment_pair.len(),
                segment_pair_multi.len(),
                start_source.len(),
                start_source_multi.len(),
                end_target.len(),
                end_target_multi.len(),
            ));
        }

        if let Some(points) = used_points {
            if points.len() < cell_count {
                return Err(format!(
                    "usedPoints 长度不足: used_points.len={} < cell_count={cell_count}",
                    points.len()
                ));
            }
        }

        let from = from_idx as usize;
        let to = to_idx as usize;
        if from >= cell_count || to >= cell_count {
            return Ok(None);
        }

        if max_x as usize >= stride || max_y as usize >= height {
            return Err(format!(
                "bounds 越界: max_x={max_x}, max_y={max_y}, stride={stride}, height={height}"
            ));
        }

        let stamp = self.next_stamp();
        self.heap.clear();

        self.cost_stamp[from] = stamp;
        self.cost_so_far[from] = 0;
        self.came_from[from] = -1;
        self.heap.push(from_idx, 0, 0);

        let to_y = to / stride;
        let to_x = to - to_y * stride;
        let stride_i32 = stride as i32;
        let route_to_i32 = route_to_idx as i32;
        let edge_pair_id = if edge_from_id <= 0xffff && edge_to_id <= 0xffff {
            Some((edge_from_id << 16) | edge_to_id)
        } else {
            None
        };

        while let Some((current_idx_u32, current_cost_at_push)) = self.heap.pop() {
            let current = current_idx_u32 as usize;

            if self.cost_stamp[current] != stamp {
                continue;
            }
            if current_cost_at_push != self.cost_so_far[current] {
                continue;
            }

            if current == to {
                let cost = self.cost_so_far[current];
                return Ok(Some((self.reconstruct_path(current_idx_u32), cost)));
            }

            let current_cost = self.cost_so_far[current];
            let current_y = current / stride;
            let current_x = current - current_y * stride;

            // -----------------------------------------------------------------
            // 4 邻居扩展（顺序与 TS 保持一致：右/左/下/上）
            // -----------------------------------------------------------------

            // 右
            if current_x < max_x as usize {
                let next = current + 1;
                if blocked[next] == 0 || next == to {
                    let mut penalty = 0;
                    let mut ok = true;
                    let seg_key = current * 2;
                    let same_pair_segment = match edge_pair_id {
                        Some(pair) => {
                            segment_pair_multi[seg_key] == 0 && segment_pair[seg_key] == pair
                        }
                        None => false,
                    };
                    if let Some(points) = used_points {
                        penalty += crossing_penalty(points[current], CONNECT_RIGHT);
                        penalty += crossing_penalty(points[next], CONNECT_LEFT);

                        if next != to {
                            let mask = points[next];
                            if mask != 0 {
                                let diff_to_target = route_to_i32 - next as i32;
                                let is_pre_target = diff_to_target == 1
                                    || diff_to_target == -1
                                    || diff_to_target == stride_i32
                                    || diff_to_target == -stride_i32;

                                // point overlap hard rule（与 TS 侧保持一致）：
                                // - 非起点第一步: 禁止走进任何已占用点；
                                // - 起点第一步 / 终点前一步: 只允许走进“不会形成强歧义 junction”的点位。
                                let next_mask = (mask | CONNECT_LEFT) & 0x0F;
                                let arms = BITCOUNT_4[next_mask as usize];

                                // 同端点平行边共享干线:
                                // - 允许“走进已占用点”,从而复用整条路径；
                                // - 但仍然禁止制造 3+ arms junction(否则又会变线团)。
                                if same_pair_segment && arms <= 2 {
                                    // ok: 复用既有直线段不会增加 arms
                                } else if current_idx_u32 != route_from_idx && !is_pre_target {
                                    ok = false;
                                } else if current_idx_u32 == route_from_idx {
                                    // 起点第一步: 不允许制造 3+ arms junction
                                    if arms >= 3 {
                                        ok = false;
                                    }
                                } else {
                                    // 终点前一步: 允许 T junction(3 arms) 汇入,但禁止 `┼`(4 arms)
                                    if arms >= 4 {
                                        ok = false;
                                    }
                                }
                            }
                        }
                    }

                    if ok
                        && is_segment_allowed_relaxed(
                            current,
                            next,
                            /*seg_key=*/ seg_key,
                            segment_used,
                            used_as_middle,
                            segment_pair,
                            segment_pair_multi,
                            start_source,
                            start_source_multi,
                            end_target,
                            end_target_multi,
                            route_from_idx,
                            route_to_idx,
                            edge_from_id,
                            edge_to_id,
                            allow_end_segment_reuse,
                        )
                    {
                        let new_cost = current_cost + 1 + penalty;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_x = current_x + 1;
                            let abs_x = if next_x >= to_x {
                                next_x - to_x
                            } else {
                                to_x - next_x
                            };
                            let abs_y = if current_y >= to_y {
                                current_y - to_y
                            } else {
                                to_y - current_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 左
            if current_x > 0 {
                let next = current - 1;
                if blocked[next] == 0 || next == to {
                    let mut penalty = 0;
                    let mut ok = true;
                    let seg_key = next * 2;
                    let same_pair_segment = match edge_pair_id {
                        Some(pair) => {
                            segment_pair_multi[seg_key] == 0 && segment_pair[seg_key] == pair
                        }
                        None => false,
                    };
                    if let Some(points) = used_points {
                        penalty += crossing_penalty(points[current], CONNECT_LEFT);
                        penalty += crossing_penalty(points[next], CONNECT_RIGHT);

                        if next != to {
                            let mask = points[next];
                            if mask != 0 {
                                let diff_to_target = route_to_i32 - next as i32;
                                let is_pre_target = diff_to_target == 1
                                    || diff_to_target == -1
                                    || diff_to_target == stride_i32
                                    || diff_to_target == -stride_i32;

                                let next_mask = (mask | CONNECT_RIGHT) & 0x0F;
                                let arms = BITCOUNT_4[next_mask as usize];

                                if same_pair_segment && arms <= 2 {
                                    // ok: 同端点平行边复用直线段
                                } else if current_idx_u32 != route_from_idx && !is_pre_target {
                                    ok = false;
                                } else if current_idx_u32 == route_from_idx {
                                    if arms >= 3 {
                                        ok = false;
                                    }
                                } else if arms >= 4 {
                                    ok = false;
                                }
                            }
                        }
                    }

                    if ok
                        && is_segment_allowed_relaxed(
                            current,
                            next,
                            /*seg_key=*/ seg_key,
                            segment_used,
                            used_as_middle,
                            segment_pair,
                            segment_pair_multi,
                            start_source,
                            start_source_multi,
                            end_target,
                            end_target_multi,
                            route_from_idx,
                            route_to_idx,
                            edge_from_id,
                            edge_to_id,
                            allow_end_segment_reuse,
                        )
                    {
                        let new_cost = current_cost + 1 + penalty;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_x = current_x - 1;
                            let abs_x = if next_x >= to_x {
                                next_x - to_x
                            } else {
                                to_x - next_x
                            };
                            let abs_y = if current_y >= to_y {
                                current_y - to_y
                            } else {
                                to_y - current_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 下
            if current_y < max_y as usize {
                let next = current + stride;
                if blocked[next] == 0 || next == to {
                    let mut penalty = 0;
                    let mut ok = true;
                    let seg_key = current * 2 + 1;
                    let same_pair_segment = match edge_pair_id {
                        Some(pair) => {
                            segment_pair_multi[seg_key] == 0 && segment_pair[seg_key] == pair
                        }
                        None => false,
                    };
                    if let Some(points) = used_points {
                        penalty += crossing_penalty(points[current], CONNECT_DOWN);
                        penalty += crossing_penalty(points[next], CONNECT_UP);

                        if next != to {
                            let mask = points[next];
                            if mask != 0 {
                                let diff_to_target = route_to_i32 - next as i32;
                                let is_pre_target = diff_to_target == 1
                                    || diff_to_target == -1
                                    || diff_to_target == stride_i32
                                    || diff_to_target == -stride_i32;

                                let next_mask = (mask | CONNECT_UP) & 0x0F;
                                let arms = BITCOUNT_4[next_mask as usize];

                                if same_pair_segment && arms <= 2 {
                                    // ok: 同端点平行边复用直线段
                                } else if current_idx_u32 != route_from_idx && !is_pre_target {
                                    ok = false;
                                } else if current_idx_u32 == route_from_idx {
                                    if arms >= 3 {
                                        ok = false;
                                    }
                                } else if arms >= 4 {
                                    ok = false;
                                }
                            }
                        }
                    }

                    if ok
                        && is_segment_allowed_relaxed(
                            current,
                            next,
                            /*seg_key=*/ seg_key,
                            segment_used,
                            used_as_middle,
                            segment_pair,
                            segment_pair_multi,
                            start_source,
                            start_source_multi,
                            end_target,
                            end_target_multi,
                            route_from_idx,
                            route_to_idx,
                            edge_from_id,
                            edge_to_id,
                            allow_end_segment_reuse,
                        )
                    {
                        let new_cost = current_cost + 1 + penalty;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_y = current_y + 1;
                            let abs_x = if current_x >= to_x {
                                current_x - to_x
                            } else {
                                to_x - current_x
                            };
                            let abs_y = if next_y >= to_y {
                                next_y - to_y
                            } else {
                                to_y - next_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }

            // 上
            if current_y > 0 {
                let next = current - stride;
                if blocked[next] == 0 || next == to {
                    let mut penalty = 0;
                    let mut ok = true;
                    let seg_key = next * 2 + 1;
                    let same_pair_segment = match edge_pair_id {
                        Some(pair) => {
                            segment_pair_multi[seg_key] == 0 && segment_pair[seg_key] == pair
                        }
                        None => false,
                    };
                    if let Some(points) = used_points {
                        penalty += crossing_penalty(points[current], CONNECT_UP);
                        penalty += crossing_penalty(points[next], CONNECT_DOWN);

                        if next != to {
                            let mask = points[next];
                            if mask != 0 {
                                let diff_to_target = route_to_i32 - next as i32;
                                let is_pre_target = diff_to_target == 1
                                    || diff_to_target == -1
                                    || diff_to_target == stride_i32
                                    || diff_to_target == -stride_i32;

                                let next_mask = (mask | CONNECT_DOWN) & 0x0F;
                                let arms = BITCOUNT_4[next_mask as usize];

                                if same_pair_segment && arms <= 2 {
                                    // ok: 同端点平行边复用直线段
                                } else if current_idx_u32 != route_from_idx && !is_pre_target {
                                    ok = false;
                                } else if current_idx_u32 == route_from_idx {
                                    if arms >= 3 {
                                        ok = false;
                                    }
                                } else if arms >= 4 {
                                    ok = false;
                                }
                            }
                        }
                    }

                    if ok
                        && is_segment_allowed_relaxed(
                            current,
                            next,
                            /*seg_key=*/ seg_key,
                            segment_used,
                            used_as_middle,
                            segment_pair,
                            segment_pair_multi,
                            start_source,
                            start_source_multi,
                            end_target,
                            end_target_multi,
                            route_from_idx,
                            route_to_idx,
                            edge_from_id,
                            edge_to_id,
                            allow_end_segment_reuse,
                        )
                    {
                        let new_cost = current_cost + 1 + penalty;
                        if self.cost_stamp[next] != stamp || new_cost < self.cost_so_far[next] {
                            self.cost_stamp[next] = stamp;
                            self.cost_so_far[next] = new_cost;
                            self.came_from[next] = current_idx_u32 as i32;

                            let next_y = current_y - 1;
                            let abs_x = if current_x >= to_x {
                                current_x - to_x
                            } else {
                                to_x - current_x
                            };
                            let abs_y = if next_y >= to_y {
                                next_y - to_y
                            } else {
                                to_y - next_y
                            };
                            let h = abs_x as u32
                                + abs_y as u32
                                + if abs_x == 0 || abs_y == 0 { 0 } else { 1 };
                            self.heap.push(next as u32, new_cost + h, new_cost);
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 回溯路径（包含 fromIdx 与 toIdx），与 TS 行为一致。
    fn reconstruct_path(&self, mut current_idx: u32) -> Vec<u32> {
        let mut path: Vec<u32> = Vec::new();
        loop {
            path.push(current_idx);
            let parent = self.came_from[current_idx as usize];
            if parent < 0 {
                break;
            }
            current_idx = parent as u32;
        }
        path.reverse();
        path
    }
}

/// strict 约束判定（单步）。
///
/// 说明：
/// - 这里用 free cell 的 UsedPointSet 禁止形成 `┼`（四向交叉）
/// - 并用 SegmentUsage 禁止“不同 source/target 的边复用同一段线”
#[allow(clippy::too_many_arguments)]
fn is_step_allowed_strict(
    step_from: usize,
    step_to: usize,
    seg_key: usize,
    from_bit: u8,
    to_bit: u8,
    segment_used: &[u8],
    used_as_middle: &[u8],
    start_source: &[u32],
    start_source_multi: &[u8],
    end_target: &[u32],
    end_target_multi: &[u8],
    used_points: Option<&[u8]>,
    route_from_idx: u32,
    route_to_idx: u32,
    edge_from_id: u32,
    edge_to_id: u32,
) -> bool {
    // 1) usedPoints：禁止形成 `┼` 四向交叉
    if let Some(points) = used_points {
        let from_mask = points[step_from];
        if from_mask != 0 {
            let next_mask = from_mask | from_bit;
            if (next_mask & H_MASK) == H_MASK && (next_mask & V_MASK) == V_MASK {
                return false;
            }
        }
        let to_mask = points[step_to];
        if to_mask != 0 {
            let next_mask = to_mask | to_bit;
            if (next_mask & H_MASK) == H_MASK && (next_mask & V_MASK) == V_MASK {
                return false;
            }
        }
    }

    // 2) segmentUsage：严格共线共享规则
    if segment_used[seg_key] == 0 {
        return true;
    }

    // 一旦有边把这段当“中间段”用过，那么任何共享都会让语义变得更难读。
    if used_as_middle[seg_key] != 0 {
        return false;
    }

    let step_from_u32 = step_from as u32;
    let step_to_u32 = step_to as u32;
    let is_start_step = step_from_u32 == route_from_idx;
    let is_end_step = step_to_u32 == route_to_idx;

    let ss = start_source[seg_key];
    let et = end_target[seg_key];
    let ss_multi = start_source_multi[seg_key] != 0;
    let et_multi = end_target_multi[seg_key] != 0;

    // 特殊情况：from 与 to 紧挨着时，这一段既是起点段也是终点段。
    // 只允许“同源 + 同靶”的边共享它（例如多条平行边）。
    if is_start_step && is_end_step {
        let start_ok = !ss_multi && (ss == 0 || ss == edge_from_id);
        let end_ok = !et_multi && (et == 0 || et == edge_to_id);
        return start_ok && end_ok;
    }

    // 同源：只允许“起点段”共线，并且该段不能混入终点共享
    if is_start_step {
        return !et_multi && et == 0 && !ss_multi && ss == edge_from_id;
    }

    // 同靶：只允许“终点段”共线，并且该段不能混入起点共享
    if is_end_step {
        return !ss_multi && ss == 0 && !et_multi && et == edge_to_id;
    }

    false
}

/// relaxed：计算“形成 `┼`（四向交叉）”的惩罚（与 TS 的 `crossingPenalty()` 一致）。
fn crossing_penalty(from_mask: u8, add_bit: u8) -> u32 {
    if from_mask == 0 {
        return 0;
    }
    let next_mask = from_mask | add_bit;
    if (next_mask & H_MASK) == H_MASK && (next_mask & V_MASK) == V_MASK {
        RELAXED_PENALTY_CROSSING
    } else {
        0
    }
}

/// relaxed：segment reuse hard rule（禁止共线重叠；仅允许受控的起点/终点段共享）。
///
/// 注意：
/// - 与 strict 不同：relaxed 默认 **禁止终点段复用**（更符合直觉），
///   只有在 JS 侧进入 fallback（不可达）时才会把 `allow_end_segment_reuse` 打开。
#[allow(clippy::too_many_arguments)]
fn is_segment_allowed_relaxed(
    step_from: usize,
    step_to: usize,
    seg_key: usize,
    segment_used: &[u8],
    used_as_middle: &[u8],
    segment_pair: &[u32],
    segment_pair_multi: &[u8],
    start_source: &[u32],
    start_source_multi: &[u8],
    end_target: &[u32],
    end_target_multi: &[u8],
    route_from_idx: u32,
    route_to_idx: u32,
    edge_from_id: u32,
    edge_to_id: u32,
    allow_end_segment_reuse: bool,
) -> bool {
    if segment_used[seg_key] == 0 {
        return true;
    }

    // ---------------------------------------------------------------------
    // 同端点平行边共享干线（终端可读性关键改良）
    //
    // 病灶:
    // - 同一对节点(from->to)存在多条带 label 的边时,如果强行“禁止 segment overlap”,
    //   这些边会被挤到不同通道,最后必然绕外圈形成大矩形,并制造大量 junction。
    //
    // 目标:
    // - 仅对“完全相同端点的平行边”允许复用已占用 segment,
    //   让它们共享同一条干线(视觉上更像同一关系的多种事件)。
    //
    // 安全阈:
    // - segment_pair_multi=1 表示该 segment 曾被多个不同 pair 使用过,
    //   此时禁止共享,避免把不相关的边合并成一条线(误连线灾难)。
    // ---------------------------------------------------------------------
    if edge_from_id <= 0xffff && edge_to_id <= 0xffff {
        let edge_pair_id = (edge_from_id << 16) | edge_to_id;
        if segment_pair_multi[seg_key] == 0 && segment_pair[seg_key] == edge_pair_id {
            return true;
        }
    }

    // 中间段永不允许复用：它必然意味着“合并后再分开”的重叠，读图会崩。
    if used_as_middle[seg_key] != 0 {
        return false;
    }

    let step_from_u32 = step_from as u32;
    let step_to_u32 = step_to as u32;
    let is_start_step = step_from_u32 == route_from_idx;
    let is_end_step = step_to_u32 == route_to_idx;

    // relaxed 默认仍优先“禁止终点段复用”（更符合直觉）。
    // 只有在 JS 侧进入 fallback（不可达）时，才会打开 allow_end_segment_reuse。
    if is_end_step && !allow_end_segment_reuse {
        return false;
    }

    let ss = start_source[seg_key];
    let et = end_target[seg_key];
    let ss_multi = start_source_multi[seg_key] != 0;
    let et_multi = end_target_multi[seg_key] != 0;

    // 特殊情况：from 与 to 紧挨着时，这一段既是起点段也是终点段。
    // 我们只允许“同源 + 同靶”的边共享它，避免引入混淆。
    if is_start_step && is_end_step {
        let start_ok = !ss_multi && (ss == 0 || ss == edge_from_id);
        let end_ok = !et_multi && (et == 0 || et == edge_to_id);
        return start_ok && end_ok;
    }

    // 同源：只允许“起点段”复用，并且该段不能混入任何 end 复用（避免读图歧义）。
    if is_start_step {
        // 重要取舍(终端可读性优先):
        // - 不允许 start 段与 end 段共享同一 unit segment。
        // - 否则双向边会在端口附近“合并成一条线”,人类很难追踪每条 label 的归属。
        //
        // 因此这里保持更强的约束:
        // - start 段只允许与“同 source 的 start 段”复用；
        // - 该 segment 不能同时作为任何边的 end 段(et 必须为 0)。
        return !et_multi && et == 0 && !ss_multi && ss == edge_from_id;
    }

    // 同靶：允许“终点段”复用（最后一段；仅 fallback 开启）。
    if is_end_step {
        // 同靶：允许“终点段”复用（最后一段；仅 fallback 开启）。
        // 但仍然禁止与任何 start 段混用(ss 必须为 0),否则会在 target 端口附近形成难以读懂的合并线。
        return !ss_multi && ss == 0 && !et_multi && et == edge_to_id;
    }

    false
}
