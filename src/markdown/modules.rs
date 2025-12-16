//! Collapsible module wrapping for specific H2 sections.
//!
//! Wraps designated H2 sections in `<details>` elements for collapsible
//! content. The heading text becomes the `<summary>`.
//!
//! ## Recognized Modules
//!
//! - "Worked Example" - Practical code walkthrough
//! - "Reference & Resources" / "Reference and Resources" - Links and references
//!
//! ## Output Structure
//!
//! ```html
//! <details class="tutorial-module">
//!   <summary>Worked Example</summary>
//!   <div class="module-content">
//!     <!-- Original H2 content until next H2 or EOF -->
//!   </div>
//! </details>
//! ```

use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

/// H2 headings that should be wrapped in collapsible modules.
const MODULE_HEADINGS: &[&str] = &[
    "worked example",
    "reference & resources",
    "reference and resources",
    "references & resources",
    "references and resources",
];

/// Check if a heading text matches a module heading (case-insensitive).
fn is_module_heading(text: &str) -> bool {
    let normalized = text.trim().to_lowercase();
    MODULE_HEADINGS.iter().any(|&h| normalized == h)
}

/// State machine for tracking module wrapping.
#[derive(Debug)]
enum ModuleState {
    /// Not inside a module section
    Outside,
    /// Inside a module, collecting content
    #[allow(dead_code)]
    InsideModule { heading_text: String },
}

/// Wrap designated H2 sections in collapsible `<details>` elements.
///
/// Detects H2 headings matching module patterns and wraps all content
/// from that heading until the next H2 (or end of document) in a
/// `<details>` element.
pub fn wrap_modules(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len() + 20);
    let mut state = ModuleState::Outside;
    let mut pending_heading_events: Vec<Event<'static>> = Vec::new();
    let mut collecting_heading_text = false;
    let mut heading_text_buffer = String::new();

    let mut iter = events.into_iter().peekable();

    while let Some(event) = iter.next() {
        match (&mut state, &event) {
            // Detect H2 start
            (
                _,
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    ..
                }),
            ) => {
                // First, close any existing module
                if let ModuleState::InsideModule { .. } = &state {
                    close_module(&mut result);
                    state = ModuleState::Outside;
                }

                // Start collecting heading text to determine if this is a module
                collecting_heading_text = true;
                heading_text_buffer.clear();
                pending_heading_events.clear();
                pending_heading_events.push(into_static(event));
            }

            // Collect heading text
            (_, Event::Text(text)) if collecting_heading_text => {
                heading_text_buffer.push_str(text);
                pending_heading_events.push(into_static(Event::Text(text.clone())));
            }

            // Handle other events while collecting heading
            (_, _) if collecting_heading_text => {
                // Check if this is the end of the heading
                if matches!(event, Event::End(TagEnd::Heading(HeadingLevel::H2))) {
                    collecting_heading_text = false;

                    // Determine if this is a module heading
                    if is_module_heading(&heading_text_buffer) {
                        // Start a new module
                        open_module(&mut result, &heading_text_buffer);
                        state = ModuleState::InsideModule {
                            heading_text: heading_text_buffer.clone(),
                        };
                        // Don't emit the original heading events - we used the text in summary
                    } else {
                        // Not a module heading - emit pending events normally
                        for e in pending_heading_events.drain(..) {
                            result.push(e);
                        }
                        result.push(into_static(event));
                    }
                    pending_heading_events.clear();
                } else {
                    // Continue collecting
                    pending_heading_events.push(into_static(event));
                }
            }

            // Normal content while inside a module
            (ModuleState::InsideModule { .. }, _) => {
                result.push(into_static(event));
            }

            // Normal content while outside a module
            (ModuleState::Outside, _) => {
                result.push(into_static(event));
            }
        }
    }

    // Close any open module at the end of the document
    if let ModuleState::InsideModule { .. } = state {
        close_module(&mut result);
    }

    result
}

/// Emit opening tags for a module.
fn open_module(result: &mut Vec<Event<'static>>, heading_text: &str) {
    // Opening <details>
    result.push(Event::Html(CowStr::Boxed(
        r#"<details class="tutorial-module">"#
            .to_string()
            .into_boxed_str(),
    )));

    // <summary> with heading text
    result.push(Event::Html(CowStr::Boxed(
        format!("<summary>{}</summary>", html_escape(heading_text)).into_boxed_str(),
    )));

    // Opening content wrapper
    result.push(Event::Html(CowStr::Boxed(
        r#"<div class="module-content">"#
            .to_string()
            .into_boxed_str(),
    )));
}

/// Emit closing tags for a module.
fn close_module(result: &mut Vec<Event<'static>>) {
    // Close content wrapper
    result.push(Event::Html(CowStr::Boxed(
        r#"</div>"#.to_string().into_boxed_str(),
    )));

    // Close <details>
    result.push(Event::Html(CowStr::Boxed(
        r#"</details>"#.to_string().into_boxed_str(),
    )));
}

/// Basic HTML escaping for summary text.
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
        let parser = Parser::new_ext(markdown, Options::all());
        let events: Vec<Event> = parser.collect();
        let transformed = wrap_modules(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, transformed.into_iter());
        html_output
    }

    #[test]
    fn test_worked_example_wrapped() {
        let md = "## Overview\n\nSome intro.\n\n## Worked Example\n\nCode here.\n\n## Reference & Resources\n\nLinks here.";
        let html = parse_transform_render(md);

        // Overview should NOT be wrapped
        assert!(html.contains("<h2>Overview</h2>"));

        // Worked Example should be wrapped
        assert!(html.contains(r#"<details class="tutorial-module">"#));
        assert!(html.contains("<summary>Worked Example</summary>"));

        // Reference & Resources should also be wrapped
        assert!(html.contains("<summary>Reference &amp; Resources</summary>"));
    }

    #[test]
    fn test_non_module_headings_unchanged() {
        let md = "## Introduction\n\nText.\n\n## Methods\n\nMore text.";
        let html = parse_transform_render(md);

        // Should not contain any module wrappers
        assert!(!html.contains("tutorial-module"));
        assert!(html.contains("<h2>Introduction</h2>"));
        assert!(html.contains("<h2>Methods</h2>"));
    }

    #[test]
    fn test_module_heading_case_insensitive() {
        let md = "## WORKED EXAMPLE\n\nContent.";
        let html = parse_transform_render(md);

        assert!(html.contains("tutorial-module"));
        assert!(html.contains("<summary>WORKED EXAMPLE</summary>"));
    }
}
