/// Parse inline colon pattern: "Label: value" -> (label, value)
/// Only matches if there's a colon followed by a space
pub fn parse_colon_pattern(text: &str) -> Option<(String, String)> {
    // Look for ": " (colon followed by space)
    if let Some(pos) = text.find(": ") {
        let label = text[..pos].trim();
        let value = text[pos + 2..].trim();

        if !label.is_empty() && !value.is_empty() {
            return Some((
                format!("{label}:"), // Add colon to label
                value.to_string(),
            ));
        }
    }
    None
}

/// Parse bold label pattern: "**Label:** value" -> (label, value)
pub fn parse_bold_label(text: &str) -> Option<(String, String)> {
    // Match **Label:** value pattern
    if let Some(stripped) = text.strip_prefix("**") {
        if let Some(end_bold) = stripped.find("**") {
            let label = &stripped[..end_bold];
            let remaining = stripped[end_bold + 2..].trim();

            if label.ends_with(':') && !remaining.is_empty() {
                return Some((label.to_string(), remaining.to_string()));
            }
        }
    }
    None
}

/// Check if text has " (Featured)" suffix and return (is_featured, text_without_suffix)
pub fn has_featured_suffix(text: &str) -> (bool, String) {
    if let Some(stripped) = text.strip_suffix(" (Featured)") {
        (true, stripped.to_string())
    } else {
        (false, text.to_string())
    }
}

/// Extract marker from heading text: "Title {.marker}" -> Some(("marker", "Title"))
/// Returns (marker, title_without_marker) if marker found, None otherwise
pub fn extract_marker(text: &str) -> Option<(String, String)> {
    // Look for {.marker} pattern
    if let Some(start) = text.rfind("{.") {
        if let Some(end) = text[start..].find('}') {
            let marker = text[start + 2..start + end].trim().to_string();
            let title = text[..start].trim().to_string();
            return Some((marker, title));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_colon_pattern() {
        // Valid patterns
        assert_eq!(
            parse_colon_pattern("Approach: Mixed-Effects"),
            Some(("Approach:".to_string(), "Mixed-Effects".to_string()))
        );

        assert_eq!(
            parse_colon_pattern("Data Type: Longitudinal data"),
            Some(("Data Type:".to_string(), "Longitudinal data".to_string()))
        );

        // Invalid patterns
        assert_eq!(parse_colon_pattern("No colon here"), None);
        assert_eq!(parse_colon_pattern("Colon:NoSpace"), None);
        assert_eq!(parse_colon_pattern(": No label"), None);
    }

    #[test]
    fn test_parse_bold_label() {
        assert_eq!(
            parse_bold_label("**File:** SCRIPT.R"),
            Some(("File:".to_string(), "SCRIPT.R".to_string()))
        );

        assert_eq!(
            parse_bold_label("**Actions:** Load data"),
            Some(("Actions:".to_string(), "Load data".to_string()))
        );

        assert_eq!(parse_bold_label("Not bold: value"), None);
    }

    #[test]
    fn test_has_featured_suffix() {
        assert_eq!(
            has_featured_suffix("Method Name (Featured)"),
            (true, "Method Name".to_string())
        );

        assert_eq!(
            has_featured_suffix("Regular Method"),
            (false, "Regular Method".to_string())
        );
    }

    #[test]
    fn test_extract_marker() {
        // Valid marker
        assert_eq!(
            extract_marker("Analytical Approach {.stats}"),
            Some(("stats".to_string(), "Analytical Approach".to_string()))
        );

        assert_eq!(
            extract_marker("Summary {.summary}"),
            Some(("summary".to_string(), "Summary".to_string()))
        );

        // No marker
        assert_eq!(extract_marker("Regular Heading"), None);

        // Malformed
        assert_eq!(extract_marker("Heading {."), None);
        assert_eq!(extract_marker("Heading .marker}"), None);
    }
}
