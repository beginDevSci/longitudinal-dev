//! Tests for Discussion section structure (prototype-aligned).
//!
//! Verifies that posts render the Discussion section with:
//! - Narrative intro paragraphs (discussion:narrative)
//! - Two side-by-side NotePanel components (insights and limitations)
//! - Proper ARIA attributes (role="note")
//! - No islands (pure SSR)
//!
//! Note: Discussion section is REQUIRED for all posts.

mod helpers;
use helpers::*;

/// Verify hello-world post renders Discussion section with correct structure.
#[test]
fn test_discussion_section_structure_hello_world() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    // 1) Section container exists
    assert_section_id_exists(&html, "discussion");
    assert_testid_exists(
        &html,
        "section:discussion",
        "Expected section:discussion testid",
    );

    // 2) Narrative container exists
    assert_testid_exists(
        &html,
        "discussion:narrative",
        "Expected discussion:narrative container",
    );

    // 3) Insights panel exists
    assert_testid_exists(
        &html,
        "discussion:insights",
        "Expected discussion:insights container",
    );

    // 4) Limitations panel exists
    assert_testid_exists(
        &html,
        "discussion:limitations",
        "Expected discussion:limitations container",
    );

    // 5) Both panels have role="note"
    let discussion_html =
        extract_section(&html, r#"data-testid="section:discussion""#, "</section>");
    let note_count = discussion_html.matches(r#"role="note""#).count();
    assert_exact_count(note_count, 2, "role=\"note\" panels");

    // 6) Check for semantic card class on panels
    assert_class_exists(
        discussion_html,
        "card",
        "Expected semantic card class on note panels",
    );

    println!("✅ Discussion structure verified:");
    println!("   - Section: present");
    println!("   - Narrative: present");
    println!("   - Note panels: {note_count}");
}

/// Verify css-only-demo post also renders Discussion section.
#[test]
fn test_discussion_section_structure_css_only_demo() {
    let Some(html) = load_post_html("css-only-demo") else {
        return;
    };

    assert_testid_exists(
        &html,
        "section:discussion",
        "Expected section:discussion testid",
    );
    assert_testid_exists(
        &html,
        "discussion:narrative",
        "Expected discussion:narrative container",
    );
    assert_testid_exists(
        &html,
        "discussion:insights",
        "Expected discussion:insights container",
    );
    assert_testid_exists(
        &html,
        "discussion:limitations",
        "Expected discussion:limitations container",
    );

    println!("✅ css-only-demo Discussion structure verified");
}

/// Verify plain-structure post renders Discussion section (required).
#[test]
fn test_discussion_section_required_plain_structure() {
    let Some(html) = load_post_html("plain-structure") else {
        return;
    };

    assert_testid_exists(
        &html,
        "section:discussion",
        "plain-structure should have Discussion section (required)",
    );

    println!("✅ plain-structure correctly includes Discussion section (required)");
}

/// Verify no islands exist within the Discussion section.
#[test]
fn test_discussion_no_islands() {
    let post_slugs = ["hello-world", "css-only-demo"];

    for slug in post_slugs {
        if let Some(html) = load_post_html(slug) {
            assert_no_islands(&html, "section:discussion", "Discussion");
        }
    }

    println!("✅ Discussion contains no islands (pure SSR)");
}

/// Verify Discussion section has correct grid layout for panels.
#[test]
fn test_discussion_grid_layout() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    assert_class_exists(
        &html,
        "md:grid-cols-2",
        "Expected md:grid-cols-2 class for panel layout",
    );

    println!("✅ Discussion grid layout verified");
}

/// Verify NotePanel bullets render as list items.
#[test]
fn test_discussion_note_panel_bullets() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    let discussion_html =
        extract_section(&html, r#"data-testid="section:discussion""#, "</section>");

    assert_class_exists(
        discussion_html,
        "list-disc",
        "Expected list-disc class for note panel bullets",
    );

    // Count <li> elements (hello-world has 2 insights + 2 limitations = 4)
    let li_count = discussion_html.matches("<li>").count();
    assert_min_count(li_count, 4, "<li> elements");

    println!("✅ Discussion note panel bullets verified: {li_count} items");
}

/// Verify Discussion section has proper aria-labelledby.
#[test]
fn test_discussion_aria_labelledby() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    assert_aria_labelledby(&html, "discussion-title");

    println!("✅ Discussion has aria-labelledby=\"discussion-title\"");
}
