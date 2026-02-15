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

  # 强制使用 strict 路由(与默认 relaxed 不同,可用于对照可读性)
  beautiful-mermaid-rs --ascii --routing strict < diagram.mmd

  # 仅校验 Mermaid 语法（stdout 输出 true/false）
  beautiful-mermaid-rs --validate < diagram.mmd

  # 校验 Markdown 中所有 ```mermaid 代码块（stdout 输出 true/false）
  beautiful-mermaid-rs --validate-markdown < README.md

选项：
  --ascii         输出 ASCII/Unicode 文本（默认输出 SVG）
  --use-ascii     仅在 --ascii 模式下生效：强制使用纯 ASCII 字符
  --routing <strict|relaxed>
                 仅在 --ascii 模式下生效：strict/relaxed 两种路由策略(输出可能差异很大)
  --validate      校验 Mermaid 语法（不输出 SVG/ASCII），stdout 输出 true/false
  --validate-markdown
                 扫描 stdin 的 Markdown，校验其中所有 ```mermaid 代码块
  -h, --help      输出帮助并退出
  -V, --version   输出版本并退出
"#
        );

        write_stdout_with_trailing_newline(&help);
    }

    // --------------------------------------------------------------------
    // 校验 Markdown 内的 Mermaid code fence：
    // - 只识别 ```mermaid ... ``` 这种 fenced code block（与 GitHub/常见 Markdown 行为一致）
    // - 输出策略：
    //   - stdout: true/false（便于脚本/CI 消费）
    //   - stderr: 失败原因（含起始行号），便于人类定位
    // --------------------------------------------------------------------
    fn validate_markdown_mermaid_blocks(markdown: &str) -> bool {
        #[derive(Debug)]
        struct MermaidBlock {
            start_line: usize,
            diagram: String,
        }

        let mut blocks: Vec<MermaidBlock> = Vec::new();
        let mut in_mermaid_block = false;
        let mut block_start_line: usize = 0;
        let mut diagram = String::new();

        for (idx, line) in markdown.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim_start();

            if !in_mermaid_block {
                if trimmed.starts_with("```mermaid") {
                    in_mermaid_block = true;
                    block_start_line = line_no;
                    diagram.clear();
                }
                continue;
            }

            // 结束 fence: 允许 ``` 后跟任意内容（大多数 Markdown 实现都会这样处理）
            if trimmed.starts_with("```") {
                blocks.push(MermaidBlock {
                    start_line: block_start_line,
                    diagram: std::mem::take(&mut diagram),
                });
                in_mermaid_block = false;
                continue;
            }

            diagram.push_str(line);
            diagram.push('\n');
        }

        if in_mermaid_block {
            eprintln!(
                "Markdown Mermaid 校验失败: 存在未闭合的 ```mermaid 代码块（起始行: {block_start_line}）。"
            );
            return false;
        }

        if blocks.is_empty() {
            // 没找到 Mermaid 块时，按“无可校验内容”处理为 true，但给出提示。
            eprintln!("提示: 未在 Markdown 中找到 ```mermaid 代码块，本次校验跳过。");
            return true;
        }

        let mut all_valid = true;
        for (idx, block) in blocks.iter().enumerate() {
            match beautiful_mermaid_rs::validate_mermaid(&block.diagram) {
                Ok(result) => {
                    if result.is_valid {
                        continue;
                    }

                    all_valid = false;
                    eprintln!(
                        "Markdown Mermaid 校验失败: 第 {} 个 mermaid block（起始行: {}）无效。",
                        idx + 1,
                        block.start_line
                    );

                    if let Some(error) = result.error.as_deref() {
                        eprintln!("错误: {error}");
                    }
                    if let Some(details) = result.details.as_deref() {
                        eprintln!("细节:\n{details}");
                    }
                }
                Err(err) => {
                    // JS 引擎错误属于“内部错误”，同样让校验失败并输出原因。
                    all_valid = false;
                    eprintln!(
                        "Markdown Mermaid 校验内部错误: 第 {} 个 mermaid block（起始行: {}）: {err}",
                        idx + 1,
                        block.start_line
                    );
                }
            }
        }

        all_valid
    }

    // --------------------------------------------------------------------
    // 先解析参数：
    // - `--help/--version` 不应该依赖 stdin
    // - 也避免用户“忘了输入”时得到 QuickJS 的堆栈错误
    // --------------------------------------------------------------------
    let args: Vec<String> = std::env::args().skip(1).collect();

    #[derive(Debug, Default)]
    struct CliArgs {
        ascii: bool,
        use_ascii: bool,
        validate: bool,
        validate_markdown: bool,
        routing: Option<beautiful_mermaid_rs::AsciiRouting>,
        help: bool,
        version: bool,
    }

    fn parse_routing(value: &str) -> Option<beautiful_mermaid_rs::AsciiRouting> {
        match value.trim().to_ascii_lowercase().as_str() {
            "strict" => Some(beautiful_mermaid_rs::AsciiRouting::Strict),
            "relaxed" => Some(beautiful_mermaid_rs::AsciiRouting::Relaxed),
            _ => None,
        }
    }

    // --------------------------------------------------------------------
    // 做一点点“强约束”：
    // - 仅支持极少量参数，避免 typo 静默被忽略，浪费排查时间
    // - 对于带 value 的参数（例如 `--routing strict`），我们也显式解析
    // --------------------------------------------------------------------
    let mut cli = CliArgs::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-h" | "--help" => cli.help = true,
            "-V" | "--version" => cli.version = true,
            "--ascii" => cli.ascii = true,
            "--use-ascii" => cli.use_ascii = true,
            "--validate" => cli.validate = true,
            "--validate-markdown" => cli.validate_markdown = true,
            "--routing" => {
                let value = args.get(idx + 1).map(|s| s.as_str()).unwrap_or("");
                if value.is_empty() {
                    eprintln!("参数错误：`--routing` 需要一个值：strict 或 relaxed。");
                    eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
                    std::process::exit(2);
                }
                match parse_routing(value) {
                    Some(routing) => cli.routing = Some(routing),
                    None => {
                        eprintln!("参数错误：`--routing` 的值无效: {value}");
                        eprintln!("可选值: strict | relaxed");
                        std::process::exit(2);
                    }
                }
                idx += 1;
            }
            arg if arg.starts_with("--routing=") => {
                let value = arg.trim_start_matches("--routing=");
                match parse_routing(value) {
                    Some(routing) => cli.routing = Some(routing),
                    None => {
                        eprintln!("参数错误：`--routing` 的值无效: {value}");
                        eprintln!("可选值: strict | relaxed");
                        std::process::exit(2);
                    }
                }
            }
            arg => {
                eprintln!("未知参数: {arg}");
                eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
                std::process::exit(2);
            }
        }
        idx += 1;
    }

    if cli.help {
        print_help();
        return;
    }

    if cli.version {
        let bin = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        write_stdout_with_trailing_newline(&format!("{bin} {version}"));
        return;
    }

    if cli.use_ascii && !cli.ascii {
        eprintln!("参数错误：`--use-ascii` 需要与 `--ascii` 一起使用。");
        eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
        std::process::exit(2);
    }

    if cli.routing.is_some() && !cli.ascii {
        eprintln!("参数错误：`--routing` 仅在 `--ascii` 模式下生效。");
        eprintln!("提示：例如 `beautiful-mermaid-rs --ascii --routing strict < diagram.mmd`。");
        std::process::exit(2);
    }

    if cli.validate && cli.validate_markdown {
        eprintln!("参数错误：`--validate` 与 `--validate-markdown` 不能同时使用。");
        eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
        std::process::exit(2);
    }

    if (cli.validate || cli.validate_markdown) && (cli.ascii || cli.use_ascii) {
        eprintln!(
            "参数错误：校验模式（`--validate*`）不能与渲染模式（`--ascii/--use-ascii`）混用。"
        );
        eprintln!("提示：可以先运行 `beautiful-mermaid-rs --help` 查看完整用法。");
        std::process::exit(2);
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

    // --------------------------------------------------------------------
    // 校验模式：不输出图，只输出 true/false，便于脚本/CI 使用。
    // --------------------------------------------------------------------
    if cli.validate_markdown {
        let is_valid = validate_markdown_mermaid_blocks(&input);
        write_stdout_with_trailing_newline(if is_valid { "true" } else { "false" });
        std::process::exit(if is_valid { 0 } else { 1 });
    }

    if cli.validate {
        match beautiful_mermaid_rs::validate_mermaid(&input) {
            Ok(result) => {
                write_stdout_with_trailing_newline(if result.is_valid { "true" } else { "false" });

                if result.is_valid {
                    return;
                }

                eprintln!("Mermaid 图语法无效。");
                if let Some(error) = result.error.as_deref() {
                    eprintln!("错误: {error}");
                }
                if let Some(details) = result.details.as_deref() {
                    eprintln!("细节:\n{details}");
                }
                std::process::exit(1);
            }
            Err(err) => {
                eprintln!("Mermaid 校验内部错误: {err}");
                std::process::exit(1);
            }
        }
    }

    let use_ascii_renderer = cli.ascii;
    let force_pure_ascii = cli.use_ascii;

    if use_ascii_renderer {
        let options = beautiful_mermaid_rs::AsciiRenderOptions {
            // `--use-ascii`：输出纯 ASCII 字符集；否则输出 Unicode 线条字符
            use_ascii: Some(force_pure_ascii),
            // 让用户可显式切换路由策略,用于在“紧凑度/可读性”之间做取舍对照。
            routing: cli.routing,
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
