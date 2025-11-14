/*!
 * Validation API for structural checks
 *
 * Provides a clean API for the validator to extract structure without
 * reimplementing markdown/YAML parsing.
 */

use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options as MdOptions, Parser, Tag, TagEnd,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ParsedStructure {
    pub frontmatter: FrontmatterData,
    pub sections: Vec<Section>,
    pub subsections: Vec<Subsection>,
}

#[derive(Debug, Clone)]
pub struct FrontmatterData {
    pub fields: HashMap<String, Value>,
    pub line_count: usize,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct Subsection {
    pub parent_section: String,
    pub title: String,
    pub marker: Option<String>,
    pub line_number: usize,
    pub has_code_fence: bool,
    pub code_fence_language: Option<String>,
}

/// Extract frontmatter and parse as raw JSON for flexible type handling
fn extract_frontmatter_raw(content: &str) -> anyhow::Result<(HashMap<String, Value>, &str, usize)> {
    // Check if content starts with "---"
    if !content.starts_with("---") {
        return Ok((HashMap::new(), content, 0));
    }

    // Find the closing "---"
    let rest = &content[3..];
    if let Some(end_pos) = rest.find("\n---\n") {
        let yaml_content = &rest[..end_pos];
        let remaining = &rest[end_pos + 5..]; // Skip past "\n---\n"

        // Parse YAML directly to serde_json::Value for flexible typing
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;

        // Convert serde_yaml::Value to serde_json::Value
        let json_str = serde_json::to_string(&yaml_value)?;
        let json_value: Value = serde_json::from_str(&json_str)?;

        let fields = if let Value::Object(map) = json_value {
            map.into_iter().collect()
        } else {
            HashMap::new()
        };

        // Calculate frontmatter line offset
        let frontmatter_line_offset = content[..content.len() - remaining.len()].lines().count();

        Ok((fields, remaining, frontmatter_line_offset))
    } else {
        // No closing delimiter found
        Ok((HashMap::new(), content, 0))
    }
}

/// Parse markdown structure for validation
pub fn parse_structure(markdown_content: &str) -> anyhow::Result<ParsedStructure> {
    // Extract frontmatter - parse raw YAML for flexible type handling
    let (fields, content_after_fm, frontmatter_line_offset) =
        extract_frontmatter_raw(markdown_content)?;

    let frontmatter_data = FrontmatterData {
        fields,
        line_count: frontmatter_line_offset,
    };

    // Build line tracker over content_after_fm (not whole file)
    let line_tracker = LineTracker::new(content_after_fm);

    // Parse markdown with pulldown-cmark
    let mut md_options = MdOptions::empty();
    md_options.insert(MdOptions::ENABLE_TABLES);
    md_options.insert(MdOptions::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(content_after_fm, md_options);
    let events: Vec<_> = parser.into_offset_iter().collect();

    let mut sections: Vec<Section> = Vec::new();
    let mut subsections: Vec<Subsection> = Vec::new();
    let mut current_section = String::new();

    let mut i = 0;
    while i < events.len() {
        let (event, range) = &events[i];
        // Get line within content_after_fm, then add frontmatter offset
        let line = line_tracker.line_for_byte(range.start) + frontmatter_data.line_count;

        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                let heading_text = extract_heading_text(&events[i..]);
                sections.push(Section {
                    title: heading_text.clone(),
                    line_number: line,
                });
                current_section = heading_text;
            }
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                ..
            }) => {
                let heading_text = extract_heading_text(&events[i..]);
                let (clean_title, marker) = extract_marker(&heading_text);

                let subsection_idx = subsections.len();
                subsections.push(Subsection {
                    parent_section: current_section.clone(),
                    title: clean_title,
                    marker: marker.clone(),
                    line_number: line,
                    has_code_fence: false,
                    code_fence_language: None,
                });

                // If marker is "code", look for next code fence
                if marker.as_deref() == Some("code") {
                    // Find next code fence
                    let mut j = i + 1;
                    while j < events.len() {
                        if let (Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))), _) =
                            &events[j]
                        {
                            subsections[subsection_idx].has_code_fence = true;
                            subsections[subsection_idx].code_fence_language =
                                Some(lang.to_string());
                            break;
                        }
                        // Stop if we hit another heading
                        if matches!(events[j].0, Event::Start(Tag::Heading { .. })) {
                            break;
                        }
                        j += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok(ParsedStructure {
        frontmatter: frontmatter_data,
        sections,
        subsections,
    })
}

/// Extract marker from heading text: "Heading {.marker}" -> ("Heading", Some("marker"))
fn extract_marker(heading_text: &str) -> (String, Option<String>) {
    // Match {.marker} at end of heading
    if let Some(start) = heading_text.rfind("{.") {
        if let Some(end) = heading_text[start..].find('}') {
            let marker = &heading_text[start + 2..start + end];
            let clean_title = heading_text[..start].trim().to_string();
            return (clean_title, Some(marker.to_string()));
        }
    }
    (heading_text.to_string(), None)
}

/// Extract heading text from event stream starting at heading
fn extract_heading_text(events: &[(Event, std::ops::Range<usize>)]) -> String {
    let mut text = String::new();
    let mut i = 1; // Skip the heading start event

    while i < events.len() {
        match &events[i].0 {
            Event::Text(t) => text.push_str(t.as_ref()),
            Event::Code(c) => text.push_str(c.as_ref()),
            Event::End(TagEnd::Heading(_)) => break,
            _ => {}
        }
        i += 1;
    }

    text
}

/// Track line numbers for byte positions
struct LineTracker {
    byte_to_line: HashMap<usize, usize>,
}

impl LineTracker {
    fn new(content: &str) -> Self {
        let mut byte_to_line = HashMap::new();
        let mut line = 1;

        byte_to_line.insert(0, 1);

        for (byte_pos, ch) in content.char_indices() {
            byte_to_line.insert(byte_pos, line);
            if ch == '\n' {
                line += 1;
                byte_to_line.insert(byte_pos + 1, line);
            }
        }

        Self { byte_to_line }
    }

    fn line_for_byte(&self, byte_pos: usize) -> usize {
        // Find closest line
        self.byte_to_line.get(&byte_pos).copied().unwrap_or(1)
    }
}
