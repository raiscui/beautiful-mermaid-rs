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

## 2026-01-30 20:47 - 补充集成使用说明

- 更新 `README.md`：补齐“在其他 Rust 项目中集成”的说明
  - `Cargo.toml` 的 `path` / `git` 依赖示例
  - 主题（`THEMES`）的使用方式
  - Tokio/async 的 `spawn_blocking` 建议
  - 多线程与构建环境注意事项

## 2026-01-30 21:00 - 修复 CLI 输出末尾换行与 BrokenPipe

- 修复 `src/main.rs`：CLI 输出统一补齐末尾换行，避免 zsh 提示符 `%` 粘在输出末尾
- 兼容管道场景：stdout 写入遇到 BrokenPipe 时不 panic，按 Unix 习惯 0 退出
- 验证：`cargo test` 全通过；pipe 到 `head` 不再触发 Broken pipe panic

## 2026-02-01 00:41 - 补齐 CLI 自说明 + 生成 code agent 命令行用法文档

- 改良 `src/main.rs`：支持 `--help/-h`、`--version/-V`，并把参数解析前置（不再因空 stdin 触发 QuickJS 异常）
- 增加参数校验：未知参数 / 错误组合直接返回 exit code `2`，减少 agent 的“盲猜式重试”
- 新增文档：`docs/code-agent-cli.md`（给 code agent 的可复制命令范式、批处理与排错指南）
- 同步 `README.md`：CLI 示例改用 `beautiful-mermaid-rs`，并指向上述文档
- 验证：`cargo test` 通过；`beautiful-mermaid-rs --help/--version` 正常；SVG 渲染冒烟通过

## 2026-02-01 20:38 - 新增仓库贡献者指南（AGENTS.md）

- 新增 `AGENTS.md`：说明项目结构、关键命令、测试约定、提交/PR 规范（面向贡献者的一页纸指南）

## 2026-02-01 20:55 - 修复 Flowchart/State 的 Unicode 节点 ID（中文 ID）渲染异常

- 根因在上游 TS 版 `beautiful-mermaid`：Flowchart/State parser 用 `\\w`/`[\\w-]` 匹配 ID，中文被解析丢失，最终进入 dagre 空图布局导致 `-Infinity`
- 同步上游修复后的 browser bundle：
  - 从 `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/dist/beautiful-mermaid.browser.global.js`
  - 拷贝到 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- 新增 Rust 侧回归测试：`tests/unicode_id_smoke.rs`
- 验证：`cargo test` 通过；`graph TD\\n开始 --> 结束\\n` 的 SVG/ASCII 输出正常

## 2026-02-01 21:06 - 增加一键同步 vendor bundle 的脚本与 Make 目标

- 新增脚本：`scripts/sync-vendor-bundle.sh`（TS 侧 `bun run build` → 拷贝到 Rust vendor → 可选 `cargo test` 验证）
- Makefile 增加目标：
  - `make sync-vendor`：只同步，不跑 Rust 测试
  - `make sync-vendor-verify`：同步 + `cargo test`（推荐）
- 文档同步：`README.md` 补充“同步上游 bundle（开发者）”说明
- 验证：`make sync-vendor-verify` 执行成功

## 2026-02-01 22:12 - 同步上游“宽字符宽度”修复，解决中文边框错位

- 上游 TS 版已修复：中文/emoji 等宽字符在 ASCII/Unicode 渲染里不再把边框“顶出去”
- 本仓库同步最新 browser bundle：`vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- Rust 侧补充回归断言：`tests/unicode_id_smoke.rs` 额外校验“每一行终端显示宽度一致”
- 验证：
  - `cargo test` ✅
  - `graph TD\\n开始 --> 结束\\n` 的 ASCII 输出边框对齐 ✅

## 2026-02-02 00:46 - 将 `sync-vendor-verify` 集成进 `make install`

- 改良 `Makefile`：`make install` 先执行 `make sync-vendor-verify`（同步上游 TS bundle + `cargo test`），再做 `cargo build --release` 并安装到 `INSTALL_DIR`
- 验证：`make install INSTALL_DIR=/tmp/beautiful-mermaid-rs-install` 执行成功（tsup build + Rust tests + release build + copy）

## 2026-02-02 16:25 - 同步 vendor bundle 后更新 golden testdata（对齐最新渲染输出）

- 背景：`make install` 执行 `sync-vendor-verify` 时，ASCII/Unicode 的 golden 输出对比失败（渲染布局变更）。
- 更新 golden files（期望输出对齐最新 vendor bundle）：
  - `tests/testdata/ascii/ampersand_lhs_and_rhs.txt`
  - `tests/testdata/ascii/preserve_order_of_definition.txt`
  - `tests/testdata/ascii/self_reference_with_edge.txt`
  - 以及对应的 `tests/testdata/unicode/*.txt`
- 验证：
  - `cargo test` 全通过
  - `make install` 端到端通过（tsup build → sync vendor → cargo test → release build → install）

## 2026-02-02 21:24 - TS bundle 再次变更后修复 golden，并增加 `UPDATE_GOLDEN` 更新模式

- 背景：上游 TS 仓库本次构建产物更新（vendor sha256 变为 `18ac06ce...`），导致多个 ASCII/Unicode golden 输出发生变化，`make install` 再次被拦截。
- 更新 golden files（对齐最新 vendor bundle 输出）：
  - `tests/testdata/ascii/ampersand_lhs_and_rhs.txt`
  - `tests/testdata/ascii/cls_all_relationships.txt`
  - `tests/testdata/ascii/er_identifying.txt`
  - `tests/testdata/ascii/preserve_order_of_definition.txt`
  - `tests/testdata/ascii/self_reference_with_edge.txt`
  - 以及对应的 `tests/testdata/unicode/*.txt`
- 改良测试体验：`tests/ascii_testdata.rs` 增加 `UPDATE_GOLDEN=1` 模式
  - mismatch 时自动把渲染结果写回 golden 文件
  - 更新完成后 panic 提示“重新运行测试确认稳定”（避免 silent 更新导致误判）
- 新增 `.envrc`：提供 `UPDATE_GOLDEN=0` 默认值与说明（配合 direnv 使用）
- 验证：
  - `cargo test` 全通过
  - `make install` 端到端通过（tsup build → sync vendor → cargo test → release build → install）

## 2026-02-03 00:18 - Native pathfinder：把 A* 热循环从 JS 挪到 Rust（显著加速 CLI）

- 背景：
  - `beautiful-mermaid-rs` 的 CLI 在 ASCII/Unicode 渲染 Flowchart/State 时，会频繁运行 A* 路由。
  - 在 QuickJS（无 JIT）里跑 A* 的 heap pop + 4 邻居扩展属于典型热循环，遇到复杂用例会明显变慢。
- 方案：
  - 新增 `src/native_pathfinder.rs`：用 Rust 实现 A*（含 strict 约束版本），并复用内部大数组（stamp 技巧）减少分配/清空成本。
  - 在 `src/js.rs` 里通过 `rquickjs` 注册全局函数：
    - `globalThis.__bm_getPath(...)`
    - `globalThis.__bm_getPathStrict(...)`
  - TS bundle 会在运行时检测这两个函数；存在则走 native（Rust）实现，否则回退纯 JS 版本。
- 配套改良：
  - `.gitignore` 增加 `.DS_Store`，并从仓库移除已被误提交的 `.DS_Store`（避免无意义的噪音变更）。
- 验证：
  - `cargo test` 全通过（包含 ASCII/Unicode golden + SVG smoke + unicode id smoke）。
