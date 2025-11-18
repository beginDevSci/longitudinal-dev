use leptos::prelude::*;

use crate::base_path;
use crate::layout::{CodeDownloadData, LeftNav, NavCategory, TableOfContents, TocItem};
use crate::models::post::Post;
use crate::sections::*;
use crate::EditorModalIsland;

/// Extract code blocks from a post for downloads.
///
/// Scans Data Preparation and Statistical Analysis sections for code blocks,
/// separating by language (R, Python) and collecting all code into markdown.
fn extract_code_downloads(post: &Post) -> CodeDownloadData {
    let mut markdown_parts = Vec::new();
    let mut r_code_parts = Vec::new();
    let mut python_code_parts = Vec::new();

    // Helper to process code blocks
    let mut process_code = |language: &str, content: &str| {
        // Add to markdown collection with language fence
        markdown_parts.push(format!("```{language}\n{content}\n```\n\n"));

        // Add to language-specific collection
        match language.to_lowercase().as_str() {
            "r" => r_code_parts.push(content.to_string()),
            "python" | "py" => python_code_parts.push(content.to_string()),
            _ => {} // Ignore other languages for now
        }
    };

    // Extract from Data Preparation section
    for block in &post.data_prep.content_blocks {
        if let crate::models::data_preparation::ContentBlock::Code(code_data) = block {
            process_code(&code_data.language, &code_data.content);
        }
    }

    // Extract from Statistical Analysis section
    for block in &post.statistical_analysis.content_blocks {
        if let crate::models::statistical_analysis::ContentBlock::Code(code_data) = block {
            process_code(&code_data.language, &code_data.content);
        }
    }

    CodeDownloadData {
        markdown: markdown_parts.join(""),
        r_code: if r_code_parts.is_empty() {
            None
        } else {
            Some(r_code_parts.join("\n\n"))
        },
        python_code: if python_code_parts.is_empty() {
            None
        } else {
            Some(python_code_parts.join("\n\n"))
        },
    }
}

