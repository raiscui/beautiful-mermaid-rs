# 任务计划: ASCII/Unicode 输出可读性改良(续)

> 续档说明:
> - 旧 `task_plan.md` 已超过 1000 行,按约定已归档为: `task_plan_2026-02-09.md`
> - 本文件从 2026-02-09 起继续记录后续计划与进展

## 目标
把用户复现图在 `beautiful-mermaid-rs --ascii`(Unicode box-drawing) 下的输出,从“能渲染”提升到“人类可读”:

- 边从 box 最近侧边出/入线,避免“背向端口”导致的大外圈绕行
- 尽量避免把线画到画布最外圈形成“外框”
- 尽量避免 `┬/┴/├/┤/┼` 造成的误连线错觉
- 在不牺牲 CLI 性能的前提下(QuickJS 无 JIT),保持结果稳定可回归(有测试兜底)

## 阶段
- [x] 阶段1: 复现与量化指标
- [x] 阶段2: 方案设计(两条路线)
- [x] 阶段3: 落地实现(优先改良,少新增)
- [x] 阶段4: 测试与验收(含 golden 更新)
- [x] 阶段5: 记录沉淀(四文件 + continuous-learning)

## 关键问题
1. “糟糕”的根因是哪一类?
   - 当前判断: 主要是 **backward edge(在 TD 下向上走的边)** 被错误地选到“背向端口”,从而被迫绕外圈。
2. 我们是否允许改变上游 TS baseline?
   - 允许。因为现在输出已回到 TS 基线,但用户仍不满意,下一步必然是“改良算法”,同步 vendor 并更新 golden。

## 两种实现路线(需要先定,避免反复摇摆)
1. 路线A(不惜代价/最佳方案,推荐): 改良 relaxed 路由策略,让候选端口组合更贴近“最近侧边”直觉
   - 核心动作(优先低风险项):
     - relaxed 候选集增加“朝向过滤”: 能不背向就绝不背向(必要时再降级)
     - relaxed backward edge 端口组合改良: 允许 Up/Down/Left/Right 组合,避免被固定到 Right 侧
     - 只在必要时再考虑: 调整 edge routing 顺序(确定性排序),降低“最后一条边被挤到外圈”的概率
   - 代价: 需要同步 vendor bundle,并更新部分 golden

2. 路线B(先能用/后面再优雅): 增加 CLI 级别的“可读性开关”
   - 例: `--hide-edge-labels` / `--compact-labels(编号+legend)` / `--routing=relaxed|strict`
   - 优点: 用户可立即通过参数选出更可读的版本
   - 缺点: 这是新增功能面,需要文档+更多测试,且会分散主线精力

## 做出的决定
- [决定] 先走路线A 的“朝向过滤 + backward 端口组合改良”。
  - 理由: 属于算法层面的改良,不引入新 CLI 面,也更符合你强调的“改良胜过新增”。

## 状态
**已完成阶段5** - 变更已沉淀到四文件,并且 `cargo test --release` 通过。

## 进展日志

### 2026-02-09 16:19:24 - 阶段1: 复现与量化(进行中)

- 我为什么先做这个:
  - 你现在说“结果还是很糟糕”,这已经不是“测试不通过”的问题,而是“审美/可读性”问题。
  - 不把问题量化成可回归的指标,后面只能靠感觉盲调,很容易越修越乱。
- 我做了什么:
  - 复现你的原始 Mermaid(用 heredoc 保证输入干净):
    - `cat <<'EOF' | beautiful-mermaid-rs --ascii ... EOF`
  - 用 `cargo run --release --example debug_user_case_meta` 打印 meta:
    - 读取 node box(x,y,width,height)
    - 读取每条 edge 的 `path.first()/path.last()` 坐标
- 关键证据(定量):
  - 节点布局呈“两列极端分离”:
    - 左列: Start/Hat_ralph/Hat_experiment_runner/Complete
    - 右列: Hat_experiment_integrator/Hat_experiment_auditor
  - 多条 backward edge(在 TD 下向上走)出现“背向端口”:
    - 例如 `Hat_experiment_integrator -> Hat_ralph (integration.rejected)`:
      - `first.x=99`(贴着 integrator 的右侧边),而 ralph 在左侧
      - `last.x=6`(贴着 ralph 的左侧边),而 integrator 在右侧
    - 这会直接导致你看到的“绕外圈画大框”。
