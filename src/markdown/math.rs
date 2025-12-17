//! Math rendering via KaTeX.
//!
//! Detects LaTeX math delimiters in text and renders them to HTML using KaTeX.
//!
//! ## Supported Delimiters
//!
//! - Inline: `$...$` or `\(...\)` (pre-processed before markdown parsing)
//! - Display: `$$...$$` or `\[...\]` (including multiline blocks)
//!
//! ## Pre-processing for `\(...\)` Syntax
//!
//! The `\(...\)` delimiters are pre-processed BEFORE pulldown-cmark runs because
//! markdown consumes the backslash as an escape character. The pre-processor
//! replaces `\(...\)` with rendered KaTeX HTML, which pulldown-cmark then
//! treats as raw HTML and passes through unchanged.
//!
//! ## Multiline Display Math
//!
//! Display math blocks that span multiple lines in markdown:
//! ```markdown
//! $$
//! y = mx + b
//! $$
//! ```
//! are handled by accumulating events until the closing delimiter is found.
//!
//! ## Safety
//!
//! Math detection is skipped inside code blocks and inline code to avoid
//! false positives (e.g., shell variable syntax like `$HOME`).

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

#[cfg(feature = "ssr")]
use regex::Regex;
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

/// Regex for inline math with \(...\) delimiters.
/// Must be processed before markdown parsing.
#[cfg(feature = "ssr")]
static INLINE_MATH_PAREN: LazyLock<Regex> = LazyLock::new(|| {
    // Match \( ... \) but not inside code blocks (handled separately)
    Regex::new(r"\\\(([^)]+)\\\)").expect("Invalid regex")
});

/// Pre-process markdown to handle `\(...\)` inline math.
///
/// This runs BEFORE pulldown-cmark to prevent markdown from consuming
/// the backslash escapes. Replaces `\(...\)` with rendered KaTeX HTML.
///
/// Note: This does NOT handle `\(...\)` inside code blocks - those are
/// skipped by the regex not matching multi-line patterns and by the
/// nature of fenced code blocks being separate from prose.
#[cfg(feature = "ssr")]
pub fn preprocess_inline_math(content: &str) -> String {
    INLINE_MATH_PAREN.replace_all(content, |caps: &regex::Captures| {
        let expr = &caps[1];
        render_katex(expr, false)
    }).into_owned()
}

/// Non-SSR fallback: return content unchanged (math processing happens at SSG build time)
#[cfg(not(feature = "ssr"))]
pub fn preprocess_inline_math(content: &str) -> String {
    content.to_string()
}

/// State for tracking multiline display math capture.
#[derive(Debug)]
enum MathCaptureState {
    /// Not currently capturing display math
    None,
    /// Capturing display math content (delimiter type, accumulated text)
    Capturing { delimiter: DisplayDelimiter, content: String },
}

#[derive(Debug, Clone, Copy)]
enum DisplayDelimiter {
    DoubleDollar,  // $$
    Bracket,       // \[...\]
}

/// Render math expressions in text events.
///
/// Scans text events for math delimiters and replaces them with KaTeX-rendered
/// HTML. Handles both single-line and multiline display math blocks.
/// Skips code blocks and inline code to avoid false positives.
pub fn render_math(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len());
    let mut in_code_block = false;
    let mut capture_state = MathCaptureState::None;

    for event in events {
        match &event {
            // Track code block state - flush any pending math capture
            Event::Start(Tag::CodeBlock(_)) => {
                flush_capture(&mut capture_state, &mut result);
                in_code_block = true;
                result.push(into_static(event));
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                result.push(into_static(event));
            }

            // Skip inline code - don't process math in code spans
            Event::Code(_) => {
                flush_capture(&mut capture_state, &mut result);
                result.push(into_static(event));
            }

            // Process text events (only outside code blocks)
            Event::Text(text) if !in_code_block => {
                process_text_event(text, &mut capture_state, &mut result);
            }

            // SoftBreak and HardBreak can be part of multiline math
            Event::SoftBreak if !in_code_block => {
                if let MathCaptureState::Capturing { content, .. } = &mut capture_state {
                    content.push('\n');
                } else {
                    result.push(Event::SoftBreak);
                }
            }
            Event::HardBreak if !in_code_block => {
                if let MathCaptureState::Capturing { content, .. } = &mut capture_state {
                    content.push('\n');
                } else {
                    result.push(Event::HardBreak);
                }
            }

            // Structural events flush any pending math capture
            Event::Start(_) | Event::End(_) => {
                flush_capture(&mut capture_state, &mut result);
                result.push(into_static(event));
            }

            // Pass through everything else
            _ => {
                flush_capture(&mut capture_state, &mut result);
                result.push(into_static(event));
            }
        }
    }

    // Flush any remaining capture at end of document
    flush_capture(&mut capture_state, &mut result);

    result
}

