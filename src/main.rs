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
    load_curations_config, write_index_files, CurationsOutput,
};
use longitudinal_dev::layout::{GuideLayout, PostLayout, SiteLayout};
use longitudinal_dev::models::guide::GuideCatalogItem;
use longitudinal_dev::posts::posts;
use longitudinal_dev::tutorial_catalog::{TutorialCatalog, TutorialData};
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

    // Convert posts to TutorialData for the catalog
    let tutorial_data: Vec<TutorialData> = posts_with_metadata
        .iter()
        .map(TutorialData::from_post)
        .collect();

    let tutorial_catalog_html = view! {
        <SiteLayout options=opts.clone()>
            <main class="min-h-screen bg-surface">
                <section class="relative overflow-hidden bg-subtle">
                    <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                        <h1 class="text-4xl md:text-5xl font-bold text-primary">"ABCD Examples"</h1>
                        <p class="mt-3 text-lg md:text-xl text-secondary">
                            "Examples of longitudinal analysis methods using data from the ABCD Study® dataset."
                        </p>
                        <div class="mt-6 flex flex-col sm:flex-row gap-3">
                            <a href="#catalog" class="inline-block px-6 py-3 rounded-lg bg-accent text-white hover:bg-accent/90 transition-colors">
                                "Browse Examples"
                            </a>
                            <a href={base_path::join("")} class="inline-block px-6 py-3 rounded-lg border border-default text-primary hover:bg-accent-subtle transition-colors">
                                "Back to Home"
                            </a>
                        </div>
                    </div>
                </section>

                <section id="catalog" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-10 mb-10">
                    <TutorialCatalog tutorials=tutorial_data.clone() />
                </section>
            </main>
        </SiteLayout>
    }
    .to_html();

    let tutorials_dir = site_root.join("tutorials");
    create_dir_all(&tutorials_dir)?;
    write(tutorials_dir.join("index.html"), &tutorial_catalog_html)?;
    eprintln!("Wrote {}", tutorials_dir.join("index.html").display());

    // 2.5 Generate tutorial index JSON artifacts for client-side loading
    let tutorial_index = generate_tutorial_index(&posts_with_metadata);
    let family_index = generate_family_index(&tutorial_index);

    // Load curations config or use defaults if not found
    let curations_path = PathBuf::from("content/tutorial_curations.yaml");
    let curations_output = if curations_path.exists() {
        match load_curations_config(&curations_path) {
            Ok(config) => generate_curations_output(&config, &tutorial_index),
            Err(e) => {
                eprintln!("Warning: Failed to load curations config: {}", e);
                eprintln!("Using default empty curations");
                CurationsOutput {
                    getting_started: vec![],
                    workflows: std::collections::HashMap::new(),
                    recently_updated: tutorial_index.iter().take(8).cloned().collect(),
                }
            }
        }
    } else {
        eprintln!("Note: No curations file found at content/tutorial_curations.yaml");
        eprintln!("Using auto-generated recently_updated only");
        CurationsOutput {
            getting_started: vec![],
            workflows: std::collections::HashMap::new(),
            recently_updated: tutorial_index.iter().take(8).cloned().collect(),
        }
    };

    // Write JSON artifacts to dist/api/
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

    // Ensure /posts directory exists for individual tutorials
    let posts_dir = site_root.join("posts");
    create_dir_all(&posts_dir)?;

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

    // 4. Generate one page per post at /posts/<slug>/index.html
    for post in all_posts.into_iter() {
        let slug = post.slug.to_string(); // capture before moving into the view

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

        let html = view! {
            <SiteLayout options=opts.clone()>
                <PostLayout post prefill_markdown baseline_hash/>
            </SiteLayout>
        }
        .to_html();

        let post_dir = posts_dir.join(&slug);
        create_dir_all(&post_dir)?;
        write(post_dir.join("index.html"), html)?;
        eprintln!("Wrote {}", post_dir.join("index.html").display());
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
