//! Validation helpers for ensuring consistency between surfaces and overlays.
//!
//! These helpers verify that overlay data is compatible with surface geometry,
//! preventing runtime errors from mismatched vertex counts.

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::statistics::StatisticData;
use crate::OverlaySource;

/// Ensure an overlay's vertex count matches a surface's vertex count.
///
/// This is a critical validation step before rendering, as overlays with
/// mismatched vertex counts will cause index-out-of-bounds errors or
/// incorrect rendering.
///
/// # Arguments
/// * `overlay` - The overlay source to validate
/// * `geometry` - The brain geometry to validate against
///
/// # Returns
/// `Ok(())` if the overlay is compatible, or a `FormatError` describing the mismatch.
///
/// # Example
/// ```ignore
/// let surface = load_surface(&path)?;
/// let overlay = load_overlay(&overlay_path)?;
/// ensure_overlay_matches_surface(&overlay, &geometry)?;
/// ```
pub fn ensure_overlay_matches_surface(
    overlay: &OverlaySource,
    geometry: &BrainGeometry,
) -> Result<(), FormatError> {
    let overlay_vertices = match overlay {
        OverlaySource::FreeSurferCurv(values) => values.len(),
        OverlaySource::GiftiFunc(volumes) => volumes.first().map(|v| v.len()).unwrap_or(0),
        OverlaySource::Nifti(nifti) => nifti.data.len(),
        OverlaySource::BlmmStatistics(stats) => stats.n_vertices,
    };

    let surface_vertices = geometry.vertices.len();

    if overlay_vertices != surface_vertices {
        return Err(FormatError::VertexCountMismatch {
            overlay_vertices,
            surface_vertices,
        });
    }

    Ok(())
}

/// Ensure a StatisticData's vertex count matches a surface's vertex count.
///
/// This is the same validation as `ensure_overlay_matches_surface` but works
/// directly with `StatisticData` after conversion from `OverlaySource`.
///
/// # Arguments
/// * `stats` - The statistic data to validate
/// * `geometry` - The brain geometry to validate against
///
/// # Returns
/// `Ok(())` if compatible, or a `FormatError` describing the mismatch.
pub fn ensure_stats_match_surface(
    stats: &StatisticData,
    geometry: &BrainGeometry,
) -> Result<(), FormatError> {
    if stats.n_vertices != geometry.vertices.len() {
        return Err(FormatError::VertexCountMismatch {
            overlay_vertices: stats.n_vertices,
            surface_vertices: geometry.vertices.len(),
        });
    }
    Ok(())
}

/// Validate that all volumes in an overlay have consistent vertex counts.
///
/// For multi-volume overlays (like GIFTI functional or BLMM statistics),
/// this ensures all volumes have the same number of vertices.
///
/// # Arguments
/// * `overlay` - The overlay source to validate
///
/// # Returns
/// `Ok(())` if all volumes are consistent, or a `FormatError` if not.
pub fn ensure_volumes_consistent(overlay: &OverlaySource) -> Result<(), FormatError> {
    match overlay {
        OverlaySource::GiftiFunc(volumes) => {
            if volumes.is_empty() {
                return Ok(());
            }
            let expected = volumes[0].len();
            for (i, vol) in volumes.iter().enumerate().skip(1) {
                if vol.len() != expected {
                    return Err(FormatError::InconsistentVolumes {
                        volume_index: i,
                        expected_vertices: expected,
                        actual_vertices: vol.len(),
                    });
                }
            }
            Ok(())
        }
        OverlaySource::BlmmStatistics(stats) => {
            // StatisticData is already validated during parsing
            let expected_total = stats.n_vertices * stats.n_volumes;
            if stats.values.len() != expected_total {
                return Err(FormatError::SizeMismatch {
                    expected: expected_total,
                    actual: stats.values.len(),
                });
            }
            Ok(())
        }
        // Single-volume formats are trivially consistent
        OverlaySource::FreeSurferCurv(_) | OverlaySource::Nifti(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statistics::Hemisphere;

    fn make_test_geometry(n_vertices: usize) -> BrainGeometry {
        BrainGeometry {
            hemisphere: Hemisphere::Left,
            vertices: vec![[0.0, 0.0, 0.0]; n_vertices],
            normals: vec![[0.0, 0.0, 1.0]; n_vertices],
            indices: vec![],
        }
    }

    #[test]
    fn test_ensure_overlay_matches_surface_ok() {
        let geom = make_test_geometry(100);
        let overlay = OverlaySource::FreeSurferCurv(vec![0.0; 100]);

        assert!(ensure_overlay_matches_surface(&overlay, &geom).is_ok());
    }

    #[test]
    fn test_ensure_overlay_matches_surface_mismatch() {
        let geom = make_test_geometry(100);
        let overlay = OverlaySource::FreeSurferCurv(vec![0.0; 200]);

        let result = ensure_overlay_matches_surface(&overlay, &geom);
        assert!(result.is_err());
        match result {
            Err(FormatError::VertexCountMismatch {
                overlay_vertices,
                surface_vertices,
            }) => {
                assert_eq!(overlay_vertices, 200);
                assert_eq!(surface_vertices, 100);
            }
            _ => panic!("Expected VertexCountMismatch error"),
        }
    }

    #[test]
    fn test_ensure_stats_match_surface_ok() {
        let geom = make_test_geometry(50);
        let stats = StatisticData {
            values: vec![0.0; 50],
            n_vertices: 50,
            n_volumes: 1,
            global_min: 0.0,
            global_max: 1.0,
            volume_ranges: vec![(0.0, 1.0)],
            nan_count: 0,
        };

        assert!(ensure_stats_match_surface(&stats, &geom).is_ok());
    }

    #[test]
    fn test_ensure_stats_match_surface_mismatch() {
        let geom = make_test_geometry(50);
        let stats = StatisticData {
            values: vec![0.0; 100],
            n_vertices: 100,
            n_volumes: 1,
            global_min: 0.0,
            global_max: 1.0,
            volume_ranges: vec![(0.0, 1.0)],
            nan_count: 0,
        };

        assert!(ensure_stats_match_surface(&stats, &geom).is_err());
    }

    #[test]
    fn test_ensure_volumes_consistent_gifti_ok() {
        let overlay = OverlaySource::GiftiFunc(vec![vec![0.0; 100], vec![0.0; 100], vec![0.0; 100]]);

        assert!(ensure_volumes_consistent(&overlay).is_ok());
    }

    #[test]
    fn test_ensure_volumes_consistent_gifti_mismatch() {
        let overlay = OverlaySource::GiftiFunc(vec![vec![0.0; 100], vec![0.0; 50]]);

        let result = ensure_volumes_consistent(&overlay);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_volumes_consistent_empty() {
        let overlay = OverlaySource::GiftiFunc(vec![]);
        assert!(ensure_volumes_consistent(&overlay).is_ok());
    }

    #[test]
    fn test_ensure_volumes_consistent_curv() {
        let overlay = OverlaySource::FreeSurferCurv(vec![0.0; 100]);
        assert!(ensure_volumes_consistent(&overlay).is_ok());
    }
}
