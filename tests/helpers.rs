//! Shared test helpers for HTML structure assertions.
//!
//! This module provides reusable functions to reduce duplication across
//! structure tests while maintaining clear, readable test assertions.

use std::fs;
use std::path::Path;

/// Load HTML from generated static site.
/// Returns None if file doesn't exist (allowing tests to skip gracefully).
pub fn load_post_html(slug: &str) -> Option<String> {
    let html_path = Path::new("target/site")
        .join("posts")
        .join(slug)
        .join("index.html");

    if !html_path.exists() {
        eprintln!("SKIP: {slug} build not found. Run SSG build first.");
        return None;
    }

    fs::read_to_string(&html_path).ok()
}

/// Assert that HTML contains a specific test ID.
pub fn assert_testid_exists(html: &str, testid: &str, description: &str) {
    assert!(
        html.contains(&format!(r#"data-testid="{testid}""#)),
        "{description}"
    );
}

/// Assert that HTML contains a section ID.
pub fn assert_section_id_exists(html: &str, section_id: &str) {
    assert!(
        html.contains(&format!(r#"id="{section_id}""#)),
        "Expected section id=\"{section_id}\""
    );
}

/// Assert that HTML contains specific CSS class.
pub fn assert_class_exists(html: &str, class: &str, description: &str) {
    assert!(
        html.contains(class),
        "{description}: Expected class '{class}'"
    );
}

/// Count occurrences of a test ID in HTML.
#[allow(dead_code)]
pub fn count_testid(html: &str, testid: &str) -> usize {
    html.matches(&format!(r#"data-testid="{testid}""#)).count()
}

/// Extract a section of HTML between two markers.
/// Returns the substring from start_marker to end_marker (or end of string).
pub fn extract_section<'a>(html: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = html
        .find(start_marker)
        .unwrap_or_else(|| panic!("Start marker '{start_marker}' not found"));

    let remaining = &html[start..];
    let end = remaining.find(end_marker).unwrap_or(remaining.len());

    &remaining[..end]
}

/// Assert that a section contains no Leptos island markers (pure SSR).
pub fn assert_no_islands(html: &str, section_testid: &str, section_name: &str) {
    if let Some(section_start) = html.find(&format!(r#"data-testid="{section_testid}""#)) {
        let section_html = extract_section(&html[section_start..], "", "</section>");

        assert!(
            !section_html.contains("leptos-island"),
            "{section_name} section contains island markers (should be pure SSR)"
        );
        assert!(
            !section_html.contains("data-leptos-island"),
            "{section_name} section contains data-leptos-island attribute"
        );
    }
}

/// Assert minimum count with helpful error message.
pub fn assert_min_count(actual: usize, expected: usize, item_name: &str) {
    assert!(
        actual >= expected,
        "Expected at least {expected} {item_name}, found {actual}"
    );
}

/// Assert exact count with helpful error message.
#[allow(dead_code)]
pub fn assert_exact_count(actual: usize, expected: usize, item_name: &str) {
    assert_eq!(
        actual, expected,
        "Expected exactly {expected} {item_name}, found {actual}"
    );
}

/// Assert that an element with a test ID has a specific aria-label.
#[allow(dead_code)]
pub fn assert_aria_label(html: &str, testid: &str, expected_label: &str) {
    let marker = format!(r#"data-testid="{testid}""#);
    let marker_pos = html
        .find(&marker)
        .unwrap_or_else(|| panic!("Test ID '{testid}' not found"));

    // Find opening tag
    let tag_start = html[..marker_pos]
        .rfind('<')
        .expect("Opening tag not found");
    let tag_end = html[marker_pos..]
        .find('>')
        .expect("Closing bracket not found");
    let opening_tag = &html[tag_start..marker_pos + tag_end + 1];

    assert!(
        opening_tag.contains(&format!(r#"aria-label="{expected_label}""#)),
        "Expected aria-label=\"{expected_label}\" on element with testid=\"{testid}\". Found: {opening_tag}"
    );
}

/// Assert that aria-labelledby exists and references a valid ID.
#[allow(dead_code)]
pub fn assert_aria_labelledby(html: &str, target_id: &str) {
    let aria_attr = format!(r#"aria-labelledby="{target_id}""#);
    let id_attr = format!(r#"id="{target_id}""#);

    assert!(
        html.contains(&id_attr),
        "Expected element with id=\"{target_id}\""
    );
    assert!(
        html.contains(&aria_attr),
        "Expected aria-labelledby=\"{target_id}\""
    );
}

/// Test helper for verifying multiple posts have basic section structure.
#[allow(dead_code)]
pub fn assert_section_in_posts(slugs: &[&str], testid: &str, section_name: &str) {
    for slug in slugs {
        if let Some(html) = load_post_html(slug) {
            assert_testid_exists(
                &html,
                testid,
                &format!("Post '{slug}' missing {section_name} section"),
            );
        }
    }
}
