fn main() {
    // --------------------------------------------------------------------
    // 一个极简 CLI：
    // - 默认从 stdin 读取 Mermaid 文本
    // - 默认输出 SVG
    // - 传 `--ascii` 切换为 ASCII/Unicode 文本输出
    //
    // 目的：
    // - 方便在本地快速验证渲染结果
    // - 不强制引入 clap 之类的重依赖，保持项目轻量
    // --------------------------------------------------------------------

    use std::io::{self, Read};

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("读取 stdin 失败: {err}");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let use_ascii_renderer = args.iter().any(|arg| arg == "--ascii");
    let force_pure_ascii = args.iter().any(|arg| arg == "--use-ascii");

    if use_ascii_renderer {
        let options = beautiful_mermaid_rs::AsciiRenderOptions {
            // `--use-ascii`：输出纯 ASCII 字符集；否则输出 Unicode 线条字符
            use_ascii: Some(force_pure_ascii),
            ..Default::default()
        };

        match beautiful_mermaid_rs::render_mermaid_ascii(&input, &options) {
            Ok(output) => print!("{output}"),
            Err(err) => {
                eprintln!("渲染 ASCII 失败: {err}");
                std::process::exit(1);
            }
        }
    } else {
        let options = beautiful_mermaid_rs::RenderOptions::default();
        match beautiful_mermaid_rs::render_mermaid(&input, &options) {
            Ok(svg) => print!("{svg}"),
            Err(err) => {
                eprintln!("渲染 SVG 失败: {err}");
                std::process::exit(1);
            }
        }
    }
}
