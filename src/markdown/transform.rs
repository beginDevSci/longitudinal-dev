//! Main transformation pipeline for markdown events.
//!
//! Orchestrates all AST-based transformations in the correct order.

use pulldown_cmark::Event;

use super::callouts;
use super::code_blocks;
use super::math;
use super::modules;
use super::tables;

/// Transform a stream of pulldown-cmark events.
///
/// Applies transformations in order:
/// 1. Callouts - Convert blockquotes to styled callout boxes
/// 2. Tables - Wrap tables for responsive scrolling
/// 3. Modules - Wrap specific H2 sections in collapsible details
/// 4. Math - Render LaTeX expressions via KaTeX
/// 5. Code blocks - Add unique IDs for copy button support
///
/// Each transformation operates on the full event stream and returns
/// a new event stream, allowing clean composition.
pub fn transform_markdown_events(events: Vec<Event<'_>>) -> Vec<Event<'static>> {
    let events = callouts::transform_callouts(events);
    let events = tables::wrap_tables(events);
    let events = modules::wrap_modules(events);
    let events = math::render_math(events);
    let events = code_blocks::add_code_block_ids(events);
    events
}
