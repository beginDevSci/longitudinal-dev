//! Data Preparation section models v2 - Flexible block-based system.
//!
//! This module defines the v2 structure for Data Preparation content:
//! - Fully flexible ordering of content blocks
//! - Three block types: code, output, note
//! - No constraints on quantity or relationships
//!
//! Content is authored in JSON and validated at build time.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Root Data Preparation model v2
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataPrepModel {
    /// Ordered sequence of content blocks (1-50 blocks)
    pub content_blocks: Vec<ContentBlock>,
}

/// Content block discriminated union
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ContentBlock {
    #[serde(rename = "code")]
    Code(CodeData),

    #[serde(rename = "output")]
    Output(OutputData),

    #[serde(rename = "note")]
    Note(NoteData),
}

/// Code block data - displays code snippets with syntax highlighting
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CodeData {
    /// Code block title (required)
    pub title: Cow<'static, str>,

    /// Code content (required)
    pub content: Cow<'static, str>,

    /// Programming language for syntax highlighting (defaults to "r")
    #[serde(default = "default_language_r")]
    pub language: Cow<'static, str>,

    /// Optional source filename reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<Cow<'static, str>>,

    /// Whether code block should be expanded by default (defaults to false/collapsed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_open: Option<bool>,
}

/// Output block data - displays results (text, tables, or images)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputData {
    /// Output content (interpretation depends on format field)
    pub content: Cow<'static, str>,

    /// Output format (text, table, or image)
    #[serde(default)]
    pub format: OutputFormat,

    /// Accessibility alt text (recommended for images)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<Cow<'static, str>>,

    /// Optional caption displayed below output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<Cow<'static, str>>,
}

/// Output format types
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Text,
    Table,
    Image,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Text
    }
}

/// Note block data - explanation/context callout cards
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteData {
    /// Note title/heading (required)
    pub title: Cow<'static, str>,

    /// Note content/explanation (required)
    pub content: Cow<'static, str>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn default_language_r() -> Cow<'static, str> {
    Cow::Borrowed("r")
}

// ============================================================================
// VALIDATION HELPERS (Optional soft validation)
// ============================================================================

impl DataPrepModel {
    /// Soft validation - returns warnings (not errors)
    pub fn validate_soft(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warn if no code blocks
        let has_code = self
            .content_blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::Code(_)));
        if !has_code {
            warnings
                .push("No code blocks found. Consider adding at least one code block.".to_string());
        }

        // Warn if image output without alt text (accessibility)
        for (i, block) in self.content_blocks.iter().enumerate() {
            if let ContentBlock::Output(data) = block {
                if data.format == OutputFormat::Image && data.alt.is_none() {
                    warnings.push(format!(
                        "Output block {} is an image but missing alt text (accessibility concern)",
                        i + 1
                    ));
                }
            }
        }

        warnings
    }

    /// Count blocks by type
    pub fn count_blocks(&self) -> (usize, usize, usize) {
        let mut code_count = 0;
        let mut output_count = 0;
        let mut note_count = 0;

        for block in &self.content_blocks {
            match block {
                ContentBlock::Code(_) => code_count += 1,
                ContentBlock::Output(_) => output_count += 1,
                ContentBlock::Note(_) => note_count += 1,
            }
        }

        (code_count, output_count, note_count)
    }
}
