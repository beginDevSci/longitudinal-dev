/// Structure integrity test
///
/// Asserts that the generated HTML maintains the fixed blog structure:
/// - Exactly 1 <h1> element (post title)
/// - Exactly 6 <section> elements (fixed sections)
use std::fs;
use std::path::Path;

#[test]
fn test_html_structure_integrity() {
    // NOTE: This test now checks the landing page (index.html)
    // Individual post structure is tested in test_each_post_has_fixed_structure
    let html_path = Path::new("target/site/index.html");

    // Skip test if HTML not yet generated (pre-build)
    if !html_path.exists() {
        println!("SKIP: target/site/index.html not found (run build first)");
        return;
    }

    let html = fs::read_to_string(html_path).expect("Failed to read generated HTML");

    // Landing page should have 1 h1 (Blog Index title)
    let h1_count = html.matches("<h1").count();
    assert_eq!(
        h1_count, 1,
        "Expected exactly 1 <h1> tag (index page title), found {h1_count}"
    );

    // Verify the page contains links to posts section
    assert!(html.contains("/posts/"), "Missing posts section link");

    println!("✅ Landing page structure verified:");
    println!("   - <h1> tags: {h1_count}");
    println!("   - Posts section link: present");
}

#[test]
fn test_islands_hydration_wiring() {
    let html_path = Path::new("target/site/index.html");

    // Skip if not built
    if !html_path.exists() {
        println!("SKIP: target/site/index.html not found (run build first)");
        return;
    }

    let html = fs::read_to_string(html_path).expect("Failed to read generated HTML");

    // Verify island wrapper exists
    let island_count = html.matches("leptos-island").count();
    assert!(
        island_count >= 1,
        "Expected at least 1 leptos-island tag (interactive components), found {island_count}"
    );

    // Verify hydration script is present
    assert!(
        html.contains("hydrateIslands"),
        "Hydration script not found in HTML"
    );

    // Verify WASM references
    assert!(
        html.contains("blog.js") || html.contains("blog_bg.wasm"),
        "WASM/JS references not found in HTML"
    );

    println!("✅ Islands hydration wiring verified:");
    println!("   - Island tags: {island_count}");
    println!("   - Hydration script: present");
    println!("   - WASM references: present");
}

/// Multi-post structure test
///
/// Verifies that each generated post maintains the strict 1+6 structure
#[test]
fn test_each_post_has_fixed_structure() {
    let site_root = Path::new("target/site");
    // Use actual tutorial posts
    let post_slugs = ["lgcm-basic", "lmm-random-slopes", "mlgcm"];

    for slug in post_slugs {
        let post_path = site_root.join("posts").join(slug).join("index.html");

        // Skip if not built yet
        if !post_path.exists() {
            println!("SKIP: {} not found (run build first)", post_path.display());
            continue;
        }

        let html = fs::read_to_string(&post_path)
            .unwrap_or_else(|_| panic!("Failed to read: {}", post_path.display()));

        // Count h1 tags
        let h1_count = html.matches("<h1").count();
        assert_eq!(
            h1_count, 1,
            "Post '{slug}' should have exactly 1 <h1>, found {h1_count}"
        );

        // Count section tags
        let section_count = html.matches("<section").count();

        // All 6 sections are required (Overview, Data Access, Data Preparation,
        // Statistical Analysis, Discussion, Additional Resources)
        let expected_sections = 6;

        assert_eq!(
            section_count, expected_sections,
            "Post '{slug}' should have exactly {expected_sections} <section> tags (6 required sections), found {section_count}"
        );

        println!("✅ Post '{slug}' structure verified (1 h1, {section_count} sections)");
    }
}

/// Data Access section test
///
/// Verifies that tutorial posts have the Data Access section
#[test]
fn test_data_access_section_present() {
    let html_path = Path::new("target/site/posts/lgcm-basic/index.html");

    if !html_path.exists() {
        println!("SKIP: lgcm-basic post not found (run build first)");
        return;
    }

    let html = fs::read_to_string(html_path).expect("Failed to read lgcm-basic post");

    // Data Access section exists
    assert!(
        html.contains(r#"data-testid="section:data-access""#),
        "Expected Data Access section"
    );

    println!("✅ Data Access section present");
}
