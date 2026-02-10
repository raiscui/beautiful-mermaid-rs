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

## 2026-02-09 - 四文件摘要(continuous-learning 触发: task_plan 超 1000 行续档)

- 任务目标(task_plan):
  - Rust 侧复刻 TS `beautiful-mermaid` 的 ASCII/Unicode 渲染,并在 CLI 中可用且可回归。
  - 用户复现图(“Hat workflow”)要求更可读: 最近侧边出入线,少外圈绕行,少误连线 junction。
- 关键决定(task_plan):
  - 保持路线B(内嵌 JS bundle)作为基线,并在 QuickJS 场景用 native pathfinder 保证性能。
  - 当输出质量回退时,优先做“语义对齐”(native 与 TS 完全一致),而不是盲目堆惩罚/参数。
- 关键发现(notes/ERRORFIX):
  - QuickJS 无 JIT: 纯 JS A* 在复杂图上会慢到不可用,因此 native 默认必须开启。
  - meta 端点语义的“隐性回归”很危险: 文本看起来没坏,但 `path.last()` 不再是箭头格会破坏消费方(UI 动画/上色)。
  - native relaxed 一旦把 TS 的 usedPoints hard rule 改成 penalty,会显著增加 junction,导致输出灾难。
- 实际变更(WORKLOG/ERRORFIX):
  - TS 上游:
    - `src/ascii/draw.ts`: 修复 `computeEdgeStrokeCoords()` 的 arrowPos 去重顺序(`pushUniqueLast`)并对齐 `drawArrowHead()` dir 推断。
    - `src/__tests__/ascii-with-meta-roundtrip.test.ts`: 增加 Unicode relaxed 回归,锁死“箭头坐标=meta.last 且贴边”的不变量。
    - 多处 ASCII 路由/绘制改良(最近侧边偏好,comb ports 扩容/单端口 nudge,crossing 拐点保护,label 避让)。
  - Rust 本仓库:
    - `src/native_pathfinder.rs`: 把 `get_path_relaxed` 的 usedPoints 规则改回与 TS 一致(避免走进占用点导致 junction)。
    - `src/js.rs` + `.envrc`: 增加 `BM_DISABLE_NATIVE_PATHFINDER` 对照开关(默认 0)。
    - `examples/debug_user_case_meta.rs`: 输出文本+meta(first/last),便于定量定位端点/贴边不变量。
    - 新增/更新 golden,并用 `cargo test --release` 验证。
- 错误与根因(ERRORFIX):
  - meta.last 在 box 内: TS `pushUnique` 导致 arrowPos 不在末尾。
  - Rust 输出崩坏: native relaxed usedPoints 语义偏离 TS,导致 A* 选择进入占用点,制造大量 junction。
- 可复用点候选(后续可沉淀):
  1) 任何“看起来只是布局变丑”的回归,先用 TS(bun) vs Rust(CLI) 对照 + diff 定位是否是 native 语义漂移。
  2) 对 meta 语义必须有硬回归: `path.last()` 贴边且字符为箭头,否则 UI 侧必坏。
  3) relaxed 路由的可读性目标优先级: 端口朝向正确 > 避免外圈绕行 > 再谈更少拐点。


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

## 2026-02-09 18:20 - 终端可读性: 多入边汇聚的 flowchart 在 ASCII/Unicode 下天然会“线团化”

### 现象(用户最新反馈)
- 用户的图是典型的“协调者节点(Ralph)收敛多个事件/结果”的结构:
  - 多条边都指向 Ralph,并且同一 source(Integrator)有多条不同 label 的边指回 Ralph。
- 在 flowchart(无论 TD 还是 LR) 的 ASCII/Unicode 渲染里:
  - 会出现大量 `┬/┴/├/┤/┼` 汇聚点,视觉上像“误连线”。
  - label 为了避让 node,经常被迫走外圈,形成“大矩形框”,进一步加剧阅读负担。

### 综合结论(更像“表达方式选择”而不是“算法还能再调一点”)
- 对这类“多消息往返 + 分支结果回到协调者”的逻辑,`sequenceDiagram` 是更适合终端阅读的表达:
  - 没有多入边的线合并,而是“泳道 + 水平消息箭头”,读起来像日志/时序。
- 实测(用本仓库 `beautiful-mermaid-rs --ascii`):
  - 同一逻辑改写成 `sequenceDiagram` 后,输出明显更清晰。
  - `Note over ...` 在当前 ASCII 输出里不会显示(被忽略),因此“legend”要用更短的 participant/label 直接表达,或把 legend 放在 Mermaid 外的普通 Markdown 文本里。

## 综合发现

- 这类 root 误判会带来两个直接后果:

## 2026-02-09 19:18 - Flowchart 终端可读性: 路由顺序是“外框(detour)”的一等根因

