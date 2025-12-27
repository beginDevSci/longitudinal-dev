use crate::math::render_math_in_html;
use crate::types::{JsonCodeBlock, JsonNoteBlock, JsonOutputBlock, JsonStats, JsonStatsBlock};
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use std::fs;
use std::path::Path;

pub fn parse_statistical_analysis_section(
    events: &[Event],
    warnings: &mut Vec<String>,
) -> Result<JsonStats> {
    let mut content_blocks = Vec::new();

    let mut i = 0;
    while i < events.len() {
        // Look for H2 headings with markers
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            ..
        }) = &events[i]
        {
            // Extract heading text and marker from current position
            if let Some((marker, title)) = extract_heading_marker(&events[i..]) {
                match marker.as_str() {
                    "code" => {
                        if let Some(block) = parse_code_block(events, i, &title, warnings) {
                            content_blocks.push(JsonStatsBlock::Code(block));
                        }
                        i = skip_to_next_h2(events, i);
                        continue;
                    }
                    "output" => {
                        if let Some(block) = parse_output_block(events, i, warnings) {
                            content_blocks.push(JsonStatsBlock::Output(block));
                        }
                        i = skip_to_next_h2(events, i);
                        continue;
                    }
                    "note" => {
                        if let Some(block) = parse_note_block(events, i, &title, warnings) {
                            content_blocks.push(JsonStatsBlock::Note(block));
                        }
                        i = skip_to_next_h2(events, i);
                        continue;
                    }
                    _ => {
                        // Unknown marker, skip this heading
                    }
                }
            }
        }

        i += 1;
    }

    if content_blocks.is_empty() {
        warnings.push("Statistical Analysis v2 section has no content blocks".to_string());
    }

    Ok(JsonStats { content_blocks })
}

/// Extract marker and title from heading at current position
/// Returns (marker, title_without_marker) if marker found
fn extract_heading_marker(events: &[Event]) -> Option<(String, String)> {
    use crate::utils::extract_marker;

    // First event should be Start(Heading)
    if !matches!(events.first(), Some(Event::Start(Tag::Heading { .. }))) {
        return None;
    }

    // Collect text from heading
    let mut heading_text = String::new();
    for event in events.iter().skip(1) {
        match event {
            Event::Text(t) => heading_text.push_str(t.as_ref()),
            Event::End(TagEnd::Heading(_)) => break,
            _ => {}
        }
    }

    extract_marker(&heading_text)
}

fn parse_code_block(
    events: &[Event],
    start_pos: usize,
    title: &str,
    warnings: &mut Vec<String>,
) -> Option<JsonCodeBlock> {
    // Find the code fence after the heading
    let mut i = start_pos + 1;
    let mut code_content = String::new();
    let mut language = "r".to_string();
    let mut found_code = false;

    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::CodeBlock(kind)) => {
                found_code = true;
                // Extract language if specified
                if let CodeBlockKind::Fenced(lang) = kind {
                    if !lang.is_empty() {
                        language = lang.to_string();
                    }
                }
            }
            Event::Text(text) if found_code => {
                code_content.push_str(text.as_ref());
            }
            Event::End(TagEnd::CodeBlock) => {
                break;
            }
            Event::Start(Tag::Heading { .. }) => {
                // Reached next heading
                break;
            }
            _ => {}
        }
        i += 1;
    }

    if code_content.is_empty() {
        warnings.push(format!("Code block '{title}' has no content"));
        return None;
    }

    Some(JsonCodeBlock {
        title: title.to_string(),
        content: code_content.trim().to_string(),
        language,
        filename: None,
        default_open: None,
    })
}

fn parse_output_block(
    events: &[Event],
    start_pos: usize,
    warnings: &mut Vec<String>,
) -> Option<JsonOutputBlock> {
    // Skip past the heading to find content
    let mut i = start_pos + 1;

    // Skip until we're past the heading end tag
    while i < events.len() {
        if matches!(&events[i], Event::End(TagEnd::Heading(_))) {
            i += 1;
            break;
        }
        i += 1;
    }

    // Check if this is an image output
    let mut image_src: Option<String> = None;
    let mut image_alt: Option<String> = None;

    // Look ahead for image tag
    let mut j = i;
    while j < events.len() {
        match &events[j] {
            Event::Start(Tag::Image { dest_url, .. }) => {
                image_src = Some(dest_url.to_string());
            }
            Event::Text(text) if image_src.is_some() && image_alt.is_none() => {
                // This is alt text for the image
                image_alt = Some(text.to_string());
            }
            Event::End(TagEnd::Image) if image_src.is_some() => {
                // Found complete image, return it
                return Some(JsonOutputBlock {
                    content: image_src.unwrap(),
                    format: "image".to_string(),
                    alt: image_alt,
                    caption: None,
                });
            }
            Event::Start(Tag::Heading { .. }) => {
                // Hit next heading without finding complete image
                break;
            }
            Event::End(TagEnd::Paragraph) if image_src.is_none() => {
                // No image found in this paragraph
                break;
            }
            _ => {}
        }
        j += 1;
    }

    // Not an image, collect text content
    let mut content = String::new();
    let mut in_code_block = false;

    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::Text(text) => {
                content.push_str(text.as_ref());
            }
            Event::SoftBreak | Event::HardBreak => {
                content.push('\n');
            }
            Event::End(TagEnd::CodeBlock) => {
                break;
            }
            Event::End(TagEnd::Paragraph) if !in_code_block => {
                break;
            }
            Event::Start(Tag::Heading { .. }) => {
                // Reached next heading
                break;
            }
            _ => {}
        }
        i += 1;
    }

    if content.is_empty() {
        warnings.push("Output block has no content".to_string());
        return None;
    }

    let content_trimmed = content.trim();

    // Check if content is a path to an HTML file
    if content_trimmed.ends_with(".html") {
        // Attempt to read and process the HTML file
        match load_html_file(content_trimmed, warnings) {
            Some(html_content) => {
                return Some(JsonOutputBlock {
                    content: html_content,
                    format: "table".to_string(),
                    alt: None,
                    caption: None,
                });
            }
            None => {
                // Fall through to return as text if file loading fails
                warnings.push(format!("Failed to load HTML file: {content_trimmed}"));
            }
        }
    }

    Some(JsonOutputBlock {
        content: content_trimmed.to_string(),
        format: "text".to_string(),
        alt: None,
        caption: None,
    })
}

