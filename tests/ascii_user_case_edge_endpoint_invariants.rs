// ============================================================================
// 回归测试: 用户复现图的“端点不漂移”不变量
//
// 你反馈的现象:
// - `experiment.complete` 绕路;
// - “实验执行器”附近出现游离箭头;
//
// 我们先用 meta 做最基础但强约束的检查:
// - 每条 edge 的第一个坐标必须落在 source box 边框上;
// - 每条 edge 的最后一个坐标必须贴着 target box 外侧一格;
//
// 这样可以快速捕捉:
// - 箭头头部漂到空白处(游离箭头);
// - 绘制端点与 meta 不一致导致的断线/错位。
// ============================================================================

use std::collections::HashMap;
use std::collections::HashSet;

use beautiful_mermaid_rs::{
    AsciiBox, AsciiDrawingCoord, AsciiRenderOptions, render_mermaid_ascii_with_meta,
};
use unicode_width::UnicodeWidthChar;

/// 判断一个点是否“贴着 box 外侧一格”。
fn is_adjacent_to_box(box_rect: AsciiBox, point: AsciiDrawingCoord) -> bool {
    let min_x = box_rect.x;
    let min_y = box_rect.y;
    let max_x = box_rect.x + box_rect.width - 1;
    let max_y = box_rect.y + box_rect.height - 1;

    let right_side = point.x == max_x + 1 && point.y >= min_y && point.y <= max_y;
    let left_side = point.x == min_x - 1 && point.y >= min_y && point.y <= max_y;
    let top_side = point.y == min_y - 1 && point.x >= min_x && point.x <= max_x;
    let bottom_side = point.y == max_y + 1 && point.x >= min_x && point.x <= max_x;

    right_side || left_side || top_side || bottom_side
}

/// 判断一个点是否落在 box 的边框上。
fn is_on_box_border(box_rect: AsciiBox, point: AsciiDrawingCoord) -> bool {
    let min_x = box_rect.x;
    let min_y = box_rect.y;
    let max_x = box_rect.x + box_rect.width - 1;
    let max_y = box_rect.y + box_rect.height - 1;

    let on_left_or_right =
        (point.x == min_x || point.x == max_x) && point.y >= min_y && point.y <= max_y;
    let on_top_or_bottom =
        (point.y == min_y || point.y == max_y) && point.x >= min_x && point.x <= max_x;

    on_left_or_right || on_top_or_bottom
}

// ============================================================================
// 终端 cell 网格构建
//
// 目的:
// - meta 的坐标是“终端 cell 坐标”,而 Rust 的 `String` 是 Unicode 字符序列;
// - 一旦输出里出现 emoji 等宽字符(宽度=2),字符串索引就不再等于 cell 列号;
// - 这里把每行展开成 cell 网格,保证 (x,y) 可稳定索引。
// ============================================================================

fn build_cell_grid(text: &str) -> Vec<Vec<Option<char>>> {
    text.split('\n')
        .map(|line| {
            let mut cells: Vec<Option<char>> = Vec::new();
            for ch in line.chars() {
                let width = UnicodeWidthChar::width(ch).unwrap_or(0);
                if width == 0 {
                    // 组合字符/零宽字符: 简单跳过,避免破坏 cell 对齐。
                    continue;
                }

                cells.push(Some(ch));
                if width == 2 {
                    // 宽字符的“影子格”(终端会占位,但通常不显示任何字符)。
                    cells.push(None);
                }
            }
            cells
        })
        .collect()
}

fn cell_at(grid: &[Vec<Option<char>>], x: i32, y: i32) -> Option<char> {
    if x < 0 || y < 0 {
        return None;
    }
    let Some(row) = grid.get(y as usize) else {
        return None;
    };
    row.get(x as usize).and_then(|c| *c)
}

fn is_unicode_arrow_char(c: char) -> bool {
    matches!(c, '▲' | '▼' | '◄' | '►' | '◥' | '◤' | '◢' | '◣' | '●')
}

fn is_ascii_arrow_char(c: char) -> bool {
    matches!(c, '^' | 'v' | '<' | '>' | '*')
}

fn char_has_vertical_stroke(c: char, use_ascii: bool) -> bool {
    if use_ascii {
        return matches!(c, '|' | '+');
    }

    matches!(
        c,
        '│' | '┼' | '┬' | '┴' | '├' | '┤' | '┌' | '┐' | '└' | '┘' | '╷' | '╵'
    )
}