- 复现结论:
  - 在 Unicode relaxed + “禁止 segment overlap”的 hard rule 下,edge routing 的顺序会直接决定:
    - 哪条边拿到内圈直通通道,
    - 哪条边被迫绕到画布最外圈,形成看起来像 subgraph 的“大外框”。
  - 用户复现图里,`Hat_ralph -->|integration.task| Hat_experiment_integrator` 写在 Mermaid 文本最后,
    会稳定触发“外框”。

- 可复用的改良策略(不新增 CLI 参数):
  - 不再死守 insertion order,而是对 Unicode relaxed 启用:
    - spanning forest(生成树主干边)优先路由,
    - 回边/补充边后路由。
  - 直觉收益:
    - 主干更直更短,反馈边绕行也更像“围绕主干”,不会把整图框起来。
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

## 2026-02-08 02:33:08 - Flowchart TD 箭头贴边错位排查结论

### 现象复现
- 用户复现图中,多条指向 `Hat_ralph` 的边显示为“箭头与 box 脱离”,特别是 `integration.applied` / `integration.blocked`。
- 复现命令:
  - `beautiful-mermaid-rs --ascii < /tmp/repro_user_case.mmd`

### 关键证据
- TS 调试显示 `edge.path` 的终点确实落在目标端口网格,但绘制坐标层出现偏移:
  - `columnWidth[2]` 被 label 逻辑扩宽到 `17`。
  - `drawBox` 仅使用节点前两列绘制 box,导致 box 本体不随第3列扩宽移动。
  - `drawArrowHead` 仍把箭头落在“末段 lastPos”,于是箭头可能停在远离 box 的位置。

### 根因本质
- 这是“布局列宽变化”和“箭头落点语义”不一致导致的显示层错位。
- 简言之: 端口列可以被扩宽,但箭头仍按旧末端语义绘制,没有重新锚定到目标 box 附近。

### 修复策略与取舍
- 采用最小风险修复: 只改 `draw.ts` 绘制层,不改 `edge-routing.ts` 路由/选线。
- 具体措施:
  1. 按目标 node box 实际坐标计算箭头新落点(贴边外一格)。
  2. 若新落点与旧末端有距离,补“桥接线”保持连续。
  3. 同步更新 meta 与 label 避让中的箭头坐标,防止渲染与元数据分叉。

### 验证
- TS 侧回归通过:
  - `bun test src/__tests__/ascii-relaxed-routing.test.ts src/__tests__/ascii-label-avoid-junction.test.ts src/__tests__/ascii-no-collinear-overlap.test.ts src/__tests__/unicode-relaxed-no-collinear-overlap.test.ts`
- Rust 侧全量通过:
  - `cargo test`
- 端到端复现确认:
  - 修复后 `beautiful-mermaid-rs --ascii` 输出中,指向 `Hat_ralph` 的箭头已贴边。

## 2026-02-08 03:09:16 - 二次修复: source 侧出边仍悬空

### 新现象
- 用户二次反馈: “还是有箭头指向不到”。
- 具体是 source 侧出边(`integration.task`、`experiment.task`)在 `Hat_ralph` 右侧/下侧仍可能出现“线头离开 box”。

### 新根因
- 第一轮仅修复了 target 箭头锚定(`drawArrowHead`)。
- source marker 仍沿用旧语义,在端口列扩宽后会漂移到 box 外侧远处。

### 二次修复
- `drawBoxStart` 改为直接根据 source box 边框锚定 marker。
- 给 marker 与 fallback 端点之间补桥接线(`drawEndpointBridge`),避免断裂。
- `computeBoxStartPositionNearSourceBox` 与 `computeArrowHeadPositionNearTargetBox` 增加 clamp,确保锚点不越过 box 边界范围。
- `computeEdgeStrokeCoords` 与 `computeBoxStartPosForLabelAvoid` 同步使用新锚点语义。

### 结果
- source/target 两侧都能保证“端点贴边 + 连续线段”。
- 用户复现图下不再出现“指向不到”的视觉断裂。

## 2026-02-08 11:33:04 - 修复 experiment.task “游离箭头”: endpoint bridge 支持 L 型 + 竖向箭头 stem

### 现象(用户反馈)
- 用户复现图中,`Hat_ralph -> Hat_experiment_runner (experiment.task)` 在“实验执行器”上方出现 `▼` 游离箭头。
- 具体可验证特征:
  - 箭头上一格是空格(没有 `│/┌/┐/┬/┼/...` 等竖向笔画),看起来像断线。

