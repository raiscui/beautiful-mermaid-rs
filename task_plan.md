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

### 2026-02-02 16:25
- 现象：执行 `make install`（内部会跑 `sync-vendor-verify`）后，`tests/ascii_testdata.rs` 的 golden 输出对比失败。
  - 失败用例集中在“自环/多边合并”的布局：`ampersand_lhs_and_rhs`、`preserve_order_of_definition`、`self_reference_with_edge`。
- 判断：这是**上游 TS bundle 更新**导致 ASCII/Unicode 渲染布局有变化，属于“golden 参考输出过期”，不是 Rust 侧逻辑回退。
- 计划：
  - [x] 更新对应 golden files：让测试期望与最新 vendor bundle 对齐
  - [x] 验证：`cargo test` 全通过
  - [x] 验证：重新执行 `make install`，确认端到端（tsup build → sync vendor → cargo test → release install）完全恢复
- 状态：
  - 已完成：`make install` 端到端验证通过

### 2026-02-02 21:06
- 现象：你再次执行 `make install` 后，`tests/ascii_testdata.rs` 的 golden 输出对比再次失败。
  - 本次只暴露了 `ampersand_lhs_and_rhs` 的差异（ASCII/Unicode 都变了）。
  - 从输出看，上游 TS bundle 也确实变了（例如 IIFE 体积从 143.65KB 变到 147.29KB，sha256 变为 `18ac06ce...`）。
- 两种处理路线（先摆出来，避免“修一次又来一次”）：
  1. **路线A（不惜代价 / 最佳方案）**：固定上游 TS 版本（pin 到某个 commit/tag），同步脚本只在版本变更时才更新 vendor，并配套更新 golden。
     - 优点：golden 不会被“上游随手改动”频繁打断，CI/安装更稳定。
     - 缺点：需要你接受“上游改了但我们不立刻跟”的节奏管理。
  2. **路线B（先能用 / 立即恢复）**：继续按 golden 的定义办事：既然 vendor 更新了，就更新 golden 期望输出对齐新 bundle。
     - 优点：最快恢复 `make install`，符合当前工作流（install 前强校验）。
     - 缺点：上游只要一改布局算法，就可能再次需要更新 golden。
- 决定（本次先按你当前诉求）：先走路线B，把测试修到通过，保证 `make install` 恢复。
- 计划：
  - [x] 重新确认 vendor bundle 已更新为本次构建产物（sha256 `18ac06ce...`）
  - [x] 跑 `cargo test --test ascii_testdata`，枚举所有 mismatch 的 golden 文件
  - [x] 更新所有受影响的 `tests/testdata/{ascii,unicode}/*.txt` 期望输出
  - [x] 验证：`cargo test` 全通过
  - [x] 验证：`make install` 端到端通过
- 状态：
  - 已完成：golden 已对齐最新 vendor bundle，`make install` 恢复可用
  - 额外改良：`tests/ascii_testdata.rs` 新增 `UPDATE_GOLDEN=1` 自动更新 golden 的模式，避免下次再手工改一堆文件
  - 观察：`preserve_order_of_definition` 这一类“自环/循环边”案例在本次 vendor bundle 下渲染明显变慢，导致 `cargo test`（尤其是 ascii golden）耗时大幅上升

### 2026-02-03 00:07
- 新需求：git 提交（把当前工作区变更提交到仓库）。
- 动机：
  - 让本次“native pathfinder 加速 + golden 更新 + agent/openspec 配置”等改动具备可追溯性。
  - 避免工作区长期漂移，后续排查问题会变得困难。
- 计划：
  - [x] 清理不该入库的文件（重点：`.DS_Store`），并补齐 `.gitignore`
  - [x] 补齐缺失的新增文件纳入版本控制（例如 `src/native_pathfinder.rs`）
  - [x] 本地验证：`cargo fmt --all` + `cargo test`（有 error/warn 就当作失败处理）
  - [x] 组织提交：按 conventional commits 写 commit message，并完成提交
  - [x] 记录产出：更新 `WORKLOG.md`（必要时补充 `notes.md` / `ERRORFIX.md`）

### 2026-02-03 14:08
- 新需求：重写 `README.md`，介绍本项目；并把“原 TS 版 beautiful-mermaid 存在的问题”与“本项目做的变动”写清楚。
- 动机：
  - README 是仓库入口，决定了读者的第一印象与理解成本。
  - 当前 README 偏“用法速查”，但缺少：项目定位、与上游关系、修复了什么问题、为什么要这样实现。
