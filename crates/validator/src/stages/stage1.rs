/*!
 * Stage 1: Structural Validation
 *
 * Checks:
 * - Frontmatter has required fields
 * - All required sections present (and ONLY those sections)
 * - Sections in correct order
 * - Subsection markers valid
 * - Code fences present for {.code} blocks
 */

use crate::config::Stage1Config;
use crate::validators::ValidationResult;
use anyhow::Result;
use std::fs;
use std::path::Path;

// Import parser validation API
use longitudinal_parser::validation::parse_structure;

pub struct Stage1Validator<'a> {
    config: &'a Stage1Config,
}

impl<'a> Stage1Validator<'a> {
    pub fn new(config: &'a Stage1Config) -> Self {
        Self { config }
    }

    pub fn validate(&self, markdown_path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new("Stage 1: Structural Validation");

        // Read markdown file
        let content = fs::read_to_string(markdown_path)?;

        // Parse structure using parser crate
        let structure = parse_structure(&content)?;

        // 1. Validate frontmatter
        self.validate_frontmatter(&structure.frontmatter, &mut result);

        // 2. Validate sections
        self.validate_sections(&structure.sections, &mut result);

        // 3. Validate markers
        self.validate_markers(&structure.subsections, &mut result);

        // 4. Validate code fences
        self.validate_code_fences(&structure.subsections, &mut result);

        Ok(result)
    }

    fn validate_frontmatter(
        &self,
        frontmatter: &longitudinal_parser::validation::FrontmatterData,
        result: &mut ValidationResult,
    ) {
        // 1. Check required fields are present
        for required_field in &self.config.required_frontmatter {
            if !frontmatter.fields.contains_key(required_field) {
                // Get expected type for better error message
                let expected_type = self.config.frontmatter_types.get(required_field);
                result.error(
                    Some(1),
                    format!("Missing required frontmatter field: '{required_field}'"),
                    Some(match expected_type.map(|s| s.as_str()) {
                        Some("string") => format!(
                            "Add '{required_field}: \"value\"' to frontmatter YAML block"
                        ),
                        Some("array") => format!(
                            "Add '{required_field}: [\"item1\", \"item2\"]' to frontmatter YAML block"
                        ),
                        _ => format!("Add '{required_field}:' to frontmatter YAML block"),
                    }),
                );
            }
        }

        // 2. Type-check fields that are present (regardless of whether required)
        for (field_name, value) in &frontmatter.fields {
            if let Some(expected_type) = self.config.frontmatter_types.get(field_name) {
                let actual_type = match value {
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Object(_) => "object",
                    serde_json::Value::Null => "null",
                };

                if actual_type != expected_type {
                    result.error(
                        Some(1),
                        format!(
                            "Frontmatter field '{field_name}' has wrong type: expected {expected_type}, got {actual_type}"
                        ),
                        Some(match expected_type.as_str() {
                            "string" => format!(
                                "Change '{field_name}' to a string value (e.g., {field_name}: \"value\")"
                            ),
                            "array" => format!(
                                "Change '{field_name}' to an array (e.g., {field_name}: [\"item1\", \"item2\"])"
                            ),
                            _ => format!("Change '{field_name}' to type {expected_type}"),
                        }),
                    );
                }
            }
            // Ignore fields not in frontmatter_types map (extra fields are allowed)
        }
    }

    fn validate_sections(
        &self,
        sections: &[longitudinal_parser::validation::Section],
        result: &mut ValidationResult,
    ) {
        let required = &self.config.required_sections;

        // Check all required sections present
        for req_section in required {
            if !sections.iter().any(|s| &s.title == req_section) {
                result.error(
                    None,
                    format!("Missing required section: '{req_section}'"),
                    Some(format!("Add '# {req_section}' section to your markdown")),
                );
            }
        }

        // Check NO extra H1 sections (only the 6 required are allowed)
        for section in sections {
            if !required.contains(&section.title) {
                result.error(
                    Some(section.line_number),
                    format!(
                        "Unknown H1 section: '{}' (only these are allowed: {})",
                        section.title,
                        required.join(", ")
                    ),
                    Some(
                        "Remove this H1 section or nest it as an H2/H3 under a valid section"
                            .to_string(),
                    ),
                );
            }
        }

        // Check section order (for sections that exist)
        let mut present_sections: Vec<_> = required
            .iter()
            .filter_map(|req| sections.iter().find(|s| &s.title == req))
            .collect();

        // Sort by line number to get actual order
        present_sections.sort_by_key(|s| s.line_number);

        for i in 1..present_sections.len() {
            let prev_idx = required
                .iter()
                .position(|r| r == &present_sections[i - 1].title)
                .unwrap();
            let curr_idx = required
                .iter()
                .position(|r| r == &present_sections[i].title)
                .unwrap();

            if curr_idx < prev_idx {
                result.error(
                    Some(present_sections[i].line_number),
                    format!(
                        "Section '{}' appears before '{}' (wrong order)",
                        present_sections[i].title,
                        present_sections[i - 1].title
                    ),
                    Some(format!(
                        "Move '# {}' to appear after '# {}'",
                        present_sections[i].title,
                        present_sections[i - 1].title
                    )),
                );
            }
        }
    }

    fn validate_markers(
        &self,
        subsections: &[longitudinal_parser::validation::Subsection],
        result: &mut ValidationResult,
    ) {
        for subsection in subsections {
            if let Some(marker) = &subsection.marker {
                if !self.config.valid_markers.contains(marker) {
                    // Unknown marker - emit warning (not error)
                    result.warning(
                        Some(subsection.line_number),
                        format!(
                            "Unknown marker '{{.{}}}' in subsection '{}' (not enforced by Stage 1)",
                            marker, subsection.title
                        ),
                        Some(format!(
                            "Only structural markers are enforced: {}",
                            self.config
                                .valid_markers
                                .iter()
                                .map(|m| format!("{{.{m}}}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    );
                }
            }
            // Subsections without markers are allowed (no warning)
        }
    }

    fn validate_code_fences(
        &self,
        subsections: &[longitudinal_parser::validation::Subsection],
        result: &mut ValidationResult,
    ) {
        // Check every .code subsection for code fence
        for subsection in subsections {
            if subsection.marker.as_deref() == Some("code") {
                if !subsection.has_code_fence {
                    result.error(
                        Some(subsection.line_number),
                        format!(
                            "Subsection '{{.code}}' has no code fence: '{}'",
                            subsection.title
                        ),
                        Some("Add '```r' code block after this heading".to_string()),
                    );
                } else if let Some(lang) = &subsection.code_fence_language {
                    if lang != "r" {
                        result.error(
                            Some(subsection.line_number),
                            format!(
                                "Code fence language '{}' should be 'r' in: '{}'",
                                lang, subsection.title
                            ),
                            Some("Change to '```r'".to_string()),
                        );
                    }
                }
            }
        }
        // Continue after errors - report ALL fence issues (no early exit)
    }
}
