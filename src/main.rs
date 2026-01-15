#![recursion_limit = "512"]

mod pages;

use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;
use longitudinal_dev::{base_path, method_family_to_slug};
use longitudinal_dev::guide_catalog::GroupedGuideCatalog;
use longitudinal_dev::guides::{group_guides_by_method, guides};
use longitudinal_dev::index_generator::{
    generate_curations_output, generate_family_index, generate_tutorial_index,
    load_curations_config, validate_curations_slugs, write_index_files, CurationsOutput,
};
use longitudinal_dev::layout::{GuideLayout, PostLayout, SiteLayout};
use longitudinal_dev::models::guide::GuideCatalogItem;
use longitudinal_dev::posts::posts;
// Feature flag: embedded-catalog uses legacy mode with props, otherwise uses fetch mode
#[cfg(feature = "embedded-catalog")]
use longitudinal_dev::tutorial_catalog::{TutorialCatalog, TutorialData};
#[cfg(not(feature = "embedded-catalog"))]
use longitudinal_dev::tutorial_catalog::{
    FamilySummary, LandingSectionsData, TutorialCatalogFetch, TutorialData, WorkflowGroup,
};
use longitudinal_writer::WriterApp;
use pages::abcd_overview::AbcdOverviewPage;
use pages::about::AboutPage;
use pages::resources::{load_resources, ResourcesPage};
use pages::toolkit::ToolkitPage;
use pages::tools::{load_tools, ToolsPage};
use sha2::{Digest, Sha256};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

/// Write an HTML file to a directory, creating the directory if needed.
fn write_html_page(dir: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(dir)?;
    write(dir.join("index.html"), content)?;
    eprintln!("Wrote {}", dir.join("index.html").display());
    Ok(())
}

