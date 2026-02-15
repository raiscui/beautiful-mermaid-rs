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
use crate::native_pathfinder::NativeAStar;
use crate::types::{AsciiRenderOptions, AsciiRenderWithMeta, RenderOptions};
use once_cell::unsync::OnceCell;
use rquickjs::FromJs;
use rquickjs::function::{FromParams, IntoJsFunc, ParamRequirement, Params};
use rquickjs::{Context, Exception, Function, IntoJs, Object, Promise, Runtime, TypedArray, Value};
use std::cell::RefCell;
use std::rc::Rc;

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

// ============================================================================
// Native pathfinder -> JS glue
//
// 说明：
// - rquickjs 的 `Function::new` 要求传入的类型实现 `IntoJsFunc<'js, P>`。
// - 对于带 `'js` 生命周期的参数（例如 `TypedArray<'js, u8>` / `Object<'js>`），
//   直接写闭包很容易陷入生命周期推断问题。
// - 因此这里用“小结构体 + 手写 IntoJsFunc”来稳定地把 native A* 暴露给 JS。
// ============================================================================

#[derive(Clone)]
struct NativeGetPathFn {
    astar: Rc<RefCell<NativeAStar>>,
}

impl<'js> IntoJsFunc<'js, (u32, u32, u32, u32, u32, TypedArray<'js, u8>)> for NativeGetPathFn {
    fn param_requirements() -> ParamRequirement {
        <(u32, u32, u32, u32, u32, TypedArray<'js, u8>)>::param_requirements()
    }

    fn call<'a>(&self, params: Params<'a, 'js>) -> rquickjs::Result<Value<'js>> {
        let ctx = params.ctx().clone();
        let (stride, from_idx, to_idx, max_x, max_y, blocked) =
            <(u32, u32, u32, u32, u32, TypedArray<'js, u8>)>::from_params(&mut params.access())?;

        let stride_usize = stride as usize;
        if stride_usize == 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPath",
                "stride 不能为 0",
            ));
        }

        let blocked_slice: &[u8] = blocked.as_ref();
        if blocked_slice.len() % stride_usize != 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPath",
                format!(
                    "blocked.len() 必须能被 stride 整除: blocked.len()={}, stride={stride}",
                    blocked_slice.len()
                ),
            ));
        }
        let height_usize = blocked_slice.len() / stride_usize;

        let path = self
            .astar
            .borrow_mut()
            .get_path(
                stride_usize,
                height_usize,
                from_idx,
                to_idx,
                max_x,
                max_y,
                blocked_slice,
            )
            .map_err(|message| {
                rquickjs::Error::new_from_js_message("native_pathfinder", "getPath", message)
            })?;

        match path {
            Some(path) => path.into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }
}

#[derive(Clone)]
struct NativeGetPathStrictFn {
    astar: Rc<RefCell<NativeAStar>>,
}

impl<'js> IntoJsFunc<'js, (u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>
    for NativeGetPathStrictFn
{
    fn param_requirements() -> ParamRequirement {
        <(u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>::param_requirements()
    }

    fn call<'a>(&self, params: Params<'a, 'js>) -> rquickjs::Result<Value<'js>> {
        let ctx = params.ctx().clone();
        let (stride, from_idx, to_idx, max_x, max_y, blocked, constraints) =
            <(u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>::from_params(
                &mut params.access(),
            )?;

        let stride_usize = stride as usize;
        if stride_usize == 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPathStrict",
                "stride 不能为 0",
            ));
        }

        let blocked_slice: &[u8] = blocked.as_ref();
        if blocked_slice.len() % stride_usize != 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPathStrict",
                format!(
                    "blocked.len() 必须能被 stride 整除: blocked.len()={}, stride={stride}",
                    blocked_slice.len()
                ),
            ));
        }
        let height_usize = blocked_slice.len() / stride_usize;

        // constraints: StrictPathConstraints（TS 侧对象）
        let segment_usage: Object<'js> = constraints.get("segmentUsage")?;

        let segment_used: TypedArray<'js, u8> = segment_usage.get("segmentUsed")?;
        let used_as_middle: TypedArray<'js, u8> = segment_usage.get("usedAsMiddle")?;
        let start_source: TypedArray<'js, u32> = segment_usage.get("startSource")?;
        let start_source_multi: TypedArray<'js, u8> = segment_usage.get("startSourceMulti")?;
        let end_target: TypedArray<'js, u32> = segment_usage.get("endTarget")?;
        let end_target_multi: TypedArray<'js, u8> = segment_usage.get("endTargetMulti")?;

        // usedPoints 在 TS 侧是可选字段：undefined/null 都视为 None
        let used_points: Option<TypedArray<'js, u8>> = constraints.get("usedPoints")?;
        let route_from_idx: u32 = constraints.get("routeFromIdx")?;
        let route_to_idx: u32 = constraints.get("routeToIdx")?;
        let edge_from_id: u32 = constraints.get("edgeFromId")?;
        let edge_to_id: u32 = constraints.get("edgeToId")?;

        let path = self
            .astar
            .borrow_mut()
            .get_path_strict(
                stride_usize,
                height_usize,
                from_idx,
                to_idx,
                max_x,
                max_y,
                blocked_slice,
                segment_used.as_ref(),
                used_as_middle.as_ref(),
                start_source.as_ref(),
                start_source_multi.as_ref(),
                end_target.as_ref(),
                end_target_multi.as_ref(),
                used_points.as_ref().map(|p| p.as_ref()),
                route_from_idx,
                route_to_idx,
                edge_from_id,
                edge_to_id,
            )
            .map_err(|message| {
                rquickjs::Error::new_from_js_message("native_pathfinder", "getPathStrict", message)
            })?;

        match path {
            Some(path) => path.into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }
}

