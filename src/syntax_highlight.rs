//! Syntax highlighting at build time using syntect
//!
//! This module provides compile-time syntax highlighting for code blocks.
//! Only compiled in SSR mode (not included in WASM bundle).
//!
//! Supports: R, Python, Rust, JavaScript, TypeScript, JSON, YAML, etc.

#[cfg(feature = "ssr")]
use syntect::easy::HighlightLines;
#[cfg(feature = "ssr")]
use syntect::highlighting::{Style, ThemeSet};
#[cfg(feature = "ssr")]
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
#[cfg(feature = "ssr")]
use syntect::parsing::SyntaxSet;
#[cfg(feature = "ssr")]
use syntect::util::LinesWithEndings;

/// Highlight code and return HTML with inline styles
///
/// # Arguments
/// * `code` - The source code to highlight
/// * `language` - Language name (e.g., "r", "python", "rust")
///
/// # Returns
/// HTML string with syntax-highlighted code using inline styles
///
/// # Example
/// ```ignore
/// let html = highlight_code("print('hello')", "python");
/// // Returns: <span style="color:#...">print</span><span>...</span>
/// ```
#[cfg(feature = "ssr")]
pub fn highlight_code(code: &str, language: &str) -> String {
    use std::fmt::Write;

    // Load syntaxes and themes
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Use a dark theme that matches the site aesthetic (teal/slate palette)
    // "base16-ocean.dark" has a nice blue/teal palette
    let theme = &ts.themes["base16-ocean.dark"];

    // Map common language aliases to syntect names
    let syntax_name = match language.to_lowercase().as_str() {
        "r" => "R",
        "python" | "py" => "Python",
        "rust" | "rs" => "Rust",
        "javascript" | "js" => "JavaScript",
        "typescript" | "ts" => "TypeScript",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "bash" | "sh" | "shell" => "Bash",
        "sql" => "SQL",
        "markdown" | "md" => "Markdown",
        "html" => "HTML",
        "css" => "CSS",
        _ => language, // Try the language name as-is
    };

    // Find syntax definition
    let syntax = ss
        .find_syntax_by_name(syntax_name)
        .or_else(|| ss.find_syntax_by_extension(syntax_name))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut html = String::with_capacity(code.len() * 2);

    // Highlight each line
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ss).unwrap_or_default();

        let line_html = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::No)
            .unwrap_or_else(|_| {
                // Fallback: simple HTML escaping
                line.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;")
            });

        let _ = write!(html, "{line_html}");
    }

    html
}

/// Fallback for non-SSR builds (returns plain text - should not be called in practice)
#[cfg(not(feature = "ssr"))]
pub fn highlight_code(code: &str, _language: &str) -> String {
    // Simple HTML escaping without external dependency
    code.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "ssr")]
    fn test_highlight_python() {
        let code = "def hello():\n    print('world')";
        let html = highlight_code(code, "python");

        // Should contain span tags with style attributes
        assert!(html.contains("<span"));
        assert!(html.contains("style="));
        assert!(html.contains("def"));
        assert!(html.contains("hello"));
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_highlight_r() {
        let code = "x <- c(1, 2, 3)\nprint(x)";
        let html = highlight_code(code, "r");

        assert!(html.contains("<span"));
        assert!(html.contains("&lt;-")); // <- should be escaped
    }

    #[test]
    #[cfg(not(feature = "ssr"))]
    fn test_fallback_no_ssr() {
        let code = "def hello():\n    print('world')";
        let html = highlight_code(code, "python");

        // Should be plain escaped text
        assert!(!html.contains("<span"));
        assert!(html.contains("def"));
    }
}