### 根因(本质层)
- 箭头位置会被 `computeArrowHeadPositionNearTargetBox()` clamp 到 target box 外侧一格,以保证“贴边”。
- 当 clamp 后的 `arrowPos` 与末段 `lastPos` 出现“同一行不同列”的偏移时:
  - 旧 `drawEndpointBridge()` 仅支持“同轴桥接”(同 y 的水平桥接或同 x 的竖直桥接)。
  - 结果只能补出一段水平线,但箭头仍是竖向(▼/▲),
  - 视觉上就会变成“水平线的尽头挂了一个竖向箭头”,读起来就是游离箭头/断线。

### 修复(绘制层最小改动)
- `drawEndpointBridge()` 增强:
  - 支持 L 型桥接。
  - 对 `dir=Up/Down` 且 `from.y === to.y` 的特例,插入 1 格 “stem”:
    - 先向入边方向移动 1 格(`stemY`),
    - 再水平对齐,
    - 再竖直进入箭头,
    - 从而保证箭头入边方向一定存在竖向笔画。
  - 桥接拐点写入正确的 box-drawing corner 字符,保证读图连续。

### 性能优化(确定性收益,低风险)
- `drawArrow()` 内原本会提前生成 `labelCanvas = drawArrowLabel(...)`。
- 但 `drawGraph()` 已经改为:
  - 先合成 line/corner/arrow/boxStart,
  - 再统一生成 label layer 并做 junction/corner/arrow 避让。
- 因此 `drawArrow()` 内的早期 labelCanvas 不再被使用,属于纯浪费。
- 优化做法:
  - `drawArrow()` 直接返回空 label canvas,减少每条 edge 一次 label 布局计算 + 一次大 canvas 拷贝。

### Rust 回归(避免再退化)
- 新增 `tests/ascii_user_case_edge_endpoint_invariants.rs`:
  - 通过 meta 锁定 `experiment.task` 的 arrow cell。
  - 在最终文本上构建“终端 cell 网格”,验证 arrow 上方存在竖向笔画。
  - 为兼容 emoji 宽字符,测试使用 `unicode-width` 按显示宽度展开 cell 网格。

## 2026-02-08 13:54:03 - relaxed 路由“右侧外圈大矩形”(绕路)修复记录

### 现象(用户反馈)
- 你反馈 `experiment.result` 看起来“绕了个圈”。
- 我本地复现并用 meta 量化后发现:
  - `experiment.result` 自身并不算绕远(它的 bbox max_x 只有 61 左右,turns=3)。
  - 误读主要来自 integrator 相关边在右侧画了一个很大的外圈矩形。
    - 典型边: `integrator -> Hat_ralph (integration.rejected)`、`integrator -> Complete (experiment.complete)`。

### 量化证据(来自 Rust 测试的 meta debug 输出)
- 旧输出(修复前)里,右侧外圈的典型证据是:

  > [debug] wide edge: Hat_experiment_integrator -> Hat_ralph (integration.rejected) bbox=(23,7)-(110,23) len=130
  > [debug] wide edge: Hat_experiment_integrator -> Complete (experiment.complete) bbox=(25,24)-(98,36) len=113

- 新输出(修复后)里,外圈明显收敛:

  > [debug] wide edge: Hat_experiment_integrator -> Hat_ralph (integration.rejected) bbox=(23,7)-(90,27) len=109
  > [debug] wide edge: Hat_experiment_integrator -> Complete (experiment.complete) bbox=(25,23)-(88,36) len=82

### 根因(本质)
- relaxed 模式下,候选路线的 cost 主要关注:
  - 步长/避让惩罚(避免交叉、避免不允许的共线复用)。
  - 拐点数量(避免锯齿)。
- 但“大外圈矩形”这种路线有一个特点:
  - 拐点很少(往往只有 2~3 次转向)。
  - 于是会被误判为“更优雅”的路径,即使它把图拉得很宽。

### 修复策略(路线A: 不扩大 grid,先改候选排序)
- 在 TS 侧 `src/ascii/edge-routing.ts` 的 `candidateCostRelaxed()` 增加 soft 的 detour 惩罚:
  - 用 from/to 节点的 3x3 block 包围盒做参考框(不受端口选择影响)。
  - 计算路径 bbox 超出参考框的“偏离量 detour”(只统计超过 margin 的部分)。
  - 仅当 detour 很大时才惩罚(阈值 THRESHOLD=12),避免影响大多数正常图。
  - 惩罚是软的: 仍允许绕远,但当存在更贴近图中心的候选时,优先选后者。
- 然后用 `scripts/sync-vendor-bundle.sh` 把新的 bundle 同步回 Rust 仓库,确保 CLI 生效。

### 风险与影响面控制
- detour 惩罚只在“detour 很大”时生效,因此对大量既有图的影响面相对可控。
- 不增加 A* 调用次数(仍是同一批候选做比较),性能影响主要是一次 O(path) 的 bbox 扫描,可控。