/// Write an HTML file and log as a redirect.
fn write_redirect_page(dir: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(dir)?;
    write(dir.join("index.html"), content)?;
    eprintln!("Wrote redirect {}", dir.join("index.html").display());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load Leptos options (cargo-leptos provides these during builds)
    let conf = get_configuration(Some("Cargo.toml")).expect("load Leptos config");
    let opts = conf.leptos_options;

    // Check for --outdir override (for deterministic build testing)
    let site_root = if let Some(arg) = std::env::args().nth(1) {
        if arg == "--outdir" {
            if let Some(dir) = std::env::args().nth(2) {
                PathBuf::from(dir)
            } else {
                eprintln!("Error: --outdir requires a directory argument");
                std::process::exit(1);
            }
        } else {
            PathBuf::from(opts.site_root.as_ref())
        }
    } else {
        PathBuf::from(opts.site_root.as_ref())
    };

    let site_root = site_root.as_path();

    // Load posts (feature-gated: JSON or hardcoded)
    let all_posts = posts();
    let post_count = all_posts.len();

    let index_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/".to_string())>
            <main class="min-h-screen flex items-center justify-center bg-gradient-to-br from-surface via-subtle to-muted px-4">
                <div class="text-center max-w-4xl mx-auto">
                    <h1 class="text-5xl md:text-6xl lg:text-7xl font-bold mb-6">
                        <span class="text-primary">"A "</span>
                        <br/>
                        <span class="text-accent">"Longitudinal Data Science"</span>
                        <br/>
                        <span class="text-primary">"Platform"</span>
                    </h1>

                    <p class="text-lg md:text-xl text-secondary max-w-2xl mx-auto">
                        "Open source tools, code examples, and templates for reproducible longitudinal research."
                    </p>
                </div>
            </main>
        </SiteLayout>
    }
    .to_html();

    write_html_page(site_root, &index_html)?;

    // 2. Generate tutorial catalog page at /abcd/index.html
    // Filter out posts without metadata
    let posts_with_metadata: Vec<_> = posts()
        .into_iter()
        .filter(|p| p.metadata.is_some())
        .collect();

    // 2.1 Generate tutorial index JSON artifacts (needed for curations)
    let tutorial_index = generate_tutorial_index(&posts_with_metadata);
    let family_index = generate_family_index(&tutorial_index);

    // 2.2 Load curations config or use defaults
    let curations_path = PathBuf::from("content/tutorial_curations.yaml");
    let curations_output = if curations_path.exists() {
        match load_curations_config(&curations_path) {
            Ok(config) => {
                // Validate slugs before generating output (fail fast on typos)
                let validation_errors = validate_curations_slugs(&config, &tutorial_index);
                if !validation_errors.is_empty() {
                    eprintln!("ERROR: Invalid slugs in tutorial_curations.yaml:");
                    for error in &validation_errors {
                        eprintln!("  - {}", error);
                    }
                    eprintln!("\nAvailable tutorial slugs:");
                    for tutorial in &tutorial_index {
                        eprintln!("  - {}", tutorial.slug);
                    }
                    std::process::exit(1);
                }
                generate_curations_output(&config, &tutorial_index)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load curations config: {}", e);
                eprintln!("Using default empty curations");
                CurationsOutput {
                    featured: vec![],
                    workflows: std::collections::HashMap::new(),
                    recently_updated: tutorial_index.iter().take(8).cloned().collect(),
                }
            }
        }
    } else {
        eprintln!("Note: No curations file found at content/tutorial_curations.yaml");
        eprintln!("Using auto-generated recently_updated only");
        CurationsOutput {
            featured: vec![],
            workflows: std::collections::HashMap::new(),
            recently_updated: tutorial_index.iter().take(8).cloned().collect(),
        }
    };

    // 2.3 Write JSON artifacts to dist/api/
    match write_index_files(site_root, &tutorial_index, &family_index, &curations_output) {
        Ok(()) => {
            eprintln!("Wrote {}/api/tutorial_index.json ({} entries)", site_root.display(), tutorial_index.len());
            eprintln!("Wrote {}/api/tutorial_families.json ({} families)", site_root.display(), family_index.len());
            eprintln!("Wrote {}/api/tutorial_curations.json", site_root.display());
        }
        Err(e) => {
            eprintln!("Warning: Failed to write index files: {}", e);
        }
    }

    // 2.4 Build landing data for the catalog page
    #[cfg(not(feature = "embedded-catalog"))]
    let landing_data = {
        use std::collections::HashMap;

        // Convert curations to landing sections data
        let featured: Vec<TutorialData> = curations_output
            .featured
            .iter()
            .map(|e| TutorialData::from(e.clone()))
            .collect();

        let mut workflows: Vec<WorkflowGroup> = curations_output
            .workflows
            .iter()
            .map(|(key, wo)| WorkflowGroup {
                key: key.clone(),
                label: wo.label.clone(),
                tutorials: wo.tutorials.iter().map(|e| TutorialData::from(e.clone())).collect(),
            })
            .collect();
        workflows.sort_by(|a, b| a.key.cmp(&b.key)); // Ensure deterministic ordering

        let families: Vec<FamilySummary> = family_index
            .iter()
            .map(|f| FamilySummary {
                id: f.id.clone(),
                label: f.label.clone(),
                count: f.count,
            })
            .collect();

        let recently_updated: Vec<TutorialData> = curations_output
            .recently_updated
            .iter()
            .map(|e| TutorialData::from(e.clone()))
            .collect();

        // Compute facets from all tutorials for sidebar filters
        let mut method_families_map: HashMap<String, usize> = HashMap::new();
        let mut engines_map: HashMap<String, usize> = HashMap::new();
        let mut covariates_map: HashMap<String, usize> = HashMap::new();

        for entry in &tutorial_index {
            *method_families_map.entry(entry.method_family.clone()).or_insert(0) += 1;
            *engines_map.entry(entry.statistical_engine.clone()).or_insert(0) += 1;
            *covariates_map.entry(entry.covariates.clone()).or_insert(0) += 1;
        }

        let mut method_families: Vec<_> = method_families_map.into_iter().collect();
        let mut statistical_engines: Vec<_> = engines_map.into_iter().collect();
        let mut covariates: Vec<_> = covariates_map.into_iter().collect();

        method_families.sort_by(|a, b| a.0.cmp(&b.0));
        statistical_engines.sort_by(|a, b| a.0.cmp(&b.0));
        covariates.sort_by(|a, b| a.0.cmp(&b.0));

        LandingSectionsData {
            featured,
            workflows,
            families,
            recently_updated,
            method_families,
            statistical_engines,
            covariates,
        }
    };

    // Generate catalog HTML based on feature flag
    // - Default (fetch mode): TutorialCatalogFetch with landing data
    // - embedded-catalog feature: TutorialCatalog receives full data via props (legacy)
    #[cfg(feature = "embedded-catalog")]
    let tutorial_data: Vec<TutorialData> = posts_with_metadata
        .iter()
        .map(TutorialData::from_post)
        .collect();

    let tutorial_catalog_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/abcd".to_string())>
            <main class="min-h-screen bg-surface">
                // Static hero section
                <section class="relative overflow-hidden bg-subtle">
                    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 lg:py-10">
                        <h1 class="text-4xl md:text-5xl font-bold text-primary">"ABCD Analyses"</h1>
                        <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                            "Examples of longitudinal analysis methods using data from the ABCD Study® dataset."
                        </p>
                        <a
                            href={base_path::join("abcd/overview/")}
                            class="inline-flex items-center gap-1 mt-4 text-sm text-accent hover:text-accent/80 transition-colors"
                        >
                            <span>"Learn more about the ABCD Study"</span>
                            <span>"→"</span>
                        </a>
                    </div>
                </section>

                // Catalog island with landing data
                <section id="catalog" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
                    {
                        #[cfg(feature = "embedded-catalog")]
                        { view! { <TutorialCatalog tutorials=tutorial_data.clone() /> } }
                        #[cfg(not(feature = "embedded-catalog"))]
                        { view! { <TutorialCatalogFetch landing_data=landing_data.clone() /> } }
                    }
                </section>
            </main>
        </SiteLayout>
    }
    .to_html();

    let abcd_dir = site_root.join("abcd");
    write_html_page(&abcd_dir, &tutorial_catalog_html)?;

    // 2b. Generate ABCD Overview page at /abcd/overview/index.html
    let abcd_overview_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/abcd".to_string())>
            <AbcdOverviewPage/>
        </SiteLayout>
    }
    .to_html();

    let abcd_overview_dir = abcd_dir.join("overview");
    write_html_page(&abcd_overview_dir, &abcd_overview_html)?;

    // 3. Generate About page at /about/index.html
    let about_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/about".to_string())>
            <AboutPage/>
        </SiteLayout>
    }
    .to_html();

    let about_dir = site_root.join("about");
    write_html_page(&about_dir, &about_html)?;

    // 3b. Load toolkit data (used by hub page and sub-pages)
    let resources_data = load_resources();
    let tools_data = load_tools();

    // Clone data for use across multiple view macros
    let resources_for_toolkit = resources_data.clone();
    let tools_for_toolkit = tools_data.clone();

    // 3c. Generate Toolkit hub page at /toolkit/index.html
    let toolkit_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/toolkit".to_string())>
            <ToolkitPage resources=resources_for_toolkit tools=tools_for_toolkit />
        </SiteLayout>
    }
    .to_html();

    let toolkit_dir = site_root.join("toolkit");
    write_html_page(&toolkit_dir, &toolkit_html)?;

    // 3d. Generate Resources page at /toolkit/learning/index.html
    let resources_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/toolkit".to_string())>
            <ResourcesPage resources=resources_data />
        </SiteLayout>
    }
    .to_html();

    let learning_dir = toolkit_dir.join("learning");
    write_html_page(&learning_dir, &resources_html)?;

    // 3e. Generate Tools page at /toolkit/software/index.html
    let tools_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/toolkit".to_string())>
            <ToolsPage tools=tools_data />
        </SiteLayout>
    }
    .to_html();

    let software_dir = toolkit_dir.join("software");
    write_html_page(&software_dir, &tools_html)?;

    // 4. Generate one page per tutorial at /abcd/<family>/<slug>/index.html
    // Also generate redirect pages at /posts/<slug>/index.html for backward compatibility
    let posts_dir = site_root.join("posts");
    for post in all_posts.into_iter() {
        let slug = post.slug.to_string();

        // Get method family for URL structure (lowercase, default to "other" if missing)
        let method_family = post
            .metadata
            .as_ref()
            .map(|m| method_family_to_slug(&m.method_family))
            .unwrap_or_else(|| "other".to_string());

        // Read the original markdown file for prefill
        let markdown_path = PathBuf::from("content/tutorials").join(format!("{}.md", slug));
        let (prefill_markdown, baseline_hash) = if markdown_path.exists() {
            match read_to_string(&markdown_path) {
                Ok(markdown) => {
                    // Generate SHA-256 hash of the raw markdown content
                    let mut hasher = Sha256::new();
                    hasher.update(markdown.as_bytes());
                    let hash = format!("{:x}", hasher.finalize());

                    // Pass raw markdown for user-friendly editing
                    (markdown, hash)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", markdown_path.display(), e);
                    (String::new(), String::new())
                }
            }
        } else {
            eprintln!("Warning: Markdown file not found: {}", markdown_path.display());
            (String::new(), String::new())
        };

        // Build canonical URL for this tutorial
        let base_path_prefix = std::env::var("SITE_BASE_PATH").unwrap_or_default();
        let canonical_url = if base_path_prefix.is_empty() {
            format!("/abcd/{}/{}/", method_family, slug)
        } else {
            format!(
                "{}/abcd/{}/{}/",
                base_path_prefix.trim_end_matches('/'),
                method_family,
                slug
            )
        };

        let html = view! {
            <SiteLayout options=opts.clone() canonical_url=canonical_url.clone() current_path=Some("/abcd".to_string())>
                <PostLayout post prefill_markdown baseline_hash/>
            </SiteLayout>
        }
        .to_html();

        // Write tutorial to new canonical URL: /abcd/<family>/<slug>/
        let tutorial_dir = abcd_dir.join(&method_family).join(&slug);
        write_html_page(&tutorial_dir, &html)?;

        // Generate redirect page at old URL: /posts/<slug>/
        // (canonical_url already computed above)
        let redirect_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="0; url={canonical_url}">
    <link rel="canonical" href="{canonical_url}">
    <title>Redirecting...</title>
    <script>window.location.replace("{canonical_url}");</script>
</head>
<body>
    <p>This page has moved. Redirecting to <a href="{canonical_url}">{canonical_url}</a>...</p>
</body>
</html>"#
        );

        let redirect_dir = posts_dir.join(&slug);
        write_redirect_page(&redirect_dir, &redirect_html)?;
    }

    // 5. Generate Writer page at /writer/index.html
    let writer_html = view! {
        <SiteLayout options=opts.clone() current_path=None>
            <WriterApp/>
        </SiteLayout>
    }
    .to_html();

    let writer_dir = site_root.join("writer");
    write_html_page(&writer_dir, &writer_html)?;

    // 6. Generate Method Guides catalog page at /guides/index.html
    let all_guides = guides();
    let guide_count = all_guides.len();

    let guide_catalog_items: Vec<GuideCatalogItem> = all_guides
        .iter()
        .map(GuideCatalogItem::from_guide)
        .collect();

    // Group guides by method (hub + tutorial + reference)
    let method_groups = group_guides_by_method(guide_catalog_items);

    let guides_catalog_html = view! {
        <SiteLayout options=opts.clone() current_path=Some("/guides".to_string())>
            <main class="min-h-screen bg-surface">
                <section class="relative overflow-hidden bg-subtle">
                    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 lg:py-10">
                        <h1 class="text-4xl md:text-5xl font-bold text-primary">"Method Guides"</h1>
                        <p class="mt-3 text-lg md:text-xl text-secondary">
                            "Comprehensive tutorials on longitudinal analysis methods with simulated data examples."
                        </p>
                        <div class="mt-4 flex flex-col sm:flex-row gap-3">
                            <a href="#catalog" class="inline-block px-6 py-3 rounded-lg bg-accent text-white hover:bg-accent/90 transition-colors">
                                "Browse Guides"
                            </a>
                            <a href={base_path::join("")} class="inline-block px-6 py-3 rounded-lg border border-default text-primary hover:bg-accent-subtle transition-colors">
                                "Back to Home"
                            </a>
                        </div>
                    </div>
                </section>

                <section id="catalog" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-10 mb-10">
                    <GroupedGuideCatalog groups=method_groups.clone() />
                </section>

                // Coming Soon section
                <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mb-16">
                    <div class="flex items-center gap-3 mb-6">
                        <span class="text-2xl">"🚧"</span>
                        <div>
                            <h2 class="text-xl font-bold text-primary">"Coming Soon"</h2>
                            <p class="text-sm text-secondary">"Additional method guides in development"</p>
                        </div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                        // GLMM placeholder
                        <div class="rounded-xl bg-subtle border border-stroke p-6 opacity-50">
                            <div class="flex items-center gap-2 mb-2">
                                <span class="text-xs px-2 py-0.5 rounded-full bg-slate-200 text-slate-500 border border-slate-300 dark:bg-slate-700 dark:text-slate-400 dark:border-slate-600">
                                    "Coming Soon"
                                </span>
                            </div>
                            <h3 class="text-lg font-semibold text-tertiary">"Generalized Linear Mixed Models (GLMM)"</h3>
                            <p class="mt-2 text-sm text-tertiary">"Mixed-effects models for non-normal outcomes including binary, count, and categorical data."</p>
                        </div>
                        // GEE placeholder
                        <div class="rounded-xl bg-subtle border border-stroke p-6 opacity-50">
                            <div class="flex items-center gap-2 mb-2">
                                <span class="text-xs px-2 py-0.5 rounded-full bg-slate-200 text-slate-500 border border-slate-300 dark:bg-slate-700 dark:text-slate-400 dark:border-slate-600">
                                    "Coming Soon"
                                </span>
                            </div>
                            <h3 class="text-lg font-semibold text-tertiary">"Generalized Estimating Equations (GEE)"</h3>
                            <p class="mt-2 text-sm text-tertiary">"Population-averaged models for correlated data with robust standard errors."</p>
                        </div>
                        // LCS placeholder
                        <div class="rounded-xl bg-subtle border border-stroke p-6 opacity-50">
                            <div class="flex items-center gap-2 mb-2">
                                <span class="text-xs px-2 py-0.5 rounded-full bg-slate-200 text-slate-500 border border-slate-300 dark:bg-slate-700 dark:text-slate-400 dark:border-slate-600">
                                    "Coming Soon"
                                </span>
                            </div>
                            <h3 class="text-lg font-semibold text-tertiary">"Latent Change Score Models (LCS)"</h3>
                            <p class="mt-2 text-sm text-tertiary">"Structural equation models for examining true change over time while accounting for measurement error."</p>
                        </div>
                    </div>
                </section>
            </main>
        </SiteLayout>
    }
    .to_html();

    let guides_dir = site_root.join("guides");
    write_html_page(&guides_dir, &guides_catalog_html)?;

    // 7. Generate one page per guide at /guides/<slug>/index.html
    for guide in all_guides.into_iter() {
        let slug = guide.slug.to_string();

        let html = view! {
            <SiteLayout options=opts.clone() current_path=Some("/guides".to_string())>
                <GuideLayout guide />
            </SiteLayout>
        }
        .to_html();

        let guide_dir = site_root.join("guides").join(&slug);
        write_html_page(&guide_dir, &html)?;
    }

    eprintln!("\n✅ Generated {post_count} posts + {guide_count} guides + home + tutorials + about + writer");
    Ok(())
}