- 两种写法路线（先摆出来，避免写到一半又推翻）：
  1. **路线A（不惜代价 / 最佳方案）**：README 同时面向使用者与维护者，包含：
     - 项目定位与适用场景
     - 与上游 TS 的关系（vendor bundle、同步方式、许可证）
     - “问题 → 根因 → 修复 → 验证”清单（可长期维护）
     - 关键工程化改动（测试、脚本、CLI 约定、性能加速）
  2. **路线B（先能用 / 后面再优雅）**：README 只保留快速上手，细节挪到 `docs/`。
     - 优点：更短
     - 缺点：读者很难理解“为什么存在”和“到底改了什么”，不符合本次诉求
- 决定：采用路线A（你明确要求“把问题与变动写清楚”，不能只写简版）。
- 计划：
  - [x] 从 `WORKLOG.md` / `task_plan.md` / `notes.md` 提炼“上游问题 + 修复点”的事实，避免写成口号
  - [x] 重写 README 结构：先“是什么/为什么”，再“怎么用”，最后“维护/同步/测试/排错”
  - [x] 如 README 中包含 Mermaid 图，使用 `mermaid-validator` 校验语法
  - [x] 完成后把本次动作记录到 `WORKLOG.md`
- 状态：
  - 已完成：`README.md` 已补齐“项目定位 / 架构概览 / 上游问题与修复 / 本仓库改动清单”
  - 已完成：README 内 Mermaid 图已通过 `mermaid-validator` 语法校验
  - 已验证：`cargo test` 全通过

### 2026-02-03 14:17
- 新需求：git 提交（提交本次 README 重写相关改动）。
- 动机：
  - 让本次文档与工作记录进入可追溯的提交历史，避免工作区长期漂移。
  - 让其他机器/CI 拉取后即可复现当前 README 与文档描述。
- 计划：
  - [x] 查看工作区变更：`git status` / `git diff`（确认没有误改/大文件）
  - [x] 检查是否存在 submodule 变更（如有则一并提交）
  - [x] 运行格式化与测试：`cargo fmt --all` + `cargo test`（有 error/warn 视为失败）
  - [x] 组织提交：实际提交为 `feat:`（因为包含 Rust API 新增：ASCII meta）
  - [x] 提交后补写工作记录：把 commit 行为追加到 `WORKLOG.md`
- 状态：
  - 已完成：已提交本次改动（README 重写 + `render_mermaid_ascii_with_meta` + vendor 同步 + 测试）

### 2026-02-06 16:00
- 现象：你执行 `make install` 时，端到端验证阶段 `cargo test` 失败。
  - 失败用例：`tests/testdata/unicode/ampersand_lhs_and_rhs.txt`
  - 报错点：`tests/ascii_testdata.rs` 的 `unicode_testdata_matches_reference`
  - 判断：上游 TS bundle 更新后，Unicode 渲染布局发生变化，导致 golden 参考输出过期。
- 两种处理路线（先摆出来，避免修完立刻又遇到同类问题）：
  1. **路线A（不惜代价 / 最佳方案）**：pin 上游 TS 仓库的 commit/tag，并把该版本信息写进同步脚本输出或仓库文档。
     - 优点：golden 更稳定，安装流程更少被打断。
     - 缺点：需要接受“上游变更不立刻跟进”的节奏管理。
  2. **路线B（先能用 / 立即恢复）**：接受最新 bundle 的输出，并更新 golden files 对齐当前渲染结果。
     - 优点：最快恢复 `make install` 的可用性。
     - 缺点：上游布局算法再变时，golden 仍可能需要更新。
- 决定（按你这次的诉求）：先走路线B, 把测试修到通过, 让 `make install` 立即恢复。
- 计划：
  - [x] 使用 `UPDATE_GOLDEN=1` 模式自动更新过期 golden（避免手改拷贝字符出错）
  - [x] 重新运行 `cargo test` 确认全绿
  - [x] 重新运行 `make install` 做端到端验证
- 状态：
  - 已完成：已更新 2 个 Unicode golden（`ampersand_lhs_and_rhs`、`preserve_order_of_definition`），并验证 `cargo test`、`make install` 全通过。

### 2026-02-06 16:39
- 新需求：直接集成 `/Users/cuiluming/local_doc/l_dev/my/rust/mcp-mermaid-validator` 的 Mermaid validator 功能到本仓库，但不引入 `mcp-mermaid-validator` 这个依赖。
- 动机：
  - 目前 Mermaid 语法校验依赖外部 MCP server，不够“就地自包含”，也不利于离线/CI 场景复用。
  - 希望把“语法校验能力”变成 Rust API/CLI 的一等公民，并能在纯 Rust 环境跑通（不依赖 Node）。
