//! Renderer trait abstraction for brain visualization backends.
//!
//! This module defines the `BrainRendererBackend` trait and supporting types.
//! Some trait methods and types are scaffolding for future features (camera state
//! serialization, additional error variants) and are not yet called from the UI.

use std::any::Any;

use crate::data::{BrainGeometry, StatisticData};
use crate::types::{BrainViewPreset, ColormapType, LayoutMode, VertexInfo, ViewerEvent};

/// Result from handling an event - can contain click selection and/or hover info.
#[derive(Debug, Clone, Default)]
pub struct EventResult {
    /// Vertex that was clicked/selected (persistent until cleared).
    pub clicked: Option<VertexInfo>,
    /// Vertex currently being hovered over (transient).
    pub hovered: Option<VertexInfo>,
}

/// Abstraction over 3D rendering backends (three-d, wgpu, etc.)
#[allow(dead_code)] // Trait methods for future features (camera state, resize)
pub trait BrainRendererBackend {
    /// Returns a mutable reference to self as Any for downcasting to concrete types.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn new(
        gl_context: web_sys::WebGl2RenderingContext,
        geometry: BrainGeometry,
        statistics: StatisticData,
    ) -> Result<Self, RendererError>
    where
        Self: Sized;

    fn set_geometry(&mut self, geometry: BrainGeometry);

    fn set_statistics(&mut self, statistics: StatisticData);

    fn set_volume(&mut self, idx: u32);

    fn set_threshold(&mut self, threshold: Option<f32>);

    fn set_colormap(&mut self, colormap: ColormapType);

    fn set_symmetric(&mut self, symmetric: bool);

    /// Set the layout mode for dual-hemisphere display.
    fn set_layout(&mut self, layout: LayoutMode);

    /// Apply a predefined camera preset, if supported.
    fn set_view_preset(&mut self, preset: BrainViewPreset);

    /// Handle an input event and return any resulting vertex info.
    /// Returns clicked vertex for Click events, hovered vertex for MouseMove events.
    fn handle_event(&mut self, event: ViewerEvent) -> EventResult;

    /// Whether an async pick/poll is currently in flight (if supported).
    fn has_pending_pick(&self) -> bool;

    /// Poll for completed async pick results.
    /// Returns an EventResult with any clicked/hovered vertex info from completed picks.
    /// For renderers with immediate picking, this returns EventResult::default().
    fn poll_completed_pick(&mut self) -> EventResult;

    fn render(&mut self);

    fn resize(&mut self, width: u32, height: u32);

    fn camera_state(&self) -> CameraState;

    fn set_camera_state(&mut self, state: CameraState);

    fn is_running(&self) -> bool;

    fn stop(&mut self);
}

/// Camera state for serialization/restoration (scaffolding for URL sharing)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraState {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov_degrees: f32,
}

/// Renderer error types - some variants reserved for future error handling
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("context creation error: {0}")]
    ContextCreation(String),
    #[error("shader compilation error: {0}")]
    ShaderCompilation(String),
    #[error("buffer allocation error: {0}")]
    BufferAllocation(String),
    #[error("unsupported format version, expected {expected}, found {found}")]
    UnsupportedFormat { expected: u32, found: u32 },
    #[error("other renderer error: {0}")]
    Other(String),
}
