use pulldown_cmark::{Event, Tag, TagEnd};

/// Extract plain text from events
pub fn extract_text(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Text(t) = e {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Extract list items from events
pub fn extract_list_items(events: &[Event]) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_item = false;
    let mut current_text = String::new();

    for event in events {
        match event {
            Event::Start(Tag::Item) => {
                in_item = true;
                current_text.clear();
            }
            Event::End(TagEnd::Item) => {
                if in_item {
                    items.push(current_text.trim().to_string());
                    in_item = false;
                }
            }
            Event::Text(t) if in_item => {
                current_text.push_str(t);
            }
            _ => {}
        }
    }

    items
}

/// Extract paragraphs from events
pub fn extract_paragraphs(events: &[Event]) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut in_paragraph = false;
    let mut current_text = String::new();

    for event in events {
        match event {
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                current_text.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if in_paragraph && !current_text.trim().is_empty() {
                    paragraphs.push(current_text.trim().to_string());
                    in_paragraph = false;
                }
            }
            Event::Text(t) if in_paragraph => {
                current_text.push_str(t);
            }
            _ => {}
        }
    }

    paragraphs
}

/// Find position of H2 heading with specific text
pub fn find_h2_heading(events: &[Event], heading_text: &str) -> Option<usize> {
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            ..
        }) = events[i]
        {
            // Collect ALL text events within the heading (handles long titles)
            let mut full_heading_text = String::new();
            let mut j = i + 1;
            while j < events.len() {
                match &events[j] {
                    Event::Text(t) => full_heading_text.push_str(t.as_ref()),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
                j += 1;
            }

            if full_heading_text == heading_text {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Find H2 heading that contains a specific marker (e.g., "{.stats}")
/// Returns (position, heading_text, marker) if found
pub fn find_h2_with_marker(events: &[Event], marker: &str) -> Option<(usize, String, String)> {
    use crate::utils::extract_marker;

    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            ..
        }) = events[i]
        {
            // Collect ALL text events within the heading (handles long titles split across multiple Text events)
            let mut heading_text = String::new();
            let mut j = i + 1;
            while j < events.len() {
                match &events[j] {
                    Event::Text(t) => heading_text.push_str(t.as_ref()),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
                j += 1;
            }

            // Check for marker in complete heading text
            if !heading_text.is_empty() {
                if let Some((found_marker, title)) = extract_marker(&heading_text) {
                    if found_marker == marker {
                        return Some((i, title, found_marker));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Find all H3 headings within a range, returning (position, title) tuples
pub fn find_all_h3_headings(events: &[Event]) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut i = 0;

    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H3,
            ..
        }) = events[i]
        {
            // Extract text from next event(s) until End tag
            let mut title = String::new();
            let mut j = i + 1;
            while j < events.len() {
                match &events[j] {
                    Event::Text(t) => title.push_str(t),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
                j += 1;
            }
            if !title.is_empty() {
                results.push((i, title));
            }
        }
        i += 1;
    }

    results
}

/// Find all H3 headings that contain a specific marker (e.g., "{.resource}")
/// Returns Vec of (position, heading_text_without_marker, marker) tuples
pub fn find_all_h3_with_marker(events: &[Event], marker: &str) -> Vec<(usize, String, String)> {
    use crate::utils::extract_marker;

    let mut results = Vec::new();
    let mut i = 0;

    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H3,
            ..
        }) = events[i]
        {
            // Collect ALL text events within the heading (handles long titles split across multiple Text events)
            let mut heading_text = String::new();
            let mut j = i + 1;
            while j < events.len() {
                match &events[j] {
                    Event::Text(t) => heading_text.push_str(t.as_ref()),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
                j += 1;
            }

            // Check for marker in complete heading text
            if !heading_text.is_empty() {
                if let Some((found_marker, title)) = extract_marker(&heading_text) {
                    if found_marker == marker {
                        results.push((i, title, found_marker));
                    }
                }
            }
        }
        i += 1;
    }

    results
}

/// Extract events between two positions (for extracting content under a heading)
pub fn extract_until_next_heading<'a>(events: &'a [Event<'a>], start: usize) -> &'a [Event<'a>] {
    let mut end = events.len();

    for (i, event) in events[start..].iter().enumerate() {
        if i == 0 {
            continue; // Skip the starting heading itself
        }
        if matches!(event, Event::Start(Tag::Heading { .. })) {
            end = start + i;
            break;
        }
    }

    &events[start..end]
}

/// Extract events until next H2 or H1 (allows H3 headings within the section)
pub fn extract_until_next_h2<'a>(events: &'a [Event<'a>], start: usize) -> &'a [Event<'a>] {
    let mut end = events.len();

    for (i, event) in events[start..].iter().enumerate() {
        if i == 0 {
            continue; // Skip the starting heading itself
        }
        if let Event::Start(Tag::Heading { level, .. }) = event {
            // Stop at H1 or H2, but allow H3 and below
            if matches!(
                level,
                pulldown_cmark::HeadingLevel::H1 | pulldown_cmark::HeadingLevel::H2
            ) {
                end = start + i;
                break;
            }
        }
    }

    &events[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn test_extract_list_items() {
        let markdown = "- Item 1\n- Item 2\n- Item 3\n";
        let events: Vec<Event> = Parser::new(markdown).collect();
        let items = extract_list_items(&events);
        assert_eq!(items, vec!["Item 1", "Item 2", "Item 3"]);
    }

    #[test]
    fn test_extract_paragraphs() {
        let markdown = "First paragraph.\n\nSecond paragraph.\n";
        let events: Vec<Event> = Parser::new(markdown).collect();
        let paragraphs = extract_paragraphs(&events);
        assert_eq!(paragraphs, vec!["First paragraph.", "Second paragraph."]);
    }

    #[test]
    fn test_find_h2_heading() {
        let markdown = "## Summary\nContent\n## Stats\nMore content\n";
        let events: Vec<Event> = Parser::new(markdown).collect();
        let pos = find_h2_heading(&events, "Stats");
        assert!(pos.is_some());
    }
}
