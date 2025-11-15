//! Tests for Additional Resources section structure (prototype-aligned).
//!
//! Verifies that posts render the Additional Resources section with:
//! - Resource intro cards (resources:list and resources:item)
//! - Uppercase right badge with tracking-widest
//! - Semantic card component class
//! - No islands (pure SSR)
//!
//! Note: Additional Resources section is REQUIRED for all posts.

mod helpers;
use helpers::*;

/// Verify hello-world post renders Additional Resources section with correct structure.
#[test]
fn test_resources_section_structure_hello_world() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    // 1) Section container exists
    assert_section_id_exists(&html, "additional-resources");
    assert_testid_exists(
        &html,
        "section:resources",
        "Expected section:resources testid",
    );

    // 2) Resources list container exists
    assert_testid_exists(&html, "resources:list", "Expected resources:list container");

    // 3) Count resource items
    let item_count = count_testid(&html, "resources:item");
    assert_min_count(item_count, 1, "resource items");

    // 4) Check for semantic card class and badge classes
    let resources_html = extract_section(&html, r#"data-testid="section:resources""#, "</section>");
    assert_class_exists(
        resources_html,
        "card",
        "Expected semantic card class on resource cards",
    );
    assert_class_exists(
        resources_html,
        "tracking-widest",
        "Expected tracking-widest class on badge",
    );
    assert_class_exists(
        resources_html,
        "uppercase",
        "Expected uppercase class on badge",
    );

    println!("✅ Additional Resources structure verified:");
    println!("   - Section: present");
    println!("   - Resource items: {item_count}");
}

/// Verify css-only-demo post also renders Additional Resources section.
#[test]
fn test_resources_section_structure_css_only_demo() {
    let Some(html) = load_post_html("css-only-demo") else {
        return;
    };

    assert_testid_exists(
        &html,
        "section:resources",
        "Expected section:resources testid",
    );
    assert_testid_exists(&html, "resources:list", "Expected resources:list container");

    let item_count = count_testid(&html, "resources:item");
    assert_min_count(item_count, 1, "resource items");

    println!("✅ css-only-demo Additional Resources structure verified");
}

/// Verify plain-structure post renders Additional Resources section (required).
#[test]
fn test_resources_section_required_plain_structure() {
    let Some(html) = load_post_html("plain-structure") else {
        return;
    };

    assert_testid_exists(
        &html,
        "section:resources",
        "plain-structure should have Additional Resources section (required)",
    );

    println!("✅ plain-structure correctly includes Additional Resources section (required)");
}

/// Verify no islands exist within the Additional Resources section.
#[test]
fn test_resources_no_islands() {
    let post_slugs = ["hello-world", "css-only-demo"];

    for slug in post_slugs {
        if let Some(html) = load_post_html(slug) {
            assert_no_islands(&html, "section:resources", "Additional Resources");
        }
    }

    println!("✅ Additional Resources contains no islands (pure SSR)");
}

/// Verify ResourceIntroCard has proper article semantics.
#[test]
fn test_resources_card_semantics() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    let resources_html = extract_section(&html, r#"data-testid="section:resources""#, "</section>");

    // Count resource items and article elements
    let item_count = resources_html
        .matches(r#"data-testid="resources:item""#)
        .count();
    let article_count = resources_html.matches("<article").count();

    assert_exact_count(
        article_count,
        item_count,
        "<article> elements matching resource items",
    );

    println!("✅ Resource cards use proper <article> semantics: {item_count} cards");
}

/// Verify badge has correct font-mono styling.
#[test]
fn test_resources_badge_styling() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    let resources_html = extract_section(&html, r#"data-testid="section:resources""#, "</section>");

    assert_class_exists(
        resources_html,
        "font-mono",
        "Expected font-mono class on badge",
    );
    assert_class_exists(
        resources_html,
        "tracking-widest",
        "Expected tracking-widest class on badge",
    );
    assert_class_exists(
        resources_html,
        "uppercase",
        "Expected uppercase class on badge",
    );

    println!("✅ Resource badge styling verified");
}

/// Verify Additional Resources section has proper aria-labelledby.
#[test]
fn test_resources_aria_labelledby() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    assert_aria_labelledby(&html, "resources-title");

    println!("✅ Additional Resources has aria-labelledby=\"resources-title\"");
}
