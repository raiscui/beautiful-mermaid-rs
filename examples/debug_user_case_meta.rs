// ============================================================================
// 调试工具: 渲染用户复现图(ASCII/Unicode)并打印 meta 端点信息。
//
// 为什么要有这个 example:
// - CLI 输出只有文本,当出现“箭头不贴边/线进了 box 里”时,很难定量定位。
// - `render_mermaid_ascii_with_meta` 会返回 node box 与 edge path 的坐标,
//   我们可以直接检查每条边的起点/终点是否符合不变量。
//
// 用法:
//   cargo run --release --example debug_user_case_meta -- --ascii
//   cargo run --release --example debug_user_case_meta
//
// 说明:
// - `--ascii` 表示纯 ASCII 字符集(兼容性最好)。
// - 不带参数时使用 Unicode box-drawing(更好看,也更贴近用户的 `--ascii` 行为)。
// ============================================================================

use beautiful_mermaid_rs::{AsciiDrawingCoord, AsciiRenderOptions, render_mermaid_ascii_with_meta};

/// 判断一个点是否“贴着 box 外侧一格”。
fn is_adjacent_to_box(box_rect: beautiful_mermaid_rs::AsciiBox, point: AsciiDrawingCoord) -> bool {
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

fn main() {
    let use_ascii = std::env::args().any(|arg| arg == "--ascii");

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
            use_ascii: Some(use_ascii),
            ..Default::default()
        },
    )
    .expect("render_mermaid_ascii_with_meta should work");

    // 先输出文本,便于肉眼对照。
    print!("{}", result.text);

    // 再输出 meta,便于定量定位问题。
    eprintln!("\n[meta] nodes:");
    for node in &result.meta.nodes {
        eprintln!(
            "  - id={} box={:?} label={:?}",
            node.id, node.box_rect, node.label
        );
    }

    eprintln!("\n[meta] edges (first/last):");
    for edge in &result.meta.edges {
        let first = edge
            .path
            .first()
            .copied()
            .unwrap_or(AsciiDrawingCoord { x: -1, y: -1 });
        let last = edge
            .path
            .last()
            .copied()
            .unwrap_or(AsciiDrawingCoord { x: -1, y: -1 });

        let target_box = result
            .meta
            .nodes
            .iter()
            .find(|n| n.id == edge.to)
            .map(|n| n.box_rect);
        let adjacent = target_box
            .map(|b| is_adjacent_to_box(b, last))
            .unwrap_or(false);

        // =====================================================================
        // 统计路径长度与包围盒,用于定位“绕远/外框”问题
        // =====================================================================
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for point in &edge.path {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        eprintln!(
            "  - {} -> {} ({}) len={} bbox=({min_x},{min_y})-({max_x},{max_y}) first={first:?} last={last:?} target_adjacent={adjacent}",
            edge.from,
            edge.to,
            edge.label,
            edge.path.len()
        );
    }
}
