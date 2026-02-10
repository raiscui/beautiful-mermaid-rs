// ============================================================================
// 调试工具: 复现用户反馈的 flowchart 用例,并在失败时打印更详细的 JS 异常信息。
//
// 用法:
//   cargo run --release --example debug_user_case -- --ascii
//
// 说明:
// - 这个 example 不参与 `cargo test`,只在需要时手动运行;
// - 主要用于把 `rquickjs::Error` 的 Debug 信息打印出来,便于定位 QuickJS 异常根因。
// ============================================================================

use beautiful_mermaid_rs::{AsciiRenderOptions, render_mermaid_ascii};

fn main() {
    let mermaid = r#"flowchart TD
Hat_ralph["ralph#1 (coordinator)"]
Hat_experiment_auditor[<0001f9fe> 结果审计员]
Hat_experiment_integrator[<0001f9e9> 集成验收员]
Hat_experiment_runner[<0001f9ea> 实验执行器]
Start[task.start]
Start --> Hat_ralph
Complete[complete]
Hat_experiment_auditor -->|experiment.reviewed| Hat_ralph
Hat_experiment_integrator -->|experiment.complete| Complete
Hat_experiment_integrator -->|experiment.complete| Hat_ralph
Hat_experiment_integrator -->|integration.applied| Hat_ralph
Hat_experiment_integrator -->|integration.blocked| Hat_ralph
Hat_experiment_integrator -->|integration.rejected| Hat_ralph
Hat_experiment_runner -->|experiment.result| Hat_experiment_auditor
Hat_ralph -->|experiment.task| Hat_experiment_runner
Hat_ralph -->|integration.task| Hat_experiment_integrator
"#;

    let options = AsciiRenderOptions {
        // Unicode box-drawing(默认): 对齐用户的 `--ascii` 行为
        use_ascii: Some(false),
        ..Default::default()
    };

    match render_mermaid_ascii(mermaid, &options) {
        Ok(text) => {
            print!("{text}");
        }
        Err(err) => {
            // display: 面向用户的简洁信息
            eprintln!("[display] {err}");
            // debug: 尽量把 QuickJS exception/stack 打印出来
            eprintln!("[debug] {err:?}");
            std::process::exit(1);
        }
    }
}
