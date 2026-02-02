// ============================================================================
// 基于 TS 版 beautiful-mermaid 的 testdata 做回归测试
//
// 说明：
// - 参考仓库的 testdata 文件格式：
//   - 前半部分：可选配置行（例如 paddingX=2），随后是 Mermaid 文本
//   - 分隔符：一行 `---`
//   - 后半部分：期望的 ASCII/Unicode 输出
// - 本测试会逐个读取文件并做“严格输出对比”（只忽略末尾换行差异）
// - 额外支持黄金文件更新模式：
//   - 当设置环境变量 `UPDATE_GOLDEN=1` 时，不做 assert，而是把当前渲染输出写回 testdata 文件
//   - 更新完成后会 panic 提示你重新运行测试（确保输出稳定且无其他回归）
// ============================================================================

use beautiful_mermaid_rs::{AsciiRenderOptions, render_mermaid_ascii};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Whitespace 归一化（对齐 TS 测试里的 normalizeWhitespace）：
/// - 每行 trimEnd（去掉行尾空格）
/// - 去掉首尾空行
fn normalize_whitespace(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized
        .split('\n')
        .map(|line| line.trim_end().to_string())
        .collect();

    while !lines.is_empty() && lines.first().is_some_and(|line| line.is_empty()) {
        lines.remove(0);
    }
    while !lines.is_empty() && lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

/// 是否启用“自动更新 golden 文件”模式。
///
/// - `UPDATE_GOLDEN=1`：启用
/// - 其他值/未设置：关闭
fn should_update_golden() -> bool {
    std::env::var("UPDATE_GOLDEN")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

/// 用新的期望输出重写 golden 文件内容（保留 Mermaid 区块与分隔符 `---`）。
fn rewrite_golden_file_content(raw: &str, new_expected: &str) -> String {
    let raw = raw.replace("\r\n", "\n");

    let mut out = String::new();
    let mut found_delimiter = false;

    for line in raw.split('\n') {
        out.push_str(line);
        out.push('\n');

        if line == "---" {
            found_delimiter = true;
            break;
        }
    }

    if !found_delimiter {
        panic!("golden 文件缺少分隔符 `---`，无法更新");
    }

    out.push_str(new_expected);
    out.push('\n');

    out
}

fn collect_txt_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("txt"))
        .map(|entry| entry.path().to_path_buf())
        .collect();

    // 稳定排序，保证失败时输出可复现
    files.sort();
    files
}

/// 解析一个 golden file（对齐 TS `parseTestCase()` 的语义）。
///
/// 格式：
///   [paddingX=N]          可选，允许空格
///   [paddingY=N]          可选，允许空格
///   [boxBorderPadding=N]  可选，允许空格
///   <mermaid code>
///   ---
///   <expected output>
fn parse_test_case(content: &str, use_ascii: bool) -> (String, AsciiRenderOptions, String) {
    let content = content.replace("\r\n", "\n");

    let mut options = AsciiRenderOptions {
        use_ascii: Some(use_ascii),
        // TS 测试里会显式传入默认 5/5，这里也保持一致，避免“依赖 JS 默认值”导致的隐式差异
        padding_x: Some(5),
        padding_y: Some(5),
        ..Default::default()
    };

    let mut in_mermaid = true;
    let mut mermaid_started = false;
    let mut mermaid_lines: Vec<&str> = Vec::new();
    let mut expected_lines: Vec<&str> = Vec::new();

    for line in content.split('\n') {
        if line == "---" {
            in_mermaid = false;
            continue;
        }

        if in_mermaid {
            let trimmed = line.trim();

            // Mermaid 代码开始之前：跳过空行，并解析 padding 指令
            if !mermaid_started {
                if trimmed.is_empty() {
                    continue;
                }

                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();

                    match key {
                        "paddingX" | "paddingx" => {
                            options.padding_x = Some(value.parse::<i32>().unwrap_or_else(|err| {
                                panic!("paddingX 解析失败: value={value}, err={err}")
                            }));
                            continue;
                        }
                        "paddingY" | "paddingy" => {
                            options.padding_y = Some(value.parse::<i32>().unwrap_or_else(|err| {
                                panic!("paddingY 解析失败: value={value}, err={err}")
                            }));
                            continue;
                        }
                        "boxBorderPadding" | "boxborderpadding" => {
                            options.box_border_padding =
                                Some(value.parse::<i32>().unwrap_or_else(|err| {
                                    panic!("boxBorderPadding 解析失败: value={value}, err={err}")
                                }));
                            continue;
                        }
                        _ => {
                            // 没匹配到配置项，就当作 Mermaid 第一行（例如 "graph TD"）
                        }
                    }
                }
            }

            mermaid_started = true;
            mermaid_lines.push(line);
        } else {
            expected_lines.push(line);
        }
    }

    let mut mermaid = mermaid_lines.join("\n");
    mermaid.push('\n');

    // 对齐 TS：join 后如果末尾有一个多余的 '\n'，去掉一个
    let mut expected = expected_lines.join("\n");
    if expected.ends_with('\n') {
        expected.pop();
    }

    (mermaid, options, expected)
}

fn run_testdata_dir(dir: &Path, use_ascii: bool) {
    let update_golden = should_update_golden();
    let mut updated_files: Vec<PathBuf> = Vec::new();

    for file_path in collect_txt_files(dir) {
        let raw = fs::read_to_string(&file_path)
            .unwrap_or_else(|err| panic!("读取 testdata 失败: path={file_path:?}, err={err}"));

        let (diagram, options, expected) = parse_test_case(&raw, use_ascii);

        let actual = render_mermaid_ascii(&diagram, &options).unwrap_or_else(|err| {
            panic!("渲染失败: path={file_path:?}, err={err}");
        });

        let normalized_expected = normalize_whitespace(&expected);
        let normalized_actual = normalize_whitespace(&actual);

        if normalized_actual != normalized_expected {
            if update_golden {
                let new_content = rewrite_golden_file_content(&raw, &normalized_actual);
                fs::write(&file_path, new_content).unwrap_or_else(|err| {
                    panic!("写入 golden 失败: path={file_path:?}, err={err}");
                });
                updated_files.push(file_path);
                continue;
            }

            assert_eq!(
                normalized_actual, normalized_expected,
                "输出不一致: path={file_path:?} (dir={dir:?})"
            );
        }
    }

    if update_golden && !updated_files.is_empty() {
        let mut message = format!(
            "已更新 {} 个 golden 文件（dir={dir:?}）。请重新运行测试确认输出稳定：\n",
            updated_files.len()
        );
        for path in updated_files {
            message.push_str(&format!("  - {path:?}\n"));
        }
        panic!("{message}");
    }
}

#[test]
fn ascii_testdata_matches_reference() {
    run_testdata_dir(Path::new("tests/testdata/ascii"), true);
}

#[test]
fn unicode_testdata_matches_reference() {
    run_testdata_dir(Path::new("tests/testdata/unicode"), false);
}
