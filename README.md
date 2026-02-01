# beautiful-mermaid-rs

用 **Rust** 调用并复刻 TypeScript 版 `beautiful-mermaid` 的行为：

- Mermaid → **SVG**（美观、可主题化、无 DOM 依赖）
- Mermaid → **ASCII/Unicode**（适合终端/CLI/日志）
- 支持图表类型：Flowchart/State、Sequence、Class、ER

## 当前实现策略（很重要）

为了尽快达成“输出一致的完整复刻”，本项目目前采用：

- **内嵌 QuickJS（rquickjs）**
- 执行已打包的 **browser IIFE bundle**
- Rust 侧只做：参数转换、错误收口、对外 API

JS bundle 位于：`vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`  
许可证：MIT（见 `LICENSE`）

### 同步上游 bundle（开发者）

如果你修改了 TypeScript 版 `beautiful-mermaid`（parser/layout/renderer 等）。
Rust 侧需要重新同步 bundle。
否则运行时仍会使用旧逻辑。

在本仓库执行：

```bash
make sync-vendor-verify
```

如果你的 TS 仓库不在默认路径，可以覆盖：

```bash
make TS_REPO_DIR=/path/to/beautiful-mermaid sync-vendor-verify
```

后续如果你希望走“纯 Rust 逐步替换内部实现”，可以在保持 API + 测试不变的前提下推进。

## 用法（库）

### SVG

```rust
use beautiful_mermaid_rs::{render_mermaid, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diagram = "graph TD\nA --> B\n";
    let svg = render_mermaid(diagram, &RenderOptions::default())?;
    println!("{svg}");
    Ok(())
}
```

### ASCII / Unicode

```rust
use beautiful_mermaid_rs::{render_mermaid_ascii, AsciiRenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diagram = "graph LR\nA --> B\n";
    let ascii = render_mermaid_ascii(
        diagram,
        &AsciiRenderOptions {
            // true = 纯 ASCII（+ - | >），false = Unicode（┌ ─ │ ►）
            use_ascii: Some(false),
            ..Default::default()
        },
    )?;

    println!("{ascii}");
    Ok(())
}
```

## 在其他 Rust 项目中集成

> 说明：目前本仓库还没发布到 crates.io。
> 因此常见接入方式是 `path` 依赖或 `git` 依赖。

### 1) 添加依赖

#### 方式A：本地 `path`（本机开发最方便）

在你的项目 `Cargo.toml`：

```toml
[dependencies]
beautiful-mermaid-rs = { path = "/Users/cuiluming/local_doc/l_dev/my/rust/beautiful-mermaid-rs" }
```

#### 方式B：`git`（适合团队/CI）

在你的项目 `Cargo.toml`：

```toml
[dependencies]
beautiful-mermaid-rs = { git = "ssh://git@your-host/your-org/beautiful-mermaid-rs.git", rev = "<commit>" }
```

建议你在 CI 场景用 `rev` 固定到某个提交。
这样构建更可复现，也更容易排查问题。

### 2) 在代码里调用

注意：crate 名是 `beautiful-mermaid-rs`，但在 Rust `use` 时要写成下划线形式：`beautiful_mermaid_rs`。

```rust
use beautiful_mermaid_rs::{render_mermaid, render_mermaid_ascii, AsciiRenderOptions, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diagram = "graph LR\nA --> B\n";

    // SVG（内部会同步等待 JS Promise 完成）
    let svg = render_mermaid(diagram, &RenderOptions::default())?;
    println!("{svg}");

    // Unicode ASCII-art（更适合终端）
    let ascii = render_mermaid_ascii(
        diagram,
        &AsciiRenderOptions {
            use_ascii: Some(false),
            ..Default::default()
        },
    )?;
    println!("{ascii}");

    Ok(())
}
```

### 3) 使用内置主题（THEMES）

本仓库提供了主题数据：`beautiful_mermaid_rs::theme::THEMES`。
渲染函数的入参是 `RenderOptions`。
你可以把 `DiagramColors` 映射成 `RenderOptions` 来用：

```rust
use beautiful_mermaid_rs::theme;
use beautiful_mermaid_rs::{render_mermaid, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diagram = "graph LR\nA --> B\n";
    let colors = theme::THEMES.get("tokyo-night").expect("theme not found");

    let options = RenderOptions {
        bg: Some(colors.bg.clone()),
        fg: Some(colors.fg.clone()),
        line: colors.line.clone(),
        accent: colors.accent.clone(),
        muted: colors.muted.clone(),
        surface: colors.surface.clone(),
        border: colors.border.clone(),
        ..Default::default()
    };

    let svg = render_mermaid(diagram, &options)?;
    println!("{svg}");
    Ok(())
}
```

### 4) 在 Tokio / async 项目里使用（建议 spawn_blocking）

当前 API 是同步阻塞的。
如果你的业务是 async（Tokio），建议用 `spawn_blocking`：

```rust
use beautiful_mermaid_rs::{render_mermaid, RenderOptions};

async fn render_svg_async(diagram: String) -> Result<String, beautiful_mermaid_rs::BeautifulMermaidError> {
    tokio::task::spawn_blocking(move || render_mermaid(&diagram, &RenderOptions::default()))
        .await
        .expect("spawn_blocking join failed")
}
```

### 5) 集成注意事项（建议先读）

- 首次调用会更慢一点：每个线程第一次使用时会初始化 QuickJS 并 eval JS bundle。
- 多线程并发没问题：本 crate 使用 thread-local 方式做到“每线程一个 JS 引擎实例”，不会跨线程共享 Context。
- 构建环境：`rquickjs-sys` 会编译 QuickJS 的 C 代码，需要系统有可用的 C 编译工具链。
- SVG 字体：TS 版输出里带了 Google Fonts 的 `@import`（默认 `Inter`）。
  - 离线环境下字体可能加载不到，但 SVG 仍可正常显示。

## 用法（CLI）

从 stdin 读取 Mermaid 文本：

> 给 code agent 的更完整命令行说明见：`docs/code-agent-cli.md`。
> 如果你已安装为系统 PATH 命令，则直接用 `beautiful-mermaid-rs`；
> 否则在仓库内用 `cargo run --quiet -- ...` 也可以。

- 输出 SVG：

```bash
printf 'graph LR\nA --> B\n' | beautiful-mermaid-rs
```

- 输出 Unicode 线条字符（默认更好看）：

```bash
printf 'graph LR\nA --> B\n' | beautiful-mermaid-rs --ascii
```

- 输出纯 ASCII 字符集（兼容性最好）：

```bash
printf 'graph LR\nA --> B\n' | beautiful-mermaid-rs --ascii --use-ascii
```

## 测试

```bash
cargo test
```

测试数据来自 TS 版的 golden files：`tests/testdata/{ascii,unicode}`。
