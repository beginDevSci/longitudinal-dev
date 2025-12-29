//! Callout transformation for blockquotes.
//!
//! Transforms blockquotes with callout markers into styled callout boxes.
//!
//! ## Supported Syntax
//!
//! GitHub-style:
//! ```markdown
//! > [!tip]
//! > This is a tip.
//! ```
//!
//! Bold-prefix style:
//! ```markdown
//! > **Warning:** Be careful with XYZ.
//! ```
//!
//! ## Callout Types
//!
//! - `tip` - Helpful suggestions
//! - `warning` - Important cautions (also `[!important]`)
//! - `note` - Additional information
//! - `pitfall` - Common mistakes to avoid (also `[!caution]`)
//! - `info` - General information
//! - `didactic` - Conceptual clarifications

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};

/// Callout type with display metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutType {
    Tip,
    Warning,
    Note,
    Pitfall,
    Info,
    Didactic,
}

impl CalloutType {
    /// CSS class suffix for this callout type.
    fn class_name(&self) -> &'static str {
        match self {
            CalloutType::Tip => "tip",
            CalloutType::Warning => "warning",
            CalloutType::Note => "note",
            CalloutType::Pitfall => "pitfall",
            CalloutType::Info => "info",
            CalloutType::Didactic => "didactic",
        }
    }

    /// Display title for this callout type.
    fn title(&self) -> &'static str {
        match self {
            CalloutType::Tip => "Tip",
            CalloutType::Warning => "Warning",
            CalloutType::Note => "Note",
            CalloutType::Pitfall => "Pitfall",
            CalloutType::Info => "Info",
            CalloutType::Didactic => "Didactic",
        }
    }

    /// Parse from a GitHub-style marker like `[!tip]` or `[!important]`.
    fn from_github_marker(marker: &str) -> Option<Self> {
        let marker = marker.trim().to_lowercase();
        match marker.as_str() {
            "tip" => Some(CalloutType::Tip),
            "warning" => Some(CalloutType::Warning),
            "important" => Some(CalloutType::Warning), // Map to warning
            "note" => Some(CalloutType::Note),
            "pitfall" => Some(CalloutType::Pitfall),
            "caution" => Some(CalloutType::Pitfall), // Map to pitfall
            "info" => Some(CalloutType::Info),
            "didactic" => Some(CalloutType::Didactic),
            _ => None,
        }
    }

    /// Parse from a bold-prefix marker like `Warning:` or `Note:`.
    fn from_bold_marker(marker: &str) -> Option<Self> {
        let marker = marker.trim().trim_end_matches(':').to_lowercase();
        match marker.as_str() {
            "tip" => Some(CalloutType::Tip),
            "warning" => Some(CalloutType::Warning),
            "important" => Some(CalloutType::Warning),
            "note" => Some(CalloutType::Note),
            "pitfall" => Some(CalloutType::Pitfall),
            "caution" => Some(CalloutType::Pitfall),
            "info" => Some(CalloutType::Info),
            "didactic" => Some(CalloutType::Didactic),
            _ => None,
        }
    }
}

/// Result of parsing a callout marker from blockquote content.
#[derive(Debug)]
struct CalloutMatch {
    callout_type: CalloutType,
    /// Events to include in the callout body (with marker stripped).
    body_events: Vec<Event<'static>>,
}

