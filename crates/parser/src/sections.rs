use crate::types::SectionBoundaries;
use anyhow::{bail, Result};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

const REQUIRED_SECTIONS: [&str; 6] = [
    "Overview",
    "Data Access",
    "Data Preparation",
    "Statistical Analysis",
    "Discussion",
    "Additional Resources",
];

pub fn detect_and_validate_sections(events: &[Event]) -> Result<SectionBoundaries> {
    let mut h1_positions = Vec::new();

    // Find all H1 headings and their positions
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: HeadingLevel::H1,
            ..
        }) = events[i]
        {
            // Extract heading text from next event(s)
            let text = extract_heading_text(&events[i + 1..]);
            h1_positions.push((i, text));
        }
        i += 1;
    }

    // Validate we have exactly 6 sections
    if h1_positions.len() != 6 {
        bail!(
            "Expected 6 H1 sections, found {}. Sections found: {:?}",
            h1_positions.len(),
            h1_positions
                .iter()
                .map(|(_, name)| name.as_str())
                .collect::<Vec<_>>()
        );
    }

    // Validate section names and order
    for (i, (_, name)) in h1_positions.iter().enumerate() {
        if name != REQUIRED_SECTIONS[i] {
            bail!(
                "Section {} should be '{}', found '{}'",
                i + 1,
                REQUIRED_SECTIONS[i],
                name
            );
        }
    }

    // Create section boundaries
    let positions: Vec<usize> = h1_positions.iter().map(|(pos, _)| *pos).collect();
    let end = events.len();

    Ok(SectionBoundaries {
        overview: positions[0]..positions[1],
        data_access: positions[1]..positions[2],
        data_preparation: positions[2]..positions[3],
        statistical_analysis: positions[3]..positions[4],
        discussion: positions[4]..positions[5],
        additional_resources: positions[5]..end,
    })
}

/// Extract text from events until we hit the end tag for the heading
fn extract_heading_text(events: &[Event]) -> String {
    let mut text = String::new();

    for event in events {
        match event {
            Event::Text(t) => text.push_str(t),
            Event::End(TagEnd::Heading(_)) => break,
            _ => {}
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn test_detect_valid_sections() {
        let markdown = r#"
# Overview
Content

# Data Access
Content

# Data Preparation
Content

# Statistical Analysis
Content

# Discussion
Content

# Additional Resources
Content
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let result = detect_and_validate_sections(&events);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_section() {
        let markdown = r#"
# Overview
Content

# Data Access
Content
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let result = detect_and_validate_sections(&events);
        assert!(result.is_err());
    }
}
