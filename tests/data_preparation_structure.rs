//! Tests for Data Preparation section structure (prototype-aligned).
//!
//! Verifies that posts render the Data Preparation section with:
//! - Step pills row (4 numbered pills)
//! - Task cards stack (3-4 cards with title, filename, code, actions)
//! - No islands (pure SSR)

mod helpers;
use helpers::*;

/// Verify hello-world post renders Data Preparation section with correct structure.
#[test]
fn test_prep_section_structure_hello_world() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    // 1) Section container exists
    assert_section_id_exists(&html, "data-preparation");
    assert_testid_exists(
        &html,
        "section:data-prep",
        "Expected section:data-prep testid",
    );

    // 2) Step pills container exists
    assert_testid_exists(&html, "prep:steps", "Expected prep:steps container");

    // 3) Count step pills (now simplified without numbered circles)
    let steps_html = extract_section(
        &html,
        r#"data-testid="prep:steps""#,
        r#"data-testid="prep:tasks""#,
    );
    let pill_count = steps_html.matches(r#"class="pill""#).count();
    assert_min_count(pill_count, 4, "step pill elements (4 pills)");

    // 4) Tasks container exists
    assert_testid_exists(&html, "prep:tasks", "Expected prep:tasks container");

    // 5) Count task cards
    let task_count = count_testid(&html, "prep:task");
    assert_min_count(task_count, 2, "task cards");

    // 6) Code blocks exist within task cards
    let tasks_html = extract_section(&html, r#"data-testid="prep:tasks""#, "</section>");
    let code_count = tasks_html.matches("<pre").count();
    assert_min_count(code_count, 2, "code blocks");

    // 7) Check for uppercase tracking-widest filename style
    assert_class_exists(
        &html,
        "tracking-widest",
        "Expected tracking-widest class for filename",
    );
    assert_class_exists(&html, "uppercase", "Expected uppercase class for filename");

    println!("✅ Data Preparation structure verified:");
    println!("   - Section: present");
    println!("   - Step pills: {pill_count}");
    println!("   - Task cards: {task_count}");
    println!("   - Code blocks: {code_count}");
}

/// Verify css-only-demo post also renders Data Preparation section.
#[test]
fn test_prep_section_structure_css_only_demo() {
    let Some(html) = load_post_html("css-only-demo") else {
        return;
    };

    // Basic structure checks
    assert_testid_exists(
        &html,
        "section:data-prep",
        "Expected section:data-prep testid",
    );
    assert_testid_exists(&html, "prep:steps", "Expected prep:steps container");
    assert_testid_exists(&html, "prep:tasks", "Expected prep:tasks container");

    let task_count = count_testid(&html, "prep:task");
    assert_min_count(task_count, 3, "task cards");

    println!("✅ css-only-demo Data Preparation structure verified");
}

/// Verify plain-structure post also renders Data Preparation section.
#[test]
fn test_prep_section_structure_plain_structure() {
    let Some(html) = load_post_html("plain-structure") else {
        return;
    };

    // Basic structure checks
    assert_testid_exists(
        &html,
        "section:data-prep",
        "Expected section:data-prep testid",
    );
    assert_testid_exists(&html, "prep:steps", "Expected prep:steps container");
    assert_testid_exists(&html, "prep:tasks", "Expected prep:tasks container");

    println!("✅ plain-structure Data Preparation structure verified");
}

/// Verify no islands exist within the Data Preparation section.
#[test]
fn test_prep_no_islands() {
    let post_slugs = ["hello-world", "plain-structure", "css-only-demo"];

    for slug in post_slugs {
        if let Some(html) = load_post_html(slug) {
            assert_no_islands(&html, "section:data-prep", "Data Preparation");
        }
    }

    println!("✅ Data Preparation contains no islands (pure SSR)");
}

/// Verify Data Preparation section has correct semantic card styling.
#[test]
fn test_prep_task_card_styling() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    assert_class_exists(&html, "card", "Expected semantic card class on task cards");
    assert_class_exists(
        &html,
        "panel",
        "Expected semantic panel class on containers",
    );

    println!("✅ Data Preparation task card styling verified");
}

/// Verify prep:steps container has aria-label for screen readers.
#[test]
fn test_prep_aria_label_on_steps() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    assert_aria_label(&html, "prep:steps", "Preparation steps");

    println!("✅ prep:steps has aria-label=\"Preparation steps\"");
}

/// Verify each prep:task code container has aria-labelledby referencing its title.
#[test]
fn test_prep_aria_labelledby_on_code() {
    let Some(html) = load_post_html("hello-world") else {
        return;
    };

    let task_count = count_testid(&html, "prep:task");
    assert_min_count(task_count, 2, "task cards");

    // For each task, verify title has id and code container has aria-labelledby
    for i in 0..task_count {
        let title_id = format!("prep-task-title-{i}");
        assert_aria_labelledby(&html, &title_id);
    }

    println!("✅ All {task_count} task cards have aria-labelledby linking title to code container");
}
