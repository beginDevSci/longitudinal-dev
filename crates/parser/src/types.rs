use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Complete parsed post matching JSON schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPost {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PostMetadata>,
    pub overview: JsonOverview,
    pub data_access: JsonDataAccess,
    pub data_preparation: JsonDataPrep,
    pub statistical_analysis: JsonStats,
    pub discussion: JsonDiscussion,
    pub additional_resources: JsonAdditionalResources,
}

/// Metadata for catalog/index display and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub method_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_family_label: Option<String>,
    pub statistical_engine: String,
    /// Array of statistical engines (preferred over statistical_engine)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engines: Vec<String>,
    pub covariates: String,
    pub outcome_type: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Explicit summary for catalog display
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Difficulty level: intro, intermediate, advanced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    /// Timepoint count bucket: 2, 3_5, 6_plus, irregular
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timepoints: Option<String>,
    /// Draft flag - if true, tutorial is hidden from catalog but still renders
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

/// Stat item - either simple string or object with custom label
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonStatItem {
    /// Simple string value with auto-assigned label based on position
    Simple(String),
    /// Object with optional custom label and required value
    WithLabel {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        value: String,
    },
}

/// Feature item - either simple string or object with custom heading
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonFeatureItem {
    /// Simple string value with auto-assigned heading based on position
    Simple(String),
    /// Object with optional custom heading and required text
    WithHeading {
        #[serde(skip_serializing_if = "Option::is_none")]
        heading: Option<String>,
        text: String,
    },
}

/// Overview section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOverview {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_panel: Option<JsonStatsPanel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features_panel: Option<JsonFeaturesPanel>,
}

/// Stats panel with optional custom title
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonStatsPanel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<JsonStatItem>,
}

/// Features panel with optional custom title
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonFeaturesPanel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<JsonFeatureItem>,
}

/// Data Access section item - either collapsible or prose
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsonDataAccessItem {
    /// Collapsible section with title and content
    Collapsible {
        title: String,
        content: String,
        #[serde(default = "default_true")]
        open: bool,
    },
    /// Plain prose/HTML content
    Prose { content: String },
}

fn default_true() -> bool {
    true
}

/// Data Access section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDataAccess {
    #[serde(default)]
    pub items: Vec<JsonDataAccessItem>,
    /// Fallback for backward compatibility - plain prose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTableNote {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub copy: String,
}

/// Discussion section item - collapsible with title and HTML content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDiscussionItem {
    pub title: String,
    pub content: String,
}

/// Discussion section - structured with collapsible items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDiscussion {
    /// Structured items from H2 headings (preferred)
    #[serde(default)]
    pub items: Vec<JsonDiscussionItem>,
    /// Fallback paragraphs (for backward compatibility)
    #[serde(default)]
    pub paragraphs: Vec<String>,
}

/// Additional Resources section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAdditionalResources {
    pub cards: Vec<JsonResourceCard>,
}

// Type alias for consistency with parser code
pub type JsonResources = JsonAdditionalResources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResourceCard {
    pub title: String,
    pub badge: String,
    pub body: String,
    pub url: String,
}

/// Section boundaries in event stream
#[derive(Debug, Clone)]
pub struct SectionBoundaries {
    pub overview: Range<usize>,
    pub data_access: Range<usize>,
    pub data_preparation: Range<usize>,
    pub statistical_analysis: Range<usize>,
    pub discussion: Range<usize>,
    pub additional_resources: Range<usize>,
}

/// Frontmatter extracted from YAML block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    // Metadata fields for tutorial catalog
    pub author: Option<String>,
    #[serde(rename = "date_iso")]
    pub updated_at: Option<String>,
    pub tags: Option<Vec<String>>,
    pub family: Option<String>,
    pub family_label: Option<String>,
    /// Single engine (legacy, for backward compatibility)
    pub engine: Option<String>,
    /// Array of engines (preferred)
    pub engines: Option<Vec<String>>,
    pub covariates: Option<String>,
    pub outcome_type: Option<String>,
    /// Explicit summary for catalog display
    pub summary: Option<String>,
    /// Difficulty level: intro, intermediate, advanced
    pub difficulty: Option<String>,
    /// Timepoint count bucket: 2, 3_5, 6_plus, irregular
    pub timepoints: Option<String>,
    /// Draft flag - if true, tutorial is hidden from catalog but still renders
    pub draft: Option<bool>,
}

// ============================================================================
// V2 SCHEMA TYPES - Flexible Block-Based Sections
// ============================================================================

/// Statistical Analysis - flexible content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStats {
    pub content_blocks: Vec<JsonStatsBlock>,
}

/// Data Preparation - flexible content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDataPrep {
    pub content_blocks: Vec<JsonDataPrepBlock>,
}

/// Statistical Analysis block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum JsonStatsBlock {
    Code(JsonCodeBlock),
    Output(JsonOutputBlock),
    Note(JsonNoteBlock),
}

/// Data Preparation v2 block types (same as Stats v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum JsonDataPrepBlock {
    Code(JsonCodeBlock),
    Output(JsonOutputBlock),
    Note(JsonNoteBlock),
}

/// Code block for v2 sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCodeBlock {
    pub title: String,
    pub content: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_open: Option<bool>,
}

fn default_language() -> String {
    "r".to_string()
}

/// Output block for v2 sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutputBlock {
    pub content: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

fn default_format() -> String {
    "text".to_string()
}

/// Note/interpretation block for v2 sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonNoteBlock {
    pub title: String,
    pub content: String,
}

// ============================================================================
// VERSION ENUMS - Support both v1 and v2 schemas
// ============================================================================

// Note: V1 enum types removed - all tutorials now use V2 format exclusively
