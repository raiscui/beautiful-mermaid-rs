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