fn char_has_horizontal_stroke(c: char, use_ascii: bool) -> bool {
    if use_ascii {
        return matches!(c, '-' | '+');
    }

    matches!(
        c,
        '─' | '┼' | '┬' | '┴' | '├' | '┤' | '┌' | '┐' | '└' | '┘' | '╴' | '╶'
    )
}

#[test]
fn user_repro_case_all_edges_respect_endpoint_invariants() {
    let mermaid = r#"flowchart TD
Hat_ralph["ralph#1 (coordinator)"]
Hat_experiment_auditor[<0001f9fe> 结果审计员]
Hat_experiment_integrator[<0001f9e9> 集成验收员]
Hat_experiment_runner[<0001f9ea> 实验执行器]
Start[task.start]
Start --> Hat_ralph
Complete[complete]
Hat_experiment_auditor -->|experiment.reviewed| Hat_ralph
Hat_experiment_integrator -->|experiment.complete| Complete
Hat_experiment_integrator -->|experiment.complete| Hat_ralph
Hat_experiment_integrator -->|integration.applied| Hat_ralph
Hat_experiment_integrator -->|integration.blocked| Hat_ralph
Hat_experiment_integrator -->|integration.rejected| Hat_ralph
Hat_experiment_runner -->|experiment.result| Hat_experiment_auditor
Hat_ralph -->|experiment.task| Hat_experiment_runner
Hat_ralph -->|integration.task| Hat_experiment_integrator
"#;

    let result = render_mermaid_ascii_with_meta(
        mermaid,
        &AsciiRenderOptions {
            use_ascii: Some(false),
            ..Default::default()
        },
    )
    .expect("render_mermaid_ascii_with_meta should work");

    let cell_grid = build_cell_grid(&result.text);

    let node_boxes: HashMap<_, _> = result
        .meta
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.box_rect))
        .collect();

    // 右侧“最靠右的 node box 边界”(drawing coord)。
    // 用它做相对阈值,比写死 max_x=xx 更稳健:
    // - 未来如果整体布局间距变化,节点会整体平移,但“绕到节点外圈多远”的相对指标仍然有效。
    let max_node_right = result
        .meta
        .nodes
        .iter()
        .map(|node| node.box_rect.x + node.box_rect.width - 1)
        .max()
        .unwrap_or(0);

    if std::env::var("BM_DEBUG_NODE_BOXES").is_ok() {
        for node in &result.meta.nodes {
            eprintln!(
                "[debug] node box: {} label={:?} box={:?}",
                node.id, node.label, node.box_rect
            );
        }
    }

    assert!(
        !result.meta.edges.is_empty(),
        "repro case should contain edges"
    );

    // ---------------------------------------------------------------------
    // 回归不变量: 用户复现的“共享走线假象”必须消失
    //
    // 你反馈的最难读点是:
    // - `experiment.result` 与 `experiment.complete` 在画布中部发生 point overlap,
    //   合成后出现 `┴` 等 junction,视觉上像一条 `complete <-> auditor` 的双向边。
    //
    // 这里直接用 meta 的 drawing 坐标做强约束:
    // - 这两条边在最终画布上不允许共享任何一个 cell。
    // ---------------------------------------------------------------------
    {
        let find_edge = |from: &str, to: &str, label: &str| {
            result
                .meta
                .edges
                .iter()
                .find(|e| e.from == from && e.to == to && e.label == label)
                .unwrap_or_else(|| {
                    panic!("meta should include edge: {from} -> {to} ({label})");
                })
        };

        let exp_result = find_edge(
            "Hat_experiment_runner",
            "Hat_experiment_auditor",
            "experiment.result",
        );
        let exp_complete = find_edge(
            "Hat_experiment_integrator",
            "Complete",
            "experiment.complete",
        );

        let to_set = |path: &[AsciiDrawingCoord]| -> HashSet<(i32, i32)> {
            path.iter().map(|c| (c.x, c.y)).collect()
        };
        let overlap = to_set(&exp_result.path)
            .intersection(&to_set(&exp_complete.path))
            .count();

        // 发生回归时,优先把“重叠点位”打印出来,避免只能看到 overlap=1 却不知道在哪里。
        //
        // 使用方式:
        // - `BM_DEBUG_EDGE_OVERLAP=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
        if std::env::var("BM_DEBUG_EDGE_OVERLAP").is_ok() && overlap > 0 {
            let mut coords: Vec<(i32, i32)> = to_set(&exp_result.path)
                .intersection(&to_set(&exp_complete.path))
                .copied()
                .collect();
            coords.sort_unstable();

            eprintln!(
                "[debug] overlap coords (experiment.result vs integrator->Complete): count={overlap}"
            );
            for (x, y) in coords.iter().take(12) {
                let ch = cell_at(&cell_grid, *x, *y).unwrap_or(' ');
                eprintln!("[debug] overlap at ({x},{y}) char={ch:?}");
            }
        }

        assert_eq!(
            overlap, 0,
            "experiment.result 与 integrator->Complete(experiment.complete) 不应共享任何绘制 cell: overlap_cells={overlap}"
        );
    }

    // ---------------------------------------------------------------------
    // 调试信息(可选): 检查 `experiment.result` 是否与其它边发生大面积重叠
    //
    // 背景:
    // - 你反馈 `experiment.result` 这条线“绕圈/绕路”,另一个常见诱因是:
    //   多条边在视觉上重叠,导致读者误把“别的边的路径”当成 `experiment.result` 的一部分。
    // - 这里用 meta 的 drawing 坐标做一次简单交集统计,快速判断是否存在该类问题。
    //
    // 使用方式:
    // - `BM_DEBUG_EDGE_OVERLAP=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
    // ---------------------------------------------------------------------
    if std::env::var("BM_DEBUG_EDGE_OVERLAP").is_ok() {
        let edge_key = |from: &str, to: &str, label: &str| format!("{from}->{to}::{label}");
        let mut edges_by_key: HashMap<String, &beautiful_mermaid_rs::AsciiRenderMetaEdge> =
            HashMap::new();
        for edge in &result.meta.edges {
            edges_by_key.insert(edge_key(&edge.from, &edge.to, &edge.label), edge);
        }

        let Some(exp_result) = edges_by_key.get(&edge_key(
            "Hat_experiment_runner",
            "Hat_experiment_auditor",
            "experiment.result",
        )) else {
            panic!("meta should include experiment.result edge");
        };
        let Some(exp_complete_to_complete) = edges_by_key.get(&edge_key(
            "Hat_experiment_integrator",
            "Complete",
            "experiment.complete",
        )) else {
            panic!("meta should include integrator -> Complete (experiment.complete) edge");
        };

        let to_set = |path: &[AsciiDrawingCoord]| -> HashSet<(i32, i32)> {
            path.iter().map(|c| (c.x, c.y)).collect()
        };

        let a = to_set(&exp_result.path);
        let b = to_set(&exp_complete_to_complete.path);
        let overlap = a.intersection(&b).count();

        eprintln!(
            "[debug] edge overlap: experiment.result vs integrator->Complete(experiment.complete): overlap_cells={overlap}, result_len={}, complete_len={}",
            exp_result.path.len(),
            exp_complete_to_complete.path.len(),
        );

        if overlap > 0 {
            let mut coords: Vec<(i32, i32)> = a.intersection(&b).copied().collect();
            coords.sort_unstable();
            for (x, y) in coords.iter().take(12) {
                let ch = cell_at(&cell_grid, *x, *y).unwrap_or(' ');
                eprintln!("[debug] overlap at ({x},{y}) char={ch:?}");
            }
        }
    }

    // ---------------------------------------------------------------------
    // 调试信息(可选): 打印“走到很右侧”的边
    //
    // 目的:
    // - 你看到的“绕圈/矩形绕行”有时其实来自其它边(路径很宽),
    //   但因为线段在视觉上共线,容易被误认为是 `experiment.result` 的一部分。
    // - 这里快速列出 max_x 很大的边,辅助定位是哪条边在“画大框”。
    //
    // 使用方式:
    // - `BM_DEBUG_WIDE_EDGES=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
    // ---------------------------------------------------------------------
    if std::env::var("BM_DEBUG_WIDE_EDGES").is_ok() {
        for edge in &result.meta.edges {
            let mut max_x = i32::MIN;
            let mut min_x = i32::MAX;
            let mut max_y = i32::MIN;
            let mut min_y = i32::MAX;
            for c in &edge.path {
                max_x = max_x.max(c.x);
                min_x = min_x.min(c.x);
                max_y = max_y.max(c.y);
                min_y = min_y.min(c.y);
            }
            if max_x >= 80 {
                eprintln!(
                    "[debug] wide edge: {} -> {} ({}) bbox=({min_x},{min_y})-({max_x},{max_y}) len={}",
                    edge.from,
                    edge.to,
                    edge.label,
                    edge.path.len()
                );
            }
        }
    }

    for edge in &result.meta.edges {
        let source_box = node_boxes.get(edge.from.as_str()).unwrap_or_else(|| {
            panic!(
                "meta should include source node box: from={}, to={}, label={}",
                edge.from, edge.to, edge.label
            )
        });
        let target_box = node_boxes.get(edge.to.as_str()).unwrap_or_else(|| {
            panic!(
                "meta should include target node box: from={}, to={}, label={}",
                edge.from, edge.to, edge.label
            )
        });

        let first = edge.path.first().expect("edge.path should not be empty");
        let last = edge.path.last().expect("edge.path should not be empty");

        assert!(
            is_on_box_border(*source_box, *first),
            "edge {} -> {} ({}) should start on source border, first={first:?}, source={source_box:?}",
            edge.from,
            edge.to,
            edge.label
        );

        assert!(
            is_adjacent_to_box(*target_box, *last),
            "edge {} -> {} ({}) should end adjacent to target box, last={last:?}, target={target_box:?}",
            edge.from,
            edge.to,
            edge.label
        );

        // -----------------------------------------------------------------
        // 回归不变量: 避免 integrator 相关边“绕到最右侧外圈画大框”
        //
        // 用户感知:
        // - `experiment.complete` / `integration.rejected` 等边会把线路推到很右侧,
        //   形成一个巨大的外框,从而让 `experiment.result` 看起来像在“绕圈”。
        //
        // 我们用相对阈值锁死“不要绕出节点右边界太远”：
        // - 只约束用户这个复现图里的关键边；
        // - 阈值给足冗余(避免未来微调 layout 时误伤),
        //   但能有效阻止回到“右侧 max_x≈110”的极端外圈。
        // -----------------------------------------------------------------
        if edge.from == "Hat_experiment_integrator"
            && edge.to == "Complete"
            && edge.label == "experiment.complete"
        {
            let edge_max_x = edge.path.iter().map(|c| c.x).max().unwrap_or(0);
            let extra_right = edge_max_x - max_node_right;

            // -----------------------------------------------------------------
            // 调试信息(可选): `integrator -> Complete (experiment.complete)` 绕路问题
            //
            // 说明:
            // - 该边一旦“跑到最右侧外圈”,会让整张图出现大矩形外框,并诱发其它边绕圈。
            // - 当断言失败时,如果 debug 输出放在断言之后,将永远看不到路径细节。
            //
            // 因此这里把 debug 打印前置:
            // - 只在显式设置环境变量时输出(默认保持测试安静)。
            //
            // 使用方式:
            // - `BM_DEBUG_EXPERIMENT_COMPLETE_TO_COMPLETE=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
            // -----------------------------------------------------------------
            if std::env::var("BM_DEBUG_EXPERIMENT_COMPLETE_TO_COMPLETE").is_ok() {
                let first = edge
                    .path
                    .first()
                    .copied()
                    .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });
                let last = edge
                    .path
                    .last()
                    .copied()
                    .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });

                let mut min_x = i32::MAX;
                let mut max_x = i32::MIN;
                let mut min_y = i32::MAX;
                let mut max_y = i32::MIN;
                for c in &edge.path {
                    min_x = min_x.min(c.x);
                    max_x = max_x.max(c.x);
                    min_y = min_y.min(c.y);
                    max_y = max_y.max(c.y);
                }

                let manhattan = (first.x - last.x).abs() + (first.y - last.y).abs();
                let mut turns = 0usize;
                let mut prev_dir: Option<(i32, i32)> = None;
                for pair in edge.path.windows(2) {
                    let a = pair[0];
                    let b = pair[1];
                    let dx = (b.x - a.x).signum();
                    let dy = (b.y - a.y).signum();
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let dir = (dx, dy);
                    if let Some(prev) = prev_dir {
                        if dir != prev {
                            turns += 1;
                        }
                    }
                    prev_dir = Some(dir);
                }

                let head: Vec<String> = edge
                    .path
                    .iter()
                    .take(12)
                    .map(|c| format!("({},{})", c.x, c.y))
                    .collect();
                let tail: Vec<String> = edge
                    .path
                    .iter()
                    .rev()
                    .take(12)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .map(|c| format!("({},{})", c.x, c.y))
                    .collect();

                let arrow_char = cell_at(&cell_grid, last.x, last.y).unwrap_or(' ');

                eprintln!(
                    "[debug] experiment.complete(to Complete) meta: extra_right={}, edge_max_x={}, max_node_right={}, len={}, manhattan={}, turns={}, arrow_char={:?}, bbox=({},{})-({},{}) first={:?} last={:?} head=[{}] tail=[{}]",
                    extra_right,
                    edge_max_x,
                    max_node_right,
                    edge.path.len(),
                    manhattan,
                    turns,
                    arrow_char,
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                    first,
                    last,
                    head.join(" "),
                    tail.join(" ")
                );
            }

            assert!(
                extra_right <= 10,
                "integrator->Complete(experiment.complete) should not detour far right: extra_right={extra_right}, edge_max_x={edge_max_x}, max_node_right={max_node_right}"
            );

            assert!(
                // 说明:
                // - 这里的长度阈值主要用于防止回到“外圈大矩形”那类灾难性绕路；
                // - 但我们现在对“起点第一步走进已占用点位”做了更严格的可读性约束，
                //   在少数图里会让路径略微变长(但只要不跑到外圈,仍然是可接受的)。
                //
                // 因此这里把阈值从 100 放宽到 120,并继续用上方的 extra_right<=10
                // 锁死“不要绕到最右侧外圈”这个真正的用户痛点。
                edge.path.len() <= 120,
                "integrator->Complete(experiment.complete) path too long (likely outer detour): len={}, max_node_right={max_node_right}",
                edge.path.len()
            );
        }

        if edge.from == "Hat_experiment_integrator"
            && edge.to == "Hat_ralph"
            && edge.label == "integration.rejected"
        {
            let edge_max_x = edge.path.iter().map(|c| c.x).max().unwrap_or(0);
            let extra_right = edge_max_x - max_node_right;
            assert!(
                extra_right <= 10,
                "integrator->Hat_ralph(integration.rejected) should not detour far right: extra_right={extra_right}, edge_max_x={edge_max_x}, max_node_right={max_node_right}"
            );

            assert!(
                // 说明:
                // - 在回到“与上游 TS 基线一致”的 relaxed 路由后,
                //   `integration.rejected` 这条边会更倾向走外侧大回环,以避免走进占用点/制造歧义 junction；
                // - 这会让 path（按 cell 逐格展开的 stroke 坐标）明显变长。
                //
                // 我们仍然保留上方的 `extra_right<=10` 作为硬约束,防止它跑到最右侧外圈。
                // 这里的长度阈值只作为“防爆线”: 避免回到无限绕圈那类灾难性退化。
                edge.path.len() <= 220,
                "integrator->Hat_ralph(integration.rejected) path too long (likely outer detour): len={}, max_node_right={max_node_right}",
                edge.path.len()
            );
        }

        // -----------------------------------------------------------------
        // 调试信息(可选): `experiment.result` 绕圈问题
        //
        // 说明:
        // - 你反馈 `Hat_experiment_runner -> Hat_experiment_auditor (experiment.result)` 走线绕圈。
        // - 这里先把 meta 里的关键指标打印出来,便于我们后续把“绕圈”量化成可回归的不变量。
        //
        // 使用方式(仅在需要时打开,默认不输出):
        // - `BM_DEBUG_EXPERIMENT_RESULT=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
        // -----------------------------------------------------------------
        if edge.from == "Hat_experiment_runner"
            && edge.to == "Hat_experiment_auditor"
            && edge.label == "experiment.result"
            && std::env::var("BM_DEBUG_EXPERIMENT_RESULT").is_ok()
        {
            let first = edge
                .path
                .first()
                .copied()
                .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });
            let last = edge
                .path
                .last()
                .copied()
                .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });

            let mut min_x = i32::MAX;
            let mut max_x = i32::MIN;
            let mut min_y = i32::MAX;
            let mut max_y = i32::MIN;
            for c in &edge.path {
                min_x = min_x.min(c.x);
                max_x = max_x.max(c.x);
                min_y = min_y.min(c.y);
                max_y = max_y.max(c.y);
            }

            let manhattan = (first.x - last.x).abs() + (first.y - last.y).abs();
            let mut turns = 0usize;
            let mut prev_dir: Option<(i32, i32)> = None;
            for pair in edge.path.windows(2) {
                let a = pair[0];
                let b = pair[1];
                let dx = (b.x - a.x).signum();
                let dy = (b.y - a.y).signum();
                if dx == 0 && dy == 0 {
                    continue;
                }
                let dir = (dx, dy);
                if let Some(prev) = prev_dir {
                    if dir != prev {
                        turns += 1;
                    }
                }
                prev_dir = Some(dir);
            }

            let head: Vec<String> = edge
                .path
                .iter()
                .take(12)
                .map(|c| format!("({},{})", c.x, c.y))
                .collect();
            let tail: Vec<String> = edge
                .path
                .iter()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|c| format!("({},{})", c.x, c.y))
                .collect();

            let arrow_char = cell_at(&cell_grid, last.x, last.y).unwrap_or(' ');

            eprintln!(
                "[debug] experiment.result meta: len={}, manhattan={}, turns={}, arrow_char={:?}, source_box={:?}, target_box={:?}, bbox=({},{})-({},{}) first={:?} last={:?} head=[{}] tail=[{}]",
                edge.path.len(),
                manhattan,
                turns,
                arrow_char,
                source_box,
                target_box,
                min_x,
                min_y,
                max_x,
                max_y,
                first,
                last,
                head.join(" "),
                tail.join(" ")
            );
        }

        // -----------------------------------------------------------------
        // 调试信息(可选): `integrator -> Complete (experiment.complete)` 绕路问题
        //
        // 使用方式:
        // - `BM_DEBUG_EXPERIMENT_COMPLETE_TO_COMPLETE=1 cargo test --test ascii_user_case_edge_endpoint_invariants -- --nocapture`
        // -----------------------------------------------------------------
        if edge.from == "Hat_experiment_integrator"
            && edge.to == "Complete"
            && edge.label == "experiment.complete"
            && std::env::var("BM_DEBUG_EXPERIMENT_COMPLETE_TO_COMPLETE").is_ok()
        {
            let first = edge
                .path
                .first()
                .copied()
                .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });
            let last = edge
                .path
                .last()
                .copied()
                .unwrap_or(AsciiDrawingCoord { x: 0, y: 0 });

            let mut min_x = i32::MAX;
            let mut max_x = i32::MIN;
            let mut min_y = i32::MAX;
            let mut max_y = i32::MIN;
            for c in &edge.path {
                min_x = min_x.min(c.x);
                max_x = max_x.max(c.x);
                min_y = min_y.min(c.y);
                max_y = max_y.max(c.y);
            }

            let manhattan = (first.x - last.x).abs() + (first.y - last.y).abs();
            let mut turns = 0usize;
            let mut prev_dir: Option<(i32, i32)> = None;
            for pair in edge.path.windows(2) {
                let a = pair[0];
                let b = pair[1];
                let dx = (b.x - a.x).signum();
                let dy = (b.y - a.y).signum();
                if dx == 0 && dy == 0 {
                    continue;
                }
                let dir = (dx, dy);
                if let Some(prev) = prev_dir {
                    if dir != prev {
                        turns += 1;
                    }
                }
                prev_dir = Some(dir);
            }

            let head: Vec<String> = edge
                .path
                .iter()
                .take(12)
                .map(|c| format!("({},{})", c.x, c.y))
                .collect();
            let tail: Vec<String> = edge
                .path
                .iter()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|c| format!("({},{})", c.x, c.y))
                .collect();
            let arrow_char = cell_at(&cell_grid, last.x, last.y).unwrap_or(' ');

            eprintln!(
                "[debug] experiment.complete(to Complete) meta: len={}, manhattan={}, turns={}, arrow_char={:?}, bbox=({},{})-({},{}) first={:?} last={:?} head=[{}] tail=[{}]",
                edge.path.len(),
                manhattan,
                turns,
                arrow_char,
                min_x,
                min_y,
                max_x,
                max_y,
                first,
                last,
                head.join(" "),
                tail.join(" ")
            );
        }

        // -----------------------------------------------------------------
        // 额外不变量(仅锁定用户反馈的关键边): 箭头不能“游离”
        //
        // 解释:
        // - 你反馈“实验执行器上有个游离箭头”,定位到的具体边是:
        //   `Hat_ralph -> Hat_experiment_runner (experiment.task)`。
        // - 这里仅对这条边做强约束,避免把“多边汇聚处的复杂合并字符”误判为回归。
        // -----------------------------------------------------------------
        if edge.from != "Hat_ralph"
            || edge.to != "Hat_experiment_runner"
            || edge.label != "experiment.task"
        {
            continue;
        }

        let use_ascii = false;
        let is_arrow_char = |ch: char| {
            if use_ascii {
                is_ascii_arrow_char(ch)
            } else {
                is_unicode_arrow_char(ch)
            }
        };

        // 优先使用 meta 的最后一个点(按设计应该就是 arrow cell)。
        let arrow_cell = edge
            .path
            .last()
            .and_then(|c| cell_at(&cell_grid, c.x, c.y).map(|ch| (*c, ch)))
            .filter(|(_coord, ch)| is_arrow_char(*ch))
            .or_else(|| {
                // 兜底: 反向扫描整条 path,找最后出现的 arrow cell。
                edge.path.iter().rev().find_map(|c| {
                    let ch = cell_at(&cell_grid, c.x, c.y)?;
                    if is_arrow_char(ch) {
                        Some((*c, ch))
                    } else {
                        None
                    }
                })
            });

        let Some((arrow_coord, arrow_char)) = arrow_cell else {
            // 调试信息: 打印 path 尾部坐标与对应字符,便于定位“meta 与最终文本不一致”的问题。
            let mut tail: Vec<String> = Vec::new();
            for c in edge.path.iter().rev().take(12).rev() {
                let ch = cell_at(&cell_grid, c.x, c.y).unwrap_or(' ');
                tail.push(format!("{c:?}={ch:?}"));
            }

            panic!(
                "edge {} -> {} ({}) should contain an arrowhead cell, path_tail=[{}]",
                edge.from,
                edge.to,
                edge.label,
                tail.join(", ")
            );
        };

        match arrow_char {
            '▼' | 'v' => {
                let incoming = cell_at(&cell_grid, arrow_coord.x, arrow_coord.y - 1).unwrap_or(' ');
                assert!(
                    char_has_vertical_stroke(incoming, use_ascii),
                    "down-arrow must have vertical stroke above it: edge {} -> {} ({}), arrow={arrow_coord:?}, incoming_char={incoming:?}",
                    edge.from,
                    edge.to,
                    edge.label
                );
            }
            '▲' | '^' => {
                let incoming = cell_at(&cell_grid, arrow_coord.x, arrow_coord.y + 1).unwrap_or(' ');
                assert!(
                    char_has_vertical_stroke(incoming, use_ascii),
                    "up-arrow must have vertical stroke below it: edge {} -> {} ({}), arrow={arrow_coord:?}, incoming_char={incoming:?}",
                    edge.from,
                    edge.to,
                    edge.label
                );
            }
            '►' | '>' => {
                let incoming = cell_at(&cell_grid, arrow_coord.x - 1, arrow_coord.y).unwrap_or(' ');
                assert!(
                    char_has_horizontal_stroke(incoming, use_ascii),
                    "right-arrow must have horizontal stroke on its left: edge {} -> {} ({}), arrow={arrow_coord:?}, incoming_char={incoming:?}",
                    edge.from,
                    edge.to,
                    edge.label
                );
            }
            '◄' | '<' => {
                let incoming = cell_at(&cell_grid, arrow_coord.x + 1, arrow_coord.y).unwrap_or(' ');
                assert!(
                    char_has_horizontal_stroke(incoming, use_ascii),
                    "left-arrow must have horizontal stroke on its right: edge {} -> {} ({}), arrow={arrow_coord:?}, incoming_char={incoming:?}",
                    edge.from,
                    edge.to,
                    edge.label
                );
            }
            _ => {
                // 斜向箭头/圆点: 当前先不做强约束(真实用例较少,避免误伤其它图)。
            }
        }
    }
}
