//! Data models for blog posts and sections.
//!
//! This module defines typed structures for each of the six sections,
//! replacing the previous flat string fields and boolean flags.

use serde::Deserialize;
use std::borrow::Cow;

// Note: Version enums removed - all tutorials now use V2 format exclusively

/// Core post model with exactly six sections.
///
/// This structure enforces the 1+6 layout at compile time:
/// - One title (rendered as `<h1>`)
/// - Six section models (each rendered as a `<section>`)
///
/// All six sections are now required.
#[derive(Debug, Clone, Deserialize)]
pub struct Post {
    pub slug: Cow<'static, str>,
    pub title: Cow<'static, str>,
    #[serde(default)]
    pub layout: Option<Cow<'static, str>>,
    #[serde(default)]
    pub metadata: Option<PostMetadata>,
    pub overview: OverviewModel,
    pub data_access: crate::models::data_access::DataAccessModel,
    pub data_prep: crate::models::data_preparation::DataPrepModel,
    pub statistical_analysis: crate::models::statistical_analysis::StatsModel,
    pub discussion: crate::models::discussion::DiscussionModel,
    pub additional_resources: crate::models::additional_resources::ResourcesModel,
}

/// Metadata for catalog/index display and filtering
#[derive(Debug, Clone, Deserialize)]
pub struct PostMetadata {
    pub method_family: String,
    #[serde(default)]
    pub method_family_label: Option<String>,
    pub statistical_engine: String,
    /// Array of statistical engines (preferred over statistical_engine for new tutorials)
    #[serde(default)]
    pub engines: Vec<String>,
    pub covariates: String,
    pub outcome_type: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Explicit summary for catalog display
    #[serde(default)]
    pub summary: Option<String>,
    /// Difficulty level: intro, intermediate, advanced
    #[serde(default)]
    pub difficulty: Option<String>,
    /// Timepoint count bucket: 2, 3_5, 6_plus, irregular
    #[serde(default)]
    pub timepoints: Option<String>,
}

/// Overview section: summary and key statistics.
///
/// Rendered with four subcontainers:
/// 1. Section header (h2)
/// 2. Split layout (left: narrative paragraphs, right: stats panel)
/// 3. Features panel (optional, requires {.features} marker, 0-5 cards)
#[derive(Debug, Clone, Deserialize)]
pub struct OverviewModel {
    /// Left column paragraphs (narrative text)
    pub summary_paragraphs: Vec<Cow<'static, str>>,
    /// Stats panel (optional, requires {.stats} marker, right column)
    #[serde(default)]
    pub stats_panel: Option<crate::models::overview::StatsPanelData>,
    /// Features panel (optional, requires {.features} marker, 0-5 cards with auto-assigned headings)
    #[serde(default)]
    pub features_panel: Option<crate::models::overview::FeaturesPanelData>,
}
