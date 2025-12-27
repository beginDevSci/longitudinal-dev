//! Discussion section models.
//!
//! This module defines the structure for Discussion content:
//! - Structured items with title and HTML content (preferred)
//! - Fallback narrative paragraphs (for backward compatibility)
//!
//! Content is authored in JSON and validated at build time.

use serde::Deserialize;
use std::borrow::Cow;

/// Discussion item with title and rich HTML content.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscussionItem {
    /// Title for the discussion subsection
    pub title: Cow<'static, str>,
    /// HTML content for the subsection
    pub content: Cow<'static, str>,
}

/// Discussion section model.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscussionModel {
    /// Structured items from H2 headings (preferred)
    #[serde(default)]
    pub items: Vec<DiscussionItem>,
    /// Fallback narrative paragraphs (for backward compatibility)
    #[serde(default)]
    pub paragraphs: Vec<Cow<'static, str>>,
}
