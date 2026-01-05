#![recursion_limit = "512"]

mod pages;

use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;
use longitudinal_dev::base_path;
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
use pages::about::AboutPage;
use sha2::{Digest, Sha256};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

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
        <SiteLayout options=opts.clone()>
            <main class="min-h-screen flex items-center justify-center bg-gradient-to-br from-surface via-subtle to-muted px-4">
                <div class="text-center max-w-4xl mx-auto">
                    <h1 class="text-5xl md:text-6xl lg:text-7xl font-bold mb-6">
                        <span class="text-primary">"A "</span>
                        <br/>
                        <span class="text-accent">"Longitudinal Data Science"</span>
                        <br/>
                        <span class="text-primary">"Platform"</span>
                    </h1>

                    <p class="text-lg md:text-xl text-secondary mb-10 max-w-2xl mx-auto">
                        "Open source tools, code examples, and templates for reproducible longitudinal research."
                    </p>

                    <div class="flex flex-col sm:flex-row gap-4 justify-center items-center">
                        <a
                            href={base_path::join("tutorials/")}
                            class="px-8 py-3 bg-accent hover:bg-accent/90 text-white font-medium rounded-lg transition-colors duration-200 shadow-lg hover:shadow-xl"
                        >
                            "ABCD Examples"
                        </a>
                        <a
                            href={base_path::join("guides/")}
                            class="px-8 py-3 bg-emerald-600 hover:bg-emerald-500 text-white font-medium rounded-lg transition-colors duration-200 shadow-lg hover:shadow-xl"
                        >
                            "Method Guides"
                        </a>
                    </div>
                </div>
            </main>
        </SiteLayout>
    }
    .to_html();

    create_dir_all(site_root)?;
    write(site_root.join("index.html"), index_html)?;
    eprintln!("Wrote {}", site_root.join("index.html").display());

    // 2. Generate tutorial catalog page at /tutorials/index.html
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

        let workflows: Vec<WorkflowGroup> = curations_output
            .workflows
            .iter()
            .map(|(key, wo)| WorkflowGroup {
                key: key.clone(),
                label: wo.label.clone(),
                tutorials: wo.tutorials.iter().map(|e| TutorialData::from(e.clone())).collect(),
            })
            .collect();

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
        <SiteLayout options=opts.clone()>
            <main class="min-h-screen bg-surface">
                // Static hero section
                <section class="relative overflow-hidden bg-subtle">
                    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                        <h1 class="text-4xl md:text-5xl font-bold text-primary">"ABCD Examples"</h1>
                        <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                            "Examples of longitudinal analysis methods using data from the ABCD Study® dataset."
                        </p>
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

    let tutorials_dir = site_root.join("tutorials");
    create_dir_all(&tutorials_dir)?;
    write(tutorials_dir.join("index.html"), &tutorial_catalog_html)?;
    eprintln!("Wrote {}", tutorials_dir.join("index.html").display());

    // 3. Generate About page at /about/index.html
    let about_html = view! {
        <SiteLayout options=opts.clone()>
            <AboutPage/>
        </SiteLayout>
    }
    .to_html();

    let about_dir = site_root.join("about");
    create_dir_all(&about_dir)?;
    write(about_dir.join("index.html"), &about_html)?;
    eprintln!("Wrote {}", about_dir.join("index.html").display());

    // 4. Generate one page per tutorial at /tutorials/<family>/<slug>/index.html
    // Also generate redirect pages at /posts/<slug>/index.html for backward compatibility
    let posts_dir = site_root.join("posts");
    create_dir_all(&posts_dir)?;

    for post in all_posts.into_iter() {
        let slug = post.slug.to_string();

        // Get method family for URL structure (lowercase, default to "other" if missing)
        let method_family = post
            .metadata
            .as_ref()
            .map(|m| m.method_family.to_lowercase().replace(' ', "-"))
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
            format!("/tutorials/{}/{}/", method_family, slug)
        } else {
            format!(
                "{}/tutorials/{}/{}/",
                base_path_prefix.trim_end_matches('/'),
                method_family,
                slug
            )
        };

        let html = view! {
            <SiteLayout options=opts.clone() canonical_url=canonical_url.clone()>
                <PostLayout post prefill_markdown baseline_hash/>
            </SiteLayout>
        }
        .to_html();

        // Write tutorial to new canonical URL: /tutorials/<family>/<slug>/
        let tutorial_dir = tutorials_dir.join(&method_family).join(&slug);
        create_dir_all(&tutorial_dir)?;
        write(tutorial_dir.join("index.html"), &html)?;
        eprintln!("Wrote {}", tutorial_dir.join("index.html").display());

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
        create_dir_all(&redirect_dir)?;
        write(redirect_dir.join("index.html"), redirect_html)?;
        eprintln!("Wrote redirect {}", redirect_dir.join("index.html").display());
    }

    // 5. Generate Writer page at /writer/index.html
    let writer_html = view! {
        <SiteLayout options=opts.clone()>
            <WriterApp/>
        </SiteLayout>
    }
    .to_html();

    let writer_dir = site_root.join("writer");
    create_dir_all(&writer_dir)?;
    write(writer_dir.join("index.html"), &writer_html)?;
    eprintln!("Wrote {}", writer_dir.join("index.html").display());

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
        <SiteLayout options=opts.clone()>
            <main class="min-h-screen bg-surface">
                <section class="relative overflow-hidden bg-subtle">
                    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                        <h1 class="text-4xl md:text-5xl font-bold text-primary">"Method Guides"</h1>
                        <p class="mt-3 text-lg md:text-xl text-secondary">
                            "Comprehensive tutorials on longitudinal analysis methods with simulated data examples."
                        </p>
                        <div class="mt-6 flex flex-col sm:flex-row gap-3">
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
            </main>
        </SiteLayout>
    }
    .to_html();

    let guides_dir = site_root.join("guides");
    create_dir_all(&guides_dir)?;
    write(guides_dir.join("index.html"), &guides_catalog_html)?;
    eprintln!("Wrote {}", guides_dir.join("index.html").display());

    // 7. Generate one page per guide at /guides/<slug>/index.html
    for guide in all_guides.into_iter() {
        let slug = guide.slug.to_string();

        let html = view! {
            <SiteLayout options=opts.clone()>
                <GuideLayout guide />
            </SiteLayout>
        }
        .to_html();

        let guide_dir = site_root.join("guides").join(&slug);
        create_dir_all(&guide_dir)?;
        write(guide_dir.join("index.html"), html)?;
        eprintln!("Wrote {}", guide_dir.join("index.html").display());
    }

    eprintln!("\n✅ Generated {post_count} posts + {guide_count} guides + home + tutorials + about + writer");
    Ok(())
}
