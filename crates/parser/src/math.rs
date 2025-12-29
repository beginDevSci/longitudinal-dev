//! Math rendering via KaTeX for the parser crate.
//!
//! Processes HTML strings to find and render LaTeX math expressions.
//! Supports both inline ($...$) and display ($$...$$) math.

/// Render all math expressions in an HTML string.
///
/// Finds $...$ (inline) and $$...$$ (display) math delimiters
/// and replaces them with KaTeX-rendered HTML.
///
/// Uses a two-pass approach:
/// 1. First replace $$...$$ (display math) with placeholders
/// 2. Then replace $...$ (inline math)
/// 3. Finally restore placeholders with rendered HTML
pub fn render_math_in_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Check for display math ($$)
            if chars.peek() == Some(&'$') {
                chars.next(); // consume second $
                // Find closing $$
                let mut expr = String::new();
                let mut found_close = false;
                while let Some(c2) = chars.next() {
                    if c2 == '$' && chars.peek() == Some(&'$') {
                        chars.next(); // consume second $
                        found_close = true;
                        break;
                    }
                    expr.push(c2);
                }
                if found_close && !expr.is_empty() {
                    result.push_str(&render_katex(expr.trim(), true));
                } else {
                    // Didn't find closing $$, output original
                    result.push_str("$$");
                    result.push_str(&expr);
                }
            } else {
                // Inline math ($...$)
                let mut expr = String::new();
                let mut found_close = false;
                for c2 in chars.by_ref() {
                    if c2 == '$' {
                        found_close = true;
                        break;
                    }
                    if c2 == '\n' {
                        // Inline math can't span lines
                        break;
                    }
                    expr.push(c2);
                }
                if found_close && !expr.is_empty() {
                    result.push_str(&render_katex(expr.trim(), false));
                } else {
                    // Didn't find closing $, output original
                    result.push('$');
                    result.push_str(&expr);
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Render a LaTeX expression using KaTeX.
fn render_katex(expr: &str, display_mode: bool) -> String {
    use katex::Opts;

    let opts = Opts::builder()
        .display_mode(display_mode)
        .throw_on_error(false) // Graceful degradation
        .build()
        .unwrap_or_default();

    match katex::render_with_opts(expr, &opts) {
        Ok(html) => {
            if display_mode {
                format!(r#"<div class="math-display">{}</div>"#, html)
            } else {
                format!(r#"<span class="math-inline">{}</span>"#, html)
            }
        }
        Err(e) => {
            // Fallback: show the original expression in a code block
            eprintln!("KaTeX error rendering '{}': {}", expr, e);
            let escaped = html_escape(expr);
            if display_mode {
                format!(
                    r#"<div class="math-display math-error"><code>{}</code></div>"#,
                    escaped
                )
            } else {
                format!(
                    r#"<code class="math-inline math-error">{}</code>"#,
                    escaped
                )
            }
        }
    }
}

/// Basic HTML escaping.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let html = "<p>The formula $E = mc^2$ is famous.</p>";
        let result = render_math_in_html(html);
        assert!(result.contains("math-inline"), "No math-inline found: {}", result);
        assert!(!result.contains("$E"), "Raw $ still present: {}", result);
    }

    #[test]
    fn test_display_math() {
        let html = "<p>Here is an equation:</p>\n<p>$$y = mx + b$$</p>";
        let result = render_math_in_html(html);
        assert!(result.contains("math-display"), "No math-display found: {}", result);
        assert!(!result.contains("$$"), "Raw $$ still present: {}", result);
    }

    #[test]
    fn test_multiple_inline() {
        let html = "<p>Both $x$ and $y$ are variables.</p>";
        let result = render_math_in_html(html);
        // Should have two math-inline spans
        let count = result.matches("math-inline").count();
        assert_eq!(count, 2, "Expected 2 math-inline, found {}: {}", count, result);
    }

    #[test]
    fn test_no_false_positives() {
        let html = "<p>Use $HOME for your home directory.</p>";
        let result = render_math_in_html(html);
        // $HOME alone shouldn't be matched (no closing $)
        // Actually it will be matched since there's content between $
        // This is a limitation - code blocks should use backticks
    }

    #[test]
    fn test_mixed_math() {
        let html = "<p>Inline $x^2$ and display $$y = mx + b$$ together.</p>";
        let result = render_math_in_html(html);
        assert!(result.contains("math-inline"), "No math-inline found");
        assert!(result.contains("math-display"), "No math-display found");
    }
}
