//! Statistical Analysis section models v2 - Flexible block-based system.
//!
//! This module defines the v2 structure for Statistical Analysis content:
//! - Fully flexible ordering of content blocks
//! - Four block types: code, output, interpretation (note), viewer (interactive)
//! - No constraints on quantity or relationships
//!
//! Content is authored in JSON and validated at build time.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Root Statistical Analysis model v2
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatsModel {
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

    #[serde(rename = "viewer")]
    Viewer(ViewerData),
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Text,
    Table,
    Image,
}

/// Note block data - interpretation/explanation callout cards
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteData {
    /// Note title/heading (required)
    pub title: Cow<'static, str>,

    /// Note content/explanation (required)
    pub content: Cow<'static, str>,
}

/// Interactive viewer block data - WebGPU-powered visualizations (e.g., brain surfaces)
///
/// Renders an interactive 3D viewer when WebGPU is available, with graceful
/// fallback to a static image when not supported or on error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ViewerData {
    /// Path to manifest JSON that describes surfaces, contrasts, and defaults.
    /// Example: "/data/blmm/manifest.json"
    pub manifest_path: Cow<'static, str>,

    /// Optional override values for this viewer instance.
    /// These take precedence over defaults specified in the manifest.
    #[serde(default, skip_serializing_if = "ViewerOverrides::is_empty")]
    pub overrides: ViewerOverrides,

    /// Optional label/caption displayed below the viewer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<Cow<'static, str>>,

    /// Static fallback image path (shown if WebGPU unavailable or viewer fails).
    /// Should point to a representative static render of the visualization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_image: Option<Cow<'static, str>>,

    /// Alt text for the fallback image (accessibility).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_alt: Option<Cow<'static, str>>,

    /// If false (default), show a "Load 3D Viewer" button instead of auto-initializing.
    /// Recommended false to avoid loading heavy WASM/WebGPU code on every page view.
    #[serde(default)]
    pub auto_start: bool,
}

/// Override values for viewer configuration.
/// All fields are optional; unset fields use manifest defaults.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ViewerOverrides {
    /// Analysis design to display (e.g., "des1", "des2")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<Cow<'static, str>>,

    /// Statistic type to display (e.g., "conT", "conTlp", "beta", "sigma2")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistic: Option<Cow<'static, str>>,

    /// Contrast/volume index (0-based)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_idx: Option<u32>,

    /// Colormap name (e.g., "coolwarm", "viridis")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colormap: Option<Cow<'static, str>>,

    /// Threshold value for display (e.g., 2.0 for |t| > 2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f32>,

    /// Hemisphere to display: "lh", "rh", or "both"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hemisphere: Option<Cow<'static, str>>,
}

impl ViewerOverrides {
    /// Returns true if all override fields are None (used for skip_serializing_if)
    pub fn is_empty(&self) -> bool {
        self.analysis.is_none()
            && self.statistic.is_none()
            && self.volume_idx.is_none()
            && self.colormap.is_none()
            && self.threshold.is_none()
            && self.hemisphere.is_none()
    }
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

impl StatsModel {
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

    /// Count blocks by type (code, output, note, viewer)
    pub fn count_blocks(&self) -> (usize, usize, usize, usize) {
        let mut code_count = 0;
        let mut output_count = 0;
        let mut note_count = 0;
        let mut viewer_count = 0;

        for block in &self.content_blocks {
            match block {
                ContentBlock::Code(_) => code_count += 1,
                ContentBlock::Output(_) => output_count += 1,
                ContentBlock::Note(_) => note_count += 1,
                ContentBlock::Viewer(_) => viewer_count += 1,
            }
        }

        (code_count, output_count, note_count, viewer_count)
    }
}
