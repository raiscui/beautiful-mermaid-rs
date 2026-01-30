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

## 用法（CLI）

从 stdin 读取 Mermaid 文本：

- 输出 SVG：

```bash
printf 'graph LR\nA --> B\n' | cargo run --quiet
```

- 输出 Unicode 线条字符（默认更好看）：

```bash
printf 'graph LR\nA --> B\n' | cargo run --quiet -- --ascii
```

- 输出纯 ASCII 字符集（兼容性最好）：

```bash
printf 'graph LR\nA --> B\n' | cargo run --quiet -- --ascii --use-ascii
```

## 测试

```bash
cargo test
```

测试数据来自 TS 版的 golden files：`tests/testdata/{ascii,unicode}`。

