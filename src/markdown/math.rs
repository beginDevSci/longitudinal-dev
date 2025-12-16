//! Math rendering via KaTeX.
//!
//! Detects LaTeX math delimiters in text and renders them to HTML using KaTeX.
//!
//! ## Supported Delimiters
//!
//! - Inline: `$...$` or `\(...\)`
//! - Display: `$$...$$` or `\[...\]`
//!
//! ## Safety
//!
//! Math detection is skipped inside code blocks and inline code to avoid
//! false positives (e.g., shell variable syntax like `$HOME`).

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

/// Render math expressions in text events.
///
/// Scans text events for math delimiters and replaces them with KaTeX-rendered
/// HTML. Skips code blocks and inline code to avoid false positives.
pub fn render_math(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len());
    let mut in_code_block = false;

    for event in events {
        match &event {
            // Track code block state
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                result.push(into_static(event));
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                result.push(into_static(event));
            }

            // Skip inline code - don't process math in code spans
            Event::Code(_) => {
                result.push(into_static(event));
            }

            // Process text events (only outside code blocks)
            Event::Text(text) if !in_code_block => {
                let processed = process_math_in_text(text);
                for e in processed {
                    result.push(e);
                }
            }

            // Pass through everything else
            _ => {
                result.push(into_static(event));
            }
        }
    }

    result
}

/// Process math delimiters in a text string.
///
/// Returns a sequence of events (text and HTML) with math expressions rendered.
fn process_math_in_text(text: &str) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Try to find the next math delimiter
        if let Some((math_start, math_end, is_display, expr)) = find_next_math(remaining) {
            // Emit text before the math
            if math_start > 0 {
                let before = &remaining[..math_start];
                if !before.is_empty() {
                    events.push(Event::Text(CowStr::Boxed(
                        before.to_string().into_boxed_str(),
                    )));
                }
            }

            // Render the math expression
            let rendered = render_katex(&expr, is_display);
            events.push(Event::Html(CowStr::Boxed(rendered.into_boxed_str())));

            // Continue after the math
            remaining = &remaining[math_end..];
        } else {
            // No more math, emit remaining text
            if !remaining.is_empty() {
                events.push(Event::Text(CowStr::Boxed(
                    remaining.to_string().into_boxed_str(),
                )));
            }
            break;
        }
    }

    // If no math was found, return the original text
    if events.is_empty() {
        events.push(Event::Text(CowStr::Boxed(text.to_string().into_boxed_str())));
    }

    events
}

/// Find the next math expression in the string.
///
/// Returns `Some((start, end, is_display, expression))` if found.
fn find_next_math(s: &str) -> Option<(usize, usize, bool, String)> {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Scan for delimiters
    let mut i = 0;
    while i < len {
        // Check for display math: $$ or \[
        if i + 1 < len && bytes[i] == b'$' && bytes[i + 1] == b'$' {
            // Find closing $$
            if let Some((expr, end)) = find_closing_display_dollar(&s[i + 2..]) {
                return Some((i, i + 2 + end, true, expr));
            }
        } else if i + 1 < len && bytes[i] == b'\\' && bytes[i + 1] == b'[' {
            // Find closing \]
            if let Some((expr, end)) = find_closing_bracket(&s[i + 2..]) {
                return Some((i, i + 2 + end, true, expr));
            }
        }
        // Check for inline math: $ or \(
        else if bytes[i] == b'$' {
            // Make sure this isn't part of $$ (already checked above)
            // and isn't escaped
            if i > 0 && bytes[i - 1] == b'\\' {
                i += 1;
                continue;
            }
            // Find closing $
            if let Some((expr, end)) = find_closing_inline_dollar(&s[i + 1..]) {
                return Some((i, i + 1 + end, false, expr));
            }
        } else if i + 1 < len && bytes[i] == b'\\' && bytes[i + 1] == b'(' {
            // Find closing \)
            if let Some((expr, end)) = find_closing_paren(&s[i + 2..]) {
                return Some((i, i + 2 + end, false, expr));
            }
        }

        i += 1;
    }

    None
}

/// Find closing $$ for display math.
fn find_closing_display_dollar(s: &str) -> Option<(String, usize)> {
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'$' {
            let expr = s[..i].to_string();
            return Some((expr, i + 2));
        }
        i += 1;
    }

    None
}

/// Find closing \] for display math.
fn find_closing_bracket(s: &str) -> Option<(String, usize)> {
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b']' {
            let expr = s[..i].to_string();
            return Some((expr, i + 2));
        }
        i += 1;
    }

    None
}

/// Find closing $ for inline math.
fn find_closing_inline_dollar(s: &str) -> Option<(String, usize)> {
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        // Found closing $
        if bytes[i] == b'$' {
            // Make sure it's not escaped
            if i > 0 && bytes[i - 1] == b'\\' {
                i += 1;
                continue;
            }
            // Make sure it's not empty (would be ambiguous)
            if i == 0 {
                return None;
            }
            let expr = s[..i].to_string();
            return Some((expr, i + 1));
        }
        // Don't span across newlines for inline math
        if bytes[i] == b'\n' {
            return None;
        }
        i += 1;
    }

    None
}

