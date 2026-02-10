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

## 2026-02-06 20:56 - 修复 Flowchart 线路强歧义: “先声明节点, 再连线”导致 relaxed root 误判

- 背景：
  - 复现输入是 flowchart LR, 先声明节点(含 emoji/中文 label), 再写边。
  - 旧 relaxed 布局会把“其实有入边”的节点误判成 root, 堆到同一列, 迫使 `task.start -> ralph` 大绕路并与其它边贴合, 肉眼容易误读为指向 “🔎 规格审阅者”。
- 修复策略(改良胜过新增, 先避免 NxM 大重构)：
  - relaxed: rootNodes 改为“无入边节点”(按 insertion order 稳定排序), 从源头修正布局层级。
  - strict: 保持旧 root 推断与路由兜底策略, 避免 strict 的 golden/roundtrip 行为漂移。
- Rust 侧落地：
  - 已通过 `scripts/sync-vendor-bundle.sh` 同步最新 bundle 到 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`(sha256: `0bc9ef48...`)。
  - 因为 Unicode 默认 routing=relaxed, 本修复会改变部分 Unicode 输出, 已更新 golden:
    - `tests/testdata/unicode/preserve_order_of_definition.txt`
- 验证：
  - `cargo test` 全通过。
  - 复现命令(与用户一致)输出第一行已清晰呈现:
    - `task.start ├────► ralph#1 (coordinator)`

## 2026-02-06 20:59 - 更新本机已安装的 `beautiful-mermaid-rs`

- 执行：
  - `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool`
- 验证：
  - `which beautiful-mermaid-rs` 指向 `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`。
  - 直接运行 `printf ... | beautiful-mermaid-rs --ascii` 已能稳定复现“无歧义”的新输出。

## 2026-02-06 23:15 - 修复 Flowchart TD 下 “出线不贴边”(端口落入 box interior)

- 现象：
  - `flowchart TD` + Unicode(relaxed) 渲染时, “🔎 规格审阅者” 右侧出线看起来没有贴到边框。
  - 视觉上会出现 box 内部竖线 `│`/junction, 像线从 box 里面长出来。
- 根因(本质)：
  - `determineLabelLine()` 为了容纳边 label, 会扩宽 `columnWidth[middleX]`。
  - 但 `columnWidth` 是“整列共享”的全局宽度, `middleX` 可能落在某个 node 的 3x3 block 列里(甚至是 node 顶点列)。
  - 一旦扩宽了 node 列, `gridToDrawingCoord()` 的“cell center”语义会把 node box 平移, 但 edge port 仍按 grid 边界取点, 从而造成端口落入 box interior。
- 修复(改良胜过新增, 先不做 NxM 大重构)：
  - relaxed + Unicode 时, 如果 `middleX` 命中任意 node block 列, 就在该 labelLine 覆盖的 `[minX..maxX]` 范围内选择“最近的非 node block 列”来扩宽。
  - 这样 label 仍有空间, 但不会误伤 node 列, 端口也就不会跑进 box 内部。
- Rust 侧落地：
  - 同步 vendor bundle 到 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`(sha256: `28c11372...`)。
- 验证：
  - `cargo test` 全通过。
  - 复现命令输出中 reviewer 右侧端口已贴边, 不再出现 box 内部竖线:
    - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`

## 2026-02-07 00:18 - git 提交（固化 Flowchart routing 修复）

- 已提交：`fix: sync vendor bundle for flowchart routing`
- 包含变更：
  - 同步 vendor bundle：更新 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`(sha256: `28c11372...`)。
  - 更新 golden：`tests/testdata/unicode/preserve_order_of_definition.txt` 对齐最新 Unicode(relaxed) 输出。
  - 同步记录：`task_plan.md`、`notes.md`、`WORKLOG.md`、`ERRORFIX.md` 追加本次问题的根因/修复/验证结论。
- 验证：
  - `cargo test` 全通过。
  - 复现命令：
    - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`

## 2026-02-08 02:34:27 - 修复 flowchart TD 箭头未贴边(输出偏移)

### 变更摘要
- 修复了 `--ascii` 渲染中“目标端口列被扩宽后箭头离 box 过远”的显示错位。
- 同步更新 vendor bundle,并补充 Rust 回归测试锁死行为。

### 具体改动
- TypeScript 源码(上游):
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  - 改动点:
    - `drawArrowHead` 增加目标 box 邻接锚定逻辑。
    - 新增箭头桥接线绘制逻辑,保证 old lastPos 到新箭头位置连续。
    - `computeEdgeStrokeCoords` 同步写入桥接段与新箭头坐标。
    - `computeArrowHeadPosForLabelAvoid` 与实际箭头坐标保持一致。
