//! Table wrapping for responsive scrolling.
//!
//! Wraps markdown tables in container divs for responsive horizontal scrolling.
//!
//! ## Output Structure
//!
//! ```html
//! <div class="table-wrapper">
//!   <div class="table-container">
//!     <table>...</table>
//!   </div>
//! </div>
//! ```

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};

/// Wrap tables in responsive container divs.
///
/// Detects `Start(Tag::Table)` events and wraps the entire table (until
/// `End(TagEnd::Table)`) in container divs for responsive scrolling.
pub fn wrap_tables(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len() + 10); // Extra capacity for wrapper tags
    let mut iter = events.into_iter().peekable();

    while let Some(event) = iter.next() {
        if let Event::Start(Tag::Table(alignments)) = event {
            // Emit wrapper opening
            result.push(Event::Html(CowStr::Boxed(
                r#"<div class="table-wrapper"><div class="table-container">"#
                    .to_string()
                    .into_boxed_str(),
            )));

            // Emit table start (converted to static)
            result.push(Event::Start(Tag::Table(alignments)));

            // Collect and emit all table events until End(Table)
            let mut depth = 1;
            while let Some(inner_event) = iter.next() {
                match &inner_event {
                    Event::Start(Tag::Table(_)) => depth += 1,
                    Event::End(TagEnd::Table) => {
                        depth -= 1;
                        if depth == 0 {
                            // Emit table end
                            result.push(Event::End(TagEnd::Table));
                            break;
                        }
                    }
                    _ => {}
                }
                result.push(into_static(inner_event));
            }

            // Emit wrapper closing
            result.push(Event::Html(CowStr::Boxed(
                r#"</div></div>"#.to_string().into_boxed_str(),
            )));
        } else {
            result.push(into_static(event));
        }
    }

    result
}

/// Convert an event to 'static lifetime by cloning string data.
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
        let transformed = wrap_tables(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, transformed.into_iter());
        html_output
    }

    #[test]
    fn test_table_wrapped() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = parse_transform_render(md);

        assert!(html.contains(r#"<div class="table-wrapper">"#));
        assert!(html.contains(r#"<div class="table-container">"#));
        assert!(html.contains("<table>"));
        assert!(html.contains("</table>"));
        assert!(html.contains("</div></div>"));
    }

    #[test]
    fn test_non_table_content_unchanged() {
        let md = "# Heading\n\nSome text.";
        let html = parse_transform_render(md);

        assert!(!html.contains("table-wrapper"));
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("Some text."));
    }
}
