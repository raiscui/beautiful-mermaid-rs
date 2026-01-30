// ============================================================================
// 错误类型
//
// 设计目标：
// - Rust 侧对外暴露清晰的错误信息
// - 内部使用 rquickjs 执行打包后的 JS bundle，所有 JS 异常都统一收口
// ============================================================================

use thiserror::Error;

/// 本 crate 的统一 Result 类型。
pub type Result<T> = std::result::Result<T, BeautifulMermaidError>;

/// beautiful-mermaid-rs 的错误枚举。
#[derive(Debug, Error)]
pub enum BeautifulMermaidError {
    /// JS 引擎（QuickJS）在 eval / 调用 / Promise 执行过程中产生的错误。
    #[error("JS 引擎错误: {0}")]
    Js(#[from] rquickjs::Error),

    /// JS bundle 初始化失败（比如 bundle 文件缺失或 eval 失败）。
    #[error("初始化失败: {message}")]
    Init { message: String },
}
