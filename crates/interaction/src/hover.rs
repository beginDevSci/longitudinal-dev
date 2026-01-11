//! Hover state management for GPU picking.
//!
//! This module provides hover state tracking designed to work with GPU-based
//! vertex picking systems. It supports throttled picks and smooth hover decay.
//!
//! # Example: Integrating with GPU Picking
//!
//! ```ignore
//! use interaction::HoverState;
//!
//! const HOVER_THROTTLE_MS: f64 = 50.0;  // Pick at most every 50ms
//! const HOVER_DECAY_MS: f64 = 200.0;    // Keep hover for 200ms after leaving
//!
//! struct Renderer {
//!     hover_state: HoverState,
//!     last_pick_time: f64,
//! }
//!
//! impl Renderer {
//!     fn on_mouse_move(&mut self, x: f32, y: f32) {
//!         let now = get_current_time_ms();
//!
//!         // Throttle: Skip pick if too soon
//!         if now - self.last_pick_time < HOVER_THROTTLE_MS {
//!             return;
//!         }
//!         self.last_pick_time = now;
//!
//!         // Perform GPU pick
//!         if let Some(result) = gpu_pick(x, y) {
//!             // Hit: Update hover state
//!             self.hover_state.set_hit(
//!                 result.vertex_id,
//!                 result.surface_id,
//!                 result.region_name,
//!                 now,
//!             );
//!             show_hover_ui(&result);
//!         } else {
//!             // Miss: Check for decay
//!             if self.hover_state.should_decay(now, HOVER_DECAY_MS) {
//!                 // Decay expired, clear hover
//!                 self.hover_state.clear();
//!                 hide_hover_ui();
//!             } else {
//!                 // Still within decay window, keep showing last hover
//!                 // (no action needed, UI still shows previous state)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! The separation of throttle and decay allows for efficient picking while
//! maintaining smooth visual feedback when the cursor moves between vertices.

use serde::{Deserialize, Serialize};

/// Surface identifier for multi-surface scenes.
pub type SurfaceId = u32;

/// Current hover state for the viewer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoverState {
    /// Index of the vertex under the cursor, if any.
    pub vertex: Option<u32>,
    /// Surface ID of the hovered vertex, if any.
    pub surface_id: Option<SurfaceId>,
    /// Identifier of the region under the cursor, if any.
    pub region: Option<String>,
    /// Timestamp of the last hover hit (for decay/debounce).
    #[serde(skip)]
    pub last_hit_time_ms: f64,
}

impl HoverState {
    /// Reset hover information.
    pub fn clear(&mut self) {
        self.vertex = None;
        self.surface_id = None;
        self.region = None;
        self.last_hit_time_ms = 0.0;
    }

    /// Check if hover state is currently active.
    pub fn is_active(&self) -> bool {
        self.vertex.is_some()
    }

    /// Update hover state with a new hit.
    pub fn set_hit(&mut self, vertex: u32, surface_id: SurfaceId, region: Option<String>, time_ms: f64) {
        self.vertex = Some(vertex);
        self.surface_id = Some(surface_id);
        self.region = region;
        self.last_hit_time_ms = time_ms;
    }

    /// Check if hover should decay (no hit for decay_ms milliseconds).
    /// Returns true if the hover should be cleared.
    pub fn should_decay(&self, current_time_ms: f64, decay_ms: f64) -> bool {
        if self.vertex.is_none() {
            return false;
        }
        current_time_ms - self.last_hit_time_ms > decay_ms
    }
}

