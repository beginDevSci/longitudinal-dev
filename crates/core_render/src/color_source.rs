//! Color source and parcellation display modes for surface rendering.
//!
//! This module defines how surface colors are determined during rendering:
//! - Overlay mode uses per-vertex scalar data with colormap lookup
//! - Parcellation mode uses per-vertex region labels with region colors

/// Determines the source of surface vertex colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSource {
    /// Use overlay scalar data with colormap (default behavior).
    #[default]
    Overlay,
    /// Use parcellation region labels with region colors.
    Parcellation,
}

impl ColorSource {
    /// Convert to a u32 value for shader uniforms.
    pub fn as_u32(&self) -> u32 {
        match self {
            ColorSource::Overlay => 0,
            ColorSource::Parcellation => 1,
        }
    }

    /// Get all available color source modes.
    pub fn all() -> &'static [ColorSource] {
        &[ColorSource::Overlay, ColorSource::Parcellation]
    }

    /// Get the display name for this mode.
    pub fn name(&self) -> &'static str {
        match self {
            ColorSource::Overlay => "Overlay",
            ColorSource::Parcellation => "Parcellation",
        }
    }
}

/// Controls how parcellation regions are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParcellationDisplay {
    /// Show filled region colors only.
    #[default]
    Fill,
    /// Show region boundary edges only.
    Edges,
    /// Show filled regions with boundary edges overlaid.
    FillAndEdges,
}

impl ParcellationDisplay {
    /// Convert to a u32 value for shader uniforms.
    pub fn as_u32(&self) -> u32 {
        match self {
            ParcellationDisplay::Fill => 0,
            ParcellationDisplay::Edges => 1,
            ParcellationDisplay::FillAndEdges => 2,
        }
    }

    /// Get all available display modes.
    pub fn all() -> &'static [ParcellationDisplay] {
        &[
            ParcellationDisplay::Fill,
            ParcellationDisplay::Edges,
            ParcellationDisplay::FillAndEdges,
        ]
    }

    /// Get the display name for this mode.
    pub fn name(&self) -> &'static str {
        match self {
            ParcellationDisplay::Fill => "Fill",
            ParcellationDisplay::Edges => "Edges",
            ParcellationDisplay::FillAndEdges => "Fill & Edges",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_source_default() {
        assert_eq!(ColorSource::default(), ColorSource::Overlay);
    }

    #[test]
    fn test_color_source_values() {
        assert_eq!(ColorSource::Overlay.as_u32(), 0);
        assert_eq!(ColorSource::Parcellation.as_u32(), 1);
    }

    #[test]
    fn test_parcellation_display_default() {
        assert_eq!(ParcellationDisplay::default(), ParcellationDisplay::Fill);
    }

    #[test]
    fn test_parcellation_display_values() {
        assert_eq!(ParcellationDisplay::Fill.as_u32(), 0);
        assert_eq!(ParcellationDisplay::Edges.as_u32(), 1);
        assert_eq!(ParcellationDisplay::FillAndEdges.as_u32(), 2);
    }
}
