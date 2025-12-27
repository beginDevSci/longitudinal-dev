use crate::math::render_math_in_html;
use crate::types::{JsonDataAccess, JsonDataAccessItem};
use crate::utils::extract_marker;
use anyhow::Result;
use pulldown_cmark::{html, Event, HeadingLevel, Tag, TagEnd};

pub fn parse_data_access_section(
    events: &[Event],
    _warnings: &mut [String],
) -> Result<JsonDataAccess> {
    // Parse Data Access section into structured items
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

            // Render any math expressions in the HTML
            let content_with_math = render_math_in_html(&content_html);

            items.push(JsonDataAccessItem::Collapsible {
                title,
                content: content_with_math,
                open: false,
            });

            continue; // Continue to next iteration
        }

        i += 1;
    }

    // Fallback: if no items found, treat entire section as prose
    let prose = if items.is_empty() {
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.iter().cloned());
        // Render any math expressions in the HTML
        Some(render_math_in_html(&html_output))
    } else {
        None
    };

    Ok(JsonDataAccess { items, prose })
}