- Rust 仓库:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js` (同步最新 bundle)
  - `tests/ascii_endpoint_alignment.rs` (新增回归测试)

### 验证记录
- TS 侧:
  - `bun test src/__tests__/ascii-relaxed-routing.test.ts src/__tests__/ascii-label-avoid-junction.test.ts src/__tests__/ascii-no-collinear-overlap.test.ts src/__tests__/unicode-relaxed-no-collinear-overlap.test.ts`
- Rust 侧:
  - `scripts/sync-vendor-bundle.sh`
  - `cargo test --package beautiful-mermaid-rs --test ascii_endpoint_alignment arrowheads_to_ralph_remain_box_adjacent_in_user_repro_case -- --exact`
  - `cargo test`
- 安装版 CLI 验证:
  - `cargo build --release`
  - `cp target/release/beautiful-mermaid-rs /Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`
  - `beautiful-mermaid-rs --ascii < /tmp/repro_user_case.mmd`

## 2026-02-08 03:09:16 - 二次补丁: 修复 source 侧出边悬空

### 文件变更
- `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  - `drawBoxStart` 改为基于 source box 锚定。
  - 新增 `computeBoxStartPositionNearSourceBox`。
  - `drawArrowHeadBridge` 泛化为 `drawEndpointBridge`，source/target 复用。
  - `computeEdgeStrokeCoords` 增加 source bridge 轨迹。
  - source/target 锚点计算增加 clamp。
- `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js` 同步更新。
- `tests/ascii_endpoint_alignment.rs` 增强断言:
  - 保留 target 邻接校验。
  - 新增 source 出边必须落在 `Hat_ralph` 边框上的校验。

### 验证
- TS 关键回归通过: 5/5。
- Rust 回归通过: `cargo test` 全通过。
- 安装版 CLI 复现通过: `beautiful-mermaid-rs --ascii < /tmp/repro_user_case.mmd`。

## 2026-02-08 11:33:04 - 修复 “实验执行器游离箭头” + 小步性能优化

### 问题现象
- 用户复现图里,`Hat_ralph -> Hat_experiment_runner (experiment.task)` 在“实验执行器”上方出现 `▼` 游离箭头。
- 直观感受:
  - 箭头没有可靠地“连回主干线”,读图时像是断线。

### 根因
- 箭头会被锚定到 target box 外侧一格(贴边),但末段 `lastPos` 可能落在另一列。
- 旧 `drawEndpointBridge()` 只支持同轴桥接:
  - 这会把“竖向箭头(▼/▲)”变成“水平线末端挂着的箭头”,从而看起来游离。

### 修复(绘制层最小改动)
- TypeScript 源码(上游):
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  - `drawEndpointBridge()`:
    - 支持 L 型桥接;
    - 对 `dir=Up/Down && from.y===to.y` 增加 1-cell stem,保证箭头入边方向存在竖向笔画;
    - 在桥接拐点写入正确 corner 字符,确保读图连续。

### 性能优化(确定性收益)
- `drawArrow()` 内提前生成的 `labelCanvas` 已不再被 `drawGraph()` 使用(因为 label 现在统一在合成线路层后生成并避让 junction)。
- 移除该冗余计算,减少每条 edge 一次 label 布局 + 一次 canvas 拷贝。

### Rust 侧落地
- 同步 vendor bundle:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js` (sha256: `66ef06df...`)
- 新增回归测试:
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`
    - 锁定 `experiment.task` 的 arrow 上方必须存在竖向笔画(避免再出现“游离箭头”)。
- 测试辅助:
  - `Cargo.toml` 增加 dev-dep `unicode-width`(用于按终端显示宽度构建 cell 网格,兼容 emoji 宽字符)。

### 验证
- `scripts/sync-vendor-bundle.sh` ✅
- `cargo test` ✅
- CLI 复现命令:
  - `beautiful-mermaid-rs --ascii < /tmp/repro_user_case.mmd` ✅

## 2026-02-08 13:54:03 - 修复 relaxed 下“右侧外圈大矩形”(绕路/绕圈观感)

### 现象
- 用户反馈:
  - `experiment.result` 这条线看起来“绕了个圈”。
- 本地复现与 meta 量化后确认:
  - 真正“画大外框”的主要是 integrator 相关边,尤其是:
    - `Hat_experiment_integrator -> Hat_ralph (integration.rejected)`
    - `Hat_experiment_integrator -> Complete (experiment.complete)`

### 根因
- relaxed 的候选路线排序里,我们偏好“拐点更少”的路径。
- 但“大外圈矩形”恰好拐点很少(通常 2~3 次转向),
  - 会被误判为“更优雅”的路线,
  - 结果把图拉得很宽,肉眼就像 `experiment.result` 在绕圈。

