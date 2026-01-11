//! Pure interaction state types and logic for brain surface visualization.
//!
//! This crate provides UI-agnostic interaction state management with no GPU,
//! rendering, or UI framework dependencies. It can be used with any renderer
//! backend and UI framework.
//!
//! # Core Types
//!
//! ## Selection
//!
//! [`Selection`] tracks which vertices and regions are currently selected.
//! It supports both single-select and multi-select modes, with a primary
//! vertex for highlighting.
//!
//! ```
//! use interaction::Selection;
//!
//! let mut selection = Selection::new();
//!
//! // Single-select mode: click replaces selection
//! selection.set_single(0, 42);  // surface 0, vertex 42
//!
//! // Multi-select mode: shift+click adds to selection
//! selection.add_vertex(0, 100);
//! selection.add_vertex(1, 50);  // different surface
//!
//! // Toggle selection (for ctrl+click behavior)
//! selection.toggle_vertex(0, 42);  // removes vertex 42
//! ```
//!
//! ## Hover State
//!
//! [`HoverState`] tracks what's under the cursor with built-in support for
//! throttling and decay. This is designed to work with GPU picking systems.
//!
//! ```
//! use interaction::HoverState;
//!
//! let mut hover = HoverState::default();
//!
//! // Update on pick hit
//! hover.set_hit(vertex_id, surface_id, region_name, current_time_ms);
//!
//! // Check for decay (clear hover after timeout with no hits)
//! if hover.should_decay(current_time_ms, decay_ms) {
//!     hover.clear();
//! }
//! ```
//!
//! ## History
//!
//! [`History`] provides undo/redo functionality for any serializable state.
//! It maintains a stack of snapshots with configurable maximum size.
//!
//! ```
//! use interaction::History;
//!
//! let mut history: History<MyState> = History::new(50);  // max 50 entries
//!
//! // Push state changes
//! history.push(current_state.clone());
//!
//! // Navigate history
//! if let Some(prev) = history.undo() {
//!     restore_state(prev);
//! }
//! ```
//!
//! ## Regions of Interest (ROI)
//!
//! The ROI system allows users to define and manage custom regions on brain
//! surfaces for analysis:
//!
//! - [`RoiDefinition`]: A named set of vertices with a display color
//! - [`RoiVertex`]: A vertex identified by surface ID and index
//! - [`RoiStatistics`]: Statistics computed over ROI vertices (mean, std, etc.)
//! - [`RoiManager`]: Manages multiple ROIs with save/load support
//!
//! ```
//! use interaction::{RoiManager, RoiStatistics};
//!
//! let mut manager = RoiManager::new();
//!
//! // Start drawing a new ROI
//! manager.start_new_roi("Motor Cortex".to_string());
//!
//! // Add vertices (from click/drag interaction)
//! manager.add_vertex_to_current(0, vertex_1);
//! manager.add_vertex_to_current(0, vertex_2);
//!
//! // Save the ROI
//! let roi_id = manager.save_current();
//!
//! // Compute statistics over ROI vertices
//! let values: Vec<f32> = get_values_for_roi_vertices(&manager.get_roi(roi_id).unwrap());
//! let stats = RoiStatistics::from_values(&values);
//! println!("Mean: {:.3}, Std: {:.3}", stats.mean, stats.std_dev);
//! ```
//!
//! # Design Principles
//!
//! - **Pure state**: No side effects, easy to test
//! - **Serializable**: All types implement `Serialize`/`Deserialize` for persistence
//! - **Framework-agnostic**: Works with any UI or rendering system

pub mod selection;
pub mod hover;
pub mod history;
pub mod roi;

// Re-export main types for convenience
pub use selection::{Selection, SelectedVertex, SurfaceId as SelectionSurfaceId};
pub use hover::HoverState;
pub use history::History;
pub use roi::{RoiDefinition, RoiVertex, RoiStatistics, RoiManager, SurfaceId as RoiSurfaceId};

