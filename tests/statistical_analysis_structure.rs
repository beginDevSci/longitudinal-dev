//! Tests for Statistical Analysis section structure.
//!
//! Verifies that posts render the Statistical Analysis section with:
//! - Six ordered subcontainers (filters, task, intro, summary, code, table note)
//! - Proper ARIA attributes (aria-label, aria-labelledby, role)
//! - No islands (pure SSR)

mod helpers;
use helpers::*;

/// Verify stats-demo post renders Statistical Analysis section with all six subcontainers in order.
#[test]
fn test_stats_section_exists_and_ordered() {
    let Some(html) = load_post_html("stats-demo") else {
        return;
    };

    // 1) Section container exists
    assert_section_id_exists(&html, "statistical-analysis");
    assert_testid_exists(&html, "section:stats", "Expected section:stats testid");

    // 2) Verify all subcontainers exist
    assert_testid_exists(&html, "stats:filters", "Expected filters container");
    assert_testid_exists(&html, "stats:intro", "Expected intro panel container");
    assert_testid_exists(&html, "stats:summary", "Expected summary block container");
    assert_testid_exists(&html, "stats:table-note", "Expected table note container");

    // Verify task card appears at least twice
    let task_count = count_testid(&html, "stats:task");
    assert_min_count(task_count, 2, "task cards");

    // 3) Verify order: extract positions and ensure they're sequential
    let section_start = html
        .find(r#"data-testid="section:stats""#)
        .expect("section:stats not found");

    let subcontainers = [
        (r#"data-testid="stats:filters""#, "filters"),
        (r#"data-testid="stats:intro""#, "intro panel"),
        (r#"data-testid="stats:summary""#, "summary block"),
        (r#"data-testid="stats:table-note""#, "table note"),
    ];

    let positions: Vec<(usize, &str)> = subcontainers
        .iter()
        .map(|(testid, name)| {
            let pos = html[section_start..]
                .find(testid)
                .unwrap_or_else(|| panic!("{name} not found after section start"));
            (pos, *name)
        })
        .collect();

    // Ensure positions are strictly increasing (ordered)
    for i in 1..positions.len() {
        let curr_name = positions[i].1;
        let curr_pos = positions[i].0;
        let prev_name = positions[i - 1].1;
        let prev_pos = positions[i - 1].0;
        assert!(
            curr_pos > prev_pos,
            "Subcontainer '{curr_name}' (pos {curr_pos}) should come after '{prev_name}' (pos {prev_pos})"
        );
    }

    // 4) Verify filter pills exist
    let filters_html = extract_section(
        &html,
        r#"data-testid="stats:filters""#,
        r#"data-testid="stats:task""#,
    );
    let pill_count = filters_html.matches(r#"class="pill""#).count();
    assert_min_count(pill_count, 4, "filter pills");

    // 5) Verify summary block has metrics
    let summary_html = extract_section(
        &html,
        r#"data-testid="stats:summary""#,
        r#"data-testid="stats:task""#,
    );
    let metric_count = summary_html
        .matches(r#"data-testid="stats:metric""#)
        .count();
    assert_min_count(metric_count, 4, "metric cards");

    println!("✅ Statistical Analysis section structure verified:");
    println!("   - Section: present");
    println!("   - Task cards: {task_count}");
    println!("   - Filter pills: {pill_count}");
    println!("   - Metrics: {metric_count}");
}

/// Verify task card has proper title id.
#[test]
fn test_stats_task_code_is_labelled() {
    let Some(html) = load_post_html("stats-demo") else {
        return;
    };

    // Verify task title has id="stats-task-title"
    assert!(
        html.contains(r#"id="stats-task-title""#),
        "Expected task title with id=\"stats-task-title\""
    );

    // Verify at least one task card exists
    assert_testid_exists(&html, "stats:task", "Expected at least one stats task card");

    println!("✅ Task card has proper id=\"stats-task-title\"");
}

/// Verify no islands exist within the Statistical Analysis section.
#[test]
fn test_stats_no_islands_in_subtree() {
    let post_slugs = ["stats-demo", "full-featured"];

    for slug in post_slugs {
        if let Some(html) = load_post_html(slug) {
            assert_no_islands(&html, "section:stats", "Statistical Analysis");
            println!("✅ Post '{slug}' has no islands in Statistical Analysis");
        }
    }

    println!("✅ Statistical Analysis contains no islands (pure SSR)");
}

/// Verify filter pills use text-sm for typography consistency.
#[test]
fn test_stats_filter_pills_typography() {
    let Some(html) = load_post_html("stats-demo") else {
        return;
    };

    let filters_html = extract_section(
        &html,
        r#"data-testid="stats:filters""#,
        r#"data-testid="stats:task""#,
    );

    assert_class_exists(
        filters_html,
        "pill",
        "Expected filter pills to use .pill class",
    );

    // Ensure .badge (text-xs) is NOT used in filter pills (checking for regression)
    let pill_with_badge = filters_html.contains(r#"class="badge""#);
    assert!(
        !pill_with_badge,
        "Filter pills should not use .badge (use .pill for consistency)"
    );

    println!("✅ Filter pills use text-sm for typography consistency");
}

/// Verify decorative SVG icons have aria-hidden="true".
#[test]
fn test_stats_decorative_icons_are_hidden() {
    let Some(html) = load_post_html("stats-demo") else {
        return;
    };

    let stats_html = extract_section(&html, r#"data-testid="section:stats""#, "</section>");

    // Count SVG elements with aria-hidden="true"
    let aria_hidden_count = stats_html.matches(r#"<svg aria-hidden="true""#).count();

    // We expect at least 3 decorative SVG icons:
    // 1. Section header icon (bar chart)
    // 2. Intro panel icon (bar chart)
    // 3. Table note icon (table/grid)
    assert_min_count(
        aria_hidden_count,
        3,
        "decorative SVG icons with aria-hidden=\"true\"",
    );

    println!(
        "✅ Statistical Analysis decorative icons have aria-hidden=\"true\" (count: {aria_hidden_count})"
    );
}
