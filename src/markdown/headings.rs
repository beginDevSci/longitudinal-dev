//! Heading ID generation for anchor navigation.
//!
//! Adds IDs to H2 and H3 headings for anchor linking.

use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

/// Add IDs to H2 and H3 headings for anchor navigation.
///
/// Transforms headings to include an id attribute based on the heading text.
/// E.g., "## Conceptual Foundations" becomes `<h2 id="conceptual-foundations">`.
pub fn add_heading_ids(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut output = Vec::with_capacity(events.len());
    let mut in_heading: Option<HeadingLevel> = None;
    let mut heading_text = String::new();

    for event in events {
        match &event {
            Event::Start(Tag::Heading { level, .. }) if *level == HeadingLevel::H2 || *level == HeadingLevel::H3 => {
                in_heading = Some(*level);
                heading_text.clear();
                // Don't emit yet - wait until we have the text
            }
            Event::Text(text) if in_heading.is_some() => {
                heading_text.push_str(text);
            }
            Event::End(TagEnd::Heading(level)) if in_heading == Some(*level) => {
                // Generate slug from heading text
                let slug = slugify(&heading_text);

                // Emit the heading with ID
                let level = in_heading.take().unwrap();
                output.push(Event::Html(CowStr::from(format!(
                    "<h{} id=\"{}\">",
                    level_to_num(level),
                    slug
                ))));
                output.push(Event::Text(CowStr::from(heading_text.clone())));
                output.push(Event::Html(CowStr::from(format!("</h{}>", level_to_num(level)))));
                heading_text.clear();
            }
            _ => {
                if in_heading.is_some() {
                    // Still collecting heading content
                    match &event {
                        Event::Code(code) => {
                            heading_text.push_str(code);
                        }
                        Event::SoftBreak | Event::HardBreak => {
                            heading_text.push(' ');
                        }
                        _ => {}
                    }
                } else {
                    output.push(into_static(event));
                }
            }
        }
    }

    output
}

/// Convert Event to static lifetime.
fn into_static(event: Event<'_>) -> Event<'static> {
    match event {
        Event::Start(tag) => Event::Start(tag_into_static(tag)),
        Event::End(tag) => Event::End(tag),
        Event::Text(s) => Event::Text(CowStr::from(s.into_string())),
        Event::Code(s) => Event::Code(CowStr::from(s.into_string())),
        Event::Html(s) => Event::Html(CowStr::from(s.into_string())),
        Event::InlineHtml(s) => Event::InlineHtml(CowStr::from(s.into_string())),
        Event::FootnoteReference(s) => Event::FootnoteReference(CowStr::from(s.into_string())),
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(b) => Event::TaskListMarker(b),
    }
}

/// Convert Tag to static lifetime.
fn tag_into_static(tag: Tag<'_>) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading { level, id, classes, attrs } => Tag::Heading {
            level,
            id: id.map(|s| CowStr::from(s.into_string())),
            classes: classes.into_iter().map(|s| CowStr::from(s.into_string())).collect(),
            attrs: attrs.into_iter().map(|(k, v)| {
                (CowStr::from(k.into_string()), v.map(|s| CowStr::from(s.into_string())))
            }).collect(),
        },
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
            pulldown_cmark::CodeBlockKind::Indented => pulldown_cmark::CodeBlockKind::Indented,
            pulldown_cmark::CodeBlockKind::Fenced(s) => {
                pulldown_cmark::CodeBlockKind::Fenced(CowStr::from(s.into_string()))
            }
        }),
        Tag::List(n) => Tag::List(n),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(s) => Tag::FootnoteDefinition(CowStr::from(s.into_string())),
        Tag::Table(alignments) => Tag::Table(alignments),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link { link_type, dest_url, title, id } => Tag::Link {
            link_type,
            dest_url: CowStr::from(dest_url.into_string()),
            title: CowStr::from(title.into_string()),
            id: CowStr::from(id.into_string()),
        },
        Tag::Image { link_type, dest_url, title, id } => Tag::Image {
            link_type,
            dest_url: CowStr::from(dest_url.into_string()),
            title: CowStr::from(title.into_string()),
            id: CowStr::from(id.into_string()),
        },
        Tag::HtmlBlock => Tag::HtmlBlock,
        Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
    }
}

/// Convert heading level enum to number.
fn level_to_num(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Convert heading text to URL-friendly slug.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-' // Replace special chars like & with -
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("Overview"), "overview");
        assert_eq!(slugify("Conceptual Foundations"), "conceptual-foundations");
    }

    #[test]
    fn test_slugify_with_ampersand() {
        assert_eq!(slugify("Model Specification & Fit"), "model-specification-fit");
        assert_eq!(slugify("Reference & Resources"), "reference-resources");
    }
}
