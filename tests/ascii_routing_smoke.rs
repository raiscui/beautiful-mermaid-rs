use beautiful_mermaid_rs::{AsciiRenderOptions, AsciiRouting};

// ============================================================================
// CLI/库的“routing”选项回归测试
//
// 背景:
// - JS bundle 支持 `routing: strict|relaxed`，用于控制边路由的约束与代价函数。
// - 这两种模式没有绝对“更好”，最终可读性取决于图结构与个人偏好。
//
// 我们在这里做两件事:
// 1) 确认 strict/relaxed 都能稳定渲染(不 panic,不报错)。
// 2) 确认默认值(未显式传 routing)与显式 relaxed 等价,并且 strict 会产生可观测差异。
//
// 说明:
// - 我们用 `render_mermaid_ascii_with_meta`，避免用字符画做脆弱匹配。
// ============================================================================

#[test]
fn routing_default_is_relaxed_and_strict_is_distinct() {
    // 这个用例刻意包含多条边,并包含 subgraph,以稳定触发路由差异。
    let diagram = r#"
flowchart TD
  %% 说明:
  %% - 该图用于测试 strict/relaxed 的路由差异(不评价好坏)。
  %% - 节点名写短一些,避免文字过长主导布局。
  subgraph B["Build(tree_build)"]
    Shaping["Shaping(AnimationE)"] --> Bind["bind props.watch -> layout"]
  end
  subgraph T["Tick(per frame when running)"]
    O["Orders::process_after_render_queue(now_ms)"] --> GE["global_elapsed.set(now)"]
    GE --> AF["after_fn(insert_after_fn_in_topo)"]
    AF -->|if running| RV["resolve_steps(dt)"]
    RV --> P["props update"]
    RV --> I["interruption queue update"]
  end
  Bind --> L["edge.layout.* (e.g. w)"]
  P -->|watch/cast_into| L
  I --> R["sa_running(bool)"] --> G["global_anima_running_sa()"]
  G -->|true: schedule next frame| O
"#;

    let default_relaxed = beautiful_mermaid_rs::render_mermaid_ascii_with_meta(
        diagram,
        &AsciiRenderOptions::default(),
    )
    .expect("默认渲染不应失败");

    let explicit_relaxed = beautiful_mermaid_rs::render_mermaid_ascii_with_meta(
        diagram,
        &AsciiRenderOptions {
            routing: Some(AsciiRouting::Relaxed),
            ..Default::default()
        },
    )
    .expect("显式 relaxed 渲染不应失败");

    let strict = beautiful_mermaid_rs::render_mermaid_ascii_with_meta(
        diagram,
        &AsciiRenderOptions {
            routing: Some(AsciiRouting::Strict),
            ..Default::default()
        },
    )
    .expect("显式 strict 渲染不应失败");

    // JS bundle 侧默认值: useAscii=false 时 routing 默认 relaxed。
    assert_eq!(
        default_relaxed, explicit_relaxed,
        "默认 routing 应与显式 relaxed 等价"
    );

    // strict 与 relaxed 的策略不同,该图应产生可观测差异。
    assert_ne!(default_relaxed, strict, "strict 应该能改变渲染结果");
}