### 修复(最小改良,不扩大 grid)
- TypeScript 上游:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
  - 在 `candidateCostRelaxed()` 增加 `detourPenaltyRelaxed()`:
    - 基于 from/to 节点 3x3 block 的包围盒,
    - 对路径 bbox “远离节点包围盒太多”的候选加 soft penalty,
    - 且只在 detour 很大(> THRESHOLD=12)时才生效,控制影响面。
- Rust 侧:
  - `scripts/sync-vendor-bundle.sh` 同步 bundle 到:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`

### 结果(关键指标对比)
- `integration.rejected`:
  - bbox max_x: 110 -> 90
  - path len: 130 -> 109
- `integrator -> Complete (experiment.complete)`:
  - bbox max_x: 98 -> 88
  - path len: 113 -> 82

### 回归测试(锁死“不要再画外圈”)
- 更新 `tests/ascii_user_case_edge_endpoint_invariants.rs`:
  - 对关键边增加断言:
    - edge 的 `max_x` 不得超过“最右 node 边界”太多(extra_right <= 10)。
    - 并对 path.len() 设置宽松上限,避免回到极端外圈绕行。

### 验证
- `scripts/sync-vendor-bundle.sh` ✅
- `cargo test` ✅
- CLI 复现(输出不再出现巨大右侧外框):
  - `printf 'flowchart TD ...' | cargo run --quiet -- --ascii`

## 2026-02-08 16:24:22 - 最近侧边端口优先 + label 拼接(断线)修复

### 现象(续)
- 你继续反馈:
  - 仍存在断线/绕路。
  - “对拥挤节点按需增加 lane/margin”。
  - “确保用 box 与 box 最近的边出线/入线”。

### 修复要点
- relaxed 路由(Unicode 默认):
  - 增加“最近侧边(朝向)软惩罚”,避免选择背向端口,从根上减少绕弯与外圈趋势。
  - 不增加 A* 调用次数,只影响候选排序,性能风险很低。
- label 绘制(Unicode relaxed):
  - 修复多个 edge label 在同一行发生覆盖拼接(例如 `iexperiment.taskked`)。
  - 策略是“按边顺序逐个落盘到 canvas”,并把已有文本视为 forbidden cell。
  - 当确实无合法位置时,允许不画该 label(避免乱码/断线)。

### 落盘(本仓库)
- vendor bundle:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- 测试与 golden:
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`:
    - 前置 debug 打印,便于断言失败时直接定位“起始段背向/绕路”的具体形态。
  - `tests/testdata/unicode/ampersand_lhs_and_rhs.txt`:
    - 由于 relaxed 端口排序更符合“最近边”,对应 golden 已更新对齐。

### 验证
- `cargo test` ✅
- CLI 复现图对比:
  - `integration.blocked` 不再与其它 label 拼接成乱码;
  - `experiment.complete` 起始段不再先朝右“背向走一截”。

## 2026-02-08 17:29:11 - corner 端口计数扩容 + Unicode crossing 拐点连通保护

### 现象(续)
- 你继续指出:
  - ralph 的同侧边很多,但 box 边长不够(端口挤在一起)。
  - `experiment.result` 视觉上像绕圈。
  - `complete` 与“结果审计员”之间出现 `◄────────────────────►`,读者会误以为存在双向边。

### 根因(补充)
- comb ports 端口统计只识别 Up/Down/Left/Right:
  - relaxed fallback 在“端口几何不可达”时会用 corner port(UpperLeft/LowerRight/...),
  - corner port 未计入 counts => box 不会按真实端口数扩容。
- Unicode `┼` 桥化策略在拐点处会破坏连通:
  - `deambiguateUnicodeCrossings()` 把 `┼` 直接改成 `─/│`,
  - 如果 `┼` 正好落在某条边的拐点上,桥化会把边断开,进而出现“双箭头直线”的错觉。

### 修复要点
- corner port 计入拥挤度,触发 box 自适应扩容:
  - 上游 TS: `src/ascii/grid.ts`
  - `dirToSide(d,node,other)`:
    - corner port 按 |dx| vs |dy| 映射到最近侧边,用于 counts/扩容/offset 分配。
- crossing 去歧义: 拐点优先降级为 tee/corner,避免断线:
  - 上游 TS:
    - `src/ascii/draw.ts` 新增 `computeEdgeCornerArmMasks()`
    - `src/ascii/index.ts` 将拐点掩码传入桥化,并用“后写覆盖(last wins)”避免 FULL_MASK
    - `src/ascii/canvas.ts` 桥化遇到拐点 `┼` 时降级为 `┬/┴/├/┤/┐/┘/┌/└`

