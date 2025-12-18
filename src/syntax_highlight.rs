//! Syntax highlighting at build time using syntect
//!
//! This module provides compile-time syntax highlighting for code blocks.
//! Only compiled in SSR mode (not included in WASM bundle).
//!
//! Uses CSS classes for styling (defined in input.css) for easy customization.
//! Supports: R, Python, Rust, JavaScript, TypeScript, JSON, YAML, etc.

#[cfg(feature = "ssr")]
use syntect::easy::HighlightLines;
#[cfg(feature = "ssr")]
use syntect::highlighting::{FontStyle, Style, ThemeSet};
#[cfg(feature = "ssr")]
use syntect::parsing::SyntaxSet;
#[cfg(feature = "ssr")]
use syntect::util::LinesWithEndings;

/// Highlight code and return HTML with CSS classes for styling
///
/// # Arguments
/// * `code` - The source code to highlight
/// * `language` - Language name (e.g., "r", "python", "rust")
///
/// # Returns
/// HTML string with syntax-highlighted code using CSS classes:
/// - `.syn-comment` - Comments
/// - `.syn-keyword` - Keywords, control flow
/// - `.syn-string` - String literals
/// - `.syn-number` - Numeric literals
/// - `.syn-function` - Function names/calls
/// - `.syn-variable` - Variables, parameters
/// - `.syn-operator` - Operators
/// - `.syn-punctuation` - Punctuation, brackets
/// - `.syn-type` - Type names
///
/// # Example
/// ```ignore
/// let html = highlight_code("print('hello')", "python");
/// // Returns: <span class="syn-function">print</span>...
/// ```
#[cfg(feature = "ssr")]
pub fn highlight_code(code: &str, language: &str) -> String {
    use std::fmt::Write;

    // Load syntaxes and themes
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Use base16-ocean.dark to get scope classifications
    // (we'll ignore the colors and use CSS classes instead)
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
    let mut html = String::with_capacity(code.len() * 3);

    // Highlight each line using CSS classes instead of inline styles
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ss).unwrap_or_default();

        for (style, text) in ranges {
            let escaped = html_escape(text);

            // Map syntect style to CSS class based on color heuristics
            let class = style_to_class(&style);

            if class.is_empty() {
                let _ = write!(html, "{escaped}");
            } else {
                let _ = write!(html, r#"<span class="{class}">{escaped}</span>"#);
            }
        }
    }

    html
}

/// Map syntect style to CSS class name based on color
#[cfg(feature = "ssr")]
fn style_to_class(style: &Style) -> &'static str {
    let r = style.foreground.r;
    let g = style.foreground.g;
    let b = style.foreground.b;

    // Check for italic (usually comments)
    if style.font_style.contains(FontStyle::ITALIC) {
        return "syn-comment";
    }

    // base16-ocean.dark color mappings:
    // Comments: #65737e (gray)
    // Strings: #a3be8c (green)
    // Numbers: #d08770 (orange)
    // Keywords: #b48ead (purple)
    // Functions: #8fa1b3 (blue)
    // Variables: #c0c5ce (light gray - default)
    // Operators: #c0c5ce

    // Gray tones (comments)
    if r < 120 && g < 130 && b < 140 && (r as i32 - g as i32).abs() < 20 {
        return "syn-comment";
    }

    // Green tones (strings)
    if g > r && g > b && g > 150 {
        return "syn-string";
    }

    // Orange/red tones (numbers, constants)
    if r > 180 && g < 160 && b < 140 {
        return "syn-number";
    }

    // Purple/magenta tones (keywords)
    if r > 150 && b > 150 && g < r && g < b {
        return "syn-keyword";
    }

    // Blue tones (functions)
    if b > r && b > g && b > 140 {
        return "syn-function";
    }

    // Cyan/teal tones (types, special)
    if g > r && b > r && g > 140 && b > 140 {
        return "syn-type";
    }

    // Yellow tones (variables in some themes)
    if r > 200 && g > 180 && b < 150 {
        return "syn-variable";
    }

    // Default: no special class (uses base code color)
    ""
}

/// HTML escape for code content
#[cfg(feature = "ssr")]
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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

        // Should contain span tags with CSS classes
        assert!(html.contains("<span"));
        assert!(html.contains("class="));
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
    #[cfg(feature = "ssr")]
    fn test_highlight_has_classes() {
        let code = "# This is a comment\nx <- 'hello'";
        let html = highlight_code(code, "r");

        // Should use CSS classes for styling
        assert!(html.contains("syn-comment") || html.contains("syn-string"));
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
