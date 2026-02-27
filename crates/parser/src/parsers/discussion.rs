use crate::math::render_math_in_html;
use crate::syntax::highlight_code_in_html;
use crate::types::{JsonDiscussion, JsonDiscussionItem};
use crate::utils::extract_marker;
use anyhow::Result;
use pulldown_cmark::{html, Event, HeadingLevel, Tag, TagEnd};

pub fn parse_discussion_section(
    events: &[Event],
    warnings: &mut Vec<String>,
) -> Result<JsonDiscussion> {
    // Parse Discussion section into structured items or plain paragraphs.
    // If H2 headings are present, extract collapsible items (title + content).
    // Otherwise, collect all content as HTML paragraphs.

    let mut i = 0;

    // Skip initial H1 heading if present (section title)
    if i < events.len()
        && matches!(
            &events[i],
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            })
        )
    {
        while i < events.len() {
            if matches!(&events[i], Event::End(TagEnd::Heading(_))) {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    let content_start = i;

    // Check whether any H2 headings exist in the remaining events
    let has_h2 = events[content_start..].iter().any(|e| {
        matches!(
            e,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            })
        )
    });

    if has_h2 {
        // --- H2-structured path: extract titled items ---
        let mut items = Vec::new();

        while i < events.len() {
            if let Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) = &events[i]
            {
                i += 1; // Move past heading start

                // Extract heading text
                let mut raw_title = String::new();
                while i < events.len() {
                    match &events[i] {
                        Event::Text(t) => raw_title.push_str(t),
                        Event::Code(c) => raw_title.push_str(c),
                        Event::End(TagEnd::Heading(_)) => {
                            i += 1;
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }

                // Strip {.note} or other markers from title
                let title = if let Some((_marker, clean_title)) = extract_marker(&raw_title) {
                    clean_title
                } else {
                    raw_title.trim().to_string()
                };

                // Collect content until the next H2 or end
                let mut content_events = Vec::new();
                while i < events.len() {
                    if matches!(
                        &events[i],
                        Event::Start(Tag::Heading {
                            level: HeadingLevel::H2,
                            ..
                        })
                    ) {
                        break;
                    }
                    content_events.push(events[i].clone());
                    i += 1;
                }

                let mut content_html = String::new();
                html::push_html(&mut content_html, content_events.into_iter());

                if !title.is_empty() && !content_html.trim().is_empty() {
                    let content_with_syntax = highlight_code_in_html(content_html.trim());
                    let content_with_math = render_math_in_html(&content_with_syntax);
                    items.push(JsonDiscussionItem {
                        title,
                        content: content_with_math,
                    });
                }

                continue;
            }

            i += 1;
        }

        if items.is_empty() {
            warnings.push("Discussion section has H2 headings but no valid content".to_string());
            return Ok(JsonDiscussion {
                items: vec![],
                paragraphs: vec!["No discussion provided.".to_string()],
            });
        }

        Ok(JsonDiscussion {
            items,
            paragraphs: vec![],
        })
    } else {
        // --- Plain paragraph path: no H2 headings ---
        // Render all remaining events to HTML, then extract individual paragraphs.
        let remaining_events: Vec<_> = events[content_start..].to_vec();

        if remaining_events.is_empty() {
            warnings.push("Discussion section has no content".to_string());
            return Ok(JsonDiscussion {
                items: vec![],
                paragraphs: vec!["No discussion provided.".to_string()],
            });
        }

        let mut full_html = String::new();
        html::push_html(&mut full_html, remaining_events.into_iter());

        // Split rendered HTML into individual paragraph blocks.
        // pulldown_cmark wraps each paragraph in <p>...</p>.
        let paragraphs: Vec<String> = full_html
            .split("</p>")
            .filter_map(|chunk| {
                if let Some(start) = chunk.find("<p>") {
                    let inner = &chunk[start + 3..];
                    let trimmed = inner.trim();
                    if !trimmed.is_empty() {
                        // Apply syntax highlighting and math rendering
                        let with_syntax = highlight_code_in_html(trimmed);
                        let with_math = render_math_in_html(&with_syntax);
                        Some(with_math)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if paragraphs.is_empty() {
            warnings.push("Discussion section has no content".to_string());
            return Ok(JsonDiscussion {
                items: vec![],
                paragraphs: vec!["No discussion provided.".to_string()],
            });
        }

        Ok(JsonDiscussion {
            items: vec![],
            paragraphs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn test_parse_discussion_simple_paragraphs() {
        let markdown = r#"
This is the first paragraph of the discussion.

This is the second paragraph with more details.

Here's a third paragraph for good measure.
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_discussion_section(&events, &mut warnings);

        assert!(result.is_ok());
        let discussion = result.unwrap();

        assert_eq!(discussion.paragraphs.len(), 3);
        assert!(discussion.paragraphs[0].contains("first paragraph"));
        assert!(discussion.paragraphs[1].contains("second paragraph"));
        assert!(discussion.paragraphs[2].contains("third paragraph"));
    }

    #[test]
    fn test_parse_discussion_with_h2_headings() {
        let markdown = r#"
## Some Subheading

A paragraph after a heading.

## Another Section

Final paragraph.
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_discussion_section(&events, &mut warnings);

        assert!(result.is_ok());
        let discussion = result.unwrap();

        // Should extract structured items from H2 headings
        assert_eq!(discussion.items.len(), 2);
        assert_eq!(discussion.items[0].title, "Some Subheading");
        assert!(discussion.items[0].content.contains("paragraph after a heading"));
        assert_eq!(discussion.items[1].title, "Another Section");
        assert!(discussion.items[1].content.contains("Final paragraph"));
        assert!(discussion.paragraphs.is_empty());
    }

    #[test]
    fn test_parse_discussion_with_inline_code() {
        let markdown = r#"
The model equation is `eta(t) = eta(t-1) + delta(t)` which describes change.
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_discussion_section(&events, &mut warnings);

        assert!(result.is_ok());
        let discussion = result.unwrap();

        assert_eq!(discussion.paragraphs.len(), 1);
        assert!(discussion.paragraphs[0].contains("eta(t)"));
    }

    #[test]
    fn test_parse_discussion_empty() {
        let markdown = "";
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_discussion_section(&events, &mut warnings);

        assert!(result.is_ok());
        let discussion = result.unwrap();

        // Should have fallback message
        assert_eq!(discussion.paragraphs.len(), 1);
        assert_eq!(discussion.paragraphs[0], "No discussion provided.");
        assert_eq!(warnings.len(), 1);
    }
}
