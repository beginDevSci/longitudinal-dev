use crate::domain::*;

/// Markdown exporter that generates parser-compliant output
///
/// CRITICAL REQUIREMENTS (from MARKDOWN_CONTRACT.md):
/// 1. Must use H1 (#) for the 6 main sections
/// 2. Sections must be in exact order
/// 3. Section names must match exactly (case-sensitive)
/// 4. Frontmatter uses correct field names (date_iso, NOT publishDate)
pub struct MarkdownExporter;

impl MarkdownExporter {
    /// Export tutorial to parser-compliant Markdown
    pub fn export(tutorial: &Tutorial) -> Result<String, ExportError> {
        let mut md = String::new();

        // Generate frontmatter (optional per parser, but recommended)
        if Self::should_include_frontmatter(tutorial) {
            md.push_str(&Self::generate_frontmatter(tutorial));
            md.push_str("\n\n");
        }

        // CRITICAL: Export the 6 H1 sections in exact order
        // Parser validates this BEFORE processing content!
        for section_type in SectionType::all_required() {
            md.push_str(&format!("# {}\n\n", section_type.heading()));

            // Find the section with this type
            if let Some(section) = tutorial
                .sections
                .iter()
                .find(|s| s.section_type == *section_type)
            {
                md.push_str(&Self::export_section_content(section));
            } else {
                // Section missing - add placeholder
                md.push_str("_To be added_\n\n");
            }
        }

        Ok(md)
    }

    /// Check if we should include frontmatter
    fn should_include_frontmatter(tutorial: &Tutorial) -> bool {
        !tutorial.title.is_empty() || !tutorial.author.name.is_empty()
    }

    /// Generate YAML frontmatter with correct field names
    fn generate_frontmatter(tutorial: &Tutorial) -> String {
        let mut lines = vec!["---".to_string()];

        // Title (optional per parser)
        if !tutorial.title.is_empty() {
            lines.push(format!("title: \"{}\"", tutorial.title));
        }

        // Author (optional)
        if !tutorial.author.name.is_empty() {
            lines.push(format!("author: \"{}\"", tutorial.author.name));
        }

        // Date (ISO format - CORRECTED from publishDate)
        if let Some(date_iso) = &tutorial.metadata.date_iso {
            lines.push(format!("date_iso: \"{date_iso}\""));
        }

        // Tags (optional)
        if !tutorial.metadata.tags.is_empty() {
            let tags_str = tutorial
                .metadata
                .tags
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("tags: [{tags_str}]"));
        }

        // Method classification fields (all optional)
        if let Some(family) = &tutorial.metadata.family {
            if !family.trim().is_empty() {
                lines.push(format!("family: \"{family}\""));
            }
        }
        if let Some(family_label) = &tutorial.metadata.family_label {
            if !family_label.trim().is_empty() {
                lines.push(format!("family_label: \"{family_label}\""));
            }
        }
        if let Some(engine) = &tutorial.metadata.engine {
            if !engine.trim().is_empty() {
                lines.push(format!("engine: \"{engine}\""));
            }
        }
        if let Some(covariates) = &tutorial.metadata.covariates {
            if !covariates.trim().is_empty() {
                lines.push(format!("covariates: \"{covariates}\""));
            }
        }
        if let Some(outcome_type) = &tutorial.metadata.outcome_type {
            if !outcome_type.trim().is_empty() {
                lines.push(format!("outcome_type: \"{outcome_type}\""));
            }
        }