#[derive(Clone)]
struct NativeGetPathRelaxedFn {
    astar: Rc<RefCell<NativeAStar>>,
}

impl<'js> IntoJsFunc<'js, (u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>
    for NativeGetPathRelaxedFn
{
    fn param_requirements() -> ParamRequirement {
        <(u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>::param_requirements()
    }

    fn call<'a>(&self, params: Params<'a, 'js>) -> rquickjs::Result<Value<'js>> {
        let ctx = params.ctx().clone();
        let (stride, from_idx, to_idx, max_x, max_y, blocked, constraints) =
            <(u32, u32, u32, u32, u32, TypedArray<'js, u8>, Object<'js>)>::from_params(
                &mut params.access(),
            )?;

        let stride_usize = stride as usize;
        if stride_usize == 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPathRelaxed",
                "stride 不能为 0",
            ));
        }

        let blocked_slice: &[u8] = blocked.as_ref();
        if blocked_slice.len() % stride_usize != 0 {
            return Err(rquickjs::Error::new_from_js_message(
                "native_pathfinder",
                "getPathRelaxed",
                format!(
                    "blocked.len() 必须能被 stride 整除: blocked.len()={}, stride={stride}",
                    blocked_slice.len()
                ),
            ));
        }
        let height_usize = blocked_slice.len() / stride_usize;

        // constraints: StrictPathConstraints（TS 侧对象）
        let segment_usage: Object<'js> = constraints.get("segmentUsage")?;

        let segment_used: TypedArray<'js, u8> = segment_usage.get("segmentUsed")?;
        let used_as_middle: TypedArray<'js, u8> = segment_usage.get("usedAsMiddle")?;
        let segment_pair: TypedArray<'js, u32> = segment_usage.get("segmentPair")?;
        let segment_pair_multi: TypedArray<'js, u8> = segment_usage.get("segmentPairMulti")?;
        let start_source: TypedArray<'js, u32> = segment_usage.get("startSource")?;
        let start_source_multi: TypedArray<'js, u8> = segment_usage.get("startSourceMulti")?;
        let end_target: TypedArray<'js, u32> = segment_usage.get("endTarget")?;
        let end_target_multi: TypedArray<'js, u8> = segment_usage.get("endTargetMulti")?;

        // usedPoints 在 TS 侧是可选字段：undefined/null 都视为 None
        let used_points: Option<TypedArray<'js, u8>> = constraints.get("usedPoints")?;
        let route_from_idx: u32 = constraints.get("routeFromIdx")?;
        let route_to_idx: u32 = constraints.get("routeToIdx")?;
        let edge_from_id: u32 = constraints.get("edgeFromId")?;
        let edge_to_id: u32 = constraints.get("edgeToId")?;

        // relaxed 专用：仅在 fallback（不可达）时才会打开
        let allow_end_segment_reuse: Option<bool> =
            constraints.get("relaxedAllowEndSegmentReuse")?;
        let allow_end_segment_reuse = allow_end_segment_reuse.unwrap_or(false);

        let result = self
            .astar
            .borrow_mut()
            .get_path_relaxed(
                stride_usize,
                height_usize,
                from_idx,
                to_idx,
                max_x,
                max_y,
                blocked_slice,
                segment_used.as_ref(),
                used_as_middle.as_ref(),
                segment_pair.as_ref(),
                segment_pair_multi.as_ref(),
                start_source.as_ref(),
                start_source_multi.as_ref(),
                end_target.as_ref(),
                end_target_multi.as_ref(),
                used_points.as_ref().map(|p| p.as_ref()),
                route_from_idx,
                route_to_idx,
                edge_from_id,
                edge_to_id,
                allow_end_segment_reuse,
            )
            .map_err(|message| {
                rquickjs::Error::new_from_js_message("native_pathfinder", "getPathRelaxed", message)
            })?;

        match result {
            Some((path, cost)) => {
                let object = Object::new(ctx.clone())?;
                object.set("path", path)?;
                object.set("cost", cost)?;
                object.into_js(&ctx)
            }
            None => Ok(Value::new_null(ctx)),
        }
    }
}

