//! Discussion section models.
//!
//! This module defines the structure for Discussion content:
//! - Narrative paragraphs (at least 1 required)
//!
//! Content is authored in JSON and validated at build time.

use serde::Deserialize;
use std::borrow::Cow;

/// Discussion section model (simplified).
#[derive(Debug, Clone, Deserialize)]
pub struct DiscussionModel {
    /// Narrative paragraphs for discussion (at least 1 required)
    pub paragraphs: Vec<Cow<'static, str>>,
}