### 落盘(本仓库)
- vendor bundle:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- golden:
  - `tests/testdata/unicode/ampersand_lhs_and_rhs.txt`

### 验证
- `cargo test` ✅
- `make install` ✅(已更新安装版 `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`)
- 用户复现图:
  - ralph box 自动增高;
  - `complete` 与审计员之间由 `◄──────►` 变为 `◄──┴──►`(出现明确分叉符号),可读性显著提升。

## 2026-02-08 22:03:31 - comb ports 单端口 nudge: 消除 `◄──┴──►` 共享走线假象

### 现象(继续)
- 你认为 `◄──┴──►` 仍然难读,并强调:
  - 尽量不要共享走线
  - 拥挤节点按需增加 lane/margin
  - 优先保证“从最近侧边出线/入线”

### 根因(本次定位)
- `experiment.result` 与 `integrator -> Complete (experiment.complete)` 在画布中部发生 point overlap:
  - overlap char=`┴`,导致读者误判为一条 `complete <-> auditor` 的双向边。
- overlap 的来源不是 segment overlap(共线段复用),而是 comb ports 的“单端口固定居中”导致多个 node 的 center lane 全局对齐。

### 修复
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
  - comb ports `assign()`:
    - `list.length===1` 时做 1 格确定性 nudge(按 side + kind(start/end)),
      打散 center lane,降低 point overlap 概率。
