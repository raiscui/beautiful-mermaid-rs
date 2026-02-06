# 笔记：beautiful-mermaid（TS 版）复刻要点

## 来源

### 来源1：本地参考仓库
- 路径：`/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid`
- 要点（从 README / 源码提取）：
  - 支持 5 种图表：Flowchart/State、Sequence、Class、ER
  - 双输出：SVG（异步）与 ASCII/Unicode（同步）
  - 主题系统：`bg/fg` 两色派生 + 可选 enrich（line/accent/muted/surface/border）
  - 对外 API（TS）：
    - `renderMermaid(text, options?) -> Promise<string>`
    - `renderMermaidAscii(text, options?) -> string`
    - `THEMES` / `DEFAULTS` / `fromShikiTheme`

## 综合发现

### 复刻落地策略
- 先以“行为一致”为最高优先级，采用 Rust 内嵌 JS 引擎执行打包后的 JS bundle。
- 测试层面：优先搬运 TS 的测试用例与 testdata（ASCII/Unicode 输出对比）。
- 稳定后再逐步把内部实现替换为纯 Rust（保持 API 与测试不变）。

### TS 版 ASCII golden tests 的关键细节（必须对齐）
- TS 的对比不是“逐字符完全一致”，而是先做 whitespace 归一化：
  - 每行 `trimEnd()`（去掉行尾空格）
  - 去掉首尾空行
- 参考实现见：`/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/__tests__/ascii.test.ts`

## 2026-02-01 20:38 - 仓库贡献者指南（AGENTS.md）需要覆盖的信息

- 结构事实：`src/`（lib+cli）、`tests/`（golden+smoke）、`vendor/`（JS bundle）、`docs/`（补充文档）。
- 构建/安装：`cargo build/test/run` + `make release/install`（`INSTALL_DIR` 可通过命令行覆盖）。
- Git 约定：当前仓库只有一条提交记录，风格是 `type: summary`（例：`init: bootstrap ...`）。

## 2026-02-02 16:25 - vendor bundle 更新会触发 golden 变化（预期行为）

- `tests/ascii_testdata.rs` 是 golden test：它的职责是“锁定当前渲染输出”，任何渲染布局变化都会被当成回归提示出来。
- 因此当我们用 `scripts/sync-vendor-bundle.sh` 同步了上游 TS bundle 后：
  - 如果上游修复/调整了布局算法（尤其是“自环/多边合并”的连线策略），golden 输出变化是正常的。
  - 对应做法是：审阅差异后，更新 `tests/testdata/{ascii,unicode}/*.txt` 的期望输出，让 Rust 侧测试与最新 vendor 对齐。

## 2026-02-02 21:24 - 本次 bundle 变更的影响范围（含性能观察）

- 本次 `tsup` 产物更新后（vendor bundle sha256: `18ac06ce...`），golden 发生变化的用例包括：
  - `ampersand_lhs_and_rhs`
  - `cls_all_relationships`
  - `er_identifying`
  - `preserve_order_of_definition`
  - `self_reference_with_edge`
- 为了避免每次都“手工改很多 golden”，在 `tests/ascii_testdata.rs` 增加了 `UPDATE_GOLDEN=1` 模式：
  - 会把当前渲染输出写回 `tests/testdata/{ascii,unicode}/*.txt`，然后 panic 提示重新跑测试确认稳定。
- 配套：新增 `.envrc`，提供 `UPDATE_GOLDEN=0` 默认值与注释说明（便于用 direnv 开关）。
- 性能观察（需要关注）：
  - `preserve_order_of_definition` 这类包含自环/循环边的图，在当前 vendor bundle 下渲染耗时明显变长（单个案例可达 ~50s）。
  - 这会导致 `cargo test` 的 golden 部分整体耗时上升（本机观测可达 70-100s 级别）。

## 2026-02-03 00:18 - 性能治理方向：把 A* 路由的热循环 native 化

- 关键认知：
  - QuickJS 没有 JIT，CPU 密集型算法（如 A* + heap）在解释执行下会被放大常数开销。
  - ASCII/Unicode 渲染里，A* 的 “pop + 4 邻居扩展” 会被调用很多次，是最典型的热路径。
- 落地策略（本仓库已实现）：
  - 用 Rust 实现 A*（含 strict 约束版本），并通过 `rquickjs` 暴露 `globalThis.__bm_getPath*`。
  - TS bundle 只要做一个“存在性检测”，就能在不改外部 API 的前提下自动启用 native 加速。

## 2026-02-03 14:08 - README 重写要点（上游问题 & 本仓库改动）

### 上游 TS 版 beautiful-mermaid 暴露过的问题（本仓库已通过 vendor bundle 同步修复）

- Flowchart/State parser 的节点/子图 ID 匹配过于“ASCII 化”（例如 `\\w` / `[\\w-]`）：
  - 现象：中文/Unicode ID 解析丢失，最终进入 dagre 空图布局，输出出现 `-Infinity` 或空白。
  - 修复方向：用 Unicode 属性类（例如 `\\p{L}\\p{N}`）替代 `\\w`，并开启 `u` flag。
