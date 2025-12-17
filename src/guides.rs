//! Guide loader for method tutorials.
//!
//! Loads guide markdown files from `content/guides/*.md`, parses frontmatter,
//! and renders markdown to HTML.

use crate::markdown::{preprocess_inline_math, transform_markdown_events_with_outline};
use crate::models::guide::{Guide, GuideCatalogItem, GuideFrontmatter, OutlineNode};
use pulldown_cmark::{html, Options, Parser};
use std::borrow::Cow;
use std::fs;
use std::path::Path;

/// Parse YAML frontmatter from markdown content.
///
/// Returns (frontmatter, remaining_content) if frontmatter is found.
fn parse_frontmatter(content: &str) -> Option<(GuideFrontmatter, &str)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_first = &content[3..];
    let end_pos = after_first.find("\n---")?;

    let yaml_str = &after_first[..end_pos];
    let remaining = &after_first[end_pos + 4..]; // Skip past \n---

    // Parse YAML
    let frontmatter: GuideFrontmatter = serde_yaml::from_str(yaml_str).ok()?;

    Some((frontmatter, remaining.trim_start()))
}

/// Result of rendering markdown: HTML content plus outline.
struct RenderResult {
    html: String,
    outline: Vec<OutlineNode>,
}

/// Render markdown content to HTML and extract outline.
///
/// Applies the transformation pipeline:
/// 0. Pre-process `\(...\)` inline math (before markdown parsing)
/// 1. Parse markdown with pulldown-cmark
/// 2. Transform callouts (blockquotes with markers)
/// 3. Wrap tables for responsive scrolling
/// 4. Wrap designated H2 sections in collapsible modules
/// 5. Render math expressions via KaTeX
/// 6. Convert events to HTML
/// 7. Extract hierarchical outline from H2/H3/H4 headings
fn render_markdown_with_outline(content: &str) -> RenderResult {
    // Pre-process inline math with \(...\) delimiters before markdown parsing
    // This prevents markdown from consuming the backslash escapes
    let content = preprocess_inline_math(content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&content, options);

    // Collect events and apply transformations (with outline extraction)
    let events: Vec<_> = parser.collect();
    let transform_result = transform_markdown_events_with_outline(events);

    let mut html_output = String::new();
    html::push_html(&mut html_output, transform_result.events.into_iter());

    RenderResult {
        html: html_output,
        outline: transform_result.outline,
    }
}

/// Load a single guide from a markdown file.
fn load_guide_from_file(path: &Path) -> Option<Guide> {
    let content = fs::read_to_string(path).ok()?;

    let (frontmatter, markdown_content) = parse_frontmatter(&content)?;

    let render_result = render_markdown_with_outline(markdown_content);

    Some(Guide::from_frontmatter_and_content(
        frontmatter,
        content,
        render_result.html,
        render_result.outline,
    ))
}

/// Load all guides from the content/guides directory.
///
/// Returns guides sorted alphabetically by slug.
pub fn guides() -> Vec<Guide> {
    let guides_dir = Path::new("content/guides");

    if !guides_dir.exists() {
        eprintln!("Warning: content/guides directory does not exist");
        return Vec::new();
    }

    let mut guides = Vec::new();

    if let Ok(entries) = fs::read_dir(guides_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(guide) = load_guide_from_file(&path) {
                    guides.push(guide);
                } else {
                    eprintln!("Warning: Failed to parse guide: {}", path.display());
                }
            }
        }
    }

    // Sort by slug
    guides.sort_by(|a, b| a.slug.cmp(&b.slug));

    guides
}

/// Load guides and convert to catalog items for the index page.
pub fn guide_catalog_items() -> Vec<GuideCatalogItem> {
    guides()
        .iter()
        .map(GuideCatalogItem::from_guide)
        .collect()
}

/// Find a guide by slug.
pub fn find_guide_by_slug(slug: &str) -> Option<Guide> {
    guides().into_iter().find(|g| g.slug == slug)
}

// ============================================================================
// Static guide loading for SSG (compile-time)
// ============================================================================

/// Statically include guide content at compile time.
///
/// This macro generates the guides() function that includes guide files
/// at compile time for SSG builds.
#[macro_export]
macro_rules! include_guides {
    ($($slug:literal),* $(,)?) => {
        pub fn guides_static() -> Vec<$crate::models::guide::Guide> {
            vec![
                $(
                    {
                        let content = include_str!(concat!("../content/guides/", $slug, ".md"));
                        $crate::guides::parse_guide_content($slug, content)
                            .expect(concat!("Failed to parse guide: ", $slug))
                    },
                )*
            ]
        }
    };
}

/// Parse guide content (used by the include_guides macro).
pub fn parse_guide_content(slug: &str, content: &str) -> Option<Guide> {
    let (frontmatter, markdown_content) = parse_frontmatter(content)?;

    // Verify slug matches
    if frontmatter.slug != slug {
        eprintln!(
            "Warning: Guide slug mismatch. File: {}, Frontmatter: {}",
            slug, frontmatter.slug
        );
    }

    let render_result = render_markdown_with_outline(markdown_content);

    Some(Guide {
        slug: Cow::Owned(slug.to_string()),
        title: Cow::Owned(frontmatter.title),
        description: Cow::Owned(frontmatter.description),
        category: Cow::Owned(frontmatter.category),
        tags: frontmatter.tags.into_iter().map(Cow::Owned).collect(),
        r_packages: frontmatter.r_packages.into_iter().map(Cow::Owned).collect(),
        script_path: frontmatter.script_path.map(Cow::Owned),
        raw_markdown: Cow::Owned(content.to_string()),
        html_content: Cow::Owned(render_result.html),
        outline: render_result.outline,
    })
}