## 2026-02-08 16:24:22 - 最近侧边端口优先 + Unicode relaxed label 避让修复记录

### 新现象(用户反馈/测试)
- 你继续反馈:
  - 存在断线/绕路,并强调要“按 box 与 box 最近的边出线/入线”。
  - 拥挤节点需要按需增加 lane/margin。
- Rust 回归测试(用户复现图)出现新失败:
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`
  - `integrator -> Complete (experiment.complete)` 的 `extra_right=11`(阈值 `<=10`)。
- 量化证据(测试 debug 输出):

  > [debug] experiment.complete(to Complete) meta: extra_right=11, edge_max_x=81, max_node_right=70, len=85, manhattan=54, turns=4, arrow_char='◄', bbox=(25,23)-(81,36) first=(70,23) ... head=[(70,23) ... (81,23)]

  关键点:
  - `first=(70,23)` 落在 source 的右边框上;
  - 起始段先从 `x=70` 走到 `x=81`,明显是“背向 target”。

### 根因(本质)
- relaxed 模式为了提升可达性,在 fallback/扩展候选阶段会尝试更多 startDir/endDir 组合。
- 但 cost 函数里:
  - 我们之前主要惩罚“整体外圈 detour”(bbox 很大时),
  - 没有足够强地惩罚“端口背向”(例如目标在左边却从右侧出线)。
- label 的“文字拼接/乱码”(例如 `iexperiment.taskked`)根因是:
  - label 绘制时各边互相看不到对方(最后一次性 merge),
  - 导致两个 label 落在同一行同一区间时发生覆盖与拼接。

### 修复策略与落点

#### 1) relaxed: 增加“最近侧边(朝向)软惩罚”(不增加 A* 次数)
- TypeScript 上游:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
- 新增 `nearestSidePenaltyRelaxed(candidate)` 并叠加到 `candidateCostRelaxed()`:
  - startDir 背向 target 时加惩罚;
  - endDir 背向 source 时加惩罚;
  - 当 |dx| 与 |dy| 明显不对称时,轻微偏好“占优轴”的端口(更接近你说的“最近边”)。
- 设计取舍:
  - 只改变候选排序,不增加 A* 调用次数;
  - 性能风险低,并且更符合“先确保最近边”的优先级。

#### 2) Unicode relaxed: label 互相避让,必要时允许不画 label(避免乱码/断线)
- TypeScript 上游:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
- 仅在 `routing=relaxed && useAscii=false` 启用:
  - label 逐边画入 `graph.canvas`,让后画 label 能看到先画 label;
  - 把“已有文本字符”(非线段字符)视为 forbidden cell,避免覆盖其它 label/node text;
  - 若线段范围内找不到合法位置,尝试放宽到全画布;仍不可行则不画 label,避免出现拼接字符串。

#### 3) 拥挤节点: comb ports 扩容时加入 breathing room(让 lane 更疏)
- TypeScript 上游:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
- `extraCapacityForPorts()`:
  - 端口数 > 3 时,按数量额外扩大 contentWidth/contentHeight(上限 4),
  - 让 offsets 分布更疏,减少拥挤处字符合并/误读。

### 验证
- `scripts/sync-vendor-bundle.sh` ✅
- `cargo test` ✅
- CLI 复现图:
  - `iexperiment.taskked` 消失(不再出现 label 拼接)。
  - `experiment.complete` 起始段不再先向右“背向走一截”。

### 2026-02-08 17:29:11 - box 边长不足 + 双箭头难读(Unicode crossing)续修

#### 1) ralph box 边长不足: corner port 未计入拥挤度
- 现象:
  - 你提出的最小规则:
    - 4 条边出线: 最少 2 角 + 4 边 = 6 个字符单位
    - 6 条边出线: 最少 2 角 + 6 边 = 8 个字符单位
  - 复现图里 `ralph#1 (coordinator)` 右侧入边多,但 box 仍只有 5 行(只有 3 个可用端口行),导致端口挤在一起。
- 证据(上游 TS debug):
  - 存在 corner port:
    - `Hat_experiment_integrator -> Hat_ralph (integration.rejected)` 的 `endDir=LowerRight`
    - `Hat_ralph -> Hat_experiment_integrator (integration.task)` 的 `endDir=UpperLeft`
  - 原 comb ports 统计函数 `dirToSide()` 只识别 Up/Down/Left/Right,corner port 不计入 counts,因此不会触发 box 扩容。
- 修复:
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
  - `dirToSide(d,node,other)`:
    - corner port 归属到最近侧边(按 |dx| vs |dy| 选择水平/竖直侧边),
    - 让 contentHeight/contentWidth 能按真实端口数扩容,从而满足你提出的最小边长要求。

