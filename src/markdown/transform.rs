//! Main transformation pipeline for markdown events.
//!
//! Orchestrates all AST-based transformations in the correct order.

use crate::models::guide::OutlineNode;
use pulldown_cmark::Event;

use super::callouts;
use super::code_blocks;
use super::headings;
use super::math;
use super::modules;
use super::tables;

/// Result of the markdown transformation pipeline.
pub struct TransformResult {
    pub events: Vec<Event<'static>>,
    pub outline: Vec<OutlineNode>,
}

/// Transform a stream of pulldown-cmark events.
///
/// Applies transformations in order:
/// 1. Headings - Add IDs to H2/H3/H4 for anchor navigation
/// 2. Callouts - Convert blockquotes to styled callout boxes
/// 3. Tables - Wrap tables for responsive scrolling
/// 4. Modules - Wrap specific H2 sections in collapsible details
/// 5. Math - Render LaTeX expressions via KaTeX
/// 6. Code blocks - Add unique IDs for copy button support
///
/// Each transformation operates on the full event stream and returns
/// a new event stream, allowing clean composition.
pub fn transform_markdown_events(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let events = headings::add_heading_ids(events);
    let events = callouts::transform_callouts(events);
    let events = tables::wrap_tables(events);
    let events = modules::wrap_modules(events);
    let events = math::render_math(events);
    code_blocks::add_code_block_ids(events)
}

/// Transform markdown events and also extract a hierarchical outline.
///
/// Same as `transform_markdown_events`, but also returns the outline
/// extracted from H2/H3/H4 headings for sidebar navigation.
pub fn transform_markdown_events_with_outline(events: Vec<Event<'_>>) -> TransformResult {
    let heading_result = headings::add_heading_ids_with_outline(events);
    let events = callouts::transform_callouts(heading_result.events);
    let events = tables::wrap_tables(events);
    let events = modules::wrap_modules(events);
    let events = math::render_math(events);
    let events = code_blocks::add_code_block_ids(events);

    TransformResult {
        events,
        outline: heading_result.outline,
    }
}