fn parse_note_block(
    events: &[Event],
    start_pos: usize,
    title: &str,
    warnings: &mut Vec<String>,
) -> Option<JsonNoteBlock> {
    use pulldown_cmark::html;

    // Skip past the heading to find content
    let mut i = start_pos + 1;

    // Skip until we're past the heading end tag
    while i < events.len() {
        if matches!(&events[i], Event::End(TagEnd::Heading(_))) {
            i += 1;
            break;
        }
        i += 1;
    }

    // Collect all events until the next H2 heading
    let mut content_events = Vec::new();
    while i < events.len() {
        // Stop at next H2 heading
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            ..
        }) = &events[i]
        {
            break;
        }
        content_events.push(events[i].clone());
        i += 1;
    }

    if content_events.is_empty() {
        warnings.push(format!("Note block '{title}' has no content"));
        return None;
    }

    // Convert to HTML to preserve all markdown formatting (lists, tables, code, etc.)
    let mut content_html = String::new();
    html::push_html(&mut content_html, content_events.into_iter());

    if content_html.trim().is_empty() {
        warnings.push(format!("Note block '{title}' has no content"));
        return None;
    }

    // Render any math expressions in the HTML
    let content_with_math = render_math_in_html(content_html.trim());

    Some(JsonNoteBlock {
        title: title.to_string(),
        content: content_with_math,
    })
}

fn skip_to_next_h2(events: &[Event], start_pos: usize) -> usize {
    let mut i = start_pos + 1;
    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            ..
        }) = &events[i]
        {
            return i;
        }
        i += 1;
    }
    events.len()
}

/// Load HTML file and extract inner content
/// Strips DOCTYPE, html, head, and body tags to get just the content
fn load_html_file(file_path: &str, warnings: &mut Vec<String>) -> Option<String> {
    // Handle both absolute paths and paths starting with /
    let path_to_read = if let Some(stripped) = file_path.strip_prefix('/') {
        // Remove leading slash and prepend "public" for paths like /stage4-artifacts/...
        Path::new("public").join(stripped)
    } else {
        Path::new(file_path).to_path_buf()
    };

    // Read the file
    let html_content = match fs::read_to_string(&path_to_read) {
        Ok(content) => content,
        Err(e) => {
            warnings.push(format!(
                "Failed to read HTML file '{}': {}",
                path_to_read.display(),
                e
            ));
            return None;
        }
    };

    // Extract inner content (strip DOCTYPE, html, head, body tags)
    // Look for content between <body> and </body>, or between first div and last div
    let inner_content = extract_inner_html(&html_content);

    if inner_content.trim().is_empty() {
        warnings.push(format!(
            "HTML file '{file_path}' contains no extractable content"
        ));
        return None;
    }

    Some(inner_content)
}

/// Extract inner HTML content, removing DOCTYPE, html, head, and body wrapper tags
fn extract_inner_html(html: &str) -> String {
    let lines: Vec<&str> = html.lines().collect();

    // Find the range of content we want to extract
    // Strategy: Skip DOCTYPE, html, head tags and body opening tag
    // Take everything until closing body tag

    let mut start_idx = 0;
    let mut end_idx = lines.len();

    // Skip DOCTYPE, <html>, <head>, and find <body>
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("<body") {
            start_idx = i + 1; // Start after <body> tag
            break;
        }
        // If we find content before body tag (e.g., a div), start there
        if trimmed.starts_with("<div") || trimmed.starts_with("<table") {
            start_idx = i;
            break;
        }
    }

    // Find </body> or </html> from the end
    for (i, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("</body") || trimmed.starts_with("</html") {
            end_idx = i;
            break;
        }
    }

    // Extract the content between start and end
    if start_idx < end_idx {
        lines[start_idx..end_idx].join("\n")
    } else {
        // Fallback: return everything if we can't find body tags
        html.to_string()
    }
}
