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
- 已补齐 CLI 的自说明能力：支持 `--help/-h`、`--version/-V`，并且在 stdin 为空时给出明确提示。
- 已新增 code agent 专用 CLI 说明：`docs/code-agent-cli.md`。
- 已同步 README 的 CLI 示例为 `beautiful-mermaid-rs ...`。
- 已将 `make sync-vendor-verify` 的动作集成进 `make install`：install 前固定同步 bundle 并做端到端验证。

## 进展日志

### 2026-01-30 14:05
- 已在参考仓库执行 `bun run build`，生成 `dist/beautiful-mermaid.browser.global.js`
- 已拷贝 JS bundle 到 `vendor/beautiful-mermaid/`，并同步根目录 `LICENSE`
- 已完成 Rust 侧封装：`rquickjs` 内嵌 QuickJS，调用 `beautifulMermaid.renderMermaid/renderMermaidAscii`
- 已复刻 TS 版 golden testdata（ASCII/Unicode）到 `tests/testdata/`
- 已实现 Rust 测试对齐 TS 的 whitespace normalize 逻辑，`cargo test` 全通过

### 2026-01-30 20:47
- 根据“在其他 Rust 程序里集成使用”的需求，补充 README 的集成说明（依赖方式、主题用法、Tokio/多线程注意事项）

### 2026-01-30 21:00
- 修复 CLI 输出末尾缺少换行：避免 zsh 把提示符 `%` 粘在渲染结果后面
- 顺手处理 stdout 的 BrokenPipe：在 pipe 场景下（下游提前关闭）不再 panic，而是按 Unix 习惯静默退出
- 验证：`cargo test` 通过；`printf ... | cargo run -- --ascii` 末尾带换行；`| head -n 1` 不再出现 Broken pipe panic

### 2026-02-01 00:35
- 新需求：已安装为系统 PATH 命令 `beautiful-mermaid-rs`，需要补一份“给 code agent 的 CLI 使用说明”。
- 计划：
  - 先以 `beautiful-mermaid-rs --help/--version` 作为事实来源，确认真实参数与默认行为。
  - 再补一份面向 agent 的“最小可用范式”（stdin/file、svg/ascii、输出目录、批处理）。
  - 最后写入项目文档，并用几条命令做冒烟验证（确保示例不跑偏）。

### 2026-02-01 00:41
- 改良 CLI：`src/main.rs` 增加 `--help/-h`、`--version/-V`，并把“参数解析”前置，避免空 stdin 直接触发 QuickJS 异常。
- 增加“强约束”提示：未知参数/错误组合直接返回 exit code `2`，让 agent 更容易定位问题。
- 新增文档：`docs/code-agent-cli.md`（面向 code agent 的可复制命令范式、批处理、排错）。
- 同步 README 的 CLI 示例为 `beautiful-mermaid-rs`，并链接到上述文档。
- 验证：`cargo test` 通过；`beautiful-mermaid-rs --help/--version` 正常；渲染 SVG 冒烟通过。

### 2026-02-01 20:38
- 新需求：为本仓库补齐 `AGENTS.md`（Repository Guidelines），作为贡献者指南。
- 计划：
  - 盘点仓库结构（`src/`/`tests/`/`vendor/`/`docs/`）与关键入口文件。
  - 总结构建/测试/运行命令（`cargo`/`make`）。
  - 从 `git log` 提取当前提交信息风格，给出可延续的约定。
  - 产出 200-400 words 的 `AGENTS.md`，内容以“可执行、可复现”为导向。
- 完成：
  - 已新增 `AGENTS.md`（贡献者指南）。
  - 验证：`cargo test` 通过。

### 2026-02-01 20:55
- 新需求：Flowchart/State 的节点 ID 使用中文/Unicode 时输出 `-Infinity` 或空白，需要修复到“真正支持 Unicode id”。
- 处理：
  - 已在上游 TS 项目修复 parser 的 ID 匹配（从 `\\w` 扩展到 Unicode 属性类），并重建 `dist/beautiful-mermaid.browser.global.js`
  - 已同步更新本仓库 vendor bundle：`vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - 已新增 Rust 侧回归测试：`tests/unicode_id_smoke.rs`
- 验证：
  - `cargo test` 通过
  - `printf 'graph TD\\n开始 --> 结束\\n' | beautiful-mermaid-rs` 输出 SVG 正常（不含 `-Infinity`）

### 2026-02-01 21:04
- 新需求：我建议补一个“一键同步 vendor bundle”的脚本/Makefile target，避免以后改了 TS 却忘记更新 Rust 的 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`。
- 计划：
  - 增加 `scripts/sync-vendor-bundle.sh`：自动 `bun run build`（TS 仓库）并拷贝产物到 Rust vendor
  - Makefile 增加 `sync-vendor` / `sync-vendor-verify` 目标，做到一条命令跑完同步 + 验证
  - 验证：跑一遍 `make sync-vendor-verify`，确认中文/Unicode ID 的测试能兜底

### 2026-02-01 22:12
- 新需求：ASCII/Unicode 输出在中文等“宽字符”场景下边框错位（终端显示宽度与字符串长度不一致）。
- 处理：
  - 已在上游 TS 项目修复宽字符显示宽度（简化版 wcwidth + 输出跳列），并通过 `bun test`。
  - 使用 `scripts/sync-vendor-bundle.sh` 重新构建并同步最新 `dist/beautiful-mermaid.browser.global.js` 到本仓库 `vendor/beautiful-mermaid/`。
  - Rust 侧补充回归断言：`tests/unicode_id_smoke.rs` 增加“每一行终端显示宽度一致”的检查，防止 vendor 回退。
- 验证：
  - `cargo test` 通过
  - `printf 'graph TD\\n开始 --> 结束\\n' | cargo run --quiet -- --ascii` 输出边框对齐（不再“右边框被顶出去”）

### 2026-02-02 00:46
- 新需求：把 `make sync-vendor-verify` 的动作集成进 `make install`。
  - 动机：安装是“交付给其他项目/脚本使用”的关键动作。
  - 如果 install 前忘了同步 vendor bundle，或没跑测试，问题会在更晚的地方爆炸，排查成本更高。
- 可选方案（这里先记录两条路，避免以后反复改来改去）：
  1. **严格模式（默认）**：`make install` 先执行 `sync-vendor-verify`（同步 bundle + `cargo test`），再 build release 并拷贝安装。
     - 优点：最稳，install 永远是“可用且已验证”的产物。
     - 缺点：install 变慢，且依赖 `bun` + TS 仓库存在。
  2. **可选开关模式**：`make install` 默认执行 `sync-vendor-verify`，但允许 `SKIP_VENDOR_VERIFY=1` 跳过（用于临时快装）。
     - 优点：兼顾速度与严谨。
     - 缺点：接口更多，容易被滥用导致“又回到忘记验证”的老问题。
- 决定：先落地方案 1（严格模式），把确定性打满；如果你后续明确需要“快装”，再补方案 2 的开关。
- 计划：
  - [x] 调整 Makefile：让 `install` 先跑 `sync-vendor-verify`，再跑 `release`
  - [x] 本地验证：跑一遍 `make install INSTALL_DIR=...`，确认顺序与失败行为正确
  - [x] 记录产出：同步更新 `WORKLOG.md`
