// ============================================================================
// beautiful-mermaid-rs — Rust 公共 API
//
// 目标：
// - 对外提供与 TS 版 `beautiful-mermaid` 等价的核心能力
// - 当前实现策略：内嵌 QuickJS 执行打包后的 JS bundle（作为“完整复刻”基线）
// ============================================================================

mod error;
mod js;
mod native_pathfinder;
pub mod theme;
pub mod types;

pub use error::{BeautifulMermaidError, Result};
pub use types::{
    AsciiBox, AsciiDrawingCoord, AsciiRenderMeta, AsciiRenderMetaEdge, AsciiRenderMetaNode,
    AsciiRenderOptions, AsciiRenderWithMeta, AsciiRouting, MermaidValidation, RenderOptions,
};

/// 渲染 Mermaid -> SVG（阻塞）。
///
/// 说明：
/// - TS 版 `renderMermaid()` 是 async（返回 Promise）
/// - Rust 版这里会在内部同步等待 Promise 完成，然后返回 SVG 字符串
pub fn render_mermaid(text: &str, options: &RenderOptions) -> Result<String> {
    js::with_js_engine(|engine| engine.render_mermaid_svg(text, options))
}

/// 渲染 Mermaid -> ASCII/Unicode（阻塞，同步）。
pub fn render_mermaid_ascii(text: &str, options: &AsciiRenderOptions) -> Result<String> {
    js::with_js_engine(|engine| engine.render_mermaid_ascii(text, options))
}

/// 渲染 Mermaid -> ASCII/Unicode + meta（阻塞，同步）。
///
/// 说明：
/// - `text` 字段等价于 `render_mermaid_ascii(...)` 的输出；
/// - `meta` 提供 node/edge 在字符画上的坐标信息，便于上层 UI 做高亮/动画。
pub fn render_mermaid_ascii_with_meta(
    text: &str,
    options: &AsciiRenderOptions,
) -> Result<AsciiRenderWithMeta> {
    js::with_js_engine(|engine| engine.render_mermaid_ascii_with_meta(text, options))
}

/// 校验 Mermaid 语法是否有效（阻塞，同步）。
///
/// 返回值约定：
/// - `Ok(MermaidValidation { is_valid: true, .. })`：有效
/// - `Ok(MermaidValidation { is_valid: false, .. })`：无效（无法被 parser 解析）
///
/// 实现说明：
/// - 当前版本使用纯 Rust 的 `selkie::parse` 做语法校验（不依赖 Node）。
/// - 当前实现不会返回 `Err`；保留 `Result` 只是为了未来可替换后端时仍能表达“内部错误”。
pub fn validate_mermaid(text: &str) -> Result<MermaidValidation> {
    // --------------------------------------------------------------------
    // 这里把“空输入”视为“无效 Mermaid”而不是内部错误：
    // - 调用方可以用 `is_valid` 统一处理；
    // - CLI 层则可以把它归类为“用法错误”(exit code=2)。
    // --------------------------------------------------------------------
    if text.trim().is_empty() {
        return Ok(MermaidValidation {
            is_valid: false,
            error: Some("输入为空".to_string()),
            details: None,
        });
    }

    match selkie::parse(text) {
        Ok(_diagram) => Ok(MermaidValidation {
            is_valid: true,
            error: None,
            details: None,
        }),
        Err(err) => Ok(MermaidValidation {
            is_valid: false,
            error: Some(err.to_string()),
            details: Some(format!("{err:?}")),
        }),
    }
}
