//! Index generator for tutorial JSON artifacts.
//!
//! Generates static JSON files for client-side tutorial catalog:
//! - tutorial_index.json: All tutorial metadata for filtering/search
//! - tutorial_families.json: Method family counts and ordering
//! - tutorial_curations.json: Curated collections (getting started, workflows)

use crate::models::post::Post;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Entry in the tutorial index (for client-side search/filter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialIndexEntry {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub method_family: String,
    pub method_family_label: Option<String>,
    pub statistical_engine: String,
    pub engines: Vec<String>,
    pub covariates: String,
    pub outcome_type: String,
    pub updated_at: String,
    pub author: String,
    pub tags: Vec<String>,
    pub difficulty: Option<String>,
    pub timepoints: Option<String>,
    /// Precomputed lowercase text for search
    pub search_text: String,
}

impl TutorialIndexEntry {
    /// Create index entry from a Post
    pub fn from_post(post: &Post) -> Option<Self> {
        let metadata = post.metadata.as_ref()?;

        // Get summary from metadata first, fallback to overview paragraph
        let summary = metadata
            .summary
            .clone()
            .or_else(|| metadata.description.clone())
            .unwrap_or_else(|| {
                post.overview
                    .summary_paragraphs
                    .first()
                    .map(|s| {
                        let text = s.to_string();
                        if text.len() > 200 {
                            format!("{}...", &text[..200])
                        } else {
                            text
                        }
                    })
                    .unwrap_or_default()
            });

        // Build engines array: prefer engines field, fallback to single engine
        let engines = if !metadata.engines.is_empty() {
            metadata.engines.clone()
        } else {
            vec![metadata.statistical_engine.clone()]
        };

        // Precompute search text (lowercase concatenation of searchable fields)
        let search_text = format!(
            "{} {} {} {} {}",
            post.title.to_lowercase(),
            summary.to_lowercase(),
            metadata.tags.join(" ").to_lowercase(),
            metadata.method_family.to_lowercase(),
            engines.join(" ").to_lowercase(),
        );

        Some(Self {
            slug: post.slug.to_string(),
            title: post.title.to_string(),
            summary,
            method_family: metadata.method_family.clone(),
            method_family_label: metadata.method_family_label.clone(),
            statistical_engine: metadata.statistical_engine.clone(),
            engines,
            covariates: metadata.covariates.clone(),
            outcome_type: metadata.outcome_type.clone(),
            updated_at: metadata.updated_at.clone(),
            author: metadata.author.clone(),
            tags: metadata.tags.clone(),
            difficulty: metadata.difficulty.clone(),
            timepoints: metadata.timepoints.clone(),
            search_text,
        })
    }
}

/// Entry in the family index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyEntry {
    pub id: String,
    pub label: String,
    pub count: usize,
    pub order: usize,
}

/// Curated collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurationsConfig {
    pub getting_started: Vec<String>,
    pub workflows: HashMap<String, WorkflowCategory>,
}

/// Workflow category with label and tutorials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCategory {
    pub label: String,
    pub tutorials: Vec<String>,
}

/// Generated curations JSON (resolved slugs to entries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurationsOutput {
    pub getting_started: Vec<TutorialIndexEntry>,
    pub workflows: HashMap<String, WorkflowOutput>,
    pub recently_updated: Vec<TutorialIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    pub label: String,
    pub tutorials: Vec<TutorialIndexEntry>,
}

/// Generate tutorial index from posts
pub fn generate_tutorial_index(posts: &[Post]) -> Vec<TutorialIndexEntry> {
    posts
        .iter()
        .filter_map(TutorialIndexEntry::from_post)
        .collect()
}

/// Generate family index from tutorial index
pub fn generate_family_index(tutorials: &[TutorialIndexEntry]) -> Vec<FamilyEntry> {
    let mut family_counts: HashMap<String, (String, usize)> = HashMap::new();

    for tutorial in tutorials {
        let label = tutorial
            .method_family_label
            .clone()
            .unwrap_or_else(|| tutorial.method_family.clone());

        family_counts
            .entry(tutorial.method_family.clone())
            .and_modify(|(_, count)| *count += 1)
            .or_insert((label, 1));
    }

    // Sort by family ID for consistent ordering
    let mut families: Vec<_> = family_counts
        .into_iter()
        .map(|(id, (label, count))| (id, label, count))
        .collect();
    families.sort_by(|a, b| a.0.cmp(&b.0));

    families
        .into_iter()
        .enumerate()
        .map(|(order, (id, label, count))| FamilyEntry {
            id,
            label,
            count,
            order,
        })
        .collect()
}

/// Load curations config from YAML file
pub fn load_curations_config(path: &Path) -> Result<CurationsConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read curations file: {}", e))?;

    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse curations YAML: {}", e))
}

/// Generate curations output by resolving slugs to tutorial entries
pub fn generate_curations_output(
    config: &CurationsConfig,
    tutorials: &[TutorialIndexEntry],
) -> CurationsOutput {
    // Build slug lookup map
    let slug_map: HashMap<&str, &TutorialIndexEntry> = tutorials
        .iter()
        .map(|t| (t.slug.as_str(), t))
        .collect();

    // Resolve getting_started slugs
    let getting_started: Vec<TutorialIndexEntry> = config
        .getting_started
        .iter()
        .filter_map(|slug| slug_map.get(slug.as_str()).cloned().cloned())
        .collect();

    // Resolve workflow slugs
    let workflows: HashMap<String, WorkflowOutput> = config
        .workflows
        .iter()
        .map(|(key, category)| {
            let tutorials: Vec<TutorialIndexEntry> = category
                .tutorials
                .iter()
                .filter_map(|slug| slug_map.get(slug.as_str()).cloned().cloned())
                .collect();

            (
                key.clone(),
                WorkflowOutput {
                    label: category.label.clone(),
                    tutorials,
                },
            )
        })
        .collect();

    // Generate recently_updated (sorted by updated_at descending, top 8)
    let mut sorted_tutorials: Vec<_> = tutorials.to_vec();
    sorted_tutorials.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let recently_updated: Vec<TutorialIndexEntry> = sorted_tutorials.into_iter().take(8).collect();

    CurationsOutput {
        getting_started,
        workflows,
        recently_updated,
    }
}

/// Write all index JSON files to the output directory
pub fn write_index_files(
    outdir: &Path,
    tutorials: &[TutorialIndexEntry],
    families: &[FamilyEntry],
    curations: &CurationsOutput,
) -> Result<(), String> {
    let api_dir = outdir.join("api");
    fs::create_dir_all(&api_dir)
        .map_err(|e| format!("Failed to create api directory: {}", e))?;

    // Write tutorial_index.json
    let index_json = serde_json::to_string_pretty(tutorials)
        .map_err(|e| format!("Failed to serialize tutorial index: {}", e))?;
    fs::write(api_dir.join("tutorial_index.json"), index_json)
        .map_err(|e| format!("Failed to write tutorial_index.json: {}", e))?;

    // Write tutorial_families.json
    let families_json = serde_json::to_string_pretty(families)
        .map_err(|e| format!("Failed to serialize family index: {}", e))?;
    fs::write(api_dir.join("tutorial_families.json"), families_json)
        .map_err(|e| format!("Failed to write tutorial_families.json: {}", e))?;

    // Write tutorial_curations.json
    let curations_json = serde_json::to_string_pretty(curations)
        .map_err(|e| format!("Failed to serialize curations: {}", e))?;
    fs::write(api_dir.join("tutorial_curations.json"), curations_json)
        .map_err(|e| format!("Failed to write tutorial_curations.json: {}", e))?;

    Ok(())
}