/// Find closing \) for inline math.
fn find_closing_paren(s: &str) -> Option<(String, usize)> {
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b')' {
            let expr = s[..i].to_string();
            return Some((expr, i + 2));
        }
        // Don't span across newlines for inline math
        if bytes[i] == b'\n' {
            return None;
        }
        i += 1;
    }

    None
}

/// Render a LaTeX expression using KaTeX.
#[cfg(feature = "ssr")]
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

/// Fallback for non-SSR builds (returns escaped expression).
#[cfg(not(feature = "ssr"))]
fn render_katex(expr: &str, display_mode: bool) -> String {
    let escaped = html_escape(expr);
    if display_mode {
        format!(r#"<div class="math-display"><code>{}</code></div>"#, escaped)
    } else {
        format!(r#"<span class="math-inline"><code>{}</code></span>"#, escaped)
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

/// Convert an event to 'static lifetime.
fn into_static(event: Event<'_>) -> Event<'static> {
    match event {
        Event::Start(tag) => Event::Start(tag_into_static(tag)),
        Event::End(tag) => Event::End(tag),
        Event::Text(s) => Event::Text(CowStr::Boxed(s.to_string().into_boxed_str())),
        Event::Code(s) => Event::Code(CowStr::Boxed(s.to_string().into_boxed_str())),
        Event::Html(s) => Event::Html(CowStr::Boxed(s.to_string().into_boxed_str())),
        Event::InlineHtml(s) => Event::InlineHtml(CowStr::Boxed(s.to_string().into_boxed_str())),
        Event::FootnoteReference(s) => {
            Event::FootnoteReference(CowStr::Boxed(s.to_string().into_boxed_str()))
        }
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(b) => Event::TaskListMarker(b),
    }
}

fn tag_into_static(tag: Tag<'_>) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading {
            level,
            id,
            classes,
            attrs,
        } => Tag::Heading {
            level,
            id: id.map(|s| CowStr::Boxed(s.to_string().into_boxed_str())),
            classes: classes
                .into_iter()
                .map(|s| CowStr::Boxed(s.to_string().into_boxed_str()))
                .collect(),
            attrs: attrs
                .into_iter()
                .map(|(k, v)| {
                    (
                        CowStr::Boxed(k.to_string().into_boxed_str()),
                        v.map(|s| CowStr::Boxed(s.to_string().into_boxed_str())),
                    )
                })
                .collect(),
        },
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
            CodeBlockKind::Indented => CodeBlockKind::Indented,
            CodeBlockKind::Fenced(s) => {
                CodeBlockKind::Fenced(CowStr::Boxed(s.to_string().into_boxed_str()))
            }
        }),
        Tag::List(start) => Tag::List(start),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(s) => {
            Tag::FootnoteDefinition(CowStr::Boxed(s.to_string().into_boxed_str()))
        }
        Tag::Table(alignments) => Tag::Table(alignments),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        } => Tag::Link {
            link_type,
            dest_url: CowStr::Boxed(dest_url.to_string().into_boxed_str()),
            title: CowStr::Boxed(title.to_string().into_boxed_str()),
            id: CowStr::Boxed(id.to_string().into_boxed_str()),
        },
        Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        } => Tag::Image {
            link_type,
            dest_url: CowStr::Boxed(dest_url.to_string().into_boxed_str()),
            title: CowStr::Boxed(title.to_string().into_boxed_str()),
            id: CowStr::Boxed(id.to_string().into_boxed_str()),
        },
        Tag::HtmlBlock => Tag::HtmlBlock,
        Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_inline_math() {
        let s = "The formula $E = mc^2$ is famous.";
        let result = find_next_math(s);
        assert!(result.is_some());
        let (start, end, is_display, expr) = result.unwrap();
        assert_eq!(start, 12); // position of $
        assert_eq!(expr, "E = mc^2");
        assert!(!is_display);
    }

    #[test]
    fn test_find_display_math() {
        let s = "Here is an equation:\n$$\ny = mx + b\n$$\nThe end.";
        let result = find_next_math(s);
        assert!(result.is_some());
        let (_, _, is_display, expr) = result.unwrap();
        assert!(is_display);
        assert!(expr.contains("y = mx + b"));
    }

    #[test]
    fn test_no_math_in_text() {
        let s = "Just regular text without math.";
        let result = find_next_math(s);
        assert!(result.is_none());
    }

    #[test]
    fn test_latex_paren_syntax() {
        let s = r"Inline \(x^2\) math.";
        let result = find_next_math(s);
        assert!(result.is_some());
        let (_, _, is_display, expr) = result.unwrap();
        assert!(!is_display);
        assert_eq!(expr, "x^2");
    }

    #[test]
    fn test_latex_bracket_syntax() {
        let s = r"Display \[y = mx + b\] math.";
        let result = find_next_math(s);
        assert!(result.is_some());
        let (_, _, is_display, expr) = result.unwrap();
        assert!(is_display);
        assert_eq!(expr, "y = mx + b");
    }
}
