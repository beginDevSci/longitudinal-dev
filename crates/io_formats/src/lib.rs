//! File format parsers and data structures for brain surface geometry
//! and statistical overlays.
//!
//! This crate is format- and domain-level only and has no GPU or UI
//! dependencies.
//!
//! # Module Organization
//!
//! Each neuroimaging format has two modules:
//!
//! | Format | Production Parser | Test Utilities |
//! |--------|-------------------|----------------|
//! | FreeSurfer | `freesurfer_real` | `freesurfer` |
//! | GIFTI | `gifti_real` | `gifti` |
//! | NIfTI | `nifti_real` | `nifti` |
//!
//! The **production parsers** (`*_real`) handle actual neuroimaging file formats.
//! The **test utilities** define simplified synthetic formats for stable testing.
//!
//! # Auto-detection
//!
//! The `load_surface` and `load_overlay` functions auto-detect file formats
//! based on magic bytes, dispatching to the appropriate parser:
//!
//! - Real format magic → production parser
//! - Synthetic format magic → test parser (for fixture files)

pub mod detect;
pub mod freesurfer;
pub mod freesurfer_real;
pub mod gifti;
pub mod gifti_real;
pub mod nifti;
pub mod nifti_real;
pub mod geometry;
pub mod statistics;
pub mod loader;
pub mod metadata;
pub mod error;
pub mod validation;
pub mod parcellation;

pub use detect::{detect_format, detect_format_from_bytes, FileFormat};
pub use freesurfer::{read_curv as read_fs_curv, read_fs_surface, read_surface};
pub use freesurfer_real::{
    is_real_annot, read_real_annot, read_real_annot_bytes, read_real_surface,
};
pub use gifti_real::{is_gifti_label, read_gifti_label, read_gifti_label_bytes};
pub use parcellation::{Parcellation, Region};
pub use validation::{
    ensure_overlay_matches_surface, ensure_stats_match_surface, ensure_volumes_consistent,
};

use std::path::Path;

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::nifti::NiftiScalar;
use crate::statistics::{Hemisphere, StatisticData};

pub enum SurfaceSource {
    FreeSurfer(BrainGeometry),
    Gifti(BrainGeometry),
}

pub enum OverlaySource {
    FreeSurferCurv(Vec<f32>),
    GiftiFunc(Vec<Vec<f32>>),
    Nifti(NiftiScalar),
    /// BLMM statistics data (BRS1 format)
    BlmmStatistics(StatisticData),
}

impl OverlaySource {
    /// Convert overlay to a normalized StatisticData representation.
    ///
    /// This allows all overlay types to be treated uniformly for rendering
    /// purposes, with automatic range computation.
    pub fn to_statistic_data(&self) -> StatisticData {
        match self {
            OverlaySource::BlmmStatistics(data) => data.clone(),
            OverlaySource::FreeSurferCurv(values) => {
                let (min, max) = compute_range(values);
                StatisticData {
                    values: values.clone(),
                    n_vertices: values.len(),
                    n_volumes: 1,
                    global_min: min,
                    global_max: max,
                    volume_ranges: vec![(min, max)],
                    nan_count: values.iter().filter(|v| v.is_nan()).count() as u32,
                }
            }
            OverlaySource::GiftiFunc(volumes) => {
                // Flatten volumes into a single values array
                let n_volumes = volumes.len();
                let n_vertices = volumes.first().map(|v| v.len()).unwrap_or(0);
                let mut values = Vec::with_capacity(n_vertices * n_volumes);
                let mut volume_ranges = Vec::with_capacity(n_volumes);
                let mut global_min = f32::INFINITY;
                let mut global_max = f32::NEG_INFINITY;
                let mut nan_count = 0u32;

                for vol in volumes {
                    let (min, max) = compute_range(vol);
                    volume_ranges.push((min, max));
                    global_min = global_min.min(min);
                    global_max = global_max.max(max);
                    nan_count += vol.iter().filter(|v| v.is_nan()).count() as u32;
                    values.extend_from_slice(vol);
                }

                StatisticData {
                    values,
                    n_vertices,
                    n_volumes,
                    global_min,
                    global_max,
                    volume_ranges,
                    nan_count,
                }
            }
            OverlaySource::Nifti(nifti) => {
                // NIfTI scalar overlay - single volume
                let values = nifti.data.clone();
                let (min, max) = compute_range(&values);
                StatisticData {
                    values: values.clone(),
                    n_vertices: values.len(),
                    n_volumes: 1,
                    global_min: min,
                    global_max: max,
                    volume_ranges: vec![(min, max)],
                    nan_count: values.iter().filter(|v| v.is_nan()).count() as u32,
                }
            }
        }
    }
}

