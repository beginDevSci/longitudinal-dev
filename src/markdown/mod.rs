//! Markdown transformation pipeline.
//!
//! This module provides AST-based transformations for markdown content,
//! operating on pulldown-cmark events before HTML rendering.
//!
//! ## Transformations
//!
//! - **Callouts**: Convert blockquotes with `[!type]` or `**Type:**` markers
//! - **Tables**: Wrap tables for responsive scrolling
//! - **Modules**: Wrap specific H2 sections in collapsible `<details>` elements
//! - **Math**: Render LaTeX math via KaTeX (server-side)
//! - **Code Blocks**: Add unique IDs for copy button support
//! - **Headings**: Add IDs to H2/H3 for anchor navigation
//!
//! ## Usage
//!
//! ```ignore
//! use crate::markdown::transform_markdown_events;
//! use pulldown_cmark::{Parser, Options, Event};
//!
//! let parser = Parser::new_ext(content, options);
//! let events: Vec<Event> = parser.collect();
//! let transformed = transform_markdown_events(events);
//! ```

mod callouts;
mod code_blocks;
mod headings;
mod math;
mod modules;
mod tables;
mod transform;

pub use headings::HeadingResult;
pub use math::preprocess_inline_math;
pub use transform::{transform_markdown_events, transform_markdown_events_with_outline, TransformResult};