- 两种实现路线（先摆出来，避免做到一半才发现方向不对）：
  1. **路线A（不惜代价 / 最佳方案）**：引入纯 Rust 的 Mermaid parser 作为 validator（选型：`selkie-rs`，目标是 mermaid.js 的 Rust port）：
     - 核心策略：只做 parse/validate，不做渲染输出；返回稳定的 `true/false + error/details` 结果模型。
     - 优点：无 Node 运行时依赖；语法校验更严格，覆盖的 Mermaid 类型也更广；适合 CI/离线使用。
     - 缺点：新增 Rust 依赖；与官方 Mermaid CLI 的行为可能存在少量差异。
  2. **路线B（先能用 / 行为最对齐）**：在 Rust 里复刻 MCP 版本的做法，spawn `npx @mermaid-js/mermaid-cli` 进行校验。
     - 优点：与 MCP 工具行为更一致，更接近“官方 Mermaid”语义。
     - 缺点：运行时依赖 Node/npx/Chromium，速度与稳定性都更不可控，不符合本仓库“运行时不依赖 Node”的定位。
- 决定：实现路线A（纯 Rust validator）。如果后续你确实需要“严格对齐官方 Mermaid CLI”的校验，再把路线B 作为可选后端补上。
- 计划：
  - [x] 阶段1: 研究并记录 MCP validator 的输入/输出/错误信息模型（写入 `notes.md`）。
  - [x] 阶段2: 在 lib 增加 `validate_mermaid` API + `MermaidValidation` 类型（实现为纯 Rust parser 校验）。
  - [x] 阶段3: 扩展 CLI: `--validate`（校验单图）与 `--validate-markdown`（扫描 Markdown 中 ```mermaid 代码块）。
  - [x] 阶段4: 增加回归测试 + 更新文档 + `cargo fmt --all` + `cargo test` 验证交付。
- 状态：
  - 已完成：新增 `validate_mermaid(...) -> MermaidValidation`（后端为 `selkie::parse`）。
  - 已完成：CLI 新增 `--validate` 与 `--validate-markdown`，stdout 输出 `true/false`，stderr 输出错误细节。
  - 已完成：新增测试 `tests/validate_smoke.rs`，并更新 `docs/code-agent-cli.md`。
  - 已验证：`cargo fmt --all` + `cargo test` 全通过。

### 2026-02-06 17:09
- 新需求：增加 `make validate-docs`，批量校验 `README.md` 与 `docs/**/*.md` 内的 ```mermaid code fence。
- 动机：
  - 把“文档 Mermaid 图的语法正确性”变成一条可重复执行的命令。
  - 这样本地与 CI 都能快速 gate, 避免 README/docs 里的图悄悄坏掉。
- 计划：
  - [x] 在 `Makefile` 增加 `validate-docs` target（逐文件执行 `--validate-markdown`，失败即退出）。
  - [x] 更新文档：补充 `make validate-docs` 的用途与示例。
  - [x] 验证：运行 `make validate-docs` 通过。
- 状态：**已完成** - `make validate-docs` 已可作为文档 Mermaid 图的快速 gate。

### 2026-02-06 19:33
- 新需求：Unicode（默认 relaxed）在 QuickJS 下渲染过慢，希望启用 Rust native A* 加速 relaxed 路由。
- 现象：
  - `cargo test --test ascii_testdata unicode_testdata_matches_reference` 在本机出现 70-100s 级别耗时。
  - 主要卡在 A* 路由热循环（QuickJS 无 JIT）。
- 根因：
  - Rust 侧只注入了 `__bm_getPath` / `__bm_getPathStrict`。
  - 但 TS 的 relaxed 路由走 `getPathRelaxed()`，之前没有 native fast path → 仍在 QuickJS 跑纯 JS A*。
- 计划：
  - [x] Rust：实现 native relaxed A*（步长 + crossing penalty + segment reuse hard rule）并注入 `globalThis.__bm_getPathRelaxed(...)`。
  - [x] TS：`getPathRelaxed()` 优先调用 `__bm_getPathRelaxed`（不存在则回退 JS）。
  - [x] 同步 vendor bundle 并跑端到端测试确认输出一致。
- 状态：
  - 已完成：`scripts/sync-vendor-bundle.sh` 通过；Unicode golden 用例耗时降到 ~3.6s。

