// ============================================================================
// SVG 渲染冒烟测试
//
// 目的：
// - 确认 QuickJS + JS bundle 的 SVG 渲染路径可用
// - 顺便验证 bg/fg 等选项能正确写入 SVG 的 CSS variables
// ============================================================================

use beautiful_mermaid_rs::{RenderOptions, render_mermaid};

#[test]
fn svg_render_smoke() {
    let diagram = "graph LR\nA --> B\n";
    let options = RenderOptions::default();

    let svg = render_mermaid(diagram, &options).expect("SVG 渲染应当成功");

    assert!(svg.starts_with("<svg"), "SVG 应以 <svg 开头");
    assert!(
        svg.contains("--bg:#FFFFFF") && svg.contains("--fg:#27272A"),
        "默认主题应写入 --bg/--fg"
    );
}

#[test]
fn svg_respects_custom_colors() {
    let diagram = "graph LR\nA --> B\n";
    let options = RenderOptions {
        bg: Some("#000000".to_string()),
        fg: Some("#FFFFFF".to_string()),
        ..Default::default()
    };

    let svg = render_mermaid(diagram, &options).expect("SVG 渲染应当成功");
    assert!(
        svg.contains("--bg:#000000") && svg.contains("--fg:#FFFFFF"),
        "自定义颜色应写入 --bg/--fg"
    );
}
