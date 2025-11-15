use crate::event_stream::{
    extract_list_items, extract_paragraphs, extract_until_next_heading, find_h2_heading,
    find_h2_with_marker,
};
use crate::types::{
    JsonFeatureItem, JsonFeaturesPanel, JsonOverview, JsonStatItem, JsonStatsPanel,
};
use crate::utils::parse_colon_pattern;
use anyhow::{Context, Result};
use pulldown_cmark::Event;

pub fn parse_overview_section(
    events: &[Event],
    warnings: &mut Vec<String>,
) -> Result<JsonOverview> {
    // Parse Summary
    let summary = parse_summary(events).context("Failed to parse Summary")?;

    // Parse Stats panel with custom label support
    let stats_panel = parse_stats_panel(events, warnings);

    // Parse Features panel with custom title support
    let features_panel = parse_features_panel(events, warnings);

    Ok(JsonOverview {
        summary,
        stats_panel,
        features_panel,
    })
}

fn parse_summary(events: &[Event]) -> Result<String> {
    // Find ## Summary heading (with or without {.summary} marker)
    // Try with marker first, then fallback to exact match
    let pos = if let Some((pos, _title, _marker)) = find_h2_with_marker(events, "summary") {
        Some(pos)
    } else {
        find_h2_heading(events, "Summary")
    };

    if let Some(pos) = pos {
        let content_events = extract_until_next_heading(events, pos);
        let paragraphs = extract_paragraphs(content_events);

        if paragraphs.is_empty() {
            return Ok("No summary provided.".to_string());
        }

        // Join all paragraphs with double newline
        return Ok(paragraphs.join("\n\n"));
    }

    Ok("No summary provided.".to_string())
}

fn parse_stats_panel(events: &[Event], warnings: &mut Vec<String>) -> Option<JsonStatsPanel> {
    // Find H2 heading with {.stats} marker
    if let Some((pos, title, _marker)) = find_h2_with_marker(events, "stats") {
        let content_events = extract_until_next_heading(events, pos);
        let list_items = extract_list_items(content_events);

        if list_items.is_empty() {
            warnings.push(format!("Stats panel '{title}' found but no list items"));
        }

        // Parse each item - check for inline colon pattern
        let items = list_items
            .into_iter()
            .map(|item| {
                if let Some((label, value)) = parse_colon_pattern(&item) {
                    // Custom label format: "Label: value"
                    JsonStatItem::WithLabel {
                        label: Some(label),
                        value,
                    }
                } else {
                    // Simple string format - will get auto-labeled
                    JsonStatItem::Simple(item)
                }
            })
            .collect();

        Some(JsonStatsPanel {
            title: Some(title),
            items,
        })
    } else {
        // No {.stats} marker found - return None (optional field)
        None
    }
}

fn parse_features_panel(events: &[Event], warnings: &mut Vec<String>) -> Option<JsonFeaturesPanel> {
    // Find H2 heading with {.features} marker
    if let Some((pos, title, _marker)) = find_h2_with_marker(events, "features") {
        let content_events = extract_until_next_heading(events, pos);
        let list_items = extract_list_items(content_events);

        if list_items.is_empty() {
            warnings.push(format!("Features panel '{title}' found but no list items"));
        }

        // Parse each item - check for inline colon pattern
        let items = list_items
            .into_iter()
            .map(|item| {
                if let Some((heading, text)) = parse_colon_pattern(&item) {
                    // Custom heading format: "Heading: text"
                    JsonFeatureItem::WithHeading {
                        heading: Some(heading),
                        text,
                    }
                } else {
                    // Simple string format - will get auto-labeled
                    JsonFeatureItem::Simple(item)
                }
            })
            .collect();

        Some(JsonFeaturesPanel {
            title: Some(title),
            items,
        })
    } else {
        // No {.features} marker found - return None (optional field)
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn test_parse_stats_with_custom_labels() {
        let markdown = r#"
## Summary
This is a test.

## Analytical Approach {.stats}
- Approach: Mixed-Effects Modeling
- Data Type: Longitudinal
- Simple string without colon

## Key Benefits {.features}
- Feature 1
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();
        let result = parse_overview_section(&events, &mut warnings);

        assert!(result.is_ok());
        let overview = result.unwrap();

        // Check stats panel
        let stats = overview.stats_panel.as_ref().unwrap();
        assert_eq!(stats.items.len(), 3);
        assert_eq!(stats.title.as_ref().unwrap(), "Analytical Approach");

        // First should have custom label
        match &stats.items[0] {
            JsonStatItem::WithLabel { label, value } => {
                assert_eq!(label.as_ref().unwrap(), "Approach:");
                assert_eq!(value, "Mixed-Effects Modeling");
            }
            _ => panic!("Expected WithLabel variant"),
        }

        // Third should be simple string
        match &stats.items[2] {
            JsonStatItem::Simple(val) => {
                assert_eq!(val, "Simple string without colon");
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_parse_features_with_marker() {
        let markdown = r#"
## Summary
Test summary.

## Method Overview {.stats}
- Item 1

## Key Benefits {.features}
- Benefit 1
- Benefit 2
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();

        let result = parse_overview_section(&events, &mut warnings);

        assert!(result.is_ok());
        let overview = result.unwrap();

        let features = overview.features_panel.as_ref().unwrap();
        assert_eq!(features.items.len(), 2);
        assert_eq!(features.title.as_ref().unwrap(), "Key Benefits");

        // Check first item is simple string
        match &features.items[0] {
            JsonFeatureItem::Simple(text) => {
                assert_eq!(text, "Benefit 1");
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_parse_features_with_custom_headings() {
        let markdown = r#"
## Summary
Test summary.

## Key Features {.features}
- When to Use: Analyze longitudinal data
- Key Benefit: Better model fit
- Simple feature without heading
"#;
        let events: Vec<Event> = Parser::new(markdown).collect();
        let mut warnings = Vec::new();

        let result = parse_overview_section(&events, &mut warnings);

        assert!(result.is_ok());
        let overview = result.unwrap();

        let features = overview.features_panel.as_ref().unwrap();
        assert_eq!(features.items.len(), 3);
        assert_eq!(features.title.as_ref().unwrap(), "Key Features");

        // First should have custom heading
        match &features.items[0] {
            JsonFeatureItem::WithHeading { heading, text } => {
                assert_eq!(heading.as_ref().unwrap(), "When to Use:");
                assert_eq!(text, "Analyze longitudinal data");
            }
            _ => panic!("Expected WithHeading variant"),
        }

        // Third should be simple string
        match &features.items[2] {
            JsonFeatureItem::Simple(text) => {
                assert_eq!(text, "Simple feature without heading");
            }
            _ => panic!("Expected Simple variant"),
        }
    }
}