/// Compute (min, max) range for a slice of floats, excluding NaN and Inf.
fn compute_range(values: &[f32]) -> (f32, f32) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for &v in values {
        if v.is_finite() {
            min = min.min(v);
            max = max.max(v);
        }
    }
    // Handle case where all values are non-finite
    if min.is_infinite() {
        min = 0.0;
        max = 1.0;
    }
    (min, max)
}

fn infer_hemisphere_from_path(path: &Path) -> Hemisphere {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if stem.starts_with("lh.") || stem == "lh" {
            return Hemisphere::Left;
        }
        if stem.starts_with("rh.") || stem == "rh" {
            return Hemisphere::Right;
        }
    }
    Hemisphere::Left
}

/// Check if a file starts with real FreeSurfer triangle magic (0xFFFFFE).
fn is_real_freesurfer_surface(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    if let Ok(mut file) = File::open(path) {
        let mut magic = [0u8; 3];
        if file.read_exact(&mut magic).is_ok() {
            return magic == [0xFF, 0xFF, 0xFE];
        }
    }
    false
}

/// Check if a file starts with real FreeSurfer "new" curv magic (0xFFFFFF).
fn is_real_freesurfer_curv(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    if let Ok(mut file) = File::open(path) {
        let mut magic = [0u8; 3];
        if file.read_exact(&mut magic).is_ok() {
            return magic == [0xFF, 0xFF, 0xFF];
        }
    }
    false
}

pub fn load_surface(path: &Path) -> Result<SurfaceSource, FormatError> {
    let format = detect::detect_format(path);
    let hemisphere = infer_hemisphere_from_path(path);
    match format {
        detect::FileFormat::FreeSurferSurface => {
            // Check if this is a real FreeSurfer file or synthetic test file
            let geom = if is_real_freesurfer_surface(path) {
                freesurfer_real::read_real_surface(path, hemisphere)?
            } else {
                freesurfer::read_fs_surface(path, hemisphere)?
            };
            Ok(SurfaceSource::FreeSurfer(geom))
        }
        detect::FileFormat::Gifti => {
            // Check if this is a real GIFTI XML file or synthetic JSON test file
            let geom = if gifti_real::is_real_gifti(path) {
                gifti_real::read_real_gifti_surface(path)?
            } else {
                gifti::read_gifti_surface(path)?
            };
            Ok(SurfaceSource::Gifti(geom))
        }
        other => Err(FormatError::InvalidMagic {
            expected: "surface format",
            found: format!("unsupported format: {:?}", other),
        }),
    }
}

pub fn load_overlay(path: &Path) -> Result<OverlaySource, FormatError> {
    let format = detect::detect_format(path);
    match format {
        detect::FileFormat::FreeSurferCurv => {
            // Check if this is a real FreeSurfer curv file or synthetic test file
            let values = if is_real_freesurfer_curv(path) {
                freesurfer_real::read_real_curv(path)?
            } else {
                freesurfer::read_fs_curv(path)?
            };
            Ok(OverlaySource::FreeSurferCurv(values))
        }
        detect::FileFormat::Gifti => {
            // Check if this is a real GIFTI XML file or synthetic JSON test file
            if gifti_real::is_real_gifti(path) {
                let volumes = gifti_real::read_real_gifti_func(path)?;
                Ok(OverlaySource::GiftiFunc(volumes))
            } else {
                let func = gifti::read_gifti_func(path)?;
                Ok(OverlaySource::GiftiFunc(func))
            }
        }
        detect::FileFormat::Nifti => {
            // Check if this is a real NIfTI-1 file or synthetic test file
            if nifti_real::is_real_nifti1(path) {
                let nifti = nifti_real::read_real_nifti1(path)?;
                Ok(OverlaySource::Nifti(nifti::NiftiScalar {
                    dims: [
                        nifti.dims[0] as u16,
                        nifti.dims[1] as u16,
                        nifti.dims[2] as u16,
                        nifti.dims[3] as u16,
                        nifti.dims[4] as u16,
                        nifti.dims[5] as u16,
                        nifti.dims[6] as u16,
                        nifti.dims[7] as u16,
                    ],
                    data: nifti.data,
                }))
            } else {
                let vol = nifti::read_nifti_scalar(path)?;
                Ok(OverlaySource::Nifti(vol))
            }
        }
        detect::FileFormat::BlmmStatistics => {
            let bytes = std::fs::read(path)?;
            let data = StatisticData::from_bytes(&bytes)?;
            Ok(OverlaySource::BlmmStatistics(data))
        }
        other => Err(FormatError::InvalidMagic {
            expected: "overlay format",
            found: format!("unsupported format: {:?}", other),
        }),
    }
}

