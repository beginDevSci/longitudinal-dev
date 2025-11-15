use crate::event_stream::extract_paragraphs;
use crate::types::JsonDiscussion;
use anyhow::Result;
use pulldown_cmark::Event;

pub fn parse_discussion_section(
    events: &[Event],
    warnings: &mut Vec<String>,
) -> Result<JsonDiscussion> {
    // Extract all paragraphs from the discussion section
    // No longer looking for specific headings - just get all paragraph text
    let paragraphs = extract_paragraphs(events);

    if paragraphs.is_empty() {
        warnings.push("Discussion section has no paragraphs".to_string());
        return Ok(JsonDiscussion {
            paragraphs: vec!["No discussion provided.".to_string()],
        });
    }

    Ok(JsonDiscussion { paragraphs })
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
