/// Structure integrity test
///
/// Asserts that the generated HTML maintains the fixed blog structure:
/// - Exactly 1 <h1> element (post title)
/// - Exactly 6 <section> elements (fixed sections)
use std::fs;
use std::path::Path;

#[test]
fn test_html_structure_integrity() {
    // NOTE: This test checks the landing page (index.html)
    // SSG output goes to dist/, not target/site/
    let html_path = Path::new("dist/index.html");

    // Skip test if HTML not yet generated (pre-build)
    if !html_path.exists() {
        println!("SKIP: dist/index.html not found (run `make ssg` first)");
        return;
    }

    let html = fs::read_to_string(html_path).expect("Failed to read generated HTML");

    // Landing page should have 1 h1 (site title)
    let h1_count = html.matches("<h1").count();
    assert_eq!(
        h1_count, 1,
        "Expected exactly 1 <h1> tag (site title), found {h1_count}"
    );

    // Verify the page contains links to ABCD analyses section
    assert!(html.contains("/abcd-analyses/"), "Missing ABCD analyses section link");

    println!("✅ Landing page structure verified:");
    println!("   - <h1> tags: {h1_count}");
    println!("   - ABCD analyses section link: present");
}

#[test]
fn test_islands_hydration_wiring() {
    let html_path = Path::new("dist/index.html");

    // Skip if not built
    if !html_path.exists() {
        println!("SKIP: dist/index.html not found (run `make ssg` first)");
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

/// Multi-tutorial structure test
///
/// Verifies that each generated tutorial maintains the strict 1+6 structure
#[test]
fn test_each_tutorial_has_fixed_structure() {
    let site_root = Path::new("dist");
    // Use actual tutorial slugs
    let tutorial_slugs = ["lgcm-basic", "lmm-random-slopes", "mlgcm"];

    for slug in tutorial_slugs {
        let tutorial_path = site_root.join("tutorials").join(slug).join("index.html");

        // Skip if not built yet
        if !tutorial_path.exists() {
            println!("SKIP: {} not found (run `make ssg` first)", tutorial_path.display());
            continue;
        }

        let html = fs::read_to_string(&tutorial_path)
            .unwrap_or_else(|_| panic!("Failed to read: {}", tutorial_path.display()));

        // Count h1 tags
        let h1_count = html.matches("<h1").count();
        assert_eq!(
            h1_count, 1,
            "Tutorial '{slug}' should have exactly 1 <h1>, found {h1_count}"
        );

        // Count section tags
        let section_count = html.matches("<section").count();

        // All 6 sections are required (Overview, Data Access, Data Preparation,
        // Statistical Analysis, Discussion, Additional Resources)
        let expected_sections = 6;

        assert_eq!(
            section_count, expected_sections,
            "Tutorial '{slug}' should have exactly {expected_sections} <section> tags (6 required sections), found {section_count}"
        );

        println!("✅ Tutorial '{slug}' structure verified (1 h1, {section_count} sections)");
    }
}

/// Data Access section test
///
/// Verifies that tutorials have the Data Access section
#[test]
fn test_data_access_section_present() {
    let html_path = Path::new("dist/abcd-analyses/lgcm/lgcm-basic/index.html");

    if !html_path.exists() {
        println!("SKIP: lgcm-basic tutorial not found (run `make ssg` first)");
        return;
    }

    let html = fs::read_to_string(html_path).expect("Failed to read lgcm-basic tutorial");

    // Data Access section exists
    assert!(
        html.contains(r#"data-testid="section:data-access""#),
        "Expected Data Access section"
    );

    println!("✅ Data Access section present");
}
