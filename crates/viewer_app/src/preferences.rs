//! User preferences persistence using localStorage.
//!
//! This module provides functions to save and load user preferences
//! to/from localStorage, allowing settings to persist across sessions.

use serde::{Deserialize, Serialize};

use crate::types::{ColormapType, LayoutMode};

/// Keys used for localStorage.
mod keys {
    pub const COLOR_MODE: &str = "bv_pref_color_mode";
    pub const PARC_DISPLAY_MODE: &str = "bv_pref_parc_display_mode";
    pub const REGION_SELECTION_ENABLED: &str = "bv_pref_region_selection";
    pub const ROI_OVERLAY_VISIBLE: &str = "bv_pref_roi_overlay";
    pub const LAYOUT: &str = "bv_pref_layout";
    pub const COLORMAP: &str = "bv_pref_colormap";
    pub const SYMMETRIC: &str = "bv_pref_symmetric";
}

/// User preferences that can be persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Color mode: "overlay" or "parcellation"
    pub color_mode: String,
    /// Parcellation display mode: "fill", "edges", "fill_edges"
    pub parc_display_mode: String,
    /// Whether region selection mode is enabled
    pub region_selection_enabled: bool,
    /// Whether ROI overlay is visible
    pub roi_overlay_visible: bool,
    /// Layout mode
    pub layout: LayoutMode,
    /// Colormap type
    pub colormap: ColormapType,
    /// Whether symmetric colorbar is enabled
    pub symmetric: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            color_mode: "overlay".to_string(),
            parc_display_mode: "fill".to_string(),
            region_selection_enabled: false,
            roi_overlay_visible: false,
            layout: LayoutMode::default(),
            colormap: ColormapType::default(),
            symmetric: false,
        }
    }
}

/// Load user preferences from localStorage.
///
/// Returns default preferences if localStorage is unavailable or values are missing.
#[cfg(target_arch = "wasm32")]
pub fn load_preferences() -> UserPreferences {
    let storage = match web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        Some(s) => s,
        None => return UserPreferences::default(),
    };

    let get_string = |key: &str, default: &str| -> String {
        storage
            .get_item(key)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    };

    let get_bool = |key: &str, default: bool| -> bool {
        storage
            .get_item(key)
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(default)
    };

    let layout = match get_string(keys::LAYOUT, "single").as_str() {
        "side-by-side" => LayoutMode::SideBySide,
        "stacked" => LayoutMode::Stacked,
        _ => LayoutMode::Single,
    };

    let colormap = match get_string(keys::COLORMAP, "rdbu").as_str() {
        "viridis" => ColormapType::Viridis,
        "hot" => ColormapType::Hot,
        "cividis" => ColormapType::Cividis,
        "plasma" => ColormapType::Plasma,
        _ => ColormapType::RdBu,
    };

    UserPreferences {
        color_mode: get_string(keys::COLOR_MODE, "overlay"),
        parc_display_mode: get_string(keys::PARC_DISPLAY_MODE, "fill"),
        region_selection_enabled: get_bool(keys::REGION_SELECTION_ENABLED, false),
        roi_overlay_visible: get_bool(keys::ROI_OVERLAY_VISIBLE, false),
        layout,
        colormap,
        symmetric: get_bool(keys::SYMMETRIC, false),
    }
}

/// Load user preferences (stub for non-wasm targets).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_preferences() -> UserPreferences {
    UserPreferences::default()
}

/// Save a single preference to localStorage.
#[cfg(target_arch = "wasm32")]
fn save_to_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
    }
}

/// Save a single preference (stub for non-wasm targets).
#[cfg(not(target_arch = "wasm32"))]
fn save_to_storage(_key: &str, _value: &str) {}

/// Save color mode preference.
pub fn save_color_mode(mode: &str) {
    save_to_storage(keys::COLOR_MODE, mode);
}

/// Save parcellation display mode preference.
pub fn save_parc_display_mode(mode: &str) {
    save_to_storage(keys::PARC_DISPLAY_MODE, mode);
}

/// Save region selection enabled preference.
pub fn save_region_selection_enabled(enabled: bool) {
    save_to_storage(
        keys::REGION_SELECTION_ENABLED,
        if enabled { "true" } else { "false" },
    );
}

/// Save ROI overlay visible preference.
pub fn save_roi_overlay_visible(visible: bool) {
    save_to_storage(
        keys::ROI_OVERLAY_VISIBLE,
        if visible { "true" } else { "false" },
    );
}

/// Save layout preference.
pub fn save_layout(layout: LayoutMode) {
    let value = match layout {
        LayoutMode::Single => "single",
        LayoutMode::SideBySide => "side-by-side",
        LayoutMode::Stacked => "stacked",
    };
    save_to_storage(keys::LAYOUT, value);
}

/// Save colormap preference.
pub fn save_colormap(colormap: ColormapType) {
    let value = match colormap {
        ColormapType::RdBu => "rdbu",
        ColormapType::Viridis => "viridis",
        ColormapType::Hot => "hot",
        ColormapType::Cividis => "cividis",
        ColormapType::Plasma => "plasma",
    };
    save_to_storage(keys::COLORMAP, value);
}

/// Save symmetric colorbar preference.
pub fn save_symmetric(symmetric: bool) {
    save_to_storage(keys::SYMMETRIC, if symmetric { "true" } else { "false" });
}
