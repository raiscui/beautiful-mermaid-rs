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
