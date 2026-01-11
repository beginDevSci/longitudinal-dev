//! Neuroimaging domain layer for brain surface visualization.
//!
//! This crate provides domain-specific abstractions for neuroimaging applications,
//! built on top of the low-level `io_formats` crate. It bridges the gap between
//! raw file formats and the rendering layer.
//!
//! # Core Concepts
//!
//! ## Brain Surfaces
//!
//! The [`BrainSurface`] type represents a complete brain with optional left and right
//! hemispheres. Each hemisphere is stored as a [`HemisphereSurface`] containing the
//! geometry from FreeSurfer or GIFTI files.
//!
//! ```ignore
//! use neuro_surface::{BrainSurface, HemisphereSurface};
//! use io_formats::statistics::Hemisphere;
//!
//! let mut brain = BrainSurface::default();
//! brain.left = Some(HemisphereSurface {
//!     hemisphere: Hemisphere::Left,
//!     geometry: left_geometry,
//! });
//!
//! for hemi in brain.hemispheres() {
//!     println!("Hemisphere: {:?}, vertices: {}", hemi.hemisphere, hemi.geometry.vertices.len());
//! }
//! ```
//!
//! ## Statistical Overlays
//!
//! The [`OverlayBinding`] and [`OverlayRange`] types manage statistical data
//! overlaid on brain surfaces. Overlays can come from various sources:
//! - BLMM analysis results (t-statistics, p-values)
//! - FreeSurfer morphometry (curvature, thickness)
//! - GIFTI functional data
//!
//! ## Camera Presets
//!
//! [`BrainViewPreset`] provides standardized camera positions for neuroimaging:
//! - **Lateral views**: Left/Right side views
//! - **Medial views**: Inside views of each hemisphere
//! - **Axial views**: Dorsal (top) and Ventral (bottom)
//! - **Coronal views**: Anterior (front) and Posterior (back)
//!
//! These correspond to the RAS (Right-Anterior-Superior) coordinate system
//! used in neuroimaging.
//!
//! ## Parcellations
//!
//! The [`Parcellation`] type represents brain region annotations (atlases),
//! mapping vertices to named regions with associated colors. Supported formats:
//! - FreeSurfer `.annot` files (aparc, aparc.a2009s)
//! - GIFTI `.label.gii` files
//!
//! # Module Overview
//!
//! - [`brain`]: `BrainSurface` and `HemisphereSurface` types
//! - [`overlay`]: `OverlayBinding` and `OverlayRange` for statistical data
//! - [`colormap`]: Colormap definitions for overlay visualization
//! - [`parcellation`]: Region annotations and atlas support
//! - [`presets`]: Standard neuroimaging camera views

pub mod brain;
pub mod overlay;
pub mod colormap;
pub mod parcellation;
pub mod presets;

pub use brain::{BrainSurface, HemisphereSurface};
pub use overlay::{OverlayBinding, OverlayRange};
pub use parcellation::{Parcellation, ParcelInfo, ParcelId, ParcelLookup, PARCEL_ID_UNKNOWN};
pub use presets::BrainViewPreset;

