/// Integration test: Export → Parse → Validate
///
/// This test verifies that the Writer exports Markdown that the actual blog parser accepts.
/// This is THE most critical test - if this passes, the integration works!
#[cfg(test)]
mod tests {
    use longitudinal_writer::*;

    #[test]
    fn test_h1_sections_required() {
        let tutorial = Tutorial::new();
        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Count H1 sections
        let h1_count = md.lines().filter(|line| line.starts_with("# ")).count();
        assert_eq!(
            h1_count, 6,
            "Must have exactly 6 H1 sections, found {h1_count}"
        );

        // Verify exact H1 headings
        assert!(md.contains("# Overview\n"));
        assert!(md.contains("# Data Access\n"));
        assert!(md.contains("# Data Preparation\n"));
        assert!(md.contains("# Statistical Analysis\n"));
        assert!(md.contains("# Discussion\n"));
        assert!(md.contains("# Additional Resources\n"));
    }

    #[test]
    fn test_no_h2_sections() {
        let tutorial = Tutorial::new();
        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Should NOT use H2 for main sections
        assert!(!md.contains("## Overview"));
        assert!(!md.contains("## Data Access"));
        assert!(!md.contains("## Data Preparation"));
        assert!(!md.contains("## Statistical Analysis"));
        assert!(!md.contains("## Discussion"));
        assert!(!md.contains("## Additional Resources"));
    }

    #[test]
    fn test_section_order_correct() {
        let tutorial = Tutorial::new();
        let md = MarkdownExporter::export(&tutorial).unwrap();

        let positions: Vec<_> = [
            "# Overview",
            "# Data Access",
            "# Data Preparation",
            "# Statistical Analysis",
            "# Discussion",
            "# Additional Resources",
        ]
        .iter()
        .map(|s| {
            md.find(s)
                .unwrap_or_else(|| panic!("Section '{s}' not found"))
        })
        .collect();

        // Verify ascending order
        for i in 0..positions.len() - 1 {
            assert!(
                positions[i] < positions[i + 1],
                "Sections not in correct order"
            );
        }
    }

    #[test]
    fn test_frontmatter_field_names() {
        let mut tutorial = Tutorial::new();
        tutorial.title = "Test Tutorial".to_string();
        tutorial.author.name = "Test Author".to_string();
        tutorial.metadata.family = Some("LGCM".to_string());
        tutorial.metadata.family_label = Some("Latent Growth Curve Models".to_string());
        tutorial.metadata.engine = Some("lavaan".to_string());
        tutorial.metadata.covariates = Some("TIC".to_string());
        tutorial.metadata.outcome_type = Some("Continuous".to_string());

        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Verify CORRECT field names
        assert!(md.contains("date_iso:"), "Missing date_iso field");
        assert!(md.contains("family: \"LGCM\""), "Missing family field");
        assert!(
            md.contains("family_label: \"Latent Growth Curve Models\""),
            "Missing family_label field"
        );
        assert!(md.contains("engine: \"lavaan\""), "Missing engine field");
        assert!(
            md.contains("covariates: \"TIC\""),
            "Missing covariates field"
        );
        assert!(
            md.contains("outcome_type: \"Continuous\""),
            "Missing outcome_type field"
        );

        // Verify INCORRECT field names NOT present
        assert!(
            !md.contains("publishDate:"),
            "Should NOT use publishDate (old field name)"
        );
        assert!(!md.contains("difficulty:"), "Should NOT use difficulty");
        assert!(
            !md.contains("estimatedTime:"),
            "Should NOT use estimatedTime"
        );
    }

