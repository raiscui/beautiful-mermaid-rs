// ============================================================================
// 对外类型：渲染选项
//
// 这里的字段名尽量保持与 TypeScript 版 `beautiful-mermaid` 的 options 对齐：
// - 颜色：bg/fg/line/accent/muted/surface/border
// - 版式：font/padding/nodeSpacing/layerSpacing/transparent
// - ASCII：useAscii/paddingX/paddingY/boxBorderPadding
// ============================================================================

use serde::{Deserialize, Serialize};

/// SVG 渲染参数（对齐 TS: `RenderOptions`）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RenderOptions {
    // --------------------------------------------------------------------
    // 颜色（CSS variables）
    // --------------------------------------------------------------------
    /// 背景色，对应 CSS 变量 `--bg`。
    pub bg: Option<String>,
    /// 前景色/主文字色，对应 CSS 变量 `--fg`。
    pub fg: Option<String>,

    /// 连接线颜色，对应 CSS 变量 `--line`。
    pub line: Option<String>,
    /// 强调色（箭头、强调元素），对应 CSS 变量 `--accent`。
    pub accent: Option<String>,
    /// 次要文字/标签色，对应 CSS 变量 `--muted`。
    pub muted: Option<String>,
    /// 节点填充/面色，对应 CSS 变量 `--surface`。
    pub surface: Option<String>,
    /// 边框色，对应 CSS 变量 `--border`。
    pub border: Option<String>,

    // --------------------------------------------------------------------
    // 版式
    // --------------------------------------------------------------------
    /// 字体族名（默认 TS 是 "Inter"）。
    pub font: Option<String>,
    /// 画布 padding（单位 px）。
    pub padding: Option<f64>,
    /// 同层节点水平间距。
    pub node_spacing: Option<f64>,
    /// 层与层之间的垂直间距。
    pub layer_spacing: Option<f64>,
    /// 是否透明背景（true 时 SVG 不画背景）。
    pub transparent: Option<bool>,
}

/// ASCII/Unicode 渲染参数（对齐 TS: `AsciiRenderOptions`）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AsciiRenderOptions {
    /// true = 纯 ASCII（+ - | >），false = Unicode 线条（┌ ─ │ ►）。
    pub use_ascii: Option<bool>,
    /// 节点水平间距。
    pub padding_x: Option<i32>,
    /// 节点垂直间距。
    pub padding_y: Option<i32>,
    /// 节点盒子内部边框 padding。
    pub box_border_padding: Option<i32>,
}
