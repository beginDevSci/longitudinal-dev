//! Heading ID generation and outline extraction for anchor navigation.
//!
//! Adds IDs to H2, H3, and H4 headings for anchor linking, and extracts
//! a hierarchical outline for sidebar navigation.

use crate::models::guide::OutlineNode;
use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};
use std::collections::HashMap;

/// Result of processing headings: transformed events plus extracted outline.
pub struct HeadingResult {
    pub events: Vec<Event<'static>>,
    pub outline: Vec<OutlineNode>,
}

/// Add IDs to H2, H3, and H4 headings for anchor navigation, and extract outline.
///
/// Transforms headings to include an id attribute based on the heading text.
/// E.g., "## Conceptual Foundations" becomes `<h2 id="conceptual-foundations">`.
///
/// Also builds a hierarchical outline structure for sidebar navigation.
pub fn add_heading_ids_with_outline(events: Vec<Event<'_>>) -> HeadingResult {
    let mut output = Vec::with_capacity(events.len());
    let mut in_heading: Option<HeadingLevel> = None;
    let mut heading_text = String::new();

    // Track used slugs for disambiguation
    let mut slug_counts: HashMap<String, u32> = HashMap::new();

    // Flat list of headings, will be converted to tree later
    let mut headings: Vec<(u8, String, String)> = Vec::new();

    for event in events {
        match &event {
            Event::Start(Tag::Heading { level, .. })
                if *level == HeadingLevel::H2
                    || *level == HeadingLevel::H3
                    || *level == HeadingLevel::H4 =>
            {
                in_heading = Some(*level);
                heading_text.clear();
                // Don't emit yet - wait until we have the text
            }
            Event::Text(text) if in_heading.is_some() => {
                heading_text.push_str(text);
            }
            Event::End(TagEnd::Heading(level)) if in_heading == Some(*level) => {
                // Generate slug from heading text (with disambiguation)
                let base_slug = slugify(&heading_text);
                let slug = disambiguate_slug(&base_slug, &mut slug_counts);

                let level = in_heading.take().unwrap();
                let level_num = level_to_num(level);

                // Collect heading for outline
                headings.push((level_num, heading_text.clone(), slug.clone()));

                // Emit heading with ID using proper Tag structure (preserves event type for downstream)
                output.push(Event::Start(Tag::Heading {
                    level,
                    id: Some(CowStr::from(slug)),
                    classes: vec![],
                    attrs: vec![],
                }));
                output.push(Event::Text(CowStr::from(heading_text.clone())));
                output.push(Event::End(TagEnd::Heading(level)));
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

    // Build hierarchical outline from flat list
    let outline = build_outline_tree(headings);

    HeadingResult {
        events: output,
        outline,
    }
}

/// Add IDs to H2, H3, and H4 headings (without returning outline).
///
/// Convenience wrapper for backward compatibility.
pub fn add_heading_ids(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    add_heading_ids_with_outline(events).events
}

/// Build a hierarchical outline tree from a flat list of headings.
fn build_outline_tree(headings: Vec<(u8, String, String)>) -> Vec<OutlineNode> {
    let mut outline: Vec<OutlineNode> = Vec::new();

    for (level, title, id) in headings {
        let node = OutlineNode {
            level,
            title,
            id,
            children: Vec::new(),
        };

        match level {
            2 => {
                // H2 is a top-level node
                outline.push(node);
            }
            3 => {
                // H3 is a child of the last H2
                if let Some(parent) = outline.last_mut() {
                    parent.children.push(node);
                } else {
                    // No H2 parent, add as top-level (shouldn't happen in well-formed docs)
                    outline.push(node);
                }
            }
            4 => {
                // H4 is a child of the last H3
                if let Some(h2_parent) = outline.last_mut() {
                    if let Some(h3_parent) = h2_parent.children.last_mut() {
                        h3_parent.children.push(node);
                    } else {
                        // No H3 parent, add under H2
                        h2_parent.children.push(node);
                    }
                } else {
                    // No parent at all, add as top-level
                    outline.push(node);
                }
            }
            _ => {}
        }
    }

    outline
}

/// Generate a unique slug by appending -2, -3, etc. for duplicates.
fn disambiguate_slug(base: &str, counts: &mut HashMap<String, u32>) -> String {
    let count = counts.entry(base.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base.to_string()
    } else {
        format!("{}-{}", base, count)
    }
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
            } else {
                '-' // Replace whitespace, hyphens, underscores, and special chars
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