### 2026-02-06 20:13
- 现象：渲染下述 Flowchart（Unicode + relaxed）时, 输出读起来像是 `task.start` 指向了 “🔎 规格审阅者”, 线路存在强歧义。
  - 复现命令：
    - `printf 'flowchart LR\n  ...\n' | beautiful-mermaid-rs --ascii`
  - 预期：`task.start` 应该只指向 `ralph#1 (coordinator)`，线路不应与其它边产生“看起来像连接”的梳状合并。
- 初步判断（先抓本质, 再决定手段）：
  - 当前 ASCII grid 布局里 “root 节点识别” 依赖 node insertion order。
  - 当 Mermaid 先声明一堆节点, 再写边时, 很多“其实有入边的节点”会被误判成 root 并堆在同一列。
  - 这会强迫 `task.start -> ralph` 走一个绕路 + 与其它边近距离贴合的路径, 从而造成肉眼误读。
- 两种处理路线（先摆出来, 避免直接跳到大重构）：
  1. **路线A（不惜代价 / 最佳方案）**：把 node 从固定 3x3 block 升级为 NxM 可变尺寸 grid block, 端口沿边框变成多个不同 toIdx, 让 A* 在路由阶段就能区分 lane。
     - 优点：从根上减少“多边挤同一个端口格子”导致的纠缠, 可读性上限更高。
     - 缺点：改动面很大（grid 占用、候选端口、native pathfinder、golden 全面变动）, 需要一轮完整回归。
  2. **路线B（先能用 / 风险可控）**：先修正 root 节点识别为“无入边的节点”, 让布局更接近 Mermaid/人类直觉; 若仍有歧义, 再在 relaxed 路由里补一个“避免中途形成 T junction”的惩罚项。
     - 优点：改动集中, 更像修 bug 而非推倒重来; 很可能直接消除该例的绕路合并。
     - 缺点：极端拥挤图仍可能需要 NxM 才能彻底解决。
- 决定：先落地路线B（root 识别修正 + 回归测试锁定）。如果该例仍然产生歧义, 再评估是否继续推进路线A。
- 计划：
  - [ ] 阶段1: 在 TS 源码定位 root 识别逻辑, 写最小复现测试用例
  - [ ] 阶段2: 修正 root 识别为“无入边节点”, 并跑 TS golden tests
  - [ ] 阶段3: `scripts/sync-vendor-bundle.sh` 同步 bundle 到 Rust vendor, 修复/更新 Rust golden
  - [ ] 阶段4: 端到端验证: `cargo test` + 复现命令确认 `task.start` 线路不再误读
- 状态：**进行中** - 先从 TS 的 `src/ascii/grid.ts` 入手（rootNodes 识别）。

### 2026-02-06 20:47
- 我正在继续处理 20:13 记录的 Flowchart 线路歧义问题, 因为目前用户例子里 `task.start` 的线路容易被肉眼误读为指向 “🔎 规格审阅者”, 这会直接影响你用终端看 Mermaid 图的可读性与可信度。
- 当前进展:
  - TS 侧已完成 relaxed 下 rootNodes 识别修正与放置流程的健壮性改良, 但 TS 测试目前还没全绿。
  - 我怀疑是 strict 路由被误引入了 corner fallback, 导致 golden/roundtrip 行为发生变化。
- 接下来要做的事(先修稳再同步 Rust, 避免把不稳定输出带进 vendor):
  - [ ] 收敛 strict 路由的 corner fallback, 让 TS `bun test` 全通过。
  - [ ] 重新构建并同步 vendor bundle 到本仓库, 然后 `cargo test` 全量验证。
  - [ ] 复现用户命令, 目测确认 `task.start -> ralph#1 (coordinator)` 的箭头清晰且不再误读。
  - [ ] 把根因/修复/验证记录追加到 `WORKLOG.md` 与 `ERRORFIX.md`, 并在 task_plan 里追加阶段勾选更新。

### 2026-02-06 20:56
- 进度更新(对应 20:13 的 4 个阶段):
  - [x] 阶段1: 在 TS 源码定位 root 识别逻辑, 写最小复现测试用例
  - [x] 阶段2: 修正 relaxed root 识别为“无入边节点”, 并跑 TS `bun test` 全绿
  - [x] 阶段3: `scripts/sync-vendor-bundle.sh` 同步 bundle 到 Rust vendor, 并按需更新 Rust Unicode golden
  - [x] 阶段4: 端到端验证: `cargo test` + 复现命令确认 `task.start` 线路不再误读
- 状态：**已完成** - 当前输出中 `task.start` 会直接、清晰地指向 `ralph#1 (coordinator)`。