- ASCII/Unicode 渲染里对宽字符（中文/emoji）用字符串长度做宽度估算：
  - 现象：边框/连线错位，右边框会被“顶出去”。
  - 修复方向：引入（简化版）`wcwidth` 逻辑，以“终端显示宽度”而不是 “string length” 计算布局。

### 本仓库需要在 README 里讲清楚的关键改动

- Rust 侧提供“库 + CLI”：
  - 公共 API：`render_mermaid()` / `render_mermaid_ascii()`。
  - CLI：stdin → SVG/ASCII，支持 `--help/--version`，并定义 exit code 约定。
- 实现策略：Rust 内嵌 QuickJS（`rquickjs`）执行 browser IIFE bundle，快速对齐 TS 行为。
  - thread-local：每线程一个 JS 引擎实例，避免跨线程共享 Context。
- vendor 同步工作流：
  - `scripts/sync-vendor-bundle.sh` + `make sync-vendor(-verify)` + `make install`（install 前强制同步并跑 `cargo test`）。
- 测试策略：
  - ASCII/Unicode golden tests（对齐 TS 的 whitespace normalize）。
  - `UPDATE_GOLDEN=1` 模式 + `.envrc`（direnv）降低维护成本。
- 性能加速（QuickJS 无 JIT 的现实补偿）：
  - Native pathfinder：把 A* 热循环挪到 Rust，并通过 `globalThis.__bm_getPath*` 注入给 JS；bundle 运行时检测并自动启用。

## 2026-02-06 16:05 - 修复 golden 过期（Unicode）

- 触发方式：`make install` 内部的 `sync-vendor-verify` 重新构建并同步了 TS bundle（sha256 `b48b9228...`）。
- 现象：`tests/ascii_testdata.rs` 的 `unicode_testdata_matches_reference` 首个 mismatch 暴露为 `ampersand_lhs_and_rhs`。
  - 因为测试遇到第一个 mismatch 会立刻 `assert_eq!` 退出, 所以只看失败输出会漏掉后续 mismatch。
- 修复方式：使用仓库内置的 golden 自动更新模式。
  - 命令：`UPDATE_GOLDEN=1 cargo test --test ascii_testdata unicode_testdata_matches_reference`
  - 实际更新了 2 个文件：
    - `tests/testdata/unicode/ampersand_lhs_and_rhs.txt`
    - `tests/testdata/unicode/preserve_order_of_definition.txt`
- 验证：
  - `cargo test` 全通过。
  - `make install` 端到端通过（tsup build → sync vendor → cargo test → release install）。

## 2026-02-06 16:39 - Mermaid validator 集成调研（来自 mcp-mermaid-validator）

## 来源

### 来源1: `/Users/cuiluming/local_doc/l_dev/my/rust/mcp-mermaid-validator/src/main.ts`
- 这是一个 MCP server, 对外提供工具 `validateMermaid`.
- 输入:
  - `diagram: string`
- 输出结构化字段:
  - `isValid: boolean`
  - `error?: string`
  - `details?: string`
- 核心校验机制:
  - 通过 `child_process.spawn` 调用:
    - `npx @mermaid-js/mermaid-cli -i /dev/stdin -o - -e png`
  - 把 Mermaid 文本写入子进程 stdin.
  - stdout 的图片数据直接丢弃, 只拿“能否成功生成”作为语法有效性的判据.
  - stderr 会累计为 errorOutput, 在失败时拼进错误信息, 作为 `details`.
- 失败模型:
  - 子进程退出码非 0, 则认为 Mermaid 无效.
  - 返回 `isValid=false`, 并把错误主信息与 stderr 细节拆分出来.

## 综合发现

- 这个 validator 的本质是“能否成功渲染”的副作用校验, 并不单独做 parse-only.
- 如果我们要把它“集成到 Rust crate”, 至少需要对齐两点:
  1. 给出稳定的 `true/false + error/details` 输出模型（便于 CLI/CI 消费）。
  2. 避免把 `mcp-mermaid-validator` 作为依赖引入（可以选择复刻其策略, 或用本仓库 QuickJS 渲染器作为校验后端）。

## 2026-02-06 16:54 - validator 后端选择: QuickJS 渲染器太宽松, 改用纯 Rust parser

- 尝试过的方案: 在 QuickJS 里调用本仓库的 `beautifulMermaid.renderMermaid(...)` 作为“是否有效”的判据。
- 发现的问题:
  - Flowchart/graph 的解析非常宽松, 很多明显不合法的输入也会返回“可渲染”, 导致校验几乎恒为 true.
  - 这不符合我们对 validator 的期望: 必须能在语法错误时给出可靠的 false + 错误信息.
- 最终选择:
  - 采用 `selkie-rs`（mermaid.js 的 Rust port）作为 parse/validate 后端.
  - `selkie::parse` 在遇到语法错误时会返回包含行列信息的 parse error 字符串, 更适合做严格校验与 CI gate.

## 2026-02-06 19:33 - QuickJS 性能：native pathfinder 覆盖范围