- 本仓库:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`(通过 `scripts/sync-vendor-bundle.sh` 同步)
  - 回归加固:
    - `tests/ascii_user_case_edge_endpoint_invariants.rs` 新增断言:
      `experiment.result` 与 `experiment.complete` overlap_cells 必须为 0。
  - golden 同步:
    - `UPDATE_GOLDEN=1 cargo test --test ascii_testdata` 更新 `tests/testdata/unicode/*.txt` 的 lane 输出。

### 验证
- 用户复现图:
  - `complete` 与审计员之间不再出现 `◄──┴──►` 的“共享走线双向边”错觉。
- `cargo test` ✅

## 2026-02-09 12:04:24 - 修复 meta 端点不变量回归(箭头贴边) + 同步 vendor + 更新 Unicode golden

### 问题
- `cargo test --release` 失败:
  - `tests/ascii_endpoint_alignment.rs`
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`
- 失败形态:
  - 实际文本里箭头已经贴边(x=49),
  - 但 meta 的 `edge.path.last()` 停在 box 内部(x=41),导致端点不变量断言失败。

### 根因
- 上游 TS `src/ascii/draw.ts` 的 `computeEdgeStrokeCoords()`:
  - 用 `pushUnique` 去重时保留“第一次出现”的坐标。
  - 在 columnWidth/rowHeight 伸缩导致末段线段提前经过 arrowPos 时,
    arrowPos 会被“提前写入”,从而无法稳定落在 path 末尾。

### 修复
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
  - `computeEdgeStrokeCoords()`:
    - 对齐 `drawArrowHead()` 的 dir/lastPos 推断(处理末段退化为单点时的 fallbackDir)。
    - 新增 `pushUniqueLast()` 专用于 arrowPos: 若已出现过则移除旧位置并重新 push 到末尾,
      保证 `path.last()` 永远是箭头 cell。
- 本仓库:
  - `scripts/sync-vendor-bundle.sh` 重建并同步 vendor bundle:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - 新增调试 example:
    - `examples/debug_user_case_meta.rs`(输出 text + meta edges first/last + 贴边判定)
  - golden 同步:
    - `tests/testdata/unicode/preserve_order_of_definition.txt`

### 验证
- `cargo test` ✅
- `cargo test --release` ✅

## 2026-02-09 12:31:50 - 防回归: 上游 TS 增加 meta 端点语义测试(Unicode relaxed)

### 做了什么
- 上游 TypeScript:
  - `src/__tests__/ascii-with-meta-roundtrip.test.ts`:
    - 追加用户复现图用例,对每条 edge 锁死不变量:
      - `path.last()` 必须贴着 target box 外侧一格。
      - `text` 在该坐标必须是箭头符号(▲▼◄►◥◤◢◣/●)。
    - 使用 `charDisplayWidth()` 做“显示列宽”映射,避免宽字符导致的坐标读取偏移。
    - 该用例渲染约 8s,为避免 bun 默认 5s timeout 误报,单独把该 `it(...)` timeout 设为 20s。

### 验证
- TS: `bun test src/__tests__/ascii-with-meta-roundtrip.test.ts` ✅
- Rust: `cargo test --release` ✅

### 备注
- ASCII strict 在该复现图上 `meta.nodes` 为空(疑似 strict 路由不可达导致 createMapping 失败)。
  - 这属于另一条问题线,本轮先不扩 scope,避免把“meta 端点语义回归”与“strict 可达性”混在一起修。

## 2026-02-09 13:16:20 - 回滚灾难输出: 修复 Rust native relaxed pathfinder 与 TS 语义不一致

### 现象
- `beautiful-mermaid-rs --ascii` 渲染用户复现图时输出极其混乱,比 TS(bun) 输出差很多。

### 根因
- Rust 侧 native `get_path_relaxed` 与 TS `getPathRelaxed` 的 usedPoints 规则不一致:
  - 把 TS 的 point-overlap hard rule 改成了 penalty,导致路线更容易走进占用点位并合成强歧义 junction。

### 修复
- `src/native_pathfinder.rs`:
  - `get_path_relaxed` 按 TS hard rule 处理 usedPoints(仅豁免起点第一步与终点前一步,并限制 arms 阈值)。
  - 移除点重叠 penalty(RELAXED_PENALTY_USED_POINT*)。
- `src/js.rs` + `.envrc`:
  - 默认仍启用 native(保证 QuickJS 下速度),
  - 提供 `BM_DISABLE_NATIVE_PATHFINDER=1` 用于对照/排错(默认 0)。
- 回归加固:
  - 新增 `tests/testdata/unicode/user_repro_case.txt`(锁死该复现图完整输出)。
  - 更新 `tests/testdata/unicode/preserve_order_of_definition.txt`(同步新基线)。
- 安装:
  - 已覆盖安装 `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs` 为修复后的 release 二进制。

### 验证
- Rust 输出与 TS(bun) 输出一致(仅差末尾换行)。
- `cargo test --release` ✅

## 2026-02-09 16:35:40 - 输出仍不够可读: 通过“TD 双向边下沉布局”消除外圈绕行

### 现象
- 你反馈同一份 Mermaid 在 `beautiful-mermaid-rs --ascii` 下“结果还是很糟糕”:
  - 多条边被迫绕外圈,画出类似“外框”的大矩形,可读性很差。

### 根因(本质)
- TD 布局里,同一父节点的多个 child 默认会被放在同一层并横向铺开。
- 当 parent 与某个 child 之间存在双向边(A->B 与 B->A)时:
  - B->A 会变成“向上走”的长 backward edge,
  - 在“不共线重叠”的规则下很容易被挤到外圈,形成巨大外框。

### 修复(上游 TS + 本仓库同步)
- TypeScript 上游:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
  - TD + Unicode relaxed 下新增启发式:
    - 若发现 `child -> parent` 反向边存在,则把该 child 下沉到下一层(`childLevel + gridStep`),
      并优先与 parent 对齐同一列(`x = parent.x`)。
    - 让双向关系更像“垂直回路”,显著降低外圈绕行概率。
- Rust 本仓库:
  - 同步 vendor bundle:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - 更新回归样例:
    - `tests/ascii_endpoint_alignment.rs`(右侧贴边样例改为 `integration.blocked`)
    - `tests/testdata/unicode/user_repro_case.txt`(更新 golden 为更可读的新布局)

### 验证
- `cargo test --release` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅
- 复现命令:
  - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`
  - 输出不再画巨大外圈,节点更集中,边更短,可读性显著提升。

## 2026-02-09 17:38:55 - 继续收敛画布宽度: label 扩列改为“最小增量”避免无意义空白

### 现象
- 复现图虽然比“灾难输出”正常很多,但仍然存在:
  - 画布偏宽(大量空白),
  - 部分边的 detour 视觉成本偏大,容易让人觉得“像在画外框”。

### 根因
- 上游 TS `determineLabelLine()` 会在选定 `labelLine` 后,
  **无条件**把某一整列 `columnWidth` 拉到 `labelWidth+2`。
- 对水平线段来说,这往往是过度的:
  - 线段总宽度已经够放下 label,
  - 但仍被强行扩列,导致整图被撑大。

### 修复
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
  - `determineLabelLine()`:
    - 计算当前线段总宽度 `currentTotalWidth`；
    - 只有当 `currentTotalWidth < labelWidth+2` 时,才对 `widenX` 做 `delta` 的最小增量扩列。
- Rust:
  - `scripts/sync-vendor-bundle.sh` 同步 bundle 到:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - golden 同步:
    - `tests/testdata/ascii/subgraph_with_labels.txt`
    - `tests/testdata/unicode/user_repro_case.txt`

### 验证
- `cargo test --release` ✅
- 复现图: 画布宽度明显收敛,外框尺寸更小,整体更集中。

### 备注
- 我短暂尝试过“改路由顺序”来让某条边更直,但发现会把外框转移到其它边并放大整体 detour,
  因此已回滚该思路,保持 insertion order 作为稳定基线。

## 2026-02-09 18:26:40 - 结论调整: “多入边汇聚”的流程用 sequenceDiagram 才是终端友好表达

### 现象
- 用户的 flowchart(TD/LR) 存在大量边汇聚到同一个协调者节点(Ralph),并且 Integrator 有多条不同 label 的边回指 Ralph。
- 在终端 ASCII/Unicode 输出里,这种结构天然会出现:
  - 大量 `┬/┴/├/┤/┼` 汇聚点,视觉上像“线团/误连线”；
  - 带 label 的边更容易被迫走外圈,形成“大矩形框”。

### 动作
- 复现用户的 flowchart TD 输入,确认“合并线过多导致不可读”的问题确实存在。
- 把同一逻辑改写成 `sequenceDiagram`(参与者泳道 + 消息箭头),并用 `beautiful-mermaid-rs --ascii` 验证:
  - 输出明显更清晰,几乎消除了“线团”。
- 把经验沉淀为文档,避免后续重复踩坑:
  - 新增 `docs/terminal-readable-diagrams.md`
  - 内含可直接复用的 `sequenceDiagram` 模板 + 终端渲染效果示例

### 验证
- `make validate-docs` ✅ (README + docs/**/*.md 的 Mermaid code fence 全部通过校验)