### 2026-02-06 23:03
- 现象(新问题, 与上一个“root 误判”同属 Flowchart 可读性问题):
  - `flowchart TD` + Unicode(relaxed) 下, “🔎 规格审阅者” 右侧出线看起来没有贴到 box 边框, 出现了 box 内部的竖线 `│`/junction, 视觉上像是线从 box 里面长出来。
- 关键证据(用 TS 的 meta 验证, 不是肉眼猜):
  - `Hat_spec_reviewer -> Hat_spec_writer (spec.rejected)` 的 edge stroke 有 4 个点落在 reviewer box interior 内部(非边框), 这是错误行为。
  - reviewer 的 `gridCoord.x`(左边框列)被 `determineLabelLine()` 为了容纳 label “spec.rejected” 扩宽到 15, 进而触发 node box 与 edge port 的坐标系错位。
- 根因(本质层):
  - `determineLabelLine()` 当前用“把 chosenLine 的中点列 middleX 扩宽到 lenLabel+2”来给 label 腾空间。
  - 但 columnWidth 是“整列共享”的全局值, middleX 可能落在某个 node 的 3x3 block 列(甚至是 node 的 gridCoord.x 顶点列)。
  - 一旦把 node 的顶点列扩宽, `gridToDrawingCoord()` 的“cell center”语义会把 node box 平移到列中央, 但 edge port 仍按 grid 列边界取点, 导致端口落进 box 内部。
- 两种处理路线(先摆出来, 再选最小风险修复):
  1. 路线A(更彻底, 影响更大): 改 node box 的放置语义, 从“cell center”改成“cell origin”, 彻底消除列宽变化导致的 node 平移。
     - 优点: 坐标系更一致, 不怕任何列宽变化。
     - 缺点: 可能引发大量 golden 变化, 风险偏大。
  2. 路线B(改良胜过新增, 风险更可控): 保持 node 放置逻辑不动, 只改 `determineLabelLine()` 的“扩宽列选择策略”:
     - relaxed + Unicode 时, 若 middleX 落在任意 node 的 3x3 block 列, 则在 chosenLine 覆盖的 [minX..maxX] 范围内选一个“非 node block 列”的最近列来扩宽。
     - 这样 label 仍有空间, 但不会扩宽 node 列, 端口也不会跑进 box 内部。
- 决定: 先落地路线B, 并新增回归测试锁死“edge stroke 不得进入 node interior”。
- 计划:
  - [x] 阶段1: 在 TS 修复 `determineLabelLine()` 的扩宽列选择(仅 relaxed + Unicode), `bun test` 全绿
  - [x] 阶段2: 新增 TS 回归测试(用 meta 断言 reviewer 内部不再被 edge stroke 命中)
  - [x] 阶段3: 同步 vendor bundle 到 Rust, 更新必要的 golden, `cargo test` 全绿
  - [x] 阶段4: 复现命令验证 “规格审阅者” 右侧出线贴边且无 box 内部竖线
- 状态：**已完成** - `flowchart TD` 下 reviewer 右侧端口已贴边, 不再出现 box 内部竖线/错位端口。

### 2026-02-06 23:15
- 收尾动机：
  - 23:03 的修复已经落地并验证, 但需要把“根因/修复/验证”同步写入 `WORKLOG.md` 与 `ERRORFIX.md`, 让后续排查有可追溯记录。
- 已完成：
  - [x] 追加记录到 `WORKLOG.md` 与 `ERRORFIX.md`(只追加到文件尾部)。
  - [x] 本仓库 `cargo test` 再次确认全绿。
- 状态：**已完成** - Flowchart TD 的“端口不贴边”问题已闭环记录并验证。

### 2026-02-07 00:18
- 新需求：git 提交（提交本次 Flowchart routing 修复相关改动）。
- 动机：
  - 把 vendor bundle + golden + 四文件记录 固化到提交历史里, 避免工作区长期漂移。
  - 让其他机器或 CI 拉取后可以稳定复现当前输出与测试结论。
- 计划：
  - [x] 查看并审阅 staged 变更：`git status` + `git diff --cached --stat`
  - [x] 检查是否存在 submodule 变更（如有则一并提交）
  - [x] 确认测试全绿：`cargo test`
  - [x] 执行提交：`git commit -m 'fix: sync vendor bundle for flowchart routing'`
  - [x] 提交后确认工作区干净：`git status`
- 状态：**已完成** - 已提交 `fix: sync vendor bundle for flowchart routing`, 工作区已干净。
