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

// ============================================================================
// Outline model for sidebar navigation
// ============================================================================

/// A node in the guide outline hierarchy.
///
/// Represents H2, H3, or H4 headings extracted from the guide markdown.
/// The structure is hierarchical: H2 nodes contain H3 children, which contain H4 children.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutlineNode {
    /// Heading level (2, 3, or 4)
    pub level: u8,
    /// The heading text
    pub title: String,
    /// URL-friendly slug for anchor linking (matches the id in rendered HTML)
    pub id: String,
    /// Child headings (H3 under H2, H4 under H3)
    #[serde(default)]
    pub children: Vec<OutlineNode>,
}

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
    /// Type of guide: "hub", "tutorial", or "reference"
    #[serde(default)]
    pub guide_type: Option<String>,
    /// Parent method slug for tutorials/references (e.g., "lgcm-pilot")
    #[serde(default)]
    pub parent_method: Option<String>,
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
    /// Type of guide: "hub", "tutorial", or "reference"
    pub guide_type: Option<Cow<'static, str>>,
    /// Parent method slug for tutorials/references
    pub parent_method: Option<Cow<'static, str>>,
    /// The full markdown content (for editing/prefill)
    pub raw_markdown: Cow<'static, str>,
    /// The rendered HTML content
    pub html_content: Cow<'static, str>,
    /// Hierarchical outline extracted from H2/H3/H4 headings
    pub outline: Vec<OutlineNode>,
}

impl Guide {
    /// Create a Guide from frontmatter and content.
    pub fn from_frontmatter_and_content(
        frontmatter: GuideFrontmatter,
        raw_markdown: String,
        html_content: String,
        outline: Vec<OutlineNode>,
    ) -> Self {
        Self {
            slug: Cow::Owned(frontmatter.slug),
            title: Cow::Owned(frontmatter.title),
            description: Cow::Owned(frontmatter.description),
            category: Cow::Owned(frontmatter.category),
            tags: frontmatter.tags.into_iter().map(Cow::Owned).collect(),
            r_packages: frontmatter.r_packages.into_iter().map(Cow::Owned).collect(),
            script_path: frontmatter.script_path.map(Cow::Owned),
            guide_type: frontmatter.guide_type.map(Cow::Owned),
            parent_method: frontmatter.parent_method.map(Cow::Owned),
            raw_markdown: Cow::Owned(raw_markdown),
            html_content: Cow::Owned(html_content),
            outline,
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
    /// Type of guide: "hub", "tutorial", or "reference"
    pub guide_type: Option<String>,
    /// Parent method slug for tutorials/references
    pub parent_method: Option<String>,
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
            guide_type: guide.guide_type.as_ref().map(|s| s.to_string()),
            parent_method: guide.parent_method.as_ref().map(|s| s.to_string()),
        }
    }
}

/// A grouped method containing hub, tutorial, and reference guides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodGroup {
    /// The hub guide (conceptual overview)
    pub hub: GuideCatalogItem,
    /// Optional tutorial guide (worked example)
    pub tutorial: Option<GuideCatalogItem>,
    /// Optional reference guide (quick lookup)
    pub reference: Option<GuideCatalogItem>,
    /// Category for the method group
    pub category: String,
}

// ============================================================================
// Category metadata for display and filtering
// ============================================================================

/// Metadata for a guide category.
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryMeta {
    /// URL-friendly identifier (e.g., "growth-models")
    pub id: &'static str,
    /// Display name (e.g., "Growth Models")
    pub name: &'static str,
    /// Emoji or icon for the category
    pub icon: &'static str,
    /// Brief description of the category
    pub description: &'static str,
    /// Sort order (lower = first)
    pub order: u8,
    /// Tailwind CSS classes for the category badge
    pub color_classes: &'static str,
}

/// All known guide categories with their display metadata.
pub const CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "growth-models",
        name: "Growth Models",
        icon: "📈",
        description: "Model change trajectories over time using latent growth curves",
        order: 1,
        color_classes: "bg-emerald-100 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800",
    },
    CategoryMeta {
        id: "mixed-models",
        name: "Mixed Models",
        icon: "🔀",
        description: "Account for nested data structure with fixed and random effects",
        order: 2,
        color_classes: "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-800",
    },
    CategoryMeta {
        id: "survival",
        name: "Survival Analysis",
        icon: "⏱️",
        description: "Model time-to-event data and hazard rates",
        order: 3,
        color_classes: "bg-orange-100 text-orange-700 border-orange-200 dark:bg-orange-900/30 dark:text-orange-400 dark:border-orange-800",
    },
    CategoryMeta {
        id: "latent-variable",
        name: "Latent Variable",
        icon: "🔮",
        description: "Uncover hidden constructs from observed indicators",
        order: 4,
        color_classes: "bg-purple-100 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800",
    },
];

impl CategoryMeta {
    /// Find category metadata by ID.
    pub fn find(id: &str) -> Option<&'static CategoryMeta> {
        CATEGORIES.iter().find(|c| c.id == id)
    }

    /// Get all categories sorted by order.
    pub fn all_sorted() -> Vec<&'static CategoryMeta> {
        let mut cats: Vec<_> = CATEGORIES.iter().collect();
        cats.sort_by_key(|c| c.order);
        cats
    }
}
