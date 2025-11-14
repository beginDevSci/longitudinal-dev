//! Additional Resources section models.
//!
//! This module defines the structure for Additional Resources content with:
//! - Exactly 3 resource cards with title, badge, body, and optional URL
//!
//! Content is authored in JSON and validated at build time.
//! Section heading is auto-assigned to "Additional Resources".

use serde::Deserialize;
use std::borrow::Cow;

/// Single resource card.
/// Badge is required for visual consistency.
/// URL is optional (chevron shown only if URL present).
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceCard {
    pub title: Cow<'static, str>,
    pub body: Cow<'static, str>,
    /// Uppercase badge label (e.g., "DOCS", "CODE", "DATA")
    pub badge_upper: Cow<'static, str>,
    #[serde(default)]
    pub url: Option<Cow<'static, str>>,
    /// Auto-calculated: true if URL is present
    pub show_chevron: bool,
}

/// Additional Resources section model.
/// Contains 0-8 resource cards (flexible, optional).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ResourcesModel {
    /// 0-8 resource cards (flexible count, optional)
    #[serde(default)]
    pub items: Vec<ResourceCard>,
}
