// ============================================================================
// 回归测试：目标端口列被扩宽时，箭头终点仍需贴边
//
// 背景：
// - 某些 flowchart 会把目标节点附近的端口列扩宽（例如长 label 导致列宽增加）；
// - 之前会出现“箭头停在远处，和 box 脱离”的视觉错位。
// ============================================================================

use beautiful_mermaid_rs::{
    AsciiBox, AsciiDrawingCoord, AsciiRenderOptions, render_mermaid_ascii_with_meta,
};

/// 判断一个点是否“贴着 box 外侧一格”。
///
/// 说明：
/// - 箭头不会直接写在边框格上，而是写在边框外一格；
/// - 因此我们断言的是“外侧一格邻接关系”。
fn is_adjacent_to_box(box_rect: AsciiBox, point: AsciiDrawingCoord) -> bool {
    let min_x = box_rect.x;
    let min_y = box_rect.y;
    let max_x = box_rect.x + box_rect.width - 1;
    let max_y = box_rect.y + box_rect.height - 1;

    let right_side = point.x == max_x + 1 && point.y >= min_y && point.y <= max_y;
    let left_side = point.x == min_x - 1 && point.y >= min_y && point.y <= max_y;
    let top_side = point.y == min_y - 1 && point.x >= min_x && point.x <= max_x;
    let bottom_side = point.y == max_y + 1 && point.x >= min_x && point.x <= max_x;

    right_side || left_side || top_side || bottom_side
}

/// 判断一个点是否落在 box 的边框上。
fn is_on_box_border(box_rect: AsciiBox, point: AsciiDrawingCoord) -> bool {
    let min_x = box_rect.x;
    let min_y = box_rect.y;
    let max_x = box_rect.x + box_rect.width - 1;
    let max_y = box_rect.y + box_rect.height - 1;

    let on_left_or_right =
        (point.x == min_x || point.x == max_x) && point.y >= min_y && point.y <= max_y;
    let on_top_or_bottom =
        (point.y == min_y || point.y == max_y) && point.x >= min_x && point.x <= max_x;

    on_left_or_right || on_top_or_bottom
}

#[test]
fn arrowheads_to_ralph_remain_box_adjacent_in_user_repro_case() {
    let mermaid = r#"flowchart TD
Hat_ralph["ralph#1 (coordinator)"]
Hat_experiment_auditor[<0001f9fe> 结果审计员]
Hat_experiment_integrator[<0001f9e9> 集成验收员]
Hat_experiment_runner[<0001f9ea> 实验执行器]
Start[task.start]
Start --> Hat_ralph
Complete[complete]
Hat_experiment_auditor -->|experiment.reviewed| Hat_ralph
Hat_experiment_integrator -->|experiment.complete| Complete
Hat_experiment_integrator -->|experiment.complete| Hat_ralph
Hat_experiment_integrator -->|integration.applied| Hat_ralph
Hat_experiment_integrator -->|integration.blocked| Hat_ralph
Hat_experiment_integrator -->|integration.rejected| Hat_ralph
Hat_experiment_runner -->|experiment.result| Hat_experiment_auditor
Hat_ralph -->|experiment.task| Hat_experiment_runner
Hat_ralph -->|integration.task| Hat_experiment_integrator
"#;

    let result = render_mermaid_ascii_with_meta(
        mermaid,
        &AsciiRenderOptions {
            use_ascii: Some(false),
            ..Default::default()
        },
    )
    .expect("render_mermaid_ascii_with_meta should work");

    let ralph = result
        .meta
        .nodes
        .iter()
        .find(|node| node.id == "Hat_ralph")
        .expect("meta should include Hat_ralph node");

    let edges_to_ralph: Vec<_> = result
        .meta
        .edges
        .iter()
        .filter(|edge| edge.to == "Hat_ralph")
        .collect();
    assert!(
        !edges_to_ralph.is_empty(),
        "repro case should contain edges targeting Hat_ralph"
    );

    for edge in &edges_to_ralph {
        let last = edge.path.last().expect("edge.path should not be empty");
        assert!(
            is_adjacent_to_box(ralph.box_rect, *last),
            "edge {} -> {} ({}) should end adjacent to Hat_ralph box, last={last:?}, box={:?}",
            edge.from,
            edge.to,
            edge.label,
            ralph.box_rect
        );
    }

    // 可选调试:
    // - 当断言失败时,打印坐标与 box,便于快速判断“到底落在了哪条边”。
    // - 用法:
    //   `BM_DEBUG_ENDPOINT_ALIGNMENT=1 cargo test --test ascii_endpoint_alignment -- --nocapture`
    if std::env::var("BM_DEBUG_ENDPOINT_ALIGNMENT").is_ok() {
        let max_x = ralph.box_rect.x + ralph.box_rect.width - 1;
        let max_y = ralph.box_rect.y + ralph.box_rect.height - 1;
        eprintln!("[debug] edges_to_ralph last coords:");
        for edge in &edges_to_ralph {
            let last = edge.path.last().expect("edge.path should not be empty");
            let side = if last.x == max_x + 1 {
                "right"
            } else if last.x == ralph.box_rect.x - 1 {
                "left"
            } else if last.y == ralph.box_rect.y - 1 {
                "top"
            } else if last.y == max_y + 1 {
                "bottom"
            } else {
                "unknown"
            };
            eprintln!("  - {}: last={last:?}, side={side}", edge.label);
        }
        eprintln!(
            "[debug] ralph box={:?} (max_x={}, max_y={})",
            ralph.box_rect, max_x, max_y
        );
    }

    let edges_from_ralph: Vec<_> = result
        .meta
        .edges
        .iter()
        .filter(|edge| edge.from == "Hat_ralph")
        .collect();
    assert!(
        !edges_from_ralph.is_empty(),
        "repro case should contain outgoing edges from Hat_ralph"
    );

    for edge in &edges_from_ralph {
        let first = edge.path.first().expect("edge.path should not be empty");
        assert!(
            is_on_box_border(ralph.box_rect, *first),
            "edge {} -> {} ({}) should start on Hat_ralph border, first={first:?}, box={:?}",
            edge.from,
            edge.to,
            edge.label,
            ralph.box_rect
        );
    }
}
