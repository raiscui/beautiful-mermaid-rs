// ============================================================================
// Unicode 节点 ID 冒烟测试
//
// 目的：
// - 防止回归：Flowchart/State 的节点 ID 使用中文/Unicode 时，不能再出现 -Infinity 或空白输出
// - 这是 Rust 侧的“端到端验证”：最终跑的是 vendor 的 JS bundle，而不是 TS 源码
// ============================================================================

use beautiful_mermaid_rs::{
    AsciiRenderOptions, RenderOptions, render_mermaid, render_mermaid_ascii,
};

// ============================================================================
// 终端“显示宽度”（简化版 wcwidth）
//
// 目的：
// - Rust 侧也要能验证“中文宽字符不会把边框顶出去”。
// - 我们用与上游 TS 侧相同的宽度模型做断言：每一行的显示宽度应一致。
// ============================================================================

fn is_combining_mark(code_point: u32) -> bool {
    matches!(
        code_point,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE20..=0xFE2F
    )
}

fn is_wide_code_point(code_point: u32) -> bool {
    // 参考：上游 TS 的实现（只取“够用且可预测”的常见范围）
    (0x1100..=0x115F).contains(&code_point)
        || (0x2E80..=0xA4CF).contains(&code_point)
        || (0xAC00..=0xD7A3).contains(&code_point)
        || (0xF900..=0xFAFF).contains(&code_point)
        || (0xFE10..=0xFE19).contains(&code_point)
        || (0xFE30..=0xFE6F).contains(&code_point)
        || (0xFF00..=0xFF60).contains(&code_point)
        || (0xFFE0..=0xFFE6).contains(&code_point)
        || (0x1F300..=0x1FAFF).contains(&code_point)
        || (0x1F900..=0x1F9FF).contains(&code_point)
}

fn char_display_width(ch: char) -> usize {
    let code_point = ch as u32;

    // 控制字符：宽度视为 0（避免把它们当成可见列宽）
    if code_point == 0 {
        return 0;
    }
    if code_point < 32 || (0x7F..0xA0).contains(&code_point) {
        return 0;
    }

    if is_combining_mark(code_point) {
        return 0;
    }
    if is_wide_code_point(code_point) {
        return 2;
    }
    1
}

fn text_display_width(text: &str) -> usize {
    text.chars().map(char_display_width).sum()
}

#[test]
fn svg_supports_unicode_node_ids() {
    let diagram = "graph TD\n开始 --> 结束\n";

    let svg = render_mermaid(diagram, &RenderOptions::default()).expect("SVG 渲染应当成功");

    // 之前的失败模式：空图进入 layout，最终把 viewBox/width/height 算成 -Infinity
    assert!(
        !svg.contains("-Infinity"),
        "SVG 不应包含 -Infinity（这意味着布局计算失败）"
    );
    assert!(
        svg.contains(">开始</text>"),
        "SVG 应渲染中文节点 label：开始"
    );
    assert!(
        svg.contains(">结束</text>"),
        "SVG 应渲染中文节点 label：结束"
    );
}

#[test]
fn ascii_supports_unicode_node_ids() {
    let diagram = "graph TD\n开始 --> 结束\n";

    let output = render_mermaid_ascii(
        diagram,
        &AsciiRenderOptions {
            use_ascii: Some(false),
            ..Default::default()
        },
    )
    .expect("ASCII/Unicode 渲染应当成功");

    // 之前的失败模式：只输出一个空格
    assert!(!output.trim().is_empty(), "ASCII 输出不应为空白");
    assert!(output.contains("开始"), "ASCII 输出应包含中文节点：开始");
    assert!(output.contains("结束"), "ASCII 输出应包含中文节点：结束");

    // 额外兜底：每一行的“终端显示宽度”应当一致
    //（否则就会出现你看到的“边框出去/不齐”）
    let mut widths = output.lines().map(text_display_width);
    let first = widths.next().expect("输出至少应有一行");
    assert!(
        widths.all(|w| w == first),
        "每一行显示宽度应一致（否则说明宽字符宽度处理回归）\n{output}"
    );
}
