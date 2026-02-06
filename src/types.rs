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

// ============================================================================
// Mermaid 语法校验（validator）
// ============================================================================
//
// 设计目标：
// - 提供一个“可机器消费”的校验输出：true/false + error/details；
// - 让 CLI/CI 能稳定判断 Mermaid 是否有效，并打印出可读的失败原因；
//
// 重要说明：
// - 当前实现是“纯语法校验”：
//   - 后端使用纯 Rust parser（`selkie::parse`）判断 Mermaid 是否可被解析。
//   - 它更适合做 CI gate, 以及在无 Node 环境下做快速检查。
// - 它不保证“本仓库渲染器一定能渲染”：
//   - 本仓库的渲染 JS bundle 目前只明确支持 Flowchart/State、Sequence、Class、ER。
//   - 语法有效 != 渲染一定成功（尤其是其他 Mermaid 图类型）。
// - 它也不保证与 Mermaid 官方 CLI（@mermaid-js/mermaid-cli）100% 一致：
//   - 但错误信息通常会带行列, 足够定位问题。

/// Mermaid 语法校验结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MermaidValidation {
    /// true 表示语法/语义被当前渲染器接受。
    pub is_valid: bool,
    /// 失败时的主错误信息（适合单行打印）。
    pub error: Option<String>,
    /// 失败时的细节信息（通常是 stack trace 或更长的上下文）。
    pub details: Option<String>,
}

// ============================================================================
// ASCII/Unicode 渲染 meta（给 TUI 上色/动画用）
// ============================================================================
//
// 设计目标：
// - 让上层 UI 能“稳定地”对 node box / edge stroke 做 cell-level 的高亮与动画；
// - 避免在 Rust 侧重新解析最终文本（那会非常脆弱，且难以处理宽字符/拐点/箭头等细节）。

/// 终端字符画上的坐标（以“终端 cell”为单位，而不是字符串字节/字符索引）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiDrawingCoord {
    pub x: i32,
    pub y: i32,
}

/// 终端字符画上的矩形区域（同样以“终端 cell”为单位）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// 节点（node box）的 meta：用于定位并高亮某个 box。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiRenderMetaNode {
    /// Mermaid node id（parser identity），例如 "Hat_planner"。
    pub id: String,
    /// 节点内部展示的 label 文本（可能包含 emoji/中文）。
    pub label: String,
    /// box 的矩形范围（坐标来自 TS 渲染器）。
    #[serde(rename = "box")]
    pub box_rect: AsciiBox,
}

/// 边（edge）的 meta：用于按 path 做逐段点亮动画。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiRenderMetaEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    /// edge stroke 的“有序坐标序列”（包含拐点/箭头等关键格子）。
    pub path: Vec<AsciiDrawingCoord>,
}

/// ASCII/Unicode 渲染的完整 meta（nodes + edges）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiRenderMeta {
    pub nodes: Vec<AsciiRenderMetaNode>,
    pub edges: Vec<AsciiRenderMetaEdge>,
}

/// ASCII/Unicode 渲染的输出：text + meta。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsciiRenderWithMeta {
    pub text: String,
    pub meta: AsciiRenderMeta,
}