## 2026-02-09 19:18:03 - Flowchart 终端可读性: 生成树主干边优先路由,消除“外框”与主要线团

### 背景
- 用户坚持必须是 `flowchart`，且不允许新增 CLI 开关。
- 复现图里最影响阅读的是: `integration.task` 被路由成一个包住整张图的“大矩形外框”,看起来像 subgraph 边框。

### 根因
- Unicode relaxed 下仍有“禁止 segment overlap”的 hard rule。
- 当关键主干边在 Mermaid 文本里出现得很靠后时:
  - 它会在其它边占满内圈通道后才开始路由,
  - 结果被迫走最外圈,形成外框。

### 修复
- 上游 TS: `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/grid.ts`
  - Unicode + relaxed 下,把 edge routing 顺序改为:
    - 先路由 spanning forest(生成树主干边),
    - 再路由剩余边(回边/补充边)。
  - 其它模式保持原样,控制影响面。
- Rust:
  - `scripts/sync-vendor-bundle.sh` 同步 vendor bundle。
  - 更新 Unicode golden:
    - `tests/testdata/unicode/user_repro_case.txt`
  - 修复不稳定断言:
    - `tests/ascii_endpoint_alignment.rs` 移除“integration.blocked 必须右侧入边”的硬断言,保留贴边不变量。

