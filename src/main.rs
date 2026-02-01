fn main() {
    // --------------------------------------------------------------------
    // 一个极简 CLI：
    // - 默认从 stdin 读取 Mermaid 文本
    // - 默认输出 SVG
    // - 传 `--ascii` 切换为 ASCII/Unicode 文本输出
    // - 传 `--help/-h` 输出帮助并退出
    // - 传 `--version/-V` 输出版本并退出
    //
    // 目的：
    // - 方便在本地快速验证渲染结果
    // - 不强制引入 clap 之类的重依赖，保持项目轻量
    // --------------------------------------------------------------------

    use std::io::{self, Read};

    // --------------------------------------------------------------------
    // 统一处理 stdout 输出：
    // 1) 始终补齐末尾换行，避免 zsh 把提示符（通常是 `%`）粘在输出后面。
    // 2) pipe 场景下如果下游提前关闭（BrokenPipe），按 Unix 习惯静默退出，不 panic。
    // --------------------------------------------------------------------
    fn write_stdout_with_trailing_newline(text: &str) {
        use std::io::{self, Write};

        fn handle_write_error(err: io::Error) -> ! {
            if err.kind() == io::ErrorKind::BrokenPipe {
                // 下游已关闭（例如 `| head -n 1`），这不是“程序错误”，直接 0 退出即可。
                std::process::exit(0);
            }

            eprintln!("写入 stdout 失败: {err}");
            std::process::exit(1);
        }

        let mut stdout = io::stdout().lock();
        if let Err(err) = stdout.write_all(text.as_bytes()) {
            handle_write_error(err);
        }

        if !text.ends_with('\n') {
            if let Err(err) = stdout.write_all(b"\n") {
                handle_write_error(err);
            }
        }
    }

    // --------------------------------------------------------------------
    // 输出 CLI 帮助：
    // - 允许 code agent 在陌生环境里快速自发现用法
    // - 避免“stdin 为空时就直接渲染失败”的困惑体验
    // --------------------------------------------------------------------
    fn print_help() {
        let bin = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");

        let help = format!(
            r#"{bin} {version}

用法：
  # 从 stdin 读取 Mermaid 文本并输出 SVG
  beautiful-mermaid-rs < diagram.mmd > diagram.svg

  # 输出 Unicode 线条字符（更好看，适合终端）
  beautiful-mermaid-rs --ascii < diagram.mmd

  # 输出纯 ASCII 字符集（兼容性最好）
  beautiful-mermaid-rs --ascii --use-ascii < diagram.mmd

选项：
  --ascii         输出 ASCII/Unicode 文本（默认输出 SVG）
  --use-ascii     仅在 --ascii 模式下生效：强制使用纯 ASCII 字符
  -h, --help      输出帮助并退出
  -V, --version   输出版本并退出
"#
        );

        write_stdout_with_trailing_newline(&help);
    }

    // --------------------------------------------------------------------
    // 先解析参数：
    // - `--help/--version` 不应该依赖 stdin
    // - 也避免用户“忘了输入”时得到 QuickJS 的堆栈错误
    // --------------------------------------------------------------------
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return;
    }

    if args.iter().any(|arg| arg == "-V" || arg == "--version") {
        let bin = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        write_stdout_with_trailing_newline(&format!("{bin} {version}"));
        return;
    }

    // --------------------------------------------------------------------
    // 做一点点“强约束”：
    // - 仅支持极少量参数，避免 typo 静默被忽略，浪费排查时间
    // - `--use-ascii` 如果没配 `--ascii`，直接报错（否则会“看起来没效果”）
    // --------------------------------------------------------------------
    let has_ascii_flag = args.iter().any(|arg| arg == "--ascii");
    let has_use_ascii_flag = args.iter().any(|arg| arg == "--use-ascii");

    if has_use_ascii_flag && !has_ascii_flag {
        eprintln!("参数错误：`--use-ascii` 需要与 `--ascii` 一起使用。");
        eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
        std::process::exit(2);
    }

    for arg in &args {
        let is_supported = matches!(
            arg.as_str(),
            "--ascii" | "--use-ascii" | "-h" | "--help" | "-V" | "--version"
        );

        if !is_supported {
            eprintln!("未知参数: {arg}");
            eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
            std::process::exit(2);
        }
    }

    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("读取 stdin 失败: {err}");
        std::process::exit(1);
    }

    if input.trim().is_empty() {
        eprintln!("stdin 为空：请通过管道或重定向输入 Mermaid 文本。");
        eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看示例。");
        std::process::exit(2);
    }

    let use_ascii_renderer = has_ascii_flag;
    let force_pure_ascii = has_use_ascii_flag;

    if use_ascii_renderer {
        let options = beautiful_mermaid_rs::AsciiRenderOptions {
            // `--use-ascii`：输出纯 ASCII 字符集；否则输出 Unicode 线条字符
            use_ascii: Some(force_pure_ascii),
            ..Default::default()
        };

        match beautiful_mermaid_rs::render_mermaid_ascii(&input, &options) {
            Ok(output) => write_stdout_with_trailing_newline(&output),
            Err(err) => {
                eprintln!("渲染 ASCII 失败: {err}");
                std::process::exit(1);
            }
        }
    } else {
        let options = beautiful_mermaid_rs::RenderOptions::default();
        match beautiful_mermaid_rs::render_mermaid(&input, &options) {
            Ok(svg) => write_stdout_with_trailing_newline(&svg),
            Err(err) => {
                eprintln!("渲染 SVG 失败: {err}");
                std::process::exit(1);
            }
        }
    }
}
