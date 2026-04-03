//! Privacy protection utilities
//!
//! Scans R output for participant IDs and other sensitive data patterns.
//! Prevents accidental exposure of individual-level data to LLM APIs.

use regex::Regex;

/// Result of scanning output for participant IDs
#[derive(Debug)]
pub struct IdScanResult {
    /// Whether any IDs were found
    pub found_ids: bool,
    /// Number of IDs found
    pub count: usize,
    /// The (potentially redacted) output - used when action is "redact"
    #[allow(dead_code)]
    pub output: String,
    /// Sample of found IDs (for error messages, truncated)
    pub sample_ids: Vec<String>,
}

/// Scan output for participant ID patterns
///
/// # Arguments
/// * `output` - The R stdout/stderr to scan
/// * `patterns` - Regex patterns that match participant IDs
/// * `action` - "redact" to replace IDs with [REDACTED], "fail" to just detect
///
/// # Returns
/// IdScanResult with detection info and potentially redacted output
pub fn scan_for_participant_ids(
    output: &str,
    patterns: &[String],
    action: &str,
) -> IdScanResult {
    let mut found_ids = Vec::new();
    let mut result_output = output.to_string();

    for pattern_str in patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            // Find all matches
            for cap in regex.find_iter(output) {
                let id = cap.as_str().to_string();
                if !found_ids.contains(&id) {
                    found_ids.push(id);
                }
            }

            // Redact if requested
            if action == "redact" {
                result_output = regex
                    .replace_all(&result_output, "[REDACTED_ID]")
                    .to_string();
            }
        }
    }

    // Truncate sample for error messages (don't expose full IDs)
    let sample_ids: Vec<String> = found_ids
        .iter()
        .take(3) // Only show first 3
        .map(|id| {
            if id.len() > 10 {
                format!("{}...", &id[..10])
            } else {
                id.clone()
            }
        })
        .collect();

    IdScanResult {
        found_ids: !found_ids.is_empty(),
        count: found_ids.len(),
        output: result_output,
        sample_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ndar_ids() {
        let output = "Processing participant NDAR_INVABCD1234 data...";
        let patterns = vec![r"NDAR_INV[A-Za-z0-9]{8}".to_string()];

        let result = scan_for_participant_ids(output, &patterns, "fail");

        assert!(result.found_ids);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_redact_ids() {
        let output = "ID: NDAR_INVABCD1234, Value: 42";
        let patterns = vec![r"NDAR_INV[A-Za-z0-9]{8}".to_string()];

        let result = scan_for_participant_ids(output, &patterns, "redact");

        assert!(result.found_ids);
        assert!(result.output.contains("[REDACTED_ID]"));
        assert!(!result.output.contains("NDAR_INV"));
    }

    #[test]
    fn test_no_ids() {
        let output = "Model summary: AIC = 1234.5, BIC = 1240.2";
        let patterns = vec![r"NDAR_INV[A-Za-z0-9]{8}".to_string()];

        let result = scan_for_participant_ids(output, &patterns, "fail");

        assert!(!result.found_ids);
        assert_eq!(result.count, 0);
    }
}