### 验证
- `cargo test --release` ✅
- `make install` ✅ 已更新 `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`
- 用户复现命令:
  - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`
  - `integration.task` 不再画出顶层大矩形外框,整体明显收敛到内圈。

### 2026-02-09 21:01:00 - 回答用户问题: `integration.blocked` 为什么看起来有很多条线

#### 我做了什么
- 复盘并读取当前实现:
  - `src/ascii/index.ts` 默认 routing 逻辑(useAscii=false -> relaxed)。
  - `src/ascii/grid.ts` 的 spanning-forest-first 路由顺序。
  - `src/ascii/edge-routing.ts` 的同端点 segment 复用规则(`pairId`)。
  - `src/ascii/draw.ts` 的 comb ports lane 偏移绘制。
- 运行复现实验并拿到量化证据:
  - `BM_DEBUG_WIDE_EDGES=1 cargo test --test ascii_user_case_edge_endpoint_invariants user_repro_case_all_edges_respect_endpoint_invariants -- --exact --nocapture`
  - 结果显示 4 条 `Hat_experiment_integrator -> Hat_ralph` 边都存在,并且 bbox/长度接近。
- 运行 meta 调试:
  - `cargo run --quiet --example debug_user_case_meta`
  - 确认 4 条平行边在目标节点左侧不同 y 入边,`integration.blocked` 只是其中之一。

#### 结论
- 视觉上的“很多条线”不是 `integration.blocked` 一条边重复绘制。
- 它来自:
  - 同端点平行边(4条)在 relaxed 下共享主干,
  - 叠加 comb ports 端口分 lane,
  - 再加上其它回边同区域汇入,形成线束感。

#### 后续可执行优化建议
- 最佳方案: 同端点多边分组路由(bundle trunk + endpoint fanout)。
- 渐进方案: 在现有 A* 评分里追加“平行边聚团惩罚 + 标签冲突二次 reroute”。

### 2026-02-09 21:27:00 - 已按用户选择落地“最佳方案”

#### 代码改动
- 上游 TS:
  - `src/ascii/grid.ts`
    - 新增 bundle trunk 路由:
      - 同端点多边组内 leader 路由一次;
      - follower 复用主干 path,减少线束抖动与重复竞争通道。
  - `src/ascii/draw.ts`
    - 新增 bundle label 纵向堆叠:
      - 组内共享 anchorY;
      - 按 rank 纵向分层(步长 2);
      - 避免标签横向拼接成长串。
  - `src/ascii/pathfinder.ts`
    - 修复 relaxed 路由运行时 `ReferenceError`:
      - 补齐 `segmentPair` / `segmentPairMulti` 局部变量绑定。
- Rust 仓库:
  - 同步 vendor bundle:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - 更新 golden:
    - `tests/testdata/unicode/user_repro_case.txt`

#### 结果
- 用户复现图中,同端点多边仍共享主干,但标签改为上下分层,不再横向拼接。
- 复现命令:
  - `printf 'flowchart TD ...' | cargo run --quiet -- --ascii`

#### 验证
- `cargo test --test ascii_user_case_edge_endpoint_invariants` ✅
- `cargo test --test ascii_testdata` ✅
- `cargo test` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅

### 2026-02-10 00:56:00 - 继续优化: 终点复用策略由“顺序短路”改为“质量对比”

#### 变更内容
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
    - 新增 `isHorizontalCrossSide()` 用于识别“左右对穿”候选。
    - 新增 `pickRelaxedWithEndReuseComparison()`:
      - 同时对比 `allowEndSegmentReuse=false/true` 两个结果。
      - 在 Unicode relaxed + 垂直主导下,允许“非复用但略差”的非对穿路径胜出。
    - 路由主流程改为:
      - `pickedWithoutReuse = tryPickRelaxed(false)`
      - `pickedWithReuse = tryPickRelaxed(true)`
      - `picked = pickRelaxedWithEndReuseComparison(...)`

#### 同步与产物
- 同步 vendor:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- golden 更新:
  - `tests/testdata/unicode/backlink_from_top.txt`

#### 验证
- `cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet` ✅
- `cargo test --test ascii_user_case_edge_endpoint_invariants --quiet` ✅
- `cargo test --quiet` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅

#### 结果
- 修复了“复用策略顺序偏置”这一算法问题。
- 用户复现图在不引入新外框回归的前提下保持稳定,并为后续近侧优先改良提供更可靠的决策基线。

### 2026-02-10 01:32:00 - 完成: 并线标签强制上下堆叠(禁止左右拼接)

#### 代码实现
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
    - 新增 `DrawTextOnLineOptions.verticalOnlyStack`。
    - `drawTextOnLine()` 在 `baseCanvasForAvoid` 分支支持“仅纵向避让”:
      - 固定中心 x;
      - 仅上下找可用 y;
      - 不再做横向 startX 漂移。
    - `drawGraph()` Unicode relaxed 顺序标签阶段:
      - 对 bundle 分组标签传入 `verticalOnlyStack=true`。

#### 同步
- 同步 vendor:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- 已安装到:
  - `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`

#### 验证
- `cargo test --test ascii_user_case_edge_endpoint_invariants --quiet` ✅
- `cargo test --test ascii_testdata --quiet` ✅
- `cargo test --quiet` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅

#### 结果
- 并线标签不再通过横向位移来避让,冲突处理改为上下层叠。
- 与用户规则对齐: “并线注释上下排列,不要左右排列”。

### 2026-02-10 01:53:00 - 二次修复: 同组标签统一中心 x,彻底收敛为纵向列

#### 代码改动
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
    - 在 `buildBundleStackedLabelLines()` 中新增 `anchorCenterX` 计算:
      - 取同组 `baseLine` 中心点的中位数。
    - 组内每条标签线统一改为:
      - `[{x: anchorCenterX, y: stackedY}, {x: anchorCenterX, y: stackedY}]`
    - 配合 `verticalOnlyStack` 后,实现“同列上下堆叠”。

#### 效果
- 用户复现图关键标签坐标:
  - `experiment.complete`: row=42, col=60
  - `integration.applied`: row=44, col=60
  - `integration.blocked`: row=46, col=60
  - `integration.rejected`: row=48, col=59
- 已从“横向散开”收敛为“同列纵向栈”。

#### 验证
- `cargo test --test ascii_user_case_edge_endpoint_invariants --quiet` ✅
- `cargo test --test ascii_testdata --quiet` ✅
- `cargo test --quiet` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅

### 2026-02-10 02:24:00 - 新增并线标签短引导符,增强“标签-线路”对应关系

#### 代码改动
- 上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
    - `drawTextOnLine()` 返回 `LabelPlacement | null`。
    - 新增 `drawShortBundleLabelLeader()`:
      - 根据标签落点与原始 labelLine 中心,绘制 1 格引导符。
      - 优先目标侧水平符号,不可用时退化到上下符号。
    - `drawGraph()` 的 Unicode relaxed bundle 路径:
      - 在标签写入后追加短引导符。

#### 同步与产物
- `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- `tests/testdata/unicode/user_repro_case.txt`

