//! Debug visualization modes for the brain viewer.
//!
//! This module provides debug view modes that can be enabled to help
//! diagnose rendering issues or inspect surface data.

/// Debug visualization modes.
///
/// These modes modify how the surface is rendered to help debug
/// rendering issues or understand the underlying data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugView {
    /// Normal rendering with overlay colormapping (default).
    #[default]
    None,

    /// Render surface normals as RGB colors.
    ///
    /// Normal components are mapped from [-1, 1] to [0, 1]:
    /// - Red = X component (left/right)
    /// - Green = Y component (anterior/posterior)
    /// - Blue = Z component (superior/inferior)
    Normals,

    /// Render raw overlay values as grayscale.
    ///
    /// Values are normalized using the current data range and displayed
    /// as grayscale intensity without colormap or thresholding.
    RawOverlay,

    /// Render vertex IDs as colors for picking debugging.
    ///
    /// Each vertex gets a unique color based on its ID, useful for
    /// verifying that picking is working correctly.
    VertexId,
}

impl DebugView {
    /// Get the debug mode as a u32 for passing to the shader.
    ///
    /// - 0 = None (normal rendering)
    /// - 1 = Normals
    /// - 2 = RawOverlay
    /// - 3 = VertexId
    pub fn as_u32(&self) -> u32 {
        match self {
            DebugView::None => 0,
            DebugView::Normals => 1,
            DebugView::RawOverlay => 2,
            DebugView::VertexId => 3,
        }
    }

    /// All available debug view modes.
    pub fn all() -> &'static [DebugView] {
        &[
            DebugView::None,
            DebugView::Normals,
            DebugView::RawOverlay,
            DebugView::VertexId,
        ]
    }

    /// Get a human-readable name for this debug mode.
    pub fn name(&self) -> &'static str {
        match self {
            DebugView::None => "Normal",
            DebugView::Normals => "Normals",
            DebugView::RawOverlay => "Raw Overlay",
            DebugView::VertexId => "Vertex ID",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_view_default() {
        assert_eq!(DebugView::default(), DebugView::None);
    }

    #[test]
    fn test_debug_view_as_u32() {
        assert_eq!(DebugView::None.as_u32(), 0);
        assert_eq!(DebugView::Normals.as_u32(), 1);
        assert_eq!(DebugView::RawOverlay.as_u32(), 2);
        assert_eq!(DebugView::VertexId.as_u32(), 3);
    }

    #[test]
    fn test_debug_view_all() {
        let all = DebugView::all();
        assert_eq!(all.len(), 4);
    }
}