/// Renders one post with the STRICT 1+6 structure using the new typed models.
///
/// This component guarantees:
/// - Exactly one <h1> (title)
/// - Exactly six <section> elements (via SectionContainer)
///
/// Inner content varies based on the Post data models, but the outer
/// structure never changes. This is our canonical post template.
///
/// # Content Variations
///
/// - **Plain text**: Simple paragraphs in most sections
/// - **CSS-only tabs**: Via DataAccessContent::Tabs
/// - **Islands**: Interactive components (ThemeToggle, CopyCodeButton, etc.)
///
/// # Layout Variants
///
/// The optional `layout` field controls UI variations:
/// - **None or "standard"**: Full layout with all UI elements (default)
/// - **"compact"**: Omits Quick Summary Details panel from Overview section
///
/// # Structure Enforcement
///
/// The six SectionContainer calls are hard-coded here. The typed models
/// provide compile-time safety for content shape.
#[component]
pub fn PostLayout(post: Post) -> impl IntoView {
    // Extract code for downloads before moving post fields
    let code_downloads = extract_code_downloads(&post);

    // Load all posts for navigation and create nav categories
    let all_posts = crate::posts::posts();
    let nav_categories = NavCategory::from_posts(&all_posts);

    // Store current post slug for active highlighting
    let current_slug = post.slug.to_string();

    // Generate page URL for suggestions
    let page_url = format!("https://swhawes.github.io/longitudinal-dev/posts/{}/", current_slug);

    // TODO: In next step, generate actual prefill_markdown and baseline_hash during SSG
    // For now, use placeholders
    let prefill_markdown = String::from("# Placeholder\n\nMarkdown content will be generated during SSG.");
    let baseline_hash = String::from("placeholder_hash");

    // Move fields out before view! to own the data
    let title = post.title;
    let metadata = post.metadata;
    let _layout = post.layout;
    let overview = post.overview;
    let data_access = post.data_access;
    let data_prep = post.data_prep;
    let statistical_analysis = post.statistical_analysis;
    let discussion = post.discussion;
    let additional_resources = post.additional_resources;

    // Generate TOC items for all 6 sections
    let toc_items = vec![
        TocItem {
            id: "overview-title".to_string(),
            title: "Overview".to_string(),
            level: 2,
        },
        TocItem {
            id: "data-access-title".to_string(),
            title: "Data Access".to_string(),
            level: 2,
        },
        TocItem {
            id: "data-prep-title".to_string(),
            title: "Data Preparation".to_string(),
            level: 2,
        },
        TocItem {
            id: "stats-title".to_string(),
            title: "Statistical Analysis".to_string(),
            level: 2,
        },
        TocItem {
            id: "discussion-title".to_string(),
            title: "Discussion".to_string(),
            level: 2,
        },
        TocItem {
            id: "resources-title".to_string(),
            title: "Additional Resources".to_string(),
            level: 2,
        },
    ];

    view! {
        <>
            // Three-column layout: left nav + main content + right TOC
            <div class="grid grid-cols-1 lg:grid-cols-[auto_1fr_max-content] gap-0">
            // Column 1: Left Navigation (hidden on mobile < lg)
            <div class="hidden lg:block">
                <LeftNav
                    categories=nav_categories.clone()
                    current_slug=current_slug.clone()
                    base_path=base_path::base_path()
                />
            </div>

            // Column 2: Main Content Area
            <div class="flex flex-col min-w-0">
                // Title container (exactly one <h1>) with optional metadata
                <header class="px-6 pt-4 pb-2">
                    <div class="flex flex-col gap-4 lg:gap-6 hero-header">
                        <div class="flex flex-col gap-2 lg:flex-1 lg:min-w-0">
                            {metadata.as_ref().and_then(|meta| {
                                meta
                                    .method_family_label
                                    .as_ref()
                                    .filter(|label| !label.is_empty())
                                    .map(|label| {
                                        view! {
                                            <p class="hero-kicker">
                                                {label.clone()}
                                            </p>
                                        }
                                    })
                            })}

                            <h1 class="post-title">
                                {match title.split_once(':') {
                                    Some((prefix, variant)) => {
                                        let prefix_trimmed = prefix.trim();
                                        let variant_trimmed = variant.trim();
                                        if prefix_trimmed.is_empty() || variant_trimmed.is_empty() {
                                            title.clone()
                                        } else {
                                            std::borrow::Cow::Owned(format!(
                                                "{variant_trimmed} ({prefix_trimmed})"
                                            ))
                                        }
                                    }
                                    None => title.clone(),
                                }}
                            </h1>

                            // Description - display if metadata exists and description is present
                            {metadata.as_ref().and_then(|meta| {
                                meta.description.as_ref().and_then(|desc| {
                                    if !desc.is_empty() {
                                        Some(view! {
                                            <p class="body-text hero-description">
                                                {desc.clone()}
                                            </p>
                                        })
                                    } else {
                                        None
                                    }
                                })
                            })}

                            // Metadata context row - only show if metadata exists
                            {metadata.as_ref().map(|meta| {
                                view! {
                                    <div class="flex flex-col gap-2.5">
                                        // Breadcrumb text (non-interactive) - method family > outcome type
                                        <div class="hero-breadcrumb">
                                            {meta.method_family.clone()} " › " {meta.outcome_type.clone()}
                                        </div>

                                        // Pills for key metadata
                                        <div class="flex flex-wrap gap-2.5 lg:flex-nowrap lg:overflow-x-auto">
                                            {{
                                                let mut pills = Vec::new();

                                                if !meta.statistical_engine.is_empty() {
                                                    pills.push(view! {
                                                        <span class="hero-pill hero-pill--primary">
                                                            {meta.statistical_engine.clone()}
                                                        </span>
                                                    });
                                                }

                                                if !meta.updated_at.is_empty() {
                                                    pills.push(view! {
                                                        <span class="hero-pill hero-pill--primary">
                                                            {format!("Updated {}", meta.updated_at)}
                                                        </span>
                                                    });
                                                }

                                                if !meta.tags.is_empty() {
                                                    for tag in &meta.tags {
                                                        pills.push(view! {
                                                            <span class="hero-pill hero-pill--tag">
                                                                {tag.clone()}
                                                            </span>
                                                        });
                                                    }
                                                }

                                                pills.into_iter().collect_view()
                                            }}
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>

                    // Draft banner shown on every tutorial page until content is finalized
                    <div class="draft-banner">
                        <span class="draft-banner-pulse" aria-hidden="true"></span>
                        <p class="leading-relaxed whitespace-nowrap">
                            "Examples are a work in progress—please share feedback if you spot gaps."
                        </p>
                    </div>
                </header>

                // Main content - Fixed six sections with responsive spacing
                <main class="px-6 pb-6 space-y-4 md:space-y-6 min-w-0">
                    // Section 1: Overview
                    <OverviewSection model=overview/>

                    // Section 2: Data Access
                    <DataAccessSection model=data_access/>

                    // Section 3: Data Preparation
                    <DataPreparationSection model=data_prep/>

                    // Section 4: Statistical Analysis
                    <StatisticalAnalysisSection model=statistical_analysis/>

                    // Section 5: Discussion
                    <DiscussionSection model=discussion/>

                    // Section 6: Additional Resources
                    <AdditionalResourcesSection model=additional_resources/>
                </main>
            </div>

            // Column 3: Right aside - Table of Contents (hidden on mobile < lg, shown on laptop+)
            <div class="hidden lg:block">
                <TableOfContents
                    toc_items=toc_items
                    download_data=code_downloads
                    repo_url="https://github.com/swhawes/leptos-test".to_string()
                    slug=current_slug.clone()
                />
            </div>
        </div>

        // Editor modal rendered at page level (outside aside container) for proper centering
        <EditorModalIsland
            slug=current_slug
            page_url=page_url
            prefill_markdown=prefill_markdown
            baseline_hash=baseline_hash
        />
        </>
    }
}