impl JsEngine {
    // ----------------------------------------------------------------
    // QuickJS exception 解包（把 Error::Exception 还原成 message/stack）
    //
    // 背景:
    // - rquickjs 在 JS 抛异常时只返回 `Error::Exception`;
    // - 如果不主动 `ctx.catch()` 取出异常对象,最终用户只能看到“Exception generated by QuickJS”，
    //   完全无法定位是哪个 JS 文件/哪一行/哪个函数崩了。
    //
    // 这里做一次统一解包,把 Exception 的 message/stack 收敛成可读的 Rust 错误。
    // ----------------------------------------------------------------
    fn map_quickjs_error<'js>(
        ctx: &rquickjs::Ctx<'js>,
        err: rquickjs::Error,
    ) -> BeautifulMermaidError {
        if matches!(err, rquickjs::Error::Exception) {
            let value = ctx.catch();

            // 优先尝试按 `Error` 对象解包 message/stack。
            if let Ok(exception) = Exception::from_js(ctx, value.clone()) {
                let message = exception
                    .message()
                    .unwrap_or_else(|| "unknown js exception".to_string());
                let details = exception
                    .stack()
                    .map(|s| format!("\n{s}"))
                    .unwrap_or_default();
                return BeautifulMermaidError::JsException { message, details };
            }

            // 兜底: 如果抛出的不是 Error 实例,就把值转成字符串(能拿到多少是多少)。
            //
            // 说明:
            // - 这里不能再抛异常(会覆盖原异常),因此用 best-effort 转换。
            let message = value
                .as_string()
                .and_then(|s| s.to_string().ok())
                .unwrap_or_else(|| "unknown js exception value".to_string());
            return BeautifulMermaidError::JsException {
                message,
                details: String::new(),
            };
        }

        BeautifulMermaidError::Js(err)
    }

    fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        // ----------------------------------------------------------------
        // 是否禁用 native pathfinder(默认不禁用)
        //
        // 背景(用户反馈 + 现象对照):
        // - Rust CLI 的 `--ascii` 输出出现明显退化(可读性显著下降)。
        // - 对比 TS(bun) 与 Rust(QuickJS) 渲染同一份 Mermaid 后发现:
        //   Rust 侧注册的 `globalThis.__bm_getPath*` 会让 bundle 走 native A*,
        //   从而改变路径选择与 tie-break,导致输出与上游 TS 基线不一致。
        //
        // 策略:
        // - native pathfinder 默认保持开启(QuickJS 无 JIT,纯 JS A* 在真实图上可能非常慢)；
        // - 但提供一个“显式禁用开关”,用于对照上游 TS 基线与定位偏差。
        // ----------------------------------------------------------------
        let disable_native_pathfinder = std::env::var("BM_DISABLE_NATIVE_PATHFINDER")
            .ok()
            .map(|v| v.trim().to_ascii_lowercase())
            .is_some_and(|v| v == "1" || v == "true");
        let enable_native_pathfinder = !disable_native_pathfinder;

        // ----------------------------------------------------------------
        // CLI 加速：注册 native pathfinder
        //
        // 说明：
        // - TS bundle 会在运行时检测 `globalThis.__bm_getPath*` 是否存在。
        // - 存在则调用 native（Rust）实现的 A*，否则回退到纯 JS 版本。
        //
        // 设计要点：
        // - native 侧维护一份可复用的 A* 缓存（stamp/heap/表），避免每次调用分配大数组
        // - TypedArray 通过 `AsRef<[T]>` 只读访问，不需要 unsafe
        // ----------------------------------------------------------------
        if enable_native_pathfinder {
            context.with(|ctx| -> Result<()> {
                let astar = Rc::new(RefCell::new(NativeAStar::new()));

                let get_path = Function::new(
                    ctx.clone(),
                    NativeGetPathFn {
                        astar: astar.clone(),
                    },
                )?
                .with_name("__bm_getPath")?;
                ctx.globals().set("__bm_getPath", get_path)?;

                let get_path_strict = Function::new(
                    ctx.clone(),
                    NativeGetPathStrictFn {
                        astar: astar.clone(),
                    },
                )?
                .with_name("__bm_getPathStrict")?;
                ctx.globals().set("__bm_getPathStrict", get_path_strict)?;

                let get_path_relaxed =
                    Function::new(ctx.clone(), NativeGetPathRelaxedFn { astar })?
                        .with_name("__bm_getPathRelaxed")?;
                ctx.globals().set("__bm_getPathRelaxed", get_path_relaxed)?;

                Ok(())
            })?;
        }

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
            let output: String = render_fn
                .call((text, js_options))
                .map_err(|err| Self::map_quickjs_error(&ctx, err))?;
            Ok(output)
        })?;

        // 保守处理：把可能残留的 Promise job 队列清空，避免跨调用累积。
        self.drain_pending_jobs()?;

        Ok(rendered)
    }

    /// 渲染 Mermaid -> ASCII/Unicode + meta（同步）。
    pub fn render_mermaid_ascii_with_meta(
        &self,
        text: &str,
        options: &AsciiRenderOptions,
    ) -> Result<AsciiRenderWithMeta> {
        let json = self.context.with(|ctx| -> Result<String> {
            let beautiful_mermaid: Object = ctx.globals().get("beautifulMermaid")?;
            let render_fn: Function = beautiful_mermaid.get("renderMermaidAsciiWithMeta")?;

            let js_options = Self::ascii_options_to_js(&ctx, options)?;
            let result: Value = render_fn
                .call((text, js_options))
                .map_err(|err| Self::map_quickjs_error(&ctx, err))?;

            // 设计取舍：
            // - 这里用 JSON.stringify 做一次“跨语言对象”的稳定传输，
            //   避免在 Rust 侧手写 Object/Array 的深度解析逻辑。
            let json_obj: Object = ctx.globals().get("JSON")?;
            let stringify: Function = json_obj.get("stringify")?;
            let output: String = stringify
                .call((result,))
                .map_err(|err| Self::map_quickjs_error(&ctx, err))?;
            Ok(output)
        })?;

        // 保守处理：把可能残留的 Promise job 队列清空，避免跨调用累积。
        self.drain_pending_jobs()?;

        serde_json::from_str::<AsciiRenderWithMeta>(&json).map_err(|err| {
            BeautifulMermaidError::Json {
                message: format!("解析 renderMermaidAsciiWithMeta 输出失败: {err}"),
            }
        })
    }

    /// 渲染 Mermaid -> SVG（TS 版返回 Promise，这里同步等待）。
    pub fn render_mermaid_svg(&self, text: &str, options: &RenderOptions) -> Result<String> {
        let rendered = self.context.with(|ctx| -> Result<String> {
            let beautiful_mermaid: Object = ctx.globals().get("beautifulMermaid")?;
            let render_fn: Function = beautiful_mermaid.get("renderMermaid")?;

            let js_options = Self::render_options_to_js(&ctx, options)?;

            // TS 版 renderMermaid 是 async，这里拿到 Promise 并 finish 阻塞等待。
            let promise: Promise = render_fn
                .call((text, js_options))
                .map_err(|err| Self::map_quickjs_error(&ctx, err))?;
            let output: String = promise
                .finish()
                .map_err(|err| Self::map_quickjs_error(&ctx, err))?;
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
        if let Some(routing) = options.routing {
            object.set("routing", routing.as_str())?;
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
