use crate::event_stream::{extract_paragraphs, find_all_h3_with_marker};
use crate::types::{JsonResourceCard, JsonResources};
use anyhow::Result;
use pulldown_cmark::{Event, Tag, TagEnd};

/// Parse **Label:** value patterns from event stream
fn parse_metadata_from_events(events: &[Event]) -> (Option<String>, Option<String>) {
    let mut badge = None;
    let mut url = None;
    let mut in_strong = false;
    let mut strong_text = String::new();
    let mut after_strong = String::new();

    for event in events {
        match event {
            Event::Start(Tag::Strong) => {
                in_strong = true;
                strong_text.clear();
                after_strong.clear();
            }
            Event::End(TagEnd::Strong) => {
                in_strong = false;
            }
            Event::Text(t) => {
                if in_strong {
                    strong_text.push_str(t);
                } else if !strong_text.is_empty() {
                    // Text after a strong tag
                    after_strong.push_str(t);

                    // Check if we have a complete Label: value pattern
                    if strong_text.ends_with(':') {
                        let value = after_strong.trim().to_string();
                        match strong_text.as_str() {
                            "Badge:" => badge = Some(value.clone()),
                            "URL:" => url = Some(value.clone()),
                            _ => {}
                        }
                        strong_text.clear();
                        after_strong.clear();
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                // End of line - finalize any pending label
                strong_text.clear();
                after_strong.clear();
            }
            _ => {}
        }
    }

    (badge, url)
}

pub fn parse_additional_resources_section(
    events: &[Event],
    warnings: &mut Vec<String>,
) -> Result<JsonResources> {
    // Find all H3 headings with {.resource} marker
    let h3_resources = find_all_h3_with_marker(events, "resource");

    if h3_resources.is_empty() {
        // No resources - that's okay, it's optional
        return Ok(JsonResources { cards: vec![] });
    }

    let mut cards = Vec::new();

    for (i, (h3_pos, title, _marker)) in h3_resources.iter().enumerate() {
        // Determine end of this H3 section (next H3 or end)
        let section_end = if i + 1 < h3_resources.len() {
            h3_resources[i + 1].0
        } else {
            events.len()
        };

        // Extract events for this H3 section
        let section = &events[*h3_pos..section_end];

        // Parse Badge and URL from strong tags in event stream
        let (badge, url) = parse_metadata_from_events(section);

        // Extract body from paragraphs (skip lines with Badge/URL)
        let paragraphs = extract_paragraphs(section);
        let mut body_text = String::new();
        for para in &paragraphs {
            // Skip paragraphs that seem to contain metadata
            if !para.contains("Badge:") && !para.contains("URL:") && !para.is_empty() {
                body_text = para.clone();
                break;
            }
        }

        // Validate required fields
        if badge.is_none() {
            warnings.push(format!("Resource '{title}': missing **Badge:**"));
        }
        if url.is_none() {
            warnings.push(format!("Resource '{title}': missing **URL:**"));
        }

        cards.push(JsonResourceCard {
            title: title.clone(),
            badge: badge.unwrap_or_else(|| "LINK".to_string()),
            body: body_text,
            url: url.unwrap_or_default(),
        });
    }

    Ok(JsonResources { cards })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn test_parse_resource_cards_with_markers() {
        let markdown = r#"
### Custom Labels Documentation {.resource}

**Badge:** DOCS
**URL:** https://example.com/custom-labels

Complete guide to using custom labels in stats panels.

### Another Resource {.resource}

**Badge:** TUTORIAL
**URL:** https://example.com/tutorial

Learn how to use the feature.
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_additional_resources_section(&events, &mut warnings);

        assert!(result.is_ok());
        let resources = result.unwrap();

        assert_eq!(resources.cards.len(), 2);

        // First card
        assert_eq!(resources.cards[0].title, "Custom Labels Documentation");
        assert_eq!(resources.cards[0].badge, "DOCS");
        assert_eq!(resources.cards[0].url, "https://example.com/custom-labels");
        assert!(resources.cards[0].body.contains("Complete guide"));

        // Second card
        assert_eq!(resources.cards[1].title, "Another Resource");
        assert_eq!(resources.cards[1].badge, "TUTORIAL");
        assert_eq!(resources.cards[1].url, "https://example.com/tutorial");
    }

    #[test]
    fn test_empty_resources() {
        let markdown = "No H3 headings here.";
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_additional_resources_section(&events, &mut warnings);

        assert!(result.is_ok());
        let resources = result.unwrap();
        assert_eq!(resources.cards.len(), 0);
    }
}
