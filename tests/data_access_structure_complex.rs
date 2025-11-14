//! Tests for Data Access Composite structure (prototype-aligned).
//!
//! Verifies that posts using the Composite model render:
//! - Data Requirements table with 4 columns
//! - Data Access Methods panel with options
//! - Optional featured option card
//! - No islands (pure SSR)

use std::fs;
use std::path::Path;

/// Verify hello-world post renders both Data Requirements and Data Access Methods subcontainers.
#[test]
fn test_composite_hello_world() {
    let html_path = Path::new("target/site/posts/hello-world/index.html");

    if !html_path.exists() {
        eprintln!("SKIP: hello-world build not found. Run SSG build first.");
        return;
    }

    let html = fs::read_to_string(html_path).expect("read hello-world");

    // Section container exists
    assert!(
        html.contains(r#"data-testid="section:data-access""#),
        "Data Access section container missing"
    );

    // Data Requirements subcontainer exists
    assert!(
        html.contains(r#"data-testid="da:reqs""#),
        "Data Requirements subcontainer missing"
    );

    // Table has 4 header columns
    assert!(
        html.contains("Component")
            && html.contains("ABCD Variable")
            && html.contains("Description")
            && html.contains("Assessment Points"),
        "Data Requirements table missing expected 4 columns"
    );

    // At least one table row exists (check for <tbody> content)
    assert!(
        html.contains("<tbody>") && html.contains("</tbody>"),
        "Data Requirements table <tbody> missing"
    );

    // Data Access Methods subcontainer exists
    assert!(
        html.contains(r#"data-testid="da:methods""#),
        "Data Access Methods subcontainer missing"
    );

    // Options rendered (3 expected: CLI, API, Files)
    assert!(
        html.contains(r#"data-testid="da:tabs""#),
        "Data Access Methods options container missing"
    );

    // No islands in Data Access section
    assert!(
        !html.contains(r#"data-component="Counter""#),
        "Data Access section should not contain islands"
    );
}

/// Verify css-only-demo post also uses Composite model.
#[test]
fn test_composite_css_only_demo() {
    let html_path = Path::new("target/site/posts/css-only-demo/index.html");

    if !html_path.exists() {
        eprintln!("SKIP: css-only-demo build not found. Run SSG build first.");
        return;
    }

    let html = fs::read_to_string(html_path).expect("read css-only-demo");

    // Both subcontainers present
    assert!(
        html.contains(r#"data-testid="da:reqs""#),
        "Data Requirements subcontainer missing in css-only-demo"
    );
    assert!(
        html.contains(r#"data-testid="da:methods""#),
        "Data Access Methods subcontainer missing in css-only-demo"
    );
}

/// Verify plain-structure post uses Composite model (as of v1.1.7).
#[test]
fn test_composite_plain_structure() {
    let html_path = Path::new("target/site/posts/plain-structure/index.html");

    if !html_path.exists() {
        eprintln!("SKIP: plain-structure build not found. Run SSG build first.");
        return;
    }

    let html = fs::read_to_string(html_path).expect("read plain-structure");

    // Should have composite subcontainers (as of v1.1.7, all posts use Composite)
    assert!(
        html.contains(r#"data-testid="da:reqs""#),
        "plain-structure should have Data Requirements subcontainer"
    );
    assert!(
        html.contains(r#"data-testid="da:methods""#),
        "plain-structure should have Data Access Methods subcontainer"
    );
}

/// Verify that Data Access section has standard testids.
#[test]
fn test_data_access_testids() {
    let post_slugs = ["hello-world", "plain-structure", "css-only-demo"];

    for slug in post_slugs {
        let html_path = Path::new("target/site")
            .join("posts")
            .join(slug)
            .join("index.html");

        if !html_path.exists() {
            eprintln!("SKIP: {slug} build not found. Run SSG build first.");
            continue;
        }

        let html = fs::read_to_string(&html_path)
            .unwrap_or_else(|_| panic!("Failed to read: {}", html_path.display()));

        // Verify standard testids exist
        assert!(
            html.contains(r#"data-testid="da:reqs""#),
            "Post '{slug}' missing da:reqs testid"
        );
        assert!(
            html.contains(r#"data-testid="da:methods""#),
            "Post '{slug}' missing da:methods testid"
        );
        assert!(
            html.contains(r#"data-testid="da:tabs""#),
            "Post '{slug}' missing da:tabs testid"
        );
    }
}

/// Verify Data Requirements table has exactly 4 <th> cells.
#[test]
fn test_data_requirements_table_structure() {
    let html_path = Path::new("target/site/posts/hello-world/index.html");

    if !html_path.exists() {
        eprintln!("SKIP: hello-world build not found. Run SSG build first.");
        return;
    }

    let html = fs::read_to_string(html_path).expect("read hello-world");

    // Extract table section (da:reqs container)
    let reqs_start = html
        .find(r#"data-testid="da:reqs""#)
        .expect("da:reqs not found");
    let reqs_section = &html[reqs_start..];

    // Count <th> elements within thead (not <thead itself)
    let thead_section = reqs_section.split("</thead>").next().unwrap_or("");

    // Count only <th scope="col" to avoid counting <thead>
    let th_count = thead_section.matches(r#"<th scope="col""#).count();

    assert_eq!(
        th_count, 4,
        "Data Requirements table should have exactly 4 <th> cells (found {th_count})"
    );
}

/// Verify no islands exist within the Data Access section.
#[test]
fn test_data_access_no_islands() {
    let post_slugs = ["hello-world", "plain-structure", "css-only-demo"];

    for slug in post_slugs {
        let html_path = Path::new("target/site")
            .join("posts")
            .join(slug)
            .join("index.html");

        if !html_path.exists() {
            eprintln!("SKIP: {slug} build not found. Run SSG build first.");
            continue;
        }

        let html = fs::read_to_string(&html_path)
            .unwrap_or_else(|_| panic!("Failed to read: {}", html_path.display()));

        // Extract Data Access section
        if let Some(da_start) = html.find(r#"data-testid="section:data-access""#) {
            // Find the end of the section (next section or end of document)
            let da_section = &html[da_start..];
            let da_end = da_section
                .find("</section>")
                .map(|pos| da_start + pos)
                .unwrap_or(html.len());
            let da_html = &html[da_start..da_end];

            // No island markers should exist
            assert!(
                !da_html.contains("leptos-island"),
                "Post '{slug}' has islands in Data Access section"
            );
        }
    }
}
