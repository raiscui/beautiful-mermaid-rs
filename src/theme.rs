// ============================================================================
// 主题（Theme）与 Shiki 主题提取
//
// 说明：
// - 这里直接复刻 TS 版 `src/theme.ts` 的默认值与 THEMES 列表
// - 目的是让 Rust 侧可以“脱离 JS 引擎”就能拿到主题数据
// - 渲染本身仍由内嵌 JS bundle 完成（作为短期基线）
// ============================================================================

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 图表配色（对齐 TS: `DiagramColors`）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagramColors {
    pub bg: String,
    pub fg: String,
    pub line: Option<String>,
    pub accent: Option<String>,
    pub muted: Option<String>,
    pub surface: Option<String>,
    pub border: Option<String>,
}

/// 默认 bg/fg（TS: DEFAULTS）。
pub static DEFAULTS: Lazy<DiagramColors> = Lazy::new(|| DiagramColors {
    bg: "#FFFFFF".to_string(),
    fg: "#27272A".to_string(),
    line: None,
    accent: None,
    muted: None,
    surface: None,
    border: None,
});

/// 预置主题（TS: THEMES）。
pub static THEMES: Lazy<HashMap<&'static str, DiagramColors>> = Lazy::new(|| {
    let mut themes: HashMap<&'static str, DiagramColors> = HashMap::new();

    // --------------------------------------------------------------------
    // 注意：这里保持与 TS 版 `src/theme.ts` 完全一致
    // --------------------------------------------------------------------
    themes.insert(
        "zinc-dark",
        DiagramColors {
            bg: "#18181B".to_string(),
            fg: "#FAFAFA".to_string(),
            line: None,
            accent: None,
            muted: None,
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "tokyo-night",
        DiagramColors {
            bg: "#1a1b26".to_string(),
            fg: "#a9b1d6".to_string(),
            line: Some("#3d59a1".to_string()),
            accent: Some("#7aa2f7".to_string()),
            muted: Some("#565f89".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "tokyo-night-storm",
        DiagramColors {
            bg: "#24283b".to_string(),
            fg: "#a9b1d6".to_string(),
            line: Some("#3d59a1".to_string()),
            accent: Some("#7aa2f7".to_string()),
            muted: Some("#565f89".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "tokyo-night-light",
        DiagramColors {
            bg: "#d5d6db".to_string(),
            fg: "#343b58".to_string(),
            line: Some("#34548a".to_string()),
            accent: Some("#34548a".to_string()),
            muted: Some("#9699a3".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "catppuccin-mocha",
        DiagramColors {
            bg: "#1e1e2e".to_string(),
            fg: "#cdd6f4".to_string(),
            line: Some("#585b70".to_string()),
            accent: Some("#cba6f7".to_string()),
            muted: Some("#6c7086".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "catppuccin-latte",
        DiagramColors {
            bg: "#eff1f5".to_string(),
            fg: "#4c4f69".to_string(),
            line: Some("#9ca0b0".to_string()),
            accent: Some("#8839ef".to_string()),
            muted: Some("#9ca0b0".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "nord",
        DiagramColors {
            bg: "#2e3440".to_string(),
            fg: "#d8dee9".to_string(),
            line: Some("#4c566a".to_string()),
            accent: Some("#88c0d0".to_string()),
            muted: Some("#616e88".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "nord-light",
        DiagramColors {
            bg: "#eceff4".to_string(),
            fg: "#2e3440".to_string(),
            line: Some("#aab1c0".to_string()),
            accent: Some("#5e81ac".to_string()),
            muted: Some("#7b88a1".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "dracula",
        DiagramColors {
            bg: "#282a36".to_string(),
            fg: "#f8f8f2".to_string(),
            line: Some("#6272a4".to_string()),
            accent: Some("#bd93f9".to_string()),
            muted: Some("#6272a4".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "github-light",
        DiagramColors {
            bg: "#ffffff".to_string(),
            fg: "#1f2328".to_string(),
            line: Some("#d1d9e0".to_string()),
            accent: Some("#0969da".to_string()),
            muted: Some("#59636e".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "github-dark",
        DiagramColors {
            bg: "#0d1117".to_string(),
            fg: "#e6edf3".to_string(),
            line: Some("#3d444d".to_string()),
            accent: Some("#4493f8".to_string()),
            muted: Some("#9198a1".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "solarized-light",
        DiagramColors {
            bg: "#fdf6e3".to_string(),
            fg: "#657b83".to_string(),
            line: Some("#93a1a1".to_string()),
            accent: Some("#268bd2".to_string()),
            muted: Some("#93a1a1".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "solarized-dark",
        DiagramColors {
            bg: "#002b36".to_string(),
            fg: "#839496".to_string(),
            line: Some("#586e75".to_string()),
            accent: Some("#268bd2".to_string()),
            muted: Some("#586e75".to_string()),
            surface: None,
            border: None,
        },
    );
    themes.insert(
        "one-dark",
        DiagramColors {
            bg: "#282c34".to_string(),
            fg: "#abb2bf".to_string(),
            line: Some("#4b5263".to_string()),
            accent: Some("#c678dd".to_string()),
            muted: Some("#5c6370".to_string()),
            surface: None,
            border: None,
        },
    );

    themes
});

/// 从 Shiki 主题对象里提取 DiagramColors（对齐 TS: `fromShikiTheme()`）。
///
/// 入参是一个“类似 Shiki ThemeRegistrationResolved 的 JSON 对象”：
/// - `type`: "dark" | "light"
/// - `colors`: Record<string, string>
/// - `tokenColors`: Array<{ scope, settings: { foreground } }>
pub fn from_shiki_theme(theme: &serde_json::Value) -> DiagramColors {
    let is_dark = theme
        .get("type")
        .and_then(|value| value.as_str())
        .map(|value| value == "dark")
        .unwrap_or(false);

    let colors = theme
        .get("colors")
        .and_then(|value| value.as_object())
        .cloned()
        .unwrap_or_default();

    // --------------------------------------------------------------------
    // Helper: 从 tokenColors 按 scope 找 foreground
    // --------------------------------------------------------------------
    let token_color = |scope: &str| -> Option<String> {
        let token_colors = theme.get("tokenColors")?.as_array()?;

        for token in token_colors {
            let scopes = token.get("scope");
            let hit = match scopes {
                Some(serde_json::Value::String(s)) => s == scope,
                Some(serde_json::Value::Array(list)) => {
                    list.iter().any(|v| v.as_str() == Some(scope))
                }
                _ => false,
            };

            if !hit {
                continue;
            }

            let foreground = token
                .get("settings")
                .and_then(|settings| settings.get("foreground"))
                .and_then(|value| value.as_str())?;

            return Some(foreground.to_string());
        }

        None
    };

    // --------------------------------------------------------------------
    // TS 版映射逻辑（逐项对齐）
    // --------------------------------------------------------------------
    let editor_background = colors
        .get("editor.background")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| if is_dark { "#1e1e1e" } else { "#ffffff" }.to_string());

    let editor_foreground = colors
        .get("editor.foreground")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| if is_dark { "#d4d4d4" } else { "#333333" }.to_string());

    let line = colors
        .get("editorLineNumber.foreground")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    let accent = colors
        .get("focusBorder")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .or_else(|| token_color("keyword"));

    let muted = token_color("comment").or_else(|| line.clone());

    let surface = colors
        .get("editor.selectionBackground")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    let border = colors
        .get("editorWidget.border")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    DiagramColors {
        bg: editor_background,
        fg: editor_foreground,
        line,
        accent,
        muted,
        surface,
        border,
    }
}