- 结论：
  - `__bm_getPath`：非 strict A*（仅 blocked + bounds）。
  - `__bm_getPathStrict`：strict A*（禁 `┼` + segment reuse 规则）。
  - `__bm_getPathRelaxed`：relaxed A*（允许 crossing，但对潜在 `┼` 加惩罚；并执行“不共线重叠”的 segment hard rule）。
- 为什么必须补 `__bm_getPathRelaxed`：
  - Unicode 默认 routing=relaxed。
  - 如果 relaxed 没有 native fast path，CLI 下仍会回退到纯 JS A*，QuickJS 无 JIT 会非常慢。

## 2026-02-06 20:13 - Flowchart: “节点先声明, 再连线”触发 root 识别偏差, 导致线路强歧义

## 来源

### 来源1: TS `beautiful-mermaid` - `src/ascii/grid.ts`
- 相关逻辑在 `createMappingOnce()` 的 rootNodes 识别段落。
- 文件内注释写的是:
  - "Identify root nodes — nodes that aren't the target of any edge"
- 但当前实现实际是按 node insertion order 的“首次出现”来推断 root:
  - 先遍历 `graph.nodes`，如果某 node.name 之前没见过就加入 rootNodes。
  - 再把该 node 的 children 也标记为“见过”。
- 这会导致一个典型偏差:
  - 如果 Mermaid 先把节点都声明完（`A[...]`、`B[...]`...），再写边，
  - 那么遍历 nodes 时, 很多“其实有入边”的节点, 在遇到它之前都不会被标记为 child,
  - 最终会被误判成 root 并堆在同一列（LR 模式下 x 相同, y 递增）。

## 综合发现

- 这类 root 误判会带来两个直接后果:
  1. 布局层面: root 堆叠会把本该在不同层级的节点塞到同一列, 使某些边不得不大绕路。
  2. 可读性层面: 绕路边更容易贴近/交错其它边, 在 Unicode 输出中形成 T junction 或“看起来像连上了别的节点”的假象。
- 对用户的这个例子（spec workflow hats）:
  - `task.start -> ralph` 被迫绕行并与 reviewer 相关的边贴合, 肉眼会误读为指向 reviewer。
- 风险评估:
  - 修正 rootNodes 为“无入边节点”属于典型 bug fix（实现与注释语义对齐）。
  - 对大多数“只用边隐式声明节点”的图, root 集合通常不变, 对 golden 的影响预计可控。

## 2026-02-06 20:56 - 落地记录: strict/relaxed 的取舍与 golden 影响面

- 关键取舍:
  - relaxed: rootNodes 改为“无入边节点”, 解决“先声明节点, 再连边”导致的 root 误判与线路强歧义。
  - strict: 保持旧 root 推断与旧路由兜底策略, 避免 strict golden/roundtrip 行为漂移。
- 一个直接影响:
  - Unicode 默认 routing=relaxed, 因此 relaxed rootNodes 修正会改变部分 Unicode golden。
  - 例如 `preserve_order_of_definition` 用例中, A 有入边但在声明顺序里排在前面, 旧实现会把 A 当 root, 新实现会把真正 root(B) 放在最左侧。
  - Rust 侧处理:
  - 同步 vendor bundle 后, 通过 `UPDATE_GOLDEN=1` 仅更新了 `tests/testdata/unicode/preserve_order_of_definition.txt`。

## 2026-02-06 23:03 - TD 输出 “出线不贴边”: labelLine 扩宽列误伤 node 顶点列

### 现象
- `flowchart TD` + Unicode(relaxed) 渲染时, “🔎 规格审阅者” 右侧出现一条 box 内部竖线, 看起来像线从 box 里面长出来, 端口没有贴到边框。

### 关键证据(来自 meta, 可量化)
- 使用 `renderMermaidAsciiWithMeta(..., { useAscii:false, routing:\"relaxed\" })` 检查:
  - reviewer box: `{ x:47, y:30, width:31, height:5 }`
  - edge `Hat_spec_reviewer -> Hat_spec_writer (spec.rejected)` 的 stroke coords 里, 有 4 个点落在 reviewer interior(非边框)。

### 根因
- `determineLabelLine()` 当前会把 chosenLine 的中点列 `middleX` 扩宽到 `lenLabel + 2`, 以便放下 label。
- 但 `columnWidth` 是“整列共享”的全局宽度:
  - 当 `middleX` 恰好落在某个 node 的 3x3 block 列(尤其是 node.gridCoord.x 顶点列)时,
  - 扩宽会触发 `gridToDrawingCoord()` 的 cell-center 平移, 让 node box 相对端口坐标系错位,
  - 结果: edge port 视觉上落入 box interior。

### 修复思路(优先低风险)
- relaxed + Unicode 时:
  - 如果 `middleX` 落在任意 node 的 3x3 block 列,
  - 就在 chosenLine 覆盖的 [minX..maxX] 里选择“最近的非 node block 列”来扩宽,
  - 让 label 仍有空间, 但不误伤 node 列。