#### 2) `◄────────────────────►` 难读: bridge 把拐点桥断
- 现象:
  - `complete` 与“结果审计员”之间出现“纯双箭头直线”,读者容易误以为存在一条双向边。
  - `experiment.result` 视觉上像绕圈,本质是拐点被桥化断开后,线路关系被误读。
- 根因:
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/canvas.ts`
  - `deambiguateUnicodeCrossings()` 会把 `┼` 桥化为 `─/│`:
    - 在“纯交叉”处很合理(避免误读为连接),
    - 但当 `┼` 恰好落在“边的拐点”上时,桥化会把拐点断开,导致线路关系被破坏。
- 修复:
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
    - 新增 `computeEdgeCornerArmMasks()` 提供“拐点需要保留的连通方向”。
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/index.ts`
    - 将拐点掩码传入桥化逻辑,并采用“后写覆盖(last wins)”避免多边共享路径时 OR 成 FULL_MASK。
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/canvas.ts`
    - 桥化遇到拐点 `┼` 时,优先降级成 `┬/┴/├/┤/┐/┘/┌/└`,保留拐点连通。

#### 3) 验证
- `cargo test` ✅
- `make install` ✅(已更新安装版 CLI)

### 2026-02-08 22:03:31

#### 4) 共享走线假象: 单端口 center lane 全局对齐导致 point overlap

- 现象(用户复现图):
  - `experiment.result` 与 `integrator -> Complete (experiment.complete)` 在画布中部仍会产生 `┴` 等 junction。
  - 视觉上像 `complete <-> auditor` 存在一条双向边(`◄──┴──►`),非常难读。
- 关键证据(可量化):
  - Rust meta(drawing 坐标) 里这两条边存在 **1 个 cell 重叠**:
    - overlap char=`┴`,coord=(52,34)
  - 修复后 overlap_cells=0,误读消失。
- 根因:
  - comb ports 对某个 side 只有 1 条边时,旧逻辑固定使用 center offset。
  - 多个节点的 center lane 在画布中部容易对齐,进而把两条无关边“接到同一个 junction cell”。
- 修复(改良胜过新增,不改 A* 结构):
  - TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
  - comb ports `assign()`:
    - 当 `list.length===1` 时,按 `side + kind(start/end)` 做 1 格确定性 nudge,
      打散 center lane,降低 point overlap 概率。
- 影响面:
  - Unicode relaxed 的多份 golden 输出会发生轻微 lane 偏移(已用 `UPDATE_GOLDEN=1` 同步更新)。

### 2026-02-09 12:04:24 - meta 端点不变量回归: “箭头已贴边,但 meta.last 在 box 内”

#### 现象(可复现 + 可量化)
- Rust `cargo test --release` 失败用例:
  - `tests/ascii_endpoint_alignment.rs`
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`
- 失败边(用户复现图):
  - `Hat_experiment_auditor -> Hat_ralph (experiment.reviewed)`:
    - box: `Hat_ralph x=8..48`
    - meta.last: `(41,19)`(落在 box 内部,不贴边)
    - 实际文本中该行存在 `│◄`(箭头在 x=49 贴边),说明“绘制正确但 meta 错位”。
  - `Hat_experiment_integrator -> Complete (experiment.complete)`:
    - box: `Complete x=8..48`
    - meta.last: `(41,47)`(落在 box 内部,不贴边)

#### 根因(本质层)
- 根因在上游 TS: `src/ascii/draw.ts` 的 `computeEdgeStrokeCoords()`。
- 关键点:
  - `computeEdgeStrokeCoords()` 用 `pushUnique` 去重,保留“第一次出现”的坐标。
  - 在 columnWidth/rowHeight 伸缩导致末段线段跨越较大 drawing 距离时:
    - arrowPos 可能先被当作“线段坐标”写入到 path 的较早位置;
    - 随后再 `pushUnique(arrowPos)` 时不会追加到末尾;
    - 结果: `edge.path.last()` 不是箭头 cell,端点不变量与 UI 动画都会出错。
- 额外触发因素:
  - 某些末段在 offsetFrom/offsetTo 下会退化为“单点 lastLine”,
    - drawArrowHead 会用 fallbackDir 决定箭头方向,
    - 但 meta 若直接用 edge.path[-2..] 推 dir,会与实际绘制的 dir 不一致。

#### 修复(结论)
- TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  1) `computeEdgeStrokeCoords()` 对齐 `drawArrowHead()` 的 dir/lastPos 推断:
     - 记录末段 lastLine(from/lastPos)与 fallbackDir。
  2) 新增 `pushUniqueLast()` 专用于 arrowPos:
     - 若已出现过,先移除旧位置,再 push 到末尾;
     - 保持“去重”同时保证 `path.last()` 稳定是箭头 cell。
