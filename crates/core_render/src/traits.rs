use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    /// Generic rendering error with a message.
    #[error("render error: {0}")]
    Message(String),
}

/// Result of a GPU picking operation.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Vertex index that was picked, if any.
    pub vertex_index: Option<u32>,
    /// Surface ID that was picked (for multi-surface scenes).
    pub surface_id: Option<u32>,
    /// World-space position of the pick point.
    pub position: Option<[f32; 3]>,
    /// Data value at the picked vertex.
    pub value: Option<f32>,
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

/// Generic input event for the renderer.
#[derive(Debug, Clone)]
pub enum ViewerEvent {
    MouseDown { position: MousePosition, button: MouseButton },
    MouseUp { position: MousePosition, button: MouseButton },
    MouseMove { position: MousePosition, delta: (f32, f32) },
    Wheel { delta_y: f32 },
    Resize { width: u32, height: u32 },
    KeyDown { key: String },
    Click { position: MousePosition },
}

/// Camera state used for saving/restoring view configuration.
#[derive(Debug, Clone)]
pub struct CameraState {
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
}

/// Abstract interface implemented by concrete renderer backends.
pub trait BrainRendererBackend {
    /// Resize the render target.
    fn resize(&mut self, width: u32, height: u32);

    /// Handle an input event; returns true if the event was consumed.
    fn handle_event(&mut self, event: ViewerEvent) -> bool;

    /// Render a frame.
    fn render(&mut self) -> Result<(), RenderError>;

    /// Read the current camera state.
    fn camera_state(&self) -> CameraState;

    /// Restore a previously saved camera state.
    fn set_camera_state(&mut self, state: CameraState);
}

