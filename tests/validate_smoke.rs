// ============================================================================
// Mermaid validator 冒烟测试
//
// 目的:
// - 确认 `validate_mermaid(...)` 能稳定返回 true/false
// - 避免未来改动 validator 后端/规则时把校验能力悄悄弄坏
// ============================================================================

use beautiful_mermaid_rs::validate_mermaid;

#[test]
fn validate_mermaid_accepts_valid_diagram() {
    // --------------------------------------------------------------------
    // 选择一个最小但明确有效的 Mermaid:
    // - flowchart 是本仓库稳定支持的类型
    // --------------------------------------------------------------------
    let diagram = "flowchart LR\nA --> B\n";
    let result = validate_mermaid(diagram).expect("validate_mermaid 内部错误");
    assert!(result.is_valid, "期望有效 Mermaid, 但返回无效: {result:?}");
}

#[test]
fn validate_mermaid_rejects_invalid_diagram() {
    // --------------------------------------------------------------------
    // 明确的语法/语义错误:
    // - `B -->` 缺少目标节点
    // --------------------------------------------------------------------
    let diagram = "flowchart LR\nA --> B\nB -->\n";
    let result = validate_mermaid(diagram).expect("validate_mermaid 内部错误");
    assert!(!result.is_valid, "期望无效 Mermaid, 但返回有效: {result:?}");
    assert!(
        result.error.as_deref().unwrap_or_default().trim().len() > 0,
        "期望 error 字段包含可读信息, 实际为: {result:?}"
    );
}