        lines.push("---".to_string());
        lines.join("\n")
    }

    /// Export section content (blocks)
    fn export_section_content(section: &Section) -> String {
        let mut md = String::new();

        for block in &section.blocks {
            md.push_str(&Self::export_block(block));
        }

        md
    }

    /// Export individual block
    fn export_block(block: &Block) -> String {
        match block {
            Block::Paragraph { content } => {
                if !content.trim().is_empty() {
                    format!("{content}\n\n")
                } else {
                    String::new()
                }
            }
            Block::Code {
                language,
                code,
                description,
                filename,
            } => {
                let mut md = String::new();

                // Add description if provided
                if let Some(desc) = description {
                    if !desc.trim().is_empty() {
                        md.push_str(&format!("**Description:** {desc}\n\n"));
                    }
                }

                // Add filename if provided
                if let Some(fname) = filename {
                    if !fname.trim().is_empty() {
                        md.push_str(&format!("**Filename:** {fname}\n\n"));
                    }
                }

                // Code block
                md.push_str(&format!("```{language}\n{code}\n```\n\n"));
                md
            }
            Block::List { ordered, items } => {
                let mut md = String::new();
                for (i, item) in items.iter().enumerate() {
                    if !item.trim().is_empty() {
                        if *ordered {
                            let num = i + 1;
                            md.push_str(&format!("{num}. {item}\n"));
                        } else {
                            md.push_str(&format!("- {item}\n"));
                        }
                    }
                }
                md.push('\n');
                md
            }
            Block::Table { headers, rows } => {
                let mut md = String::new();

                // Header row
                md.push_str("| ");
                md.push_str(&headers.join(" | "));
                md.push_str(" |\n");

                // Separator row
                md.push('|');
                for _ in headers {
                    md.push_str(" --- |");
                }
                md.push('\n');

                // Data rows
                for row in rows {
                    md.push_str("| ");
                    md.push_str(&row.join(" | "));
                    md.push_str(" |\n");
                }

                md.push('\n');
                md
            }
            Block::Image { src, alt } => format!("![{alt}]({src})\n\n"),
            Block::Note { content } => {
                if !content.trim().is_empty() {
                    format!("**Note:** {content}\n\n")
                } else {
                    String::new()
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Tutorial validation failed: {0}")]
    ValidationError(String),

    #[error("Missing required section: {0}")]
    MissingSection(String),

    #[error("IO error: {0}")]
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h1_sections_generated() {
        let tutorial = Tutorial::new();
        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Verify H1 sections (NOT H2!)
        assert!(md.contains("# Overview\n"));
        assert!(md.contains("# Data Access\n"));
        assert!(md.contains("# Data Preparation\n"));
        assert!(md.contains("# Statistical Analysis\n"));
        assert!(md.contains("# Discussion\n"));
        assert!(md.contains("# Additional Resources\n"));

        // Should NOT contain H2 versions
        assert!(!md.contains("## Overview"));
        assert!(!md.contains("## Data Access"));
    }

    #[test]
    fn test_section_order() {
        let tutorial = Tutorial::new();
        let md = MarkdownExporter::export(&tutorial).unwrap();

        let overview_pos = md.find("# Overview").unwrap();
        let data_access_pos = md.find("# Data Access").unwrap();
        let data_prep_pos = md.find("# Data Preparation").unwrap();
        let stats_pos = md.find("# Statistical Analysis").unwrap();
        let discussion_pos = md.find("# Discussion").unwrap();
        let resources_pos = md.find("# Additional Resources").unwrap();

        // Verify exact order
        assert!(overview_pos < data_access_pos);
        assert!(data_access_pos < data_prep_pos);
        assert!(data_prep_pos < stats_pos);
        assert!(stats_pos < discussion_pos);
        assert!(discussion_pos < resources_pos);
    }

    #[test]
    fn test_frontmatter_fields() {
        let mut tutorial = Tutorial::new();
        tutorial.title = "Test Tutorial".to_string();
        tutorial.author.name = "Test Author".to_string();
        tutorial.metadata.family = Some("LGCM".to_string());
        tutorial.metadata.family_label = Some("Latent Growth Curve Models".to_string());
        tutorial.metadata.engine = Some("lavaan".to_string());

        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Verify correct field names
        assert!(md.contains("date_iso:")); // CORRECT
        assert!(!md.contains("publishDate:")); // OLD (wrong)
        assert!(md.contains("family: \"LGCM\""));
        assert!(md.contains("family_label: \"Latent Growth Curve Models\""));
        assert!(md.contains("engine: \"lavaan\""));
    }
}
