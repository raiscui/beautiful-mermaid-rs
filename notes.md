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
