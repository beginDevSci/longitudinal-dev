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
    // Parse Discussion section into structured items (like Data Access)
    // Look for H2 headings to identify collapsible sections

    let mut items = Vec::new();
    let mut i = 0;

    // Skip initial H1 heading if present (section title)
    if i < events.len() && matches!(&events[i], Event::Start(Tag::Heading { .. })) {
        while i < events.len() {
            if matches!(&events[i], Event::End(TagEnd::Heading(_))) {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    while i < events.len() {
        // Look for H2 headings
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
                        i += 1; // Move past heading end
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

            // Collect all content until the next H2 heading or end of section
            let mut content_events = Vec::new();
            while i < events.len() {
                // Stop if we hit another H2 heading
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

            // Convert content to HTML
            let mut content_html = String::new();
            html::push_html(&mut content_html, content_events.into_iter());

            if !title.is_empty() && !content_html.trim().is_empty() {
                // Apply syntax highlighting to code blocks, then render math
                let content_with_syntax = highlight_code_in_html(content_html.trim());
                let content_with_math = render_math_in_html(&content_with_syntax);
                items.push(JsonDiscussionItem {
                    title,
                    content: content_with_math,
                });
            }

            continue; // Continue to next iteration
        }

        i += 1;
    }

    if items.is_empty() {
        warnings.push("Discussion section has no H2 subsections".to_string());
        return Ok(JsonDiscussion {
            items: vec![],
            paragraphs: vec!["No discussion provided.".to_string()],
        });
    }

    Ok(JsonDiscussion {
        items,
        paragraphs: vec![],
    })
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
    fn test_parse_discussion_with_headings() {
        let markdown = r#"
Initial discussion paragraph.

## Some Subheading

Another paragraph after a heading.

## Another Section

Final paragraph.
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_discussion_section(&events, &mut warnings);

        assert!(result.is_ok());
        let discussion = result.unwrap();

        // Should extract all paragraphs, ignoring headings
        assert_eq!(discussion.paragraphs.len(), 3);
        assert!(discussion.paragraphs[0].contains("Initial discussion"));
        assert!(discussion.paragraphs[1].contains("Another paragraph"));
        assert!(discussion.paragraphs[2].contains("Final paragraph"));
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
