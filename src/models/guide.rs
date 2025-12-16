//! Guide model for method tutorials.
//!
//! Guides are comprehensive method tutorials that are separate from
//! the ABCD-specific examples. They use a different structure:
//! - Overview
//! - Conceptual Foundations
//! - Model Specification & Fit
//! - Worked Example (collapsible)
//! - Reference & Resources (collapsible)

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Frontmatter for a guide markdown file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideFrontmatter {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub r_packages: Vec<String>,
    /// Optional path to downloadable R script
    pub script_path: Option<String>,
}

/// A parsed guide ready for rendering.
#[derive(Debug, Clone)]
pub struct Guide {
    pub slug: Cow<'static, str>,
    pub title: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub category: Cow<'static, str>,
    pub tags: Vec<Cow<'static, str>>,
    pub r_packages: Vec<Cow<'static, str>>,
    pub script_path: Option<Cow<'static, str>>,
    /// The full markdown content (for editing/prefill)
    pub raw_markdown: Cow<'static, str>,
    /// The rendered HTML content
    pub html_content: Cow<'static, str>,
}

impl Guide {
    /// Create a Guide from frontmatter and content.
    pub fn from_frontmatter_and_content(
        frontmatter: GuideFrontmatter,
        raw_markdown: String,
        html_content: String,
    ) -> Self {
        Self {
            slug: Cow::Owned(frontmatter.slug),
            title: Cow::Owned(frontmatter.title),
            description: Cow::Owned(frontmatter.description),
            category: Cow::Owned(frontmatter.category),
            tags: frontmatter.tags.into_iter().map(Cow::Owned).collect(),
            r_packages: frontmatter.r_packages.into_iter().map(Cow::Owned).collect(),
            script_path: frontmatter.script_path.map(Cow::Owned),
            raw_markdown: Cow::Owned(raw_markdown),
            html_content: Cow::Owned(html_content),
        }
    }
}

/// Catalog data for displaying guides in the index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideCatalogItem {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub r_packages: Vec<String>,
}

impl GuideCatalogItem {
    pub fn from_guide(guide: &Guide) -> Self {
        Self {
            slug: guide.slug.to_string(),
            title: guide.title.to_string(),
            description: guide.description.to_string(),
            category: guide.category.to_string(),
            tags: guide.tags.iter().map(|t| t.to_string()).collect(),
            r_packages: guide.r_packages.iter().map(|p| p.to_string()).collect(),
        }
    }
}
