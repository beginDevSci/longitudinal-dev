//! Integration test: Validate SSG output in dist/ directory
//!
//! This test ensures that `make ssg` produces all expected artifacts:
//! - HTML files for each post and index pages
//! - WASM binary and JS glue code
//! - Compiled CSS
//!
//! Run after `make ssg` in CI to catch missing/broken outputs.

use std::fs;
use std::path::{Path, PathBuf};

/// Helper to check if a file exists and is non-empty
fn assert_file_exists_and_non_empty(path: &Path, description: &str) {
    assert!(
        path.exists(),
        "❌ Missing {description}: {}",
        path.display()
    );

    let metadata = fs::metadata(path)
        .unwrap_or_else(|_| panic!("Failed to read metadata for {}", path.display()));

    assert!(
        metadata.len() > 0,
        "❌ Empty file {description}: {}",
        path.display()
    );

    eprintln!(
        "✓ Found {description}: {} ({} bytes)",
        path.display(),
        metadata.len()
    );
}

/// Helper to get all post slugs from content/posts/*.post.json
fn get_post_slugs() -> Vec<String> {
    let posts_dir = PathBuf::from("content/posts");
    let mut slugs = Vec::new();

    if let Ok(entries) = fs::read_dir(posts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Remove .post suffix if present
                    let slug = stem.strip_suffix(".post").unwrap_or(stem);
                    slugs.push(slug.to_string());
                }
            }
        }
    }

    slugs.sort();
    slugs
}

#[test]
fn dist_output_contains_required_files() {
    let dist = PathBuf::from("dist");

    // Skip test if dist/ doesn't exist (not run after `make ssg`)
    if !dist.exists() {
        eprintln!("⚠️  dist/ directory not found. Run `make ssg` before this test.");
        eprintln!("   Skipping validation...");
        return;
    }

    eprintln!("\n🔍 Validating SSG output in dist/...\n");

    // 1. Check core HTML files
    assert_file_exists_and_non_empty(&dist.join("index.html"), "main index");
    assert_file_exists_and_non_empty(&dist.join("tutorials/index.html"), "tutorials index");
    assert_file_exists_and_non_empty(&dist.join("writer/index.html"), "writer page");

    // 2. Check WASM artifacts
    let pkg = dist.join("pkg");
    assert_file_exists_and_non_empty(&pkg.join("blog.js"), "WASM JS glue");
    assert_file_exists_and_non_empty(&pkg.join("blog_bg.wasm"), "WASM binary");

    // 3. Check CSS
    assert_file_exists_and_non_empty(&pkg.join("blog.css"), "compiled CSS");

    // 4. Check post-specific HTML files
    let post_slugs = get_post_slugs();
    eprintln!(
        "\n📝 Found {} post(s) to validate: {:?}\n",
        post_slugs.len(),
        post_slugs
    );

    for slug in &post_slugs {
        let post_html = dist.join(format!("posts/{}/index.html", slug));
        assert_file_exists_and_non_empty(&post_html, &format!("post HTML for '{slug}'"));
    }

    eprintln!("\n✅ All required SSG outputs present and non-empty");
}

#[test]
fn dist_html_files_contain_expected_content() {
    let dist = PathBuf::from("dist");

    // Skip if dist/ doesn't exist
    if !dist.exists() {
        eprintln!("⚠️  dist/ directory not found. Skipping content validation...");
        return;
    }

    eprintln!("\n🔍 Validating HTML content...\n");

    // Check main index contains navigation
    let index_html =
        fs::read_to_string(dist.join("index.html")).expect("Failed to read dist/index.html");

    assert!(
        index_html.contains("<html") && index_html.contains("</html>"),
        "index.html missing HTML structure"
    );

    // Check that WASM is referenced
    assert!(
        index_html.contains("blog.js") || index_html.contains("/pkg/blog.js"),
        "index.html missing WASM JS reference"
    );

    // Check that CSS is referenced
    assert!(
        index_html.contains("blog.css") || index_html.contains("/pkg/blog.css"),
        "index.html missing CSS reference"
    );

    eprintln!("✓ index.html contains expected structure");

    // Check first post has proper structure
    let post_slugs = get_post_slugs();
    if let Some(slug) = post_slugs.first() {
        let post_html_path = dist.join(format!("posts/{}/index.html", slug));
        if post_html_path.exists() {
            let post_html = fs::read_to_string(&post_html_path).expect("Failed to read post HTML");

            assert!(
                post_html.contains("<html") && post_html.contains("</html>"),
                "post HTML missing HTML structure"
            );

            assert!(
                post_html.contains("blog.js") || post_html.contains("/pkg/blog.js"),
                "post HTML missing WASM JS reference"
            );

            eprintln!("✓ Post '{slug}' HTML contains expected structure");
        }
    }

    eprintln!("\n✅ HTML content validation passed");
}
