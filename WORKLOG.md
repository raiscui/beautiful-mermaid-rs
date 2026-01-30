# WORKLOG

> 这里记录每次任务的最终产出与关键动作（按时间追加）。

## 2026-01-30 14:05 - 初始化并复刻 TS 版 beautiful-mermaid（基线：嵌入 JS bundle）

- 在参考仓库构建 browser bundle（tsup iife），并拷贝到本仓库：`vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- Rust 侧完成封装：
  - `src/js.rs`：用 `rquickjs` 初始化 QuickJS，并调用 `beautifulMermaid.renderMermaid/renderMermaidAscii`
  - `src/lib.rs`：对外暴露 `render_mermaid` / `render_mermaid_ascii`
  - `src/theme.rs`：复刻 `DEFAULTS`/`THEMES`，并实现 `from_shiki_theme`
- 测试对齐：
  - 拷贝 golden files 到 `tests/testdata/{ascii,unicode}`
  - `tests/ascii_testdata.rs` 复刻 TS 的 whitespace normalize 规则
  - `tests/svg_smoke.rs` 做 SVG 冒烟测试
- 验证：`cargo test` 全通过