- Rust: 通过 `scripts/sync-vendor-bundle.sh` 同步最新 bundle 到:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`

#### 影响与后续
- golden:
  - vendor rebuild 后,`tests/testdata/unicode/preserve_order_of_definition.txt` 输出发生变化,已更新对齐。
- 建议:
  - 保持 `tests/ascii_endpoint_alignment.rs` 与 `tests/ascii_user_case_edge_endpoint_invariants.rs` 作为 meta 的底座回归,
    一旦 meta 与文本再出现不一致,可以第一时间拦截。

### 2026-02-09 12:31:10 - 上游 TS 回归测试: 锁死 meta 端点语义(Unicode relaxed)

- 动机:
  - 本次回归的本质是“最终文本输出没坏,但 meta 语义坏了”(path.last 不再是箭头格子)。
  - 如果不在 TS 上游把这个语义锁死,后续任何对 `computeEdgeStrokeCoords()` 的“去重/遍历策略”改动,
    都可能再次引入同类隐性回归,让 UI 动画与回归测试失真。
- 落点(改良胜过新增):
  - 选择在既有测试文件追加用例,而不是新建一堆文件:
    - TS: `src/__tests__/ascii-with-meta-roundtrip.test.ts`
- 新增断言(用户复现图,Unicode + relaxed):
  - 对每条 edge:
    - `path.last()` 必须贴着 target box 外侧一格(坐标语义)。
    - `text` 在 `path.last()` 坐标处必须是箭头符号(最终文本语义)。
  - 为解决宽字符(emoji/中文)导致的“字符串索引 != cell 坐标”:
    - 测试里用 `charDisplayWidth()` 按“显示列宽”做坐标映射,稳定读取 (x,y) 对应字符。
- 运行与注意事项:
  - `bun test src/__tests__/ascii-with-meta-roundtrip.test.ts` ✅
  - 该图渲染耗时约 8s,超过 bun 默认 5s timeout,因此为该 `it(...)` 单独设置了 20s timeout。
- 额外观察(未扩 scope):
- ASCII strict 在该复现图上会出现布局不可达,导致 `meta.nodes` 为空。
  - 这更像是 strict 路由可达性/布局重试策略的问题,本轮先不在这里继续展开。

### 2026-02-09 13:15:10 - Rust CLI 输出崩坏: native relaxed pathfinder 语义偏差

#### 现象
- 你提供的命令:
  - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`
  - 输出变得非常混乱(线路穿 box,大量 junction,可读性显著下降)。

#### 关键证据(对照)
- 同一份 Mermaid:
  - TS(bun) 渲染输出与 Rust CLI 输出差异巨大。
- 我把两份输出写入并 diff:
  - `/tmp/bm_ts_unicode_relaxed.txt`
  - `/tmp/bm_rust_unicode_relaxed.txt`
- 结论:
  - Rust CLI 并不是“正常执行 TS bundle”,而是被 Rust 注入的 native pathfinder 改变了路径选择。

#### 根因(本质)
- Rust 的 `src/js.rs` 会向 QuickJS 注入:
  - `globalThis.__bm_getPath*`
  - bundle 检测到这些函数后会走 native A*。
- native `get_path_relaxed` 曾把 TS 的 usedPoints 规则从 hard forbid 改成了 soft penalty:
  - A* 更容易走进已占用点位,合成 `┬/┴/├/┤` 等强歧义 junction,
  - 最终字符画出现你看到的“灾难输出”。

#### 修复(结论)
- `src/native_pathfinder.rs`:
  - 把 usedPoints 处理改回 TS hard rule(只豁免起点第一步与终点前一步,并限制 arms 阈值)。
  - 移除点重叠 penalty(RELAXED_PENALTY_USED_POINT*)。
- `src/js.rs`:
  - 默认仍启用 native(保证速度),
  - 新增 `BM_DISABLE_NATIVE_PATHFINDER=1` 用于对照/排错。
- 回归加固:
  - 新增 golden: `tests/testdata/unicode/user_repro_case.txt` 锁死该复现图完整输出。
  - 同步更新受影响的 unicode golden: `tests/testdata/unicode/preserve_order_of_definition.txt`。

#### 验证
- Rust 输出与 TS(bun) 输出一致(仅差最后换行)。
- `cargo test --release` ✅

### 2026-02-09 17:38:20 - label 扩列策略过度: “只按缺口最小增量扩列”让画布宽度收敛

#### 现象
- 同一份 Mermaid 在 `beautiful-mermaid-rs --ascii` 下虽然能渲染,
  但画布被拉得很宽,并且更容易出现“像在画外框”的大矩形绕行。
- 这个问题在 edge label 较长,且存在多条平行边/回边时尤其明显(用户复现图属于这一类)。

