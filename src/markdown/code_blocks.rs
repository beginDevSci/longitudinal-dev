//! Code block ID injection and syntax highlighting.
//!
//! Adds unique IDs to code blocks so that copy buttons can reference them,
//! and applies syntax highlighting for supported languages.
//! The IDs follow the pattern `guide-code-{n}` where n is a sequential counter.
//!
//! ## Output Structure
//!
//! Transforms:
//! ```html
//! <pre><code class="language-r">...</code></pre>
//! ```
//!
//! Into:
//! ```html
//! <div class="code-block-wrapper" data-code-id="guide-code-0" data-language="r">
//!   <pre id="guide-code-0"><code class="language-r">
//!     <span style="color:...">highlighted</span>...
//!   </code></pre>
//! </div>
//! ```

use crate::syntax_highlight::highlight_code;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use std::cell::Cell;

thread_local! {
    static CODE_BLOCK_COUNTER: Cell<usize> = const { Cell::new(0) };
}

/// Reset the code block counter (useful for testing).
#[allow(dead_code)]
pub fn reset_counter() {
    CODE_BLOCK_COUNTER.with(|c| c.set(0));
}

/// Add unique IDs, wrapper divs, and syntax highlighting to code blocks.
///
/// Wraps each fenced code block in a container div and adds an ID to the pre element.
/// Applies syntax highlighting for supported languages (R, Python, etc.).
/// This allows JavaScript/WASM to find and attach copy buttons dynamically.
pub fn add_code_block_ids(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len() + 20);
    let mut in_code_block = false;
    let mut current_language = String::new();
    #[allow(unused_assignments)]
    let mut current_code_id = String::new();
    let mut code_buffer = String::new();

    for event in events {
        match &event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_buffer.clear();

                current_language = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };

                // Generate unique ID
                current_code_id = CODE_BLOCK_COUNTER.with(|c| {
                    let id = c.get();
                    c.set(id + 1);
                    format!("guide-code-{}", id)
                });

                // Opening wrapper div
                result.push(Event::Html(CowStr::Boxed(
                    format!(
                        r#"<div class="code-block-wrapper" data-code-id="{}" data-language="{}">"#,
                        current_code_id, current_language
                    )
                    .into_boxed_str(),
                )));

                // Modified pre tag with ID
                result.push(Event::Html(CowStr::Boxed(
                    format!(r#"<pre id="{}">"#, current_code_id).into_boxed_str(),
                )));

                // Code tag with language class (if available)
                if !current_language.is_empty() {
                    result.push(Event::Html(CowStr::Boxed(
                        format!(r#"<code class="language-{}">"#, current_language).into_boxed_str(),
                    )));
                } else {
                    result.push(Event::Html(CowStr::Boxed("<code>".to_string().into_boxed_str())));
                }
            }

            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;

                // Apply syntax highlighting if language is specified
                let highlighted = if !current_language.is_empty() {
                    highlight_code(&code_buffer, &current_language)
                } else {
                    html_escape(&code_buffer)
                };

                result.push(Event::Html(CowStr::Boxed(highlighted.into_boxed_str())));

                // Close code and pre tags
                result.push(Event::Html(CowStr::Boxed(
                    "</code></pre>".to_string().into_boxed_str(),
                )));
                // Close wrapper div
                result.push(Event::Html(CowStr::Boxed(
                    "</div>".to_string().into_boxed_str(),
                )));

                code_buffer.clear();
            }

            Event::Text(text) if in_code_block => {
                // Collect code text for highlighting at end of block
                code_buffer.push_str(text);
            }

            _ => {
                result.push(into_static(event));
            }
        }
    }

    result
}

/// HTML escape for code content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
    use pulldown_cmark::CodeBlockKind;

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

    fn parse_transform_render(markdown: &str) -> String {
        reset_counter();
        let parser = Parser::new_ext(markdown, Options::all());
        let events: Vec<Event> = parser.collect();
        let transformed = add_code_block_ids(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, transformed.into_iter());
        html_output
    }

    #[test]
    fn test_code_block_gets_id() {
        let md = "```r\nx <- 1\n```";
        let html = parse_transform_render(md);

        assert!(html.contains(r#"class="code-block-wrapper""#));
        assert!(html.contains(r#"data-code-id="guide-code-0""#));
        assert!(html.contains(r#"id="guide-code-0""#));
        assert!(html.contains(r#"class="language-r""#));
    }

    #[test]
    fn test_multiple_code_blocks_sequential_ids() {
        let md = "```r\na <- 1\n```\n\n```python\nb = 2\n```";
        let html = parse_transform_render(md);

        assert!(html.contains("guide-code-0"));
        assert!(html.contains("guide-code-1"));
    }

    #[test]
    fn test_code_content_escaped() {
        let md = "```r\nif (x < 3 && y > 2) {}\n```";
        let html = parse_transform_render(md);

        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(html.contains("&amp;"));
    }
}
