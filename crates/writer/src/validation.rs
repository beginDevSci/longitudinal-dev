use crate::domain::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub section: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Error,   // Blocks export (missing required content)
    Warning, // Allows export (missing optional fields)
    Info,    // Just FYI (suggestions for improvement)
}

pub struct TutorialValidator;

impl TutorialValidator {
    /// Validate tutorial before export
    /// Returns validation issues grouped by severity
    pub fn validate(tutorial: &Tutorial) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check metadata
        issues.extend(Self::validate_metadata(tutorial));

        // Check sections
        issues.extend(Self::validate_sections(tutorial));

        issues
    }

    fn validate_metadata(tutorial: &Tutorial) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Title is recommended but not strictly required by parser
        if tutorial.title.trim().is_empty() {
            issues.push(ValidationIssue {
                level: ValidationLevel::Warning,
                section: None,
                message: "Title is recommended for better searchability".to_string(),
            });
        }

        // Author is recommended
        if tutorial.author.name.trim().is_empty() {
            issues.push(ValidationIssue {
                level: ValidationLevel::Warning,
                section: None,
                message: "Author name is recommended".to_string(),
            });
        }

        // Method classification fields are helpful but optional
        if tutorial.metadata.family.is_none() {
            issues.push(ValidationIssue {
                level: ValidationLevel::Info,
                section: None,
                message:
                    "Consider adding method family (LGCM, GLMM, etc.) for better categorization"
                        .to_string(),
            });
        }

        if tutorial.metadata.engine.is_none() {
            issues.push(ValidationIssue {
                level: ValidationLevel::Info,
                section: None,
                message: "Consider adding statistical engine (lavaan, lme4, etc.)".to_string(),
            });
        }

        issues
    }

    fn validate_sections(tutorial: &Tutorial) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check all 6 required sections exist
        for section_type in SectionType::all_required() {
            let section = tutorial
                .sections
                .iter()
                .find(|s| s.section_type == *section_type);

            match section {
                None => {
                    issues.push(ValidationIssue {
                        level: ValidationLevel::Error,
                        section: Some(section_type.heading().to_string()),
                        message: format!("Required section '{section_type}' is missing"),
                    });
                }
                Some(section) => {
                    // Check if section has content
                    if !section.has_content() {
                        issues.push(ValidationIssue {
                            level: ValidationLevel::Warning,
                            section: Some(section_type.heading().to_string()),
                            message: format!("Section '{section_type}' is empty"),
                        });
                    }
                }
            }
        }

        // Check for duplicate sections
        let mut seen = std::collections::HashSet::new();
        for section in &tutorial.sections {
            if !seen.insert(section.section_type) {
                issues.push(ValidationIssue {
                    level: ValidationLevel::Error,
                    section: Some(section.section_type.heading().to_string()),
                    message: format!("Duplicate section: {}", section.section_type),
                });
            }
        }

        issues
    }

    /// Check if tutorial can be exported (no errors)
    pub fn can_export(tutorial: &Tutorial) -> bool {
        Self::validate(tutorial)
            .iter()
            .all(|issue| issue.level != ValidationLevel::Error)
    }

    /// Get validation summary
    pub fn summary(issues: &[ValidationIssue]) -> String {
        let errors = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Error)
            .count();
        let warnings = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Warning)
            .count();
        let info = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Info)
            .count();

        if errors > 0 {
            format!("{errors} errors, {warnings} warnings, {info} suggestions")
        } else if warnings > 0 {
            format!("{warnings} warnings, {info} suggestions")
        } else if info > 0 {
            format!("{info} suggestions")
        } else {
            "No issues".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tutorial() {
        let mut tutorial = Tutorial::new();
        tutorial.title = "Test".to_string();
        tutorial.author.name = "Author".to_string();

        // Add content to sections
        for section in &mut tutorial.sections {
            section.blocks = vec![Block::Paragraph {
                content: "Some content".to_string(),
            }];
        }

        let issues = TutorialValidator::validate(&tutorial);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Error)
            .collect();

        assert_eq!(errors.len(), 0, "Should have no errors");
        assert!(TutorialValidator::can_export(&tutorial));
    }

    #[test]
    fn test_missing_section() {
        let mut tutorial = Tutorial::new();
        tutorial.sections.clear(); // Remove all sections

        let issues = TutorialValidator::validate(&tutorial);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Error)
            .collect();

        assert_eq!(errors.len(), 6, "Should have 6 errors for missing sections");
        assert!(!TutorialValidator::can_export(&tutorial));
    }
}