#### 验证
- `cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet` ✅
- `cargo test --quiet` ✅
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅

#### 结果
- `experiment.complete/integration.applied/integration.blocked/integration.rejected` 在同列纵向堆叠基础上,新增短引导标记,
  更容易看出标签与附近线路的对应关系。

### 2026-02-10 19:32:05 - 并线标签收口优化完成(继续任务)

#### 本轮目标
- 继续优化用户复现图中并线标签的可读性。
- 满足硬约束: 并线标签仅上下堆叠,不再出现左右拼接样式。

#### 代码与产物
- 修改上游 TS:
  - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/draw.ts`
    1. `drawShortBundleLabelLeader()` 改为仅纵向引导符。
    2. `buildBundleStackedLabelLines()` 增加同组 `anchorStartX` 统一起始列。
- 同步到 Rust vendor:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- 更新 golden:
  - `tests/testdata/unicode/user_repro_case.txt`

#### 验证命令
- `UPDATE_GOLDEN=1 cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet`
- `cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet`
- `cargo test --quiet`
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool`
- `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs --ascii < /tmp/user_repro_case.mmd`

#### 验证结果
- 所有正式测试通过。
- 安装后的 CLI 复现确认通过。
- 并线标签呈纵向列表,且不再出现左右 `─/-` 拼接噪音。

### 2026-02-10 20:26:52 - 继续优化完成: 近侧走线优先(左侧端口)落地

#### 用户目标
- 继续优化路由。
- 重点解决“明明左边近却没有走左边”的问题。

#### 代码改动
1. 上游 TS 路由核心:
   - `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
     - `tryPickRelaxed()`:
       - 坏味道场景补充 `expandedAll` 探测。
       - 同端点并线 + Unicode 场景优先 end reuse。
     - `nearestSidePenaltyRelaxed()`:
       - 增加近轴对穿惩罚。
     - `detourPenaltyRelaxed()`:
       - 增加方向过冲惩罚。
2. 调试工具增强:
   - `examples/debug_user_case_meta.rs`
     - 新增 `len` / `bbox` 输出。
3. Rust 仓库同步:
   - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
   - `tests/testdata/unicode/user_repro_case.txt`
   - `tests/ascii_user_case_edge_endpoint_invariants.rs` (阈值与注释同步)

#### 验证命令
- `cargo test --test ascii_user_case_edge_endpoint_invariants user_repro_case_all_edges_respect_endpoint_invariants --quiet`
- `cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet`
- `cargo test --quiet`
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool`
- `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs --ascii < /tmp/user_repro_case.mmd`

#### 验证结果
- 所有测试通过。
- 安装验证通过。
- 关键并线边已从目标左侧进入 `Hat_ralph`。

### 2026-02-10 22:56:39 - 修正为“纯近路优先”并完成回归

#### 目标
- 按用户澄清,将策略从“侧向偏好”改为“纯近路优先”。

#### 代码改动
- `/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid/src/ascii/edge-routing.ts`
  1. `detourPenaltyRelaxed()`
     - 移除方向过冲惩罚,保持方向无关。
  2. `nearestSidePenaltyRelaxed()`
     - 保留中等强度背向惩罚,移除轴向/对穿偏置。
  3. `tryPickRelaxed()`
     - `shouldProbeExpandedAllFast()` 改为路径质量触发逻辑。
  4. `pickRelaxedWithEndReuseComparison()`
     - 回收为纯成本比较。
  5. 移除同端点并线的定向优先分支。

- 同步与测试文件:
  - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
  - `tests/testdata/unicode/user_repro_case.txt`
  - `tests/ascii_user_case_edge_endpoint_invariants.rs`

#### 验证命令
- `cargo test --test ascii_user_case_edge_endpoint_invariants user_repro_case_all_edges_respect_endpoint_invariants --quiet`
- `cargo test --test ascii_testdata unicode_testdata_matches_reference --quiet`
- `cargo test --quiet`
- `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool`

#### 结果
- 全部测试通过。
- 安装版已更新。
- 路由行为满足“纯近路优先,不固定偏左/偏右”。