- 下一步(进入阶段2/3):
  - 路线A 的最小改良落点:
    1) relaxed 候选端口做“朝向过滤”(能不背向就不背向,无可用候选再降级)
    2) relaxed backward edge 的 base 候选对(Up/Down/Left/Right)改为按 dx/dy 生成,避免被固定到 Right 侧

### 2026-02-09 16:35:40 - 阶段3/4: 采用“双向边下沉布局”消除外圈绕行(已完成)

- 我为什么改布局,而不是继续硬调 A*:
  - 复现图的本质是“同一父节点的多个 child 横向铺开 + 存在双向边”,导致 backward edge 很长,
    在“不共线重叠”的规则下容易被挤到外圈。
  - 继续加惩罚项只能让 A* 更难选,但不解决“几何结构”本身的问题。
- 我做了什么(上游 TS):
  - `src/ascii/grid.ts`:
    - 在 TD + Unicode relaxed 下,如果 parent 与 child 存在反向边(child->parent),
      则把该 child 下沉到下一层(childLevel + gridStep),并优先与 parent 对齐同一列(x=parent.x)。
    - 目标是把双向关系从“左右跨列”变成“垂直回路”,显著减少外圈绕行。
  - 同步 bundle 到本仓库 vendor:
    - `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
- Rust 侧回归更新:
  - `tests/ascii_endpoint_alignment.rs`:
    - 右侧贴边样例从 `experiment.complete` 调整为 `integration.blocked`(布局变化后更稳定)。
  - `tests/testdata/unicode/user_repro_case.txt`:
    - 更新 golden,锁死新布局输出(更可读版本)。
- 验证:
  - `cargo test --release` ✅ 全绿
  - `make install INSTALL_DIR=/Users/cuiluming/local_doc/l_dev/tool` ✅ 已覆盖安装最新 CLI
  - 复现你的命令: 输出不再画巨大外圈,节点更集中,边更短(可读性显著提升)。

### 2026-02-09 16:57:10 - 用户再次反馈“结果还是很糟糕”: 重新定位 label 扩列导致的画布膨胀(进行中)

- 我正在做什么,为什么:
  - 我先用当前已安装的 `beautiful-mermaid-rs` 复现你的命令,把“糟糕”落到可观察的形态(外框/空白/宽度)。
  - 复现输出里最显眼的问题不是“渲染失败”,而是“画布被拉得过宽 + 某些边被迫绕外圈”。
  - 我怀疑根因在 label 布局策略: 当前实现会在选定 `labelLine` 后,
    **无条件**把某一整列 `columnWidth` 拉到 `labelWidth+2`,
    即使该线段的“总宽度”本来已经足够放下 label。
  - 这会直接制造大量空白,也会放大 detour 的视觉成本,让边更像是在画“外框”。
- 我准备怎么改(优先改良,不新增 CLI 开关):
  1) 在上游 TS `determineLabelLine()` 里,把“无条件扩到 `labelWidth+2`”改成:
     - 只在当前线段总宽度不足时,按缺口做**最小增量**扩列。
  2) 保持 label 不裁剪(可逆自证不变),但显著减少无意义空白与外框概率。
- 验证计划:
  - 复现图输出: 宽度收敛,外框收敛,边更短更集中。
  - `scripts/sync-vendor-bundle.sh` 同步 bundle 后,跑 `cargo test --release` 确保回归全绿。

### 2026-02-09 17:12:40 - 继续改良: 路由顺序对“平行边/回边”可读性的决定性影响(准备实施)

- 我正在做什么,为什么:
  - 复现图里,`Hat_ralph -> Hat_experiment_integrator (integration.task)` 是一条“顺着 TD 向下走”的主干边,
    但在 Mermaid 文本里它出现在最后,导致当前实现按 insertion order 路由时,
    主干通道先被多条“向上回边”(integrator->ralph)占掉,主干边被迫走外圈画矩形。
  - 这类问题的本质不是 A* 算法不行,而是“先画谁”决定了通道占用:
    在我们仍然使用 hard rule 禁止 segment overlap 的前提下,路由顺序就是一等公民。
- 我准备怎么改(仍然不新增 CLI 开关,且只在 Unicode relaxed 启用以控影响面):
  1) `src/ascii/grid.ts` 路由阶段引入一个**稳定分区**顺序:
     - 先路由“顺着图方向”的 forward edges(TD: dy>0, LR: dx>0),
     - 再路由 neutral edges,
     - 最后路由 backward edges。
  2) 这样主干边更容易拿到直通路径,回边再选择 detour,整体更像人画的流程图。
- 验证计划:
  - 重点看复现图: `integration.task` 是否能从“大矩形”收敛为更短更直的通路。
  - 跑 `cargo test --release` 看 golden 漂移范围(预计只影响少量 Unicode relaxed 用例)。

### 2026-02-09 17:23:30 - 校准方案: 全局 forward-first 影响面过大,改为“仅延后回边”(准备实施)

- 我正在做什么,为什么:
  - 我刚才尝试了“全局 forward-first”分区路由,确实能让某些主干边更直,
    但副作用非常大: 其它边的通道占用被整体打散,复现图反而出现更大的外框/更宽的画布,甚至有 label 贴到 box 边框的风险。
  - 这说明“全局排序”太粗暴,会把局部问题扩散成全局扰动。
- 新策略(更小的手术刀):
  - 不把 forward edge 提到最前面。
  - 只做一件事: **当一条 backward edge 的反向边(forward)在输入顺序里更靠后时,延后这条 backward edge 的路由**。
  - 这样可以保证“先画主干,再画回边”,但不会打乱其它无关边的顺序。
- 验证计划:
  - 复现图重点看: `integration.task` 是否摆脱外圈大矩形,同时不引入新的大范围 detour。
  - 如 golden 漂移过大,立刻回滚该排序改动,只保留前一轮的 label 扩列改良。

### 2026-02-09 17:30:50 - 结论: “改路由顺序”会把局部问题扩散成全局外框,决定回滚

- 我观察到的现象:
  - 不论是全局 forward-first,还是“延后回边”,都会让复现图出现更大的外框矩形(甚至跑到 Start 上方),
    并且更容易出现 edge label 贴近/覆盖 box 边框的风险。
  - 这说明在当前 hard rule(禁止 segment overlap)框架下,单纯调换“先画谁”只是把痛点从一条边转移到另一条边,
    并不能降低整体 detour 的上限。
- 决策:
  - **回滚**所有“路由顺序”相关改动,恢复 insertion order。
  - 保留上一轮已经验证有效且影响面可控的改良: `determineLabelLine()` 的“最小增量扩列”。
- 下一步:
  - 在不改路由顺序的前提下,继续从“减少外圈 detour 的成本函数/约束”或“多边同端点的呈现策略”入手迭代。

### 2026-02-09 18:12:10 - 用户反馈: 依然很乱(合并线太多),要求“回滚重做方案”(进行中)

- 我正在做什么,为什么:
  - 你反馈的核心不是“渲染失败”,而是“人类读不懂逻辑关系”。
  - 这类问题通常有两层原因:
    1) 渲染器算法回归(同一张图,输出比以前更差)。
    2) 图本身是“多入边汇聚到单点”的结构,哪怕算法没回归,在终端 ASCII/Unicode 里也天然容易变成“线团”。
  - 所以我先把问题拆成两条线并行推进,避免再次盲调:
    - 线1: 用当前工作区版本复现你的 Mermaid,确认是否真的存在“回归”。
    - 线2: 给出“终端可读”的 Mermaid 写法(不新增 CLI 开关),把逻辑关系表达得更清晰。

- 两种方案方向(先写清楚,避免反复摇摆):
  1) 方向A(不惜代价/最佳方案,推荐): 把你的 flowchart 改写为更适合“多消息往返”的 `sequenceDiagram`
     - 目标: 用“参与者泳道 + 消息箭头”消除大量合并线,让终端输出天然更清爽。
     - 特点: 不需要改渲染器算法,也不需要新增 CLI 参数,只改 Mermaid 表达方式。
  2) 方向B(先能用/后面再优雅): 继续改良 flowchart 的走线策略,专门优化“多入边汇聚”类图
     - 可能手段:
       - 同一 target 的多条边,在 target 附近自动“分层进入”(减少同一 cell 的多重 junction)
       - 对“密集汇聚区”引入轻量级虚拟汇聚点(内部节点,不改语义),让线先汇聚再进 box
     - 风险: 算法改动影响面更大,需要更多 golden 更新与回归验证。

- 下一步(先做最小可验证闭环):
  1) 在本仓库里复现你给的 flowchart TD 输入,把当前 `--ascii` 输出保存为证据。
  2) 我会把同一逻辑改写成 `sequenceDiagram`,并用 `beautiful-mermaid-rs --ascii` 输出对比,确认是否达到“终端可读”。
  3) 如果 sequenceDiagram 明显更好,我们就把它沉淀成 `docs/terminal-readable-diagrams.md` 的推荐模板与示例。
  4) 如果 sequenceDiagram 仍然不够清晰,再回到方向B,做渲染器层面的改良与测试兜底。

### 2026-02-09 18:33:05 - 用户决定: 坚持一定要 flowchart(进入方向B)(进行中)

- 我正在做什么,为什么:
  - 你明确要求必须是 flowchart,这意味着我们不能靠“换图类型”来绕开物理限制。
  - 但我们仍然可以通过渲染器内部的“结构化改良”,把最糟糕的线团压下去:
    - 典型目标: 针对“同一对节点之间的多条带 label 的边”和“多入边汇聚到单点”的场景,减少外圈绕行与 `┼` 密度。
  - 我会优先选择“改良胜过新增”的手段:
    - 不新增 CLI 开关。
    - 尽量让改动只影响明确的病灶场景(多边汇聚),避免波及所有用例。

- 方案方向(两条,需要择一落地;我会先做风险更小的):
  1) 方案B1(优先落地/风险更小): 多重边(同 from/to)的“共享干线 + 分离 label”
     - 思路:
       - 多条边如果端点相同,允许它们共享一条 route(避免重复占用通道,减少 junction)。
       - label 不再强行挤到同一段线上,而是在干线附近做“纵向堆叠”(每条边一行),保持每个 label 仍然可见且短。
     - 优点: 不改变语义(仍然是多条边),只是路由/标注更聪明。
     - 风险: 需要动到 label 绘制/占位逻辑,要小心不把 node port 语义搞乱。

  2) 方案B2(备选/更激进): 在目标节点附近引入“虚拟汇聚点”(内部实现,不改变输入)
     - 思路:
       - 对入边过多的 target,在其附近找一个空白 cell 作为 hub,
         让多条边先汇聚到 hub,再用一条短边进 box。
     - 优点: 结构上最像人手画的“总线/汇聚”。
     - 风险: 需要引入新的布局约束,影响面更大。

- 我接下来会怎么做(可验证闭环):
  1) 在 TS 上游仓库里定位 flowchart ASCII/Unicode 的 edge routing + label placement 代码位置。
  2) 先实现 B1 的最小版本(只对同端点 multi-edge 启用),并用你的复现图做对照。
  3) 同步 vendor bundle 到本仓库,更新必要的 golden,补充一个“线团度量”的回归测试(比如 junction 数量阈值/画布宽度阈值)。
  4) `cargo test --release` 全绿后,再把新的终端输出贴给你确认是否达到“可读”。

### 2026-02-09 19:12:31 - 复现确认: “主干边最后路由”会把边挤成外框(进行中)

- 我正在做什么,为什么:
  - 你现在的反馈点很明确: 依然很乱,而且合并线太多,终端读不懂。
  - 我先用当前工作区版本严格复现你给的 Mermaid,避免“靠感觉”改一堆算法后仍然不对。
  - 复现的目的不是证明“能不能渲染”,而是定位哪一条边在制造最强的阅读噪声(典型就是外框)。

- 复现证据(当前实现,输入顺序不变):
  - `Hat_ralph -->|integration.task| Hat_experiment_integrator` 这条边因为出现在 Mermaid 文本最后,
    会在其它边已经占满通道后才开始路由,最终被迫绕到最外圈,形成一个看起来像“subgraph 外框”的大矩形。
  - 我把同一份 Mermaid 仅做“行顺序调整”(把 `integration.task` 提前),终端输出立刻收敛,
    说明这不是“画不出来”,而是“路由顺序在 hard rule 下决定了可读性”。

- 决策(回滚重做方案,不新增 CLI 开关):
  1) 放弃“让平行边共享干线”作为主线解法(它会让语义更难追踪,且对外框问题帮助有限)。
  2) 改为更贴近人类画图直觉的策略: **生成树主干边优先路由**。
     - 先把“把节点连起来的主干边”(覆盖所有节点的 spanning forest)路由出来,让主干尽量直、短、在内圈。
     - 再路由回边/补充边(例如多条结果回到协调者),允许它们为避让主干而绕一点。
  3) 这类改良不需要任何新参数,但会改变输出,因此要同步 vendor 并更新 golden,用测试锁死回归。

- 下一步(可验证闭环):
  - 在 TS 上游的 `src/ascii/grid.ts` 实现“spanning forest 优先”的 `edgesForRouting` 排序(仅对 Unicode relaxed 启用以控影响面)。
  - 同步 vendor bundle 回本仓库,跑 `cargo test --release`,必要时更新 golden。

### 2026-02-09 19:18:03 - 已落地: 生成树主干边优先路由,外框消失(已验证)

- 我做了什么(不新增 CLI 开关):
  - 上游 TS: `src/ascii/grid.ts` 把 `edgesForRouting` 从“严格按输入顺序”调整为:
    - Unicode + relaxed 下,先路由 spanning forest 的主干边,再路由剩余边(其余模式保持原样)。
  - Rust: 同步 vendor bundle,并更新 Unicode golden:
    - `tests/testdata/unicode/user_repro_case.txt`
  - Rust 测试加固:
    - `tests/ascii_endpoint_alignment.rs` 去掉“某条边必须从右侧进入”的不稳定断言,保留真正不变量: 所有入边箭头必须贴边。

- 为什么这样改能解决你看到的“外框 + 线团”:
  - 主干边(例如 `integration.task`)被提前路由后,会占住“最直/最短/最内圈”的通道,
    后续回边再做避让,就不会把整张图绕成一个大矩形外框。

- 验证:
  - 复现命令(与你一致):
    - `printf 'flowchart TD ...' | beautiful-mermaid-rs --ascii`
    - 现在 `integration.task` 不再画出顶层大矩形外框,整体收敛到内圈。
  - `cargo test --release` ✅
  - `make install` ✅ 已更新 `/Users/cuiluming/local_doc/l_dev/tool/beautiful-mermaid-rs`

### 2026-02-09 20:45:00 - 分析 `integration.blocked` 视觉多线段问题(进行中)

- 目标:
  - 解释为什么 `integration.blocked` 在语义上是一条边,但在 Unicode 图里看起来像多条线段。
  - 评估当前路由是否还有可继续优化的算法方向。

- 阶段:
  - [x] 阶段1: 复盘历史上下文与已有修复记录
  - [x] 阶段2: 读取当前走线实现并确认根因
  - [x] 阶段3: 给出可落地的优化方向和取舍
  - [x] 阶段4: 同步记录与对外说明

- 当前状态:
  - 已完成根因确认: 视觉“多线段”不是 `integration.blocked` 单边异常,而是 4 条 `Hat_experiment_integrator -> Hat_ralph` 平行边叠加 + comb ports 分 lane 的合成效果。
  - 已确认当前路由默认是 Unicode relaxed,会启用 spanning-forest-first 排序与终点段复用优先,因此平行边会尽量靠近主干走,更容易在视觉上形成“线束”。
  - 下一步如需继续优化,可走两条路线:
    - 最佳方案: 同端点多边做 bundle trunk + endpoint fanout 的分组路由,把“主干+分叉”作为一等对象优化。
    - 渐进方案: 增加多边密度惩罚和标签避让二次 reroute,先降低右侧聚团感,保持兼容性。

### 2026-02-09 21:08:00 - 执行用户选择: 最佳方案(进行中)

- 用户决策:
  - 采用“最佳方案”。
  - 同时要求 label 支持上下堆叠,不要水平拼接在一起。

- 实施阶段:
  - [x] 阶段1: 在 TS 上游实现 bundle trunk 路由(同 from/to 多边共主干)
  - [x] 阶段2: 在绘制层实现 bundle label 纵向堆叠(避免横向拼接)
  - [x] 阶段3: 同步 vendor 到 Rust 仓库并更新必要 golden
  - [x] 阶段4: 跑测试并回归复现图验证

- 当前状态:
  - 已完成最佳方案落地并通过 Rust 全量测试:
    - `cargo test` 全部通过。
  - 用户复现图验证完成:
    - 同端点多边现在共享主干,标签按纵向堆叠展示,不再横向拼接。
  - 过程中顺手修复了上游 TS 的运行时错误:
    - `src/ascii/pathfinder.ts` 中 `segmentPairMulti` / `segmentPair` 局部变量缺失导致 relaxed 路由 ReferenceError。