/// Load a surface from bytes with a specified format.
///
/// This is useful for web-fetched data where the format is known
/// but the data is already in memory.
pub fn load_surface_bytes(
    bytes: &[u8],
    format: detect::FileFormat,
    hemisphere: Hemisphere,
) -> Result<SurfaceSource, FormatError> {
    match format {
        detect::FileFormat::FreeSurferSurface => {
            let geom = freesurfer_real::read_real_surface_bytes(bytes, hemisphere)?;
            Ok(SurfaceSource::FreeSurfer(geom))
        }
        detect::FileFormat::Gifti => {
            let geom = gifti_real::read_gifti_surface_bytes(bytes, hemisphere)?;
            Ok(SurfaceSource::Gifti(geom))
        }
        other => Err(FormatError::InvalidMagic {
            expected: "surface format",
            found: format!("unsupported format for bytes: {:?}", other),
        }),
    }
}

/// Load an overlay from bytes with a specified format.
///
/// This is useful for web-fetched data where the format is known
/// but the data is already in memory.
pub fn load_overlay_bytes(
    bytes: &[u8],
    format: detect::FileFormat,
) -> Result<OverlaySource, FormatError> {
    match format {
        detect::FileFormat::BlmmStatistics => {
            let data = StatisticData::from_bytes(bytes)?;
            Ok(OverlaySource::BlmmStatistics(data))
        }
        detect::FileFormat::FreeSurferCurv => {
            let values = freesurfer_real::read_real_curv_from_bytes(bytes)?;
            Ok(OverlaySource::FreeSurferCurv(values))
        }
        detect::FileFormat::Gifti => {
            let volumes = gifti_real::read_gifti_func_bytes(bytes)?;
            Ok(OverlaySource::GiftiFunc(volumes))
        }
        detect::FileFormat::Nifti => {
            let nifti = nifti_real::read_nifti1_bytes(bytes)?;
            Ok(OverlaySource::Nifti(nifti::NiftiScalar {
                dims: [
                    nifti.dims[0] as u16,
                    nifti.dims[1] as u16,
                    nifti.dims[2] as u16,
                    nifti.dims[3] as u16,
                    nifti.dims[4] as u16,
                    nifti.dims[5] as u16,
                    nifti.dims[6] as u16,
                    nifti.dims[7] as u16,
                ],
                data: nifti.data,
            }))
        }
        other => Err(FormatError::InvalidMagic {
            expected: "overlay format",
            found: format!("unsupported format for bytes: {:?}", other),
        }),
    }
}

/// Load an overlay from bytes, auto-detecting the format from magic bytes.
///
/// This variant detects the format from magic bytes rather than file extension.
/// For more control, use `load_overlay_bytes` with an explicit format.
pub fn load_overlay_from_bytes(bytes: &[u8]) -> Result<OverlaySource, FormatError> {
    let format = detect::detect_format_from_bytes(bytes);
    if format == detect::FileFormat::Unknown {
        return Err(FormatError::InvalidMagic {
            expected: "overlay format",
            found: format!(
                "unrecognized magic bytes: {:?}",
                &bytes[..bytes.len().min(4)]
            ),
        });
    }
    load_overlay_bytes(bytes, format)
}