/// Check if blockquote events contain a callout marker and extract it.
///
/// Returns `Some(CalloutMatch)` if a callout marker is found, with the
/// marker stripped from the body events.
fn detect_callout(events: &[Event<'_>]) -> Option<CalloutMatch> {
    // We need to find either:
    // 1. A text starting with [!type] (GitHub-style)
    // 2. A strong tag followed by text ending with ":" (bold-prefix style)

    // First, try GitHub-style: look for text starting with [!
    if let Some(match_result) = try_github_style(events) {
        return Some(match_result);
    }

    // Then try bold-prefix style: **Type:**
    if let Some(match_result) = try_bold_prefix_style(events) {
        return Some(match_result);
    }

    None
}

/// Try to detect GitHub-style callout: `> [!tip]`
///
/// Note: pulldown-cmark may split `[!tip]` into multiple text events like:
/// `"["`, `"!tip"`, `"]"` due to link detection. We need to concatenate
/// early text to detect the pattern.
fn try_github_style(events: &[Event<'_>]) -> Option<CalloutMatch> {
    // First, build a concatenated view of early text content to detect the marker
    let mut early_text = String::new();
    let mut text_event_count = 0;

    for event in events.iter() {
        match event {
            Event::Start(Tag::Paragraph) => continue,
            Event::Text(text) => {
                early_text.push_str(text);
                text_event_count += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                // End of first "line" in the blockquote
                break;
            }
            Event::End(TagEnd::Paragraph) => {
                break;
            }
            _ => break,
        }
    }

    // Check if early text matches [!type] pattern
    let trimmed = early_text.trim();
    if !trimmed.starts_with("[!") {
        return None;
    }

    let bracket_end = trimmed.find(']')?;
    let marker = &trimmed[2..bracket_end];
    let callout_type = CalloutType::from_github_marker(marker)?;

    // Check for content after the marker on the same line
    let after_marker = trimmed[bracket_end + 1..].trim();

    // Build body events, skipping the marker text events
    let mut body_events = Vec::new();
    let mut skip_count = text_event_count; // Skip the text events that form the marker
    let mut past_first_line = false;

    for (idx, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Paragraph) if idx == 0 => {
                // Skip the opening paragraph that contains the marker
                continue;
            }
            Event::Text(_) if skip_count > 0 => {
                skip_count -= 1;
                continue;
            }
            Event::SoftBreak | Event::HardBreak if !past_first_line => {
                past_first_line = true;
                // If there's text after marker, we need to include a soft break
                if !after_marker.is_empty() {
                    body_events.push(Event::SoftBreak);
                }
                continue;
            }
            Event::End(TagEnd::Paragraph) if !past_first_line => {
                past_first_line = true;
                // If there was content after marker, include it in a paragraph
                if !after_marker.is_empty() {
                    body_events.push(Event::Start(Tag::Paragraph));
                    body_events.push(Event::Text(CowStr::Boxed(
                        after_marker.to_string().into_boxed_str(),
                    )));
                    body_events.push(Event::End(TagEnd::Paragraph));
                }
                continue;
            }
            _ => {
                past_first_line = true;
                body_events.push(event.clone().into_static());
            }
        }
    }

    // Clean up
    body_events = clean_leading_whitespace(body_events);

    Some(CalloutMatch {
        callout_type,
        body_events,
    })
}