/// Process a text event, handling display math capture state.
fn process_text_event(
    text: &str,
    capture_state: &mut MathCaptureState,
    result: &mut Vec<Event<'static>>,
) {
    match capture_state {
        MathCaptureState::None => {
            // Check if this text starts a multiline display math block
            let trimmed = text.trim();

            // Check for standalone $$ that starts display math
            if trimmed == "$$" {
                *capture_state = MathCaptureState::Capturing {
                    delimiter: DisplayDelimiter::DoubleDollar,
                    content: String::new(),
                };
                return;
            }

            // Check for standalone \[ that starts display math
            if trimmed == r"\[" {
                *capture_state = MathCaptureState::Capturing {
                    delimiter: DisplayDelimiter::Bracket,
                    content: String::new(),
                };
                return;
            }

            // Check for $$ at start of text with content following
            if let Some(rest) = trimmed.strip_prefix("$$") {
                // Check if it also ends with $$ (single-line display math)
                if let Some(expr) = rest.trim().strip_suffix("$$") {
                    // Complete display math on single line
                    let rendered = render_katex(expr.trim(), true);
                    result.push(Event::Html(CowStr::Boxed(rendered.into_boxed_str())));
                    return;
                }
                // Starts with $$ but doesn't end with it - begin capture
                *capture_state = MathCaptureState::Capturing {
                    delimiter: DisplayDelimiter::DoubleDollar,
                    content: rest.to_string(),
                };
                return;
            }

            // Not starting display math - process inline math normally
            let processed = process_math_in_text(text);
            for e in processed {
                result.push(e);
            }
        }

        MathCaptureState::Capturing { delimiter, content } => {
            let trimmed = text.trim();

            // Check for closing delimiter
            let closing = match delimiter {
                DisplayDelimiter::DoubleDollar => "$$",
                DisplayDelimiter::Bracket => r"\]",
            };

            if trimmed == closing {
                // Found standalone closing delimiter - render the math
                let expr = content.trim();
                let rendered = render_katex(expr, true);
                result.push(Event::Html(CowStr::Boxed(rendered.into_boxed_str())));
                *capture_state = MathCaptureState::None;
                return;
            }

            // Check if text ends with closing delimiter
            if let Some(before) = trimmed.strip_suffix(closing) {
                content.push_str(before);
                let expr = content.trim();
                let rendered = render_katex(expr, true);
                result.push(Event::Html(CowStr::Boxed(rendered.into_boxed_str())));
                *capture_state = MathCaptureState::None;
                return;
            }

            // Continue accumulating content
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(text);
        }
    }
}

/// Flush any pending math capture state, emitting accumulated content as-is.
fn flush_capture(capture_state: &mut MathCaptureState, result: &mut Vec<Event<'static>>) {
    if let MathCaptureState::Capturing { delimiter, content } = capture_state {
        // We hit a structural element before finding closing delimiter
        // Emit the opening delimiter and content as text (fallback)
        let opening = match delimiter {
            DisplayDelimiter::DoubleDollar => "$$",
            DisplayDelimiter::Bracket => r"\[",
        };
        let text = format!("{}{}", opening, content);
        result.push(Event::Text(CowStr::Boxed(text.into_boxed_str())));
    }
    *capture_state = MathCaptureState::None;
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
    use pulldown_cmark::{html, Options, Parser};

    // Helper to parse markdown, apply math transform, and render to HTML
    fn parse_transform_render(markdown: &str) -> String {
        let parser = Parser::new_ext(markdown, Options::all());
        let events: Vec<Event> = parser.collect();
        let transformed = render_math(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, transformed.into_iter());
        html_output
    }

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

    // ========================================================================
    // Event-based multiline display math tests
    // ========================================================================

    #[test]
    fn test_multiline_display_math_simple() {
        // Tests the event-based capture of multiline display math
        let md = r#"Here is an equation:

$$
y = mx + b
$$

The end."#;
        let html = parse_transform_render(md);

        // Should NOT contain raw $$ delimiters
        assert!(!html.contains("$$"), "Raw $$ found in output: {}", html);
        // Should contain math-display class
        assert!(html.contains("math-display"), "No math-display found: {}", html);
    }

    #[test]
    fn test_multiline_display_math_with_subscripts() {
        let md = r#"The LGCM equation:

$$
y\_\{it\} = \eta\_\{0i\} + \eta\_\{1i\} \cdot \lambda\_t + \epsilon\_\{it\}
$$

Where the subscripts matter."#;
        let html = parse_transform_render(md);

        assert!(!html.contains("$$"), "Raw $$ found in output");
        assert!(html.contains("math-display"), "No math-display found");
    }

    #[test]
    fn test_single_line_display_math() {
        // Display math on a single line should also work
        let md = "Inline display: $$y = mx + b$$ done.";
        let html = parse_transform_render(md);

        assert!(!html.contains("$$"), "Raw $$ found in output");
        assert!(html.contains("math-display"), "No math-display found");
    }

    #[test]
    fn test_inline_math_preserved() {
        let md = "The formula $E = mc^2$ is famous.";
        let html = parse_transform_render(md);

        // Should contain math-inline, not raw $
        assert!(html.contains("math-inline"), "No math-inline found");
        // Should not have math-display
        assert!(!html.contains("math-display"), "Unexpected math-display found");
    }

    #[test]
    fn test_matrix_multiline() {
        // Test matrix notation spanning multiple lines
        let md = r#"Random effects covariance:

$$
\begin{pmatrix} u_{0j} \\ u_{1j} \end{pmatrix} \sim N
$$

End."#;
        let html = parse_transform_render(md);

        // The key test is that $$ delimiters are NOT in the output
        // and the content is wrapped in math-display
        assert!(!html.contains("$$"), "Raw $$ found in output");
        assert!(html.contains("math-display"), "No math-display found");
        // Note: The actual LaTeX content (\begin{pmatrix}) will be inside
        // KaTeX's rendered HTML or in a <code> fallback, which is correct behavior
    }

    #[test]
    fn test_code_block_preserved() {
        // Math inside code blocks should NOT be processed
        let md = r#"```r
# Using $$ for display math
x <- $$
```"#;
        let html = parse_transform_render(md);

        // Should NOT contain math-display (it's in a code block)
        assert!(!html.contains("math-display"), "Code block math was incorrectly processed");
    }

    #[test]
    fn test_inline_code_preserved() {
        let md = "Use `$x$` for inline math in LaTeX.";
        let html = parse_transform_render(md);

        // The backtick-wrapped content should be preserved as code
        // and not processed as math
        assert!(!html.contains("math-inline") || html.contains("<code>$x$</code>"));
    }
}