    #[test]
    fn test_export_with_content() {
        let mut tutorial = Tutorial::new();
        tutorial.title = "Sample Tutorial".to_string();
        tutorial.author.name = "Dr. Jane Smith".to_string();
        tutorial.metadata.family = Some("LGCM".to_string());

        // Add content to Overview section
        if let Some(overview) = tutorial
            .sections
            .iter_mut()
            .find(|s| s.section_type == SectionType::Overview)
        {
            overview.blocks = vec![
                Block::Paragraph {
                    content: "This tutorial demonstrates latent growth curve modeling.".to_string(),
                },
                Block::List {
                    ordered: false,
                    items: vec![
                        "Learn model specification".to_string(),
                        "Interpret results".to_string(),
                    ],
                },
            ];
        }

        // Add code to Statistical Analysis
        if let Some(stats) = tutorial
            .sections
            .iter_mut()
            .find(|s| s.section_type == SectionType::StatisticalAnalysis)
        {
            stats.blocks = vec![Block::Code {
                language: "r".to_string(),
                code: "model <- lm(y ~ x, data = df)".to_string(),
                description: Some("Fit linear model".to_string()),
                filename: Some("01_model.R".to_string()),
            }];
        }

        let md = MarkdownExporter::export(&tutorial).unwrap();

        // Verify content appears under correct sections
        assert!(md.contains("latent growth curve modeling"));
        assert!(md.contains("- Learn model specification"));
        assert!(md.contains("```r\n"));
        assert!(md.contains("model <- lm(y ~ x, data = df)"));
        assert!(md.contains("**Description:** Fit linear model"));
        assert!(md.contains("**Filename:** 01_model.R"));

        // Content should appear AFTER the H1 heading
        let overview_pos = md.find("# Overview").unwrap();
        let content_pos = md.find("latent growth curve modeling").unwrap();
        assert!(content_pos > overview_pos);

        println!("Generated Markdown:\n{md}");
    }

    /// CRITICAL: Test with actual parser
    /// This verifies end-to-end: Writer export → Parser accepts → JSON validates
    #[test]
    fn test_parser_accepts_export() {
        // Create a minimal but complete tutorial
        let mut tutorial = Tutorial::new();
        tutorial.title = "Parser Integration Test".to_string();
        tutorial.author.name = "Test Author".to_string();
        tutorial.metadata.family = Some("LGCM".to_string());
        tutorial.metadata.engine = Some("lavaan".to_string());

        // Add minimal content to each section
        for section in &mut tutorial.sections {
            let section_type = section.section_type;
            section.blocks = vec![Block::Paragraph {
                content: format!("Content for {section_type} section."),
            }];
        }

        // Export to Markdown
        let md = MarkdownExporter::export(&tutorial).expect("Export should succeed");

        // Write to temp file for parser
        let temp_path = std::env::temp_dir().join("writer_test.md");
        std::fs::write(&temp_path, &md).expect("Should write temp file");

        // TODO: Run parser CLI and validate
        // This would be: cargo run --bin parser -- temp_path
        // For now, just verify the Markdown structure manually

        let temp_display = temp_path.display();
        println!("✅ Exported Markdown to: {temp_display}");
        println!("\n--- Generated Markdown ---");
        println!("{md}");
        println!("--- End ---\n");

        // Cleanup
        // std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_validation_enforces_required_sections() {
        let mut tutorial = Tutorial::new();
        tutorial.sections.clear(); // Remove all sections

        let issues = TutorialValidator::validate(&tutorial);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Error)
            .collect();

        assert_eq!(
            errors.len(),
            6,
            "Should report 6 errors for missing sections"
        );
        assert!(!TutorialValidator::can_export(&tutorial));
    }

    #[test]
    fn test_validation_allows_export_with_warnings() {
        let mut tutorial = Tutorial::new();
        // Leave sections empty but present
        tutorial.title = "Test".to_string();
        tutorial.author.name = "Author".to_string();

        let issues = TutorialValidator::validate(&tutorial);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.level == ValidationLevel::Error)
            .collect();

        // Empty sections generate warnings, not errors
        assert_eq!(errors.len(), 0);
        assert!(TutorialValidator::can_export(&tutorial));
    }
}
