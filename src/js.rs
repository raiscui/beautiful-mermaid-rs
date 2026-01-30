// ============================================================================
// JS 运行时封装（QuickJS via rquickjs）
//
// 为什么要这样做？
// - 目标是“完整复刻 TS 版行为”，最快的方式是直接执行打包后的 JS bundle
// - Rust 侧只负责：初始化 JS 引擎、注入 bundle、把参数/返回值做类型转换
//
// 后续演进：
// - 可以在保持 Rust API 不变的前提下，把内部实现逐步替换为纯 Rust
// ============================================================================

use crate::error::{BeautifulMermaidError, Result};
use crate::types::{AsciiRenderOptions, RenderOptions};
use once_cell::unsync::OnceCell;
use rquickjs::{Context, Function, Object, Promise, Runtime};

/// 通过 `include_str!` 内嵌的 browser IIFE bundle（来自 TS 项目的 tsup 输出）。
const BEAUTIFUL_MERMAID_BUNDLE: &str =
    include_str!("../vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js");

thread_local! {
    /// 每个线程一个 JS 引擎实例：
    /// - QuickJS Context 不是线程安全的（也不应该跨线程共享）
    /// - 这样能避免频繁初始化带来的开销
    static JS_ENGINE: OnceCell<JsEngine> = OnceCell::new();
}

/// 在当前线程的 JS 引擎上执行一次操作。
pub fn with_js_engine<T>(f: impl FnOnce(&JsEngine) -> Result<T>) -> Result<T> {
    JS_ENGINE.with(|cell| {
        let engine = cell.get_or_try_init(JsEngine::new)?;
        f(engine)
    })
}

/// JS 引擎实例：包含 Runtime + Context，并在初始化时 eval bundle。
pub struct JsEngine {
    runtime: Runtime,
    context: Context,
}

impl JsEngine {
    fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        // ----------------------------------------------------------------
        // 初始化：把 browser IIFE bundle eval 进 Context
        // 这一步会创建全局对象 `beautifulMermaid`
        // ----------------------------------------------------------------
        context.with(|ctx| {
            ctx.eval::<(), _>(BEAUTIFUL_MERMAID_BUNDLE)
                .map_err(BeautifulMermaidError::from)
        })?;

        Ok(Self { runtime, context })
    }

    /// 渲染 Mermaid -> ASCII/Unicode（同步）。
    pub fn render_mermaid_ascii(&self, text: &str, options: &AsciiRenderOptions) -> Result<String> {
        let rendered = self.context.with(|ctx| -> Result<String> {
            let beautiful_mermaid: Object = ctx.globals().get("beautifulMermaid")?;
            let render_fn: Function = beautiful_mermaid.get("renderMermaidAscii")?;

            let js_options = Self::ascii_options_to_js(&ctx, options)?;
            let output: String = render_fn.call((text, js_options))?;
            Ok(output)
        })?;

        // 保守处理：把可能残留的 Promise job 队列清空，避免跨调用累积。
        self.drain_pending_jobs()?;

        Ok(rendered)
    }

    /// 渲染 Mermaid -> SVG（TS 版返回 Promise，这里同步等待）。
    pub fn render_mermaid_svg(&self, text: &str, options: &RenderOptions) -> Result<String> {
        let rendered = self.context.with(|ctx| -> Result<String> {
            let beautiful_mermaid: Object = ctx.globals().get("beautifulMermaid")?;
            let render_fn: Function = beautiful_mermaid.get("renderMermaid")?;

            let js_options = Self::render_options_to_js(&ctx, options)?;

            // TS 版 renderMermaid 是 async，这里拿到 Promise 并 finish 阻塞等待。
            let promise: Promise = render_fn.call((text, js_options))?;
            let output: String = promise.finish()?;
            Ok(output)
        })?;

        self.drain_pending_jobs()?;

        Ok(rendered)
    }

    fn drain_pending_jobs(&self) -> Result<()> {
        // ----------------------------------------------------------------
        // rquickjs 的 job queue 执行错误类型不是 `rquickjs::Error`，
        // 这里统一转成我们自己的初始化/运行时错误，避免把内部类型泄漏到 API 层。
        // ----------------------------------------------------------------
        while self.runtime.is_job_pending() {
            match self.runtime.execute_pending_job() {
                Ok(true) => continue,
                Ok(false) => break,
                Err(err) => {
                    return Err(BeautifulMermaidError::Init {
                        message: format!("执行 JS pending jobs 失败: {err}"),
                    });
                }
            }
        }
        Ok(())
    }

    // --------------------------------------------------------------------
    // Rust options -> JS object
    // --------------------------------------------------------------------

    fn render_options_to_js<'js>(
        ctx: &rquickjs::Ctx<'js>,
        options: &RenderOptions,
    ) -> Result<Object<'js>> {
        let object = Object::new(ctx.clone())?;

        // 颜色
        if let Some(value) = &options.bg {
            object.set("bg", value.as_str())?;
        }
        if let Some(value) = &options.fg {
            object.set("fg", value.as_str())?;
        }
        if let Some(value) = &options.line {
            object.set("line", value.as_str())?;
        }
        if let Some(value) = &options.accent {
            object.set("accent", value.as_str())?;
        }
        if let Some(value) = &options.muted {
            object.set("muted", value.as_str())?;
        }
        if let Some(value) = &options.surface {
            object.set("surface", value.as_str())?;
        }
        if let Some(value) = &options.border {
            object.set("border", value.as_str())?;
        }

        // 版式
        if let Some(value) = &options.font {
            object.set("font", value.as_str())?;
        }
        if let Some(value) = options.padding {
            object.set("padding", value)?;
        }
        if let Some(value) = options.node_spacing {
            object.set("nodeSpacing", value)?;
        }
        if let Some(value) = options.layer_spacing {
            object.set("layerSpacing", value)?;
        }
        if let Some(value) = options.transparent {
            object.set("transparent", value)?;
        }

        Ok(object)
    }

    fn ascii_options_to_js<'js>(
        ctx: &rquickjs::Ctx<'js>,
        options: &AsciiRenderOptions,
    ) -> Result<Object<'js>> {
        let object = Object::new(ctx.clone())?;

        if let Some(value) = options.use_ascii {
            object.set("useAscii", value)?;
        }
        if let Some(value) = options.padding_x {
            object.set("paddingX", value)?;
        }
        if let Some(value) = options.padding_y {
            object.set("paddingY", value)?;
        }
        if let Some(value) = options.box_border_padding {
            object.set("boxBorderPadding", value)?;
        }

        Ok(object)
    }
}
