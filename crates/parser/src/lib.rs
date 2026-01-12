use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub mod event_stream;
pub mod frontmatter;
pub mod math;
pub mod parsers;
pub mod sections;
pub mod syntax;
pub mod types;
pub mod utils;
pub mod validation;

use frontmatter::extract_frontmatter;
use sections::detect_and_validate_sections;
use types::{Frontmatter, ParsedPost, SectionBoundaries};

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub verbose: bool,
    pub strict: bool,
    pub validate: bool,
}

/// Main entry point: parse markdown file and write JSON
pub fn parse_markdown_file(
    input_path: &Path,
    output_path: &Path,
    options: ParseOptions,
) -> Result<Vec<String>> {
    // Read input file
    let markdown = fs::read_to_string(input_path).context(format!(
        "Failed to read input file: {}",
        input_path.display()
    ))?;

    // Parse markdown to JSON
    let (json_output, warnings) = parse_markdown(&markdown, options.clone())?;

    // Write output file
    fs::write(output_path, json_output).context(format!(
        "Failed to write output file: {}",
        output_path.display()
    ))?;

    // TODO: Schema validation if options.validate

    Ok(warnings)
}

/// Parse markdown string to JSON string
pub fn parse_markdown(markdown: &str, options: ParseOptions) -> Result<(String, Vec<String>)> {
    let mut warnings = Vec::new();

    // Step 1: Extract frontmatter (if present)
    let (frontmatter, content) = extract_frontmatter(markdown);

    if options.verbose {
        if let Some(ref fm) = frontmatter {
            eprintln!("📄 Extracted frontmatter: title={:?}", fm.title);
        } else {
            warnings.push(
                "No frontmatter found - title will be extracted from content or use default"
                    .to_string(),
            );
        }
    }

    // Step 2: Parse markdown events
    use pulldown_cmark::{Event, Options as MdOptions, Parser};
    let mut md_options = MdOptions::empty();
    md_options.insert(MdOptions::ENABLE_TABLES);
    md_options.insert(MdOptions::ENABLE_STRIKETHROUGH);
    let events: Vec<Event> = Parser::new_ext(content, md_options).collect();

    if options.verbose {
        eprintln!("📊 Parsed {} markdown events", events.len());
    }

    // Step 3: Detect and validate sections
    let sections = detect_and_validate_sections(&events)?;

    if options.verbose {
        eprintln!("✅ Detected all 6 required sections");
    }

    // Step 4: Parse each section
    let post = parse_post(frontmatter, &events, sections, &mut warnings)?;

    // Step 5: Generate JSON
    let json = serde_json::to_string_pretty(&post).context("Failed to serialize post to JSON")?;

    Ok((json, warnings))
}

/// Parse the complete post
fn parse_post(
    frontmatter: Option<Frontmatter>,
    events: &[pulldown_cmark::Event],
    sections: SectionBoundaries,
    warnings: &mut Vec<String>,
) -> Result<ParsedPost> {
    // Extract title and metadata from frontmatter
    let (title, metadata) = if let Some(mut fm) = frontmatter {
        let title = fm
            .title
            .clone()
            .unwrap_or_else(|| "Untitled Post".to_string());

        // Construct metadata if all required fields are present
        // Note: engine OR engines must be present (engines takes precedence)
        let has_engine = fm.engine.is_some() || fm.engines.is_some();
        let metadata = if fm.family.is_some()
            && has_engine
            && fm.covariates.is_some()
            && fm.outcome_type.is_some()
            && fm.updated_at.is_some()
            && fm.tags.is_some()
            && fm.author.is_some()
        {
            // Build engines array: prefer explicit engines[], fall back to wrapping engine
            let engines = if let Some(engines_arr) = fm.engines.take() {
                engines_arr
            } else if let Some(ref engine) = fm.engine {
                vec![engine.clone()]
            } else {
                vec![]
            };

            Some(types::PostMetadata {
                method_family: fm.family.take().unwrap(),
                method_family_label: fm.family_label.take(),
                statistical_engine: fm.engine.take().unwrap_or_else(|| {
                    engines.first().cloned().unwrap_or_default()
                }),
                engines,
                covariates: fm.covariates.take().unwrap(),
                outcome_type: fm.outcome_type.take().unwrap(),
                updated_at: fm.updated_at.take().unwrap(),
                tags: fm.tags.take().unwrap(),
                author: fm.author.take().unwrap(),
                description: fm.description,
                summary: fm.summary,
                difficulty: fm.difficulty,
                timepoints: fm.timepoints,
                draft: fm.draft,
            })
        } else {
            None
        };

        // Validate new schema fields and emit warnings for missing ones
        if let Some(ref meta) = metadata {
            if meta.difficulty.is_none() {
                warnings.push("Missing 'difficulty' field (intro | intermediate | advanced) - recommended for scaling".to_string());
            }
            if meta.timepoints.is_none() {
                warnings.push("Missing 'timepoints' field (2 | 3_5 | 6_plus | irregular) - recommended for scaling".to_string());
            }
            if meta.engines.is_empty() {
                warnings.push("Missing 'engines' array - using legacy 'engine' field as fallback".to_string());
            }
            if meta.summary.is_none() {
                warnings.push("Missing explicit 'summary' field - will derive from description".to_string());
            }
        }

        (title, metadata)
    } else {
        ("Untitled Post".to_string(), None)
    };

    // Parse sections that are the same across versions
    let overview =
        parsers::overview::parse_overview_section(&events[sections.overview.clone()], warnings)?;
    let data_access = parsers::data_access::parse_data_access_section(
        &events[sections.data_access.clone()],
        warnings,
    )?;
    let discussion = parsers::discussion::parse_discussion_section(
        &events[sections.discussion.clone()],
        warnings,
    )?;
    let additional_resources = parsers::additional_resources::parse_additional_resources_section(
        &events[sections.additional_resources.clone()],
        warnings,
    )?;

    // Parse Data Preparation and Statistical Analysis sections
    let data_preparation = parsers::data_preparation::parse_data_preparation_section(
        &events[sections.data_preparation.clone()],
        warnings,
    )?;

    let statistical_analysis = parsers::statistical_analysis::parse_statistical_analysis_section(
        &events[sections.statistical_analysis.clone()],
        warnings,
    )?;

    Ok(ParsedPost {
        title,
        metadata,
        overview,
        data_access,
        data_preparation,
        statistical_analysis,
        discussion,
        additional_resources,
    })
}
