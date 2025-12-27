//! Syntax highlighting for code blocks in HTML content.
//!
//! Finds `<code class="language-X">` blocks and applies syntax highlighting
//! using syntect, outputting CSS class-based spans.

use regex::Regex;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Regex to find code blocks: `<code class="language-X">...</code>`
static CODE_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<code class="language-([^"]+)">([\s\S]*?)</code>"#)
        .expect("Invalid code block regex")
});

/// Apply syntax highlighting to code blocks in HTML content.
///
/// Finds `<code class="language-X">...</code>` patterns and replaces
/// the content with syntax-highlighted HTML using CSS classes.
pub fn highlight_code_in_html(html: &str) -> String {
    CODE_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let language = caps.get(1).map_or("", |m| m.as_str());
            let code = caps.get(2).map_or("", |m| m.as_str());

            // Unescape HTML entities before highlighting
            let unescaped = html_unescape(code);

            // Apply syntax highlighting
            let highlighted = highlight_code(&unescaped, language);

            // Return the highlighted code in the same structure
            format!(r#"<code class="language-{language}">{highlighted}</code>"#)
        })
        .into_owned()
}

/// Highlight code and return HTML with CSS classes for styling.
fn highlight_code(code: &str, language: &str) -> String {
    use std::fmt::Write;

    // Load syntaxes and themes
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Use base16-ocean.dark to get scope classifications
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
        _ => language,
    };

    // Find syntax definition
    let syntax = ss
        .find_syntax_by_name(syntax_name)
        .or_else(|| ss.find_syntax_by_extension(syntax_name))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut html = String::with_capacity(code.len() * 3);

    // Highlight each line using CSS classes
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ss).unwrap_or_default();

        for (style, text) in ranges {
            let escaped = html_escape(text);
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

/// Map syntect style to CSS class name based on color.
fn style_to_class(style: &Style) -> &'static str {
    let r = style.foreground.r;
    let g = style.foreground.g;
    let b = style.foreground.b;

    // Check for italic (usually comments)
    if style.font_style.contains(FontStyle::ITALIC) {
        return "syn-comment";
    }

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

    // Yellow tones (variables)
    if r > 200 && g > 180 && b < 150 {
        return "syn-variable";
    }

    // Default: no special class
    ""
}

/// HTML escape for code content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Unescape HTML entities in code content.
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_python_block() {
        let html = r#"<p>Code:</p><pre><code class="language-python">def hello():
    print("world")</code></pre>"#;
        let result = highlight_code_in_html(html);

        assert!(result.contains("syn-keyword") || result.contains("syn-function"));
        assert!(result.contains("syn-string"));
    }

    #[test]
    fn test_highlight_r_block() {
        let html = r#"<pre><code class="language-r">x &lt;- c(1, 2, 3)
print(x)</code></pre>"#;
        let result = highlight_code_in_html(html);

        // Should have syntax classes
        assert!(result.contains("syn-"));
    }

    #[test]
    fn test_preserves_non_code_content() {
        let html = r#"<p>Hello world</p><pre><code class="language-yaml">key: value</code></pre>"#;
        let result = highlight_code_in_html(html);

        assert!(result.contains("<p>Hello world</p>"));
    }
}
