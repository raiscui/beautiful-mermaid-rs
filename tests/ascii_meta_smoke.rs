// ============================================================================
// meta API smoke test
//
// 目的：
// - 确认 vendor bundle 暴露了 `renderMermaidAsciiWithMeta`
// - 确认 Rust 侧 `render_mermaid_ascii_with_meta` 能拿到 text + meta
// - text 必须与旧 API 完全一致（避免引入渲染差异）
// ============================================================================

use beautiful_mermaid_rs::{
    AsciiRenderOptions, render_mermaid_ascii, render_mermaid_ascii_with_meta,
};

/// Whitespace 归一化（对齐 golden 测试的策略）：
/// - 每行 trimEnd（去掉行尾空格）
/// - 去掉首尾空行
fn normalize_whitespace(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized
        .split('\n')
        .map(|line| line.trim_end().to_string())
        .collect();

    while !lines.is_empty() && lines.first().is_some_and(|line| line.is_empty()) {
        lines.remove(0);
    }
    while !lines.is_empty() && lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

#[test]
fn render_mermaid_ascii_with_meta_returns_text_and_meta() {
    // 说明：
    // - 这里选用最小 flowchart 用例：2 个节点 + 1 条带 label 的边
    // - 目的是让 meta 至少包含 2 nodes + 1 edge，且 edge.path 非空
    let mermaid = "flowchart LR\nA[A]\nB[B]\nA -->|t| B\n";
    let options = AsciiRenderOptions {
        use_ascii: Some(false),
        ..Default::default()
    };

    let plain = render_mermaid_ascii(mermaid, &options).expect("render_mermaid_ascii should work");
    let with_meta = render_mermaid_ascii_with_meta(mermaid, &options)
        .expect("render_mermaid_ascii_with_meta should work");

    // text 必须严格一致（只忽略行尾空格与首尾空行差异）。
    assert_eq!(
        normalize_whitespace(&with_meta.text),
        normalize_whitespace(&plain)
    );

    // meta 至少能定位到 node/edge
    assert!(
        with_meta.meta.nodes.iter().any(|n| n.id == "A"),
        "meta.nodes should include node A"
    );
    assert!(
        with_meta.meta.nodes.iter().any(|n| n.id == "B"),
        "meta.nodes should include node B"
    );
    assert!(
        with_meta
            .meta
            .edges
            .iter()
            .any(|e| e.from == "A" && e.to == "B" && e.label == "t"),
        "meta.edges should include A -> B with label t"
    );

    let edge = with_meta
        .meta
        .edges
        .iter()
        .find(|e| e.from == "A" && e.to == "B" && e.label == "t")
        .expect("edge A->B should exist");
    assert!(
        !edge.path.is_empty(),
        "edge.path should include stroke coordinates for animation"
    );
}
