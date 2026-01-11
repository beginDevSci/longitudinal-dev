use serde::{Deserialize, Serialize};

// Core format-level types are defined in `io_formats` and re-exported
// here so the rest of the viewer can depend on a single source.
pub use io_formats::statistics::{Analysis, Hemisphere, Statistic};

// Re-export brain view presets from neuro_surface
pub use neuro_surface::BrainViewPreset;

/// Surface identifier type (matches core_render).
pub type SurfaceId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColormapType {
    #[serde(rename = "Viridis")]
    Viridis,
    #[default]
    #[serde(rename = "RdBu")]
    RdBu,
    #[serde(rename = "Hot")]
    Hot,
    /// Colorblind-friendly perceptually uniform colormap.
    #[serde(rename = "Cividis")]
    Cividis,
    /// Perceptually uniform sequential colormap (purple to yellow).
    #[serde(rename = "Plasma")]
    Plasma,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
}

/// Modifier keys state for input events.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ModifierKeys {
    /// Create modifier keys from a mouse event.
    pub fn from_mouse_event(ev: &web_sys::MouseEvent) -> Self {
        Self {
            shift: ev.shift_key(),
            ctrl: ev.ctrl_key(),
            alt: ev.alt_key(),
            meta: ev.meta_key(),
        }
    }

    /// Create modifier keys from a keyboard event.
    pub fn from_keyboard_event(ev: &web_sys::KeyboardEvent) -> Self {
        Self {
            shift: ev.shift_key(),
            ctrl: ev.ctrl_key(),
            alt: ev.alt_key(),
            meta: ev.meta_key(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewerEvent {
    MouseDown {
        position: MousePosition,
        button: MouseButton,
        modifiers: ModifierKeys,
    },
    MouseUp {
        position: MousePosition,
        button: MouseButton,
        modifiers: ModifierKeys,
    },
    MouseMove {
        position: MousePosition,
        delta: (f32, f32),
    },
    Wheel {
        delta_y: f32,
    },
    Click {
        position: MousePosition,
        modifiers: ModifierKeys,
    },
    Resize {
        width: u32,
        height: u32,
    },
    KeyDown {
        key: String,
        modifiers: ModifierKeys,
    },
}

#[derive(Debug, Clone)]
pub struct VertexInfo {
    pub index: u32,
    pub position: [f32; 3],
    pub value: f32,
    /// Surface ID this vertex belongs to (for multi-surface scenes).
    pub surface_id: Option<SurfaceId>,
}

impl VertexInfo {
    /// Get the hemisphere based on surface_id convention.
    pub fn hemisphere(&self) -> Option<Hemisphere> {
        match self.surface_id {
            Some(0) => Some(Hemisphere::Left),
            Some(1) => Some(Hemisphere::Right),
            _ => None,
        }
    }
}

/// Layout mode for dual-hemisphere display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutMode {
    /// Both hemispheres centered and overlapping (single view).
    #[default]
    #[serde(rename = "single")]
    Single,
    /// Hemispheres separated side-by-side (left on left, right on right).
    #[serde(rename = "side-by-side")]
    SideBySide,
    /// Hemispheres stacked vertically (left on top, right on bottom).
    #[serde(rename = "stacked")]
    Stacked,
}
