//! Overview section data structures
//!
//! These types define the shape of content for the Overview section's
//! four subcontainers: narrative paragraphs, stat list, feature triplet,
//! and summary strip.

use serde::Deserialize;
use std::borrow::Cow;

/// A single row in the stat list panel (right column of split)
#[derive(Clone, Debug, Deserialize)]
pub struct StatRow {
    pub label: Cow<'static, str>,
    pub value: Cow<'static, str>,
    #[serde(default)]
    pub delta: Option<Cow<'static, str>>,
}

/// A feature card in the features panel
#[derive(Clone, Debug, Deserialize, Default)]
pub struct FeatureCard {
    #[serde(default)]
    pub heading: Cow<'static, str>,
    #[serde(default)]
    pub lines: Vec<Cow<'static, str>>, // 1–2 short lines
}

/// Features panel data (expandable grid of feature cards)
#[derive(Clone, Debug, Deserialize, Default)]
pub struct FeaturesPanelData {
    /// Optional custom title for features panel (captured from markdown heading)
    #[serde(default)]
    pub title: Option<Cow<'static, str>>,
    /// 1-5 feature cards (flexible count)
    pub cards: Vec<FeatureCard>,
}

/// Stats panel data (single column list of stat rows)
#[derive(Clone, Debug, Deserialize, Default)]
pub struct StatsPanelData {
    /// Optional custom title for stats panel (defaults to "Analytical Approach" in UI)
    #[serde(default)]
    pub title: Option<Cow<'static, str>>,
    /// Stat rows with auto-assigned labels
    pub rows: Vec<StatRow>,
}

/// Stats panel labels (1-10 stats supported, flexible count)
/// First 5 have semantic labels, remaining get generic "Stat N:" labels
pub(crate) const STATS_PANEL_LABELS: [&str; 10] = [
    "Method:",
    "Parameters:",
    "Sample:",
    "Outcome:",
    "Model:",
    "Stat 6:",
    "Stat 7:",
    "Stat 8:",
    "Stat 9:",
    "Stat 10:",
];

/// Features panel headings (1-5 features supported, flexible count)
/// First 3 have semantic headings, remaining get generic "Feature N" headings
pub(crate) const FEATURES_PANEL_LABELS: [&str; 5] = [
    "When to Use It",
    "Key Advantages",
    "What You'll Learn",
    "Feature 4",
    "Feature 5",
];
