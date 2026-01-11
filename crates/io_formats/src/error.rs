use thiserror::Error;

/// Errors that can occur while reading or parsing neuroimaging file formats.
#[derive(Debug, Error)]
pub enum FormatError {
    /// The file magic number did not match the expected value.
    #[error("invalid magic, expected {expected}, found {found}")]
    InvalidMagic { expected: &'static str, found: String },

    /// The file version is not supported.
    #[error("unsupported version, expected {expected}, found {found}")]
    UnsupportedVersion { expected: u32, found: u32 },

    /// The file flags contain unsupported bits.
    #[error("unsupported flags: {flags:#x}")]
    UnsupportedFlags { flags: u32 },

    /// The file size did not match what was expected from its header.
    #[error("size mismatch, expected {expected} bytes, actual {actual} bytes")]
    SizeMismatch { expected: usize, actual: usize },

    /// Overlay vertex count does not match surface vertex count.
    #[error("vertex count mismatch: overlay has {overlay_vertices} vertices, surface has {surface_vertices}")]
    VertexCountMismatch {
        overlay_vertices: usize,
        surface_vertices: usize,
    },

    /// Volumes in a multi-volume overlay have inconsistent vertex counts.
    #[error("inconsistent volumes: volume {volume_index} has {actual_vertices} vertices, expected {expected_vertices}")]
    InconsistentVolumes {
        volume_index: usize,
        expected_vertices: usize,
        actual_vertices: usize,
    },

    /// Underlying I/O error when reading data.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