#### 根因(本质)
- 上游 TS `determineLabelLine()` 目前的策略是:
  - 选定 `labelLine` 后,直接把某一整列 `columnWidth` 拉到 `labelWidth+2`。
- 但对“水平线段”来说,真正决定 label 是否放得下的是**线段的总宽度**(跨多列的 Σ columnWidth),
  而不是“某一列必须 >= labelWidth”。
- 因此这类“无条件扩列”会制造大量无意义空白:
  - 画布宽度膨胀;
  - detour 视觉成本被放大,更像外框。

#### 修复(结论)
- 上游 TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
  - `determineLabelLine()`:
    - 计算 `currentTotalWidth = calculateLineWidth(graph, chosenLine)`；
    - 只有当 `currentTotalWidth < labelWidth+2` 时,才对 `widenX` 做 `delta` 的最小增量扩列。
- Rust: 通过 `scripts/sync-vendor-bundle.sh` 同步最新 bundle 到:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`

#### 验证
- 用户复现图: 画布明显收敛,外框尺寸变小,整体更集中。
- `UPDATE_GOLDEN=1 cargo test --test ascii_testdata --release` 更新 2 个 golden:
  - `tests/testdata/ascii/subgraph_with_labels.txt`
  - `tests/testdata/unicode/user_repro_case.txt`
- `cargo test --release` ✅

#### 备注
- 我尝试过“改路由顺序”(forward-first / 延后回边),发现会把局部问题扩散成全局外框,
  并有引入 label 贴边框风险,因此已回滚,继续保持 insertion order 作为路由基线。

### 2026-02-09 20:58:00 - `integration.blocked` 视觉“多线段”问题分析

#### 现象
- 用户观察到 `integration.blocked` 在 Mermaid 文本里只定义了一次,
  但终端 Unicode 图里右侧靠近 `ralph` 的区域像有“很多条线”。

#### 复现与证据
- 单测同款复现图(`tests/ascii_user_case_edge_endpoint_invariants.rs`)里,实际定义是 4 条平行边都从 `Hat_experiment_integrator` 指向 `Hat_ralph`:
  - `experiment.complete`
  - `integration.applied`
  - `integration.blocked`
  - `integration.rejected`
- 运行命令:
  - `BM_DEBUG_WIDE_EDGES=1 cargo test --test ascii_user_case_edge_endpoint_invariants user_repro_case_all_edges_respect_endpoint_invariants -- --exact --nocapture`
- 调试输出显示上述 4 条边 bbox 和长度都很接近(`len=212`),说明它们在主干段高度重合,视觉上会形成“线束”。
- 进一步用 `debug_user_case_meta` 验证,4 条边终点分别落在 `ralph` 左边不同 y:
  - `integration.blocked` 终点是 `(30,23)`
  - 其它平行边终点在 `(30,21)/(30,22)/(30,25)`。
  - 这是 comb ports 分 lane 的结果,不是同一条边重复绘制。

#### 根因(实现层)
- Unicode 默认 routing 是 `relaxed`:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/index.ts`
  - `routing = useAscii ? "strict" : "relaxed"`
- relaxed 下对平行边有“同端点共享干线”设计:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
  - `recordPathSegments()` 用 `pairId(from,to)` 允许同 pair segment 复用。
- 绘制层又启用了 comb ports lane 偏移:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  - `startPortOffset*/endPortOffset*` 只在首末段应用,让平行边在端口处分离。

#### 结论
- `integration.blocked` 没有被重复画多次。
- 你看到的“很多条线”是 4 条平行边 + 1 条 `experiment.reviewed` 在同区域汇入造成的视觉聚团。
- 当前行为符合现有 relaxed 设计目标(减少不可达和外框),但可读性在高密度平行边场景仍有优化空间。

#### 可优化算法方向
- 方向A(不惜代价,最佳可读性):
  - 把“同端点多边”提升为一等对象:
    - 先路由一条 bundle 主干,
    - 再在 source/target 附近做短 fanout/fanin。
  - 优点: 明显减少线束噪声和标签冲突。
  - 代价: 需要重构 edge 数据结构与 label 布局策略。
- 方向B(先能用,后面再优雅):
  - 保持现架构,新增两类惩罚:
    1) 高密度平行边区域的 lane 聚团惩罚;
    2) 标签重叠风险惩罚触发局部 reroute。
  - 优点: 改动小,兼容当前 golden 体系。
  - 代价: 只能缓解,不能像分组路由那样根治。

### 2026-02-09 21:24:00 - 最佳方案已落地: bundle trunk + 标签纵向堆叠

#### 已实现内容
- 上游 TS `src/ascii/grid.ts`:
  - 在 Unicode relaxed 下启用 bundle trunk:
    - 同端点(`from,to`)多边先路由 leader,
    - follower 复用 leader 的 `path/startDir/endDir`,
    - 端口 offset 在 comb ports 阶段重新分配。
