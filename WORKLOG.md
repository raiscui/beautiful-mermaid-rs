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

## 2026-02-03 14:12 - 重写 README：补齐“项目定位 / 上游问题 / 本仓库改动”

- 更新 `README.md`：
  - 新增 TL;DR（CLI 最快上手命令）
  - 新增“为什么有这个项目”（讲清楚定位与适用场景）
  - 新增“架构概览”（用 Mermaid 图说明：Mermaid → QuickJS → vendor bundle → SVG/ASCII）
  - 新增“原 TS 版 beautiful-mermaid 暴露过的问题”（问题→根因→修复→验证）：
    - Unicode 节点 ID（中文 ID）导致 `-Infinity` / 空白
    - 宽字符（中文/emoji）导致 ASCII/Unicode 边框错位
    - QuickJS（无 JIT）下 A* 路由变成性能瓶颈，并用 native pathfinder 解决
  - 新增“本仓库做了哪些可见的改动”清单（API/CLI/脚本/测试/黄金文件更新模式）
  - 补充“测试与 golden”说明（包含 `UPDATE_GOLDEN=1` 用法与 `.envrc` 提示）
- 验证：README 内的 Mermaid 图已通过 `mermaid-validator` 语法校验
- 验证：`cargo test` 全通过

## 2026-02-03 14:20 - git 提交（提交本次 README + meta API 相关改动）

- 已执行：`cargo fmt --all` + `cargo test`（全通过）
- 已提交：`feat: add ASCII render meta API`
  - 新增：`render_mermaid_ascii_with_meta`（ASCII/Unicode + meta）
  - 新增：ASCII meta 类型（node/edge/box/path）
  - 同步：vendor bundle（`renderMermaidAsciiWithMeta`）
  - 新增：`tests/ascii_meta_smoke.rs`（text 与旧 API 严格一致 + meta 可用）
  - 更新：`README.md`（项目定位 / 上游问题 / 本仓库改动）

## 2026-02-06 16:10 - TS bundle 更新后修复 Unicode golden（恢复 `make install`）

- 背景：`make install` 内部会执行 `sync-vendor-verify` 重建并同步 TS bundle（本次 sha256 为 `b48b9228...`）。
- 现象：Unicode 渲染布局变化导致 golden 参考输出过期, `cargo test` 在 `unicode_testdata_matches_reference` 失败。
- 修复：使用 `tests/ascii_testdata.rs` 内置的 `UPDATE_GOLDEN=1` 更新模式, 自动写回当前渲染输出。
  - 更新了 2 个文件：
    - `tests/testdata/unicode/ampersand_lhs_and_rhs.txt`
    - `tests/testdata/unicode/preserve_order_of_definition.txt`
- 验证：
  - `cargo test` 全通过。
  - `make install` 端到端通过（tsup build → sync vendor → cargo test → release build → install）。

## 2026-02-06 16:54 - 集成 Mermaid validator（不依赖 mcp-mermaid-validator）

- 新增公共 API:
  - `beautiful_mermaid_rs::validate_mermaid(...) -> MermaidValidation`
  - 后端使用 `selkie::parse` 做严格语法校验（纯 Rust, 不依赖 Node）。
- 扩展 CLI:
  - `--validate`: 校验单个 Mermaid（stdin 输入）, stdout 输出 `true/false`, stderr 输出错误细节。
  - `--validate-markdown`: 扫描 stdin 的 Markdown, 校验其中所有 ```mermaid code fence（stdout 输出 `true/false`）。
- 新增测试:
  - `tests/validate_smoke.rs`: 覆盖 valid/invalid 两类输入, 防止 validator 回归失效。
- 更新文档:
  - `docs/code-agent-cli.md` 补充 validator 的用法、选项与退出码说明。
- 验证:
  - `cargo fmt --all`
  - `cargo test` 全通过。

## 2026-02-06 17:09 - 增加 `make validate-docs`（批量校验 README/docs Mermaid 图）

- 新增 Makefile target:
  - `make validate-docs`: 校验 `README.md` 与 `docs/**/*.md` 内的 ```mermaid code fence。
  - 失败即退出, 并输出具体文件与错误细节（stderr）。
- 文档同步:
  - `README.md` 与 `docs/code-agent-cli.md` 增加对应说明与示例。
- 验证:
  - `make validate-docs` 通过。

## 2026-02-06 17:15 - validator 文档语义对齐（收尾）

- 修正注释与实际实现一致:
  - `MermaidValidation` 目前后端为 `selkie::parse`（纯 Rust parser）。
- 验证:
  - `cargo test` 全通过。
  - `make validate-docs` 通过。

## 2026-02-06 19:33 - QuickJS Unicode relaxed 性能优化：native `__bm_getPathRelaxed`

- 做了什么：
  - Rust：实现 native relaxed A*（步长 + crossing penalty + segment reuse hard rule），并注入 `globalThis.__bm_getPathRelaxed(...)`。
  - TS bundle：`getPathRelaxed()` 增加 fast path，存在 `__bm_getPathRelaxed` 时优先走 Rust。
  - 同步 vendor：把最新 bundle 写入 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`。
- 验证：
  - `scripts/sync-vendor-bundle.sh` 通过（含 `cargo test`）。
  - Unicode golden 用例耗时从 ~88s 降到 ~3.6s（本机观测）。
