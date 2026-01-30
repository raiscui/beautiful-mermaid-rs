# 任务计划: 初始化 `beautiful-mermaid-rs`（复刻 TypeScript 版 beautiful-mermaid）

## 目标
在 `/Users/cuiluming/local_doc/l_dev/my/rust/beautiful-mermaid-rs` 得到一个 **Rust crate**，对外提供与 TypeScript 版 `beautiful-mermaid` 等价的能力：

- `render_mermaid(...) -> SVG string`
- `render_mermaid_ascii(...) -> ASCII/Unicode string`
- 主题（`THEMES` / `DEFAULTS`）与基础配色选项
- 支持图表类型：Flowchart/State、Sequence、Class、ER

并且能跑通自动化测试来证明行为一致。

## 阶段
- [x] 阶段1: 计划和设置
- [x] 阶段2: 研究/收集信息（对齐 TS 版行为 & 产物）
- [x] 阶段3: 执行/构建（代码与结构落地）
- [x] 阶段4: 审查和交付（测试、文档、收尾）

## 两种实现路线（需要先定）
1. **路线A（不惜代价 / 最佳方案）**：纯 Rust 逐模块移植（parser/layout/renderer/ascii/theme 全部 Rust 化）
   - 优点：最终依赖最少，性能与可控性最好
   - 缺点：工作量最大，短期很难“完整复刻到一致输出”

2. **路线B（先能用 / 后面再优雅）**：在 Rust 内嵌 JS 引擎 + 打包后的 `beautiful-mermaid` JS bundle
   - Rust 对外暴露同等 API；内部直接执行经过打包的 JS 逻辑
   - 优点：最快达成“完整复刻”（输出与 TS 版强一致），测试也更容易对齐
   - 缺点：引入 JS 引擎依赖（体积/复杂度上升），长期需要逐步替换为纯 Rust

> 我会先按 **路线B** 把“能用 + 行为一致”做出来，作为稳定基线。
> 后续如果你明确要路线A，我们再在此基线上逐步替换实现（保持 API 与测试不变）。

## 关键问题
1. “完整复刻”的验收标准是什么？
   - 我按 **输出一致（SVG/ASCII）+ 主要 API 与选项齐全 + 测试覆盖** 来做。
2. Rust 侧需要同时提供库（lib）与命令行（CLI）吗？
   - 我会先做库（lib），再补一个很薄的 CLI（可选）。

## 做出的决定
- [决定] 先实现路线B（嵌入 JS bundle）作为基线：更快、更稳，最符合“完整复刻”的短期目标。

## 遇到错误
- 暂无

## 状态
**已完成**
- 交付物：Rust crate + vendor bundle + 测试 + 文档
- 下一步：如果要“路线A（纯 Rust）”，建议先从 ASCII 引擎开始逐模块替换（保持测试不变）

## 进展日志

### 2026-01-30 14:05
- 已在参考仓库执行 `bun run build`，生成 `dist/beautiful-mermaid.browser.global.js`
- 已拷贝 JS bundle 到 `vendor/beautiful-mermaid/`，并同步根目录 `LICENSE`
- 已完成 Rust 侧封装：`rquickjs` 内嵌 QuickJS，调用 `beautifulMermaid.renderMermaid/renderMermaidAscii`
- 已复刻 TS 版 golden testdata（ASCII/Unicode）到 `tests/testdata/`
- 已实现 Rust 测试对齐 TS 的 whitespace normalize 逻辑，`cargo test` 全通过