- 上游 TS `src/ascii/draw.ts`:
  - 引入 bundle 标签堆叠:
    - 按同端点分组;
    - 组内共享 `anchorY`;
    - 按 rank 用步长 2 做纵向分层;
    - 目标是“允许上下叠,避免横向拼接”。
- 上游 TS `src/ascii/pathfinder.ts`:
  - 修复运行时错误:
    - `segmentPair` / `segmentPairMulti` 未绑定局部变量导致 `ReferenceError`。

#### 效果确认
- 用户复现命令(`cargo run -- --ascii`)输出中:
  - `integration.applied` / `integration.blocked` / `integration.rejected` 已上下分层。
  - 不再出现 `integration.rejected integration.blocked` 横向拼接同一行。

#### 同步与验证
- 已执行:
  - `scripts/sync-vendor-bundle.sh --ts-dir /Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid --skip-rust-tests`
- Rust 侧验证:
  - `cargo test --test ascii_user_case_edge_endpoint_invariants` ✅
  - `cargo test --test ascii_testdata` (先失败后更新 golden) ✅
  - `cargo test` 全量 ✅
- Golden 更新:
  - `tests/testdata/unicode/user_repro_case.txt`

### 2026-02-10 00:55:00 - “左侧更近却没走左侧”二次诊断结论

#### 复现与证据
- 在 TS 上游用调试脚本复现(`bun /tmp/bm_grid.ts`):
  - `Hat_experiment_integrator -> Hat_ralph` 的 4 条平行边稳定选中 `start=(0,1), end=(2,1)`(左出右入)。
- 临时候选打印显示:
  - 并非“左入候选完全不存在”。
  - 左入候选在当前图下通常需要更长 detour(会绕到更外层),因此被成本函数压过。

#### 根因拆分
- 这类问题是“几何可达性 + 共享约束 + 策略顺序偏置”的叠加:
  1. `allowEndSegmentReuse=true/false` 两种策略在同一条边上会得到不同拓扑。
  2. 旧逻辑是“先试一种策略,只要有解就提前返回”,存在顺序偏置。
  3. 在垂直主导场景中,顺序偏置会放大“左右对穿”路径的稳定性。

#### 本轮改良(稳态)
- 上游 TS: `src/ascii/edge-routing.ts`
  - 保留原有两种 relaxed 策略求解(`allowEndSegmentReuse=false/true`)。
  - 从“顺序短路”改为“先分别求解,再比较质量”。
  - 在 Unicode relaxed + 垂直主导场景下新增窄范围决策:
    - 若复用方案是“左右对穿”,且非复用方案能消除对穿并且成本/长度仅略差,
      则优先非复用。

#### 为什么仍可能不是“走左边”
- 当“左入”需要显著更长路径时(明显更差),新规则会维持当前路径。
- 这属于有意设计: 避免为了“强行左入”制造更大的外框绕行。

#### 影响面评估
- 仅影响 Unicode relaxed 的窄场景。
- ASCII/strict 不受影响。
- golden 漂移仅 1 个用例(`backlink_from_top.txt`)。

### 2026-02-10 01:31:00 - 并线标签“禁止左右拼接”根因与方案

#### 根因
- 当前 bundle 标签虽然先做了 `anchorY + rank` 纵向分层,
  但真正落字时仍走 `drawTextOnLine()` 的通用避让:
  - 会在同一 `y` 上做 `startX` 最近可行搜索;
  - 于是并线标签会被横向平移,出现左右排开。

#### 改良策略
- 对“并线标签”(bundle 分组标签)启用专门模式:
  - 固定中心 `x`(仅做边界 clamp);
  - 冲突时只做 `y` 方向上下搜索;
  - Unicode relaxed 下若无合法位则不绘制,避免文字拼接。

#### 影响面
- 只在 Unicode relaxed + bundle 标签路径启用。
- ASCII/strict 与非 bundle 标签保持原行为。

### 2026-02-10 01:52:00 - 并线标签仍分散的二次根因

#### 现象复盘
- 即使开启 `verticalOnlyStack`, `integration.applied/blocked/rejected/experiment.complete` 仍出现横向分散。

#### 根因
- 纵向堆叠只共享了 `anchorY`,但没有共享 `x`:
  - `buildBundleStackedLabelLines()` 仍按每条边自己的 `baseLine(minX,maxX)` 生成 line;
  - `drawTextOnLine()` 以该 line 的 `middleX` 作为落字中心;
  - 结果是同组标签有不同中心 x,表现为左右离散。

#### 修复方向
- 同组标签共享 `anchorCenterX`(中位数中心),再交给 `verticalOnlyStack` 做 y 轴避让。