/// Try to detect bold-prefix style callout: `> **Warning:** text`
fn try_bold_prefix_style(events: &[Event<'_>]) -> Option<CalloutMatch> {
    // Look for pattern: Start(Paragraph), Start(Strong), Text("Type:"), End(Strong), Text(" rest")
    // Or: Start(Paragraph), Start(Strong), Text("Type"), End(Strong), Text(": rest")

    let mut state = BoldPrefixState::LookingForParagraph;
    let mut marker_type: Option<CalloutType> = None;
    let mut body_start_idx: Option<usize> = None;
    let mut text_after_marker: Option<String> = None;

    for (idx, event) in events.iter().enumerate() {
        match (&state, event) {
            (BoldPrefixState::LookingForParagraph, Event::Start(Tag::Paragraph)) => {
                state = BoldPrefixState::InParagraphLookingForStrong;
            }
            (BoldPrefixState::InParagraphLookingForStrong, Event::Start(Tag::Strong)) => {
                state = BoldPrefixState::InStrong;
            }
            (BoldPrefixState::InStrong, Event::Text(text)) => {
                // Check if this is a callout type label
                let trimmed = text.trim().trim_end_matches(':');
                if let Some(ct) = CalloutType::from_bold_marker(trimmed) {
                    marker_type = Some(ct);
                    state = BoldPrefixState::FoundMarkerText;
                } else {
                    // Not a recognized callout type, abort
                    return None;
                }
            }
            (BoldPrefixState::FoundMarkerText, Event::End(TagEnd::Strong)) => {
                state = BoldPrefixState::AfterStrong;
            }
            (BoldPrefixState::AfterStrong, Event::Text(text)) => {
                // This is text after **Type:** - check for leading colon and whitespace
                let trimmed = text.trim_start_matches(':').trim_start();
                if !trimmed.is_empty() {
                    text_after_marker = Some(trimmed.to_string());
                }
                body_start_idx = Some(idx + 1);
                break;
            }
            (BoldPrefixState::AfterStrong, Event::End(TagEnd::Paragraph)) => {
                // The marker was the whole paragraph content
                body_start_idx = Some(idx + 1);
                break;
            }
            // Skip soft breaks and whitespace while looking
            (BoldPrefixState::InParagraphLookingForStrong, Event::Text(t))
                if t.trim().is_empty() =>
            {
                continue;
            }
            (BoldPrefixState::InParagraphLookingForStrong, Event::SoftBreak) => continue,
            // Any other pattern means this isn't a bold-prefix callout
            _ => {
                if !matches!(state, BoldPrefixState::LookingForParagraph) {
                    return None;
                }
            }
        }
    }

    let callout_type = marker_type?;
    let start_idx = body_start_idx?;

    // Build body events starting from after the marker
    let mut body_events = Vec::new();

    // If there was text after the marker on the same line, include it
    if let Some(text) = text_after_marker {
        body_events.push(Event::Start(Tag::Paragraph));
        body_events.push(Event::Text(CowStr::Boxed(text.into_boxed_str())));
    }

    // Include remaining events
    for event in events.iter().skip(start_idx) {
        body_events.push(event.clone().into_static());
    }

    // Clean up
    body_events = clean_leading_whitespace(body_events);

    Some(CalloutMatch {
        callout_type,
        body_events,
    })
}

#[derive(Debug)]
enum BoldPrefixState {
    LookingForParagraph,
    InParagraphLookingForStrong,
    InStrong,
    FoundMarkerText,
    AfterStrong,
}

/// Remove leading empty paragraphs and whitespace from events.
fn clean_leading_whitespace(mut events: Vec<Event<'static>>) -> Vec<Event<'static>> {
    // Remove leading Start(Paragraph) if followed by End(Paragraph) with only whitespace
    while events.len() >= 2 {
        if matches!(&events[0], Event::Start(Tag::Paragraph))
            && matches!(&events[1], Event::End(TagEnd::Paragraph))
        {
            events.drain(0..2);
        } else if matches!(&events[0], Event::SoftBreak | Event::HardBreak) {
            events.remove(0);
        } else if let Event::Text(t) = &events[0] {
            if t.trim().is_empty() {
                events.remove(0);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    events
}

/// Transform blockquotes into callouts where applicable.
///
/// Scans the event stream for blockquotes, checks if they contain callout
/// markers, and replaces matching blockquotes with styled callout HTML.
pub fn transform_callouts(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let mut result = Vec::with_capacity(events.len());
    let mut iter = events.into_iter().peekable();

    while let Some(event) = iter.next() {
        if matches!(event, Event::Start(Tag::BlockQuote)) {
            // Collect all events in this blockquote
            let blockquote_events = collect_until_end_blockquote(&mut iter);

            // Check if this is a callout
            if let Some(callout_match) = detect_callout(&blockquote_events) {
                // Emit callout HTML
                emit_callout(&mut result, callout_match);
            } else {
                // Not a callout, emit original blockquote
                result.push(event.into_static());
                for e in blockquote_events {
                    result.push(e.into_static());
                }
                result.push(Event::End(TagEnd::BlockQuote));
            }
        } else {
            result.push(event.into_static());
        }
    }

    result
}

/// Collect events until we see End(BlockQuote).
fn collect_until_end_blockquote<'a>(
    iter: &mut std::iter::Peekable<impl Iterator<Item = Event<'a>>>,
) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    let mut depth = 1;

    for event in iter.by_ref() {
        match &event {
            Event::Start(Tag::BlockQuote) => depth += 1,
            Event::End(TagEnd::BlockQuote) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        events.push(event);
    }

    events
}

/// Emit callout HTML structure.
fn emit_callout(result: &mut Vec<Event<'static>>, callout: CalloutMatch) {
    let class_name = callout.callout_type.class_name();
    let title = callout.callout_type.title();

    // Opening wrapper
    result.push(Event::Html(CowStr::Boxed(
        format!(
            r#"<div class="callout callout-{}" role="note" aria-label="{}">"#,
            class_name, title
        )
        .into_boxed_str(),
    )));

    // Title
    result.push(Event::Html(CowStr::Boxed(
        format!(
            r#"<div class="callout-title"><span>{}</span></div>"#,
            title
        )
        .into_boxed_str(),
    )));

    // Body opening
    result.push(Event::Html(CowStr::Boxed(
        r#"<div class="callout-body">"#.to_string().into_boxed_str(),
    )));

    // Body content
    for event in callout.body_events {
        result.push(event);
    }

    // Body closing
    result.push(Event::Html(CowStr::Boxed(
        r#"</div>"#.to_string().into_boxed_str(),
    )));

    // Wrapper closing
    result.push(Event::Html(CowStr::Boxed(
        r#"</div>"#.to_string().into_boxed_str(),
    )));
}

/// Extension trait to convert events to 'static lifetime.
trait IntoStatic {
    fn into_static(self) -> Event<'static>;
}

impl<'a> IntoStatic for Event<'a> {
    fn into_static(self) -> Event<'static> {
        match self {
            Event::Start(tag) => Event::Start(tag_into_static(tag)),
            Event::End(tag) => Event::End(tag),
            Event::Text(s) => Event::Text(CowStr::Boxed(s.to_string().into_boxed_str())),
            Event::Code(s) => Event::Code(CowStr::Boxed(s.to_string().into_boxed_str())),
            Event::Html(s) => Event::Html(CowStr::Boxed(s.to_string().into_boxed_str())),
            Event::InlineHtml(s) => {
                Event::InlineHtml(CowStr::Boxed(s.to_string().into_boxed_str()))
            }
            Event::FootnoteReference(s) => {
                Event::FootnoteReference(CowStr::Boxed(s.to_string().into_boxed_str()))
            }
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(b) => Event::TaskListMarker(b),
        }
    }
}

fn tag_into_static(tag: Tag<'_>) -> Tag<'static> {
    use pulldown_cmark::*;
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading { level, id, classes, attrs } => Tag::Heading {
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
        Tag::Link { link_type, dest_url, title, id } => Tag::Link {
            link_type,
            dest_url: CowStr::Boxed(dest_url.to_string().into_boxed_str()),
            title: CowStr::Boxed(title.to_string().into_boxed_str()),
            id: CowStr::Boxed(id.to_string().into_boxed_str()),
        },
        Tag::Image { link_type, dest_url, title, id } => Tag::Image {
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
    use pulldown_cmark::{Options, Parser};

    fn parse_and_transform(markdown: &str) -> Vec<Event<'static>> {
        let parser = Parser::new_ext(markdown, Options::all());
        let events: Vec<Event> = parser.collect();
        transform_callouts(events)
    }

    #[test]
    fn test_github_style_tip() {
        let md = "> [!tip]\n> This is a tip.";
        let events = parse_and_transform(md);

        // Should contain callout HTML
        let html: String = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(s) = e {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert!(html.contains("callout-tip"));
        assert!(html.contains("Tip"));
    }

    #[test]
    fn test_github_style_important_maps_to_warning() {
        let md = "> [!important]\n> This is important.";
        let events = parse_and_transform(md);

        let html: String = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(s) = e {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert!(html.contains("callout-warning"));
    }

    #[test]
    fn test_bold_prefix_warning() {
        let md = "> **Warning:** Be careful!";
        let events = parse_and_transform(md);

        let html: String = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(s) = e {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        assert!(html.contains("callout-warning"));
    }

    #[test]
    fn test_regular_blockquote_preserved() {
        let md = "> This is just a regular quote.";
        let events = parse_and_transform(md);

        // Should NOT contain callout HTML
        let has_callout = events
            .iter()
            .any(|e| matches!(e, Event::Html(s) if s.contains("callout")));

        assert!(!has_callout);
    }
}
