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
    AsciiRenderOptions, AsciiRenderWithMeta, RenderOptions,
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
