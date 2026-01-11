//! Data re-exports for the viewer application.
//!
//! This module exposes format-level types from the `io_formats` crate so that
//! existing code can continue to refer to `crate::data::*`.

pub use io_formats::geometry::BrainGeometry;
pub use io_formats::loader::*;
pub use io_formats::metadata::*;
pub use io_formats::statistics::{
    Analysis, Hemisphere, NanHandling, Statistic, StatisticData, StatisticMetadata, VolumeLabel,
};

// ============================================================================
// io_formats-based surface loading (behind feature flag)
// ============================================================================

/// Load a surface using io_formats parsers (real FreeSurfer format support).
///
/// This helper loads surfaces from real FreeSurfer files (e.g., lh.pial, rh.pial)
/// using the io_formats crate's parsers. It expects files to be located at
/// `{base_path}/{hemi}.pial` where `hemi` is "lh" or "rh".
///
/// # Arguments
/// * `base_path` - Base directory containing the surface files
/// * `hemisphere` - Which hemisphere to load
///
/// # Returns
/// The loaded brain geometry, or an error string if loading fails.
#[cfg(feature = "io-formats-loader")]
pub async fn load_surface_from_io_formats(
    base_path: &str,
    hemisphere: Hemisphere,
) -> Result<BrainGeometry, String> {
    use io_formats::{load_surface, SurfaceSource};
    use std::path::PathBuf;

    // Construct the filename based on hemisphere
    let hemi_str = match hemisphere {
        Hemisphere::Left => "lh",
        Hemisphere::Right => "rh",
    };

    let path = PathBuf::from(base_path).join(format!("{hemi_str}.pial"));

    let src = load_surface(&path).map_err(|e| format!("format error: {}", e))?;
    match src {
        SurfaceSource::FreeSurfer(geom) => Ok(geom),
        SurfaceSource::Gifti(geom) => Ok(geom),
    }
}

// ============================================================================
// io_formats-based overlay loading (behind feature flag)
// ============================================================================

/// Overlay format type for specifying what kind of file to load.
#[cfg(feature = "io-formats-overlay-loader")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayFormat {
    /// BLMM statistics (.bin.gz files from BLMM analysis)
    BlmmStatistics,
    /// FreeSurfer curvature (.curv, .thickness, .sulc)
    FreeSurferCurv,
    /// GIFTI functional (.func.gii)
    GiftiFunc,
    /// NIfTI overlay (.nii, .nii.gz)
    Nifti,
    /// Auto-detect based on file extension
    Auto,
}

/// Load an overlay using io_formats parsers.
///
/// This helper loads overlays from various formats and converts them to
/// StatisticData suitable for rendering. It supports:
///
/// - BLMM statistics (.bin.gz with BRS1 magic)
/// - FreeSurfer curvature (.curv, .thickness, .sulc)
/// - GIFTI functional data (.func.gii)
/// - NIfTI scalar overlays (.nii, .nii.gz)
///
/// # Arguments
/// * `path` - Path to the overlay file
///
/// # Returns
/// The loaded statistic data, or an error string if loading fails.
#[cfg(feature = "io-formats-overlay-loader")]
pub fn load_overlay_from_io_formats(path: &std::path::Path) -> Result<StatisticData, String> {
    use io_formats::load_overlay;

    let src = load_overlay(path).map_err(|e| format!("format error: {}", e))?;

    // Convert to StatisticData using the unified conversion method
    Ok(src.to_statistic_data())
}

/// Load an overlay from a base path with analysis and statistic parameters.
///
/// This helper constructs the appropriate path for BLMM-style statistics
/// based on the base path, hemisphere, analysis name, and statistic type.
///
/// # Arguments
/// * `base_path` - Base directory containing the statistics
/// * `hemisphere` - Which hemisphere to load
/// * `analysis` - Analysis name (e.g., "main_effect")
/// * `statistic` - Statistic type (e.g., "t_stat")
///
/// # Returns
/// The loaded statistic data, or an error string if loading fails.
#[cfg(feature = "io-formats-overlay-loader")]
pub fn load_blmm_overlay(
    base_path: &str,
    hemisphere: Hemisphere,
    analysis: &str,
    statistic: &str,
) -> Result<StatisticData, String> {
    use std::path::PathBuf;

    let hemi_str = match hemisphere {
        Hemisphere::Left => "lh",
        Hemisphere::Right => "rh",
    };

    // BLMM output structure: {base_path}/{analysis}/{hemi}.{statistic}.bin.gz
    let path = PathBuf::from(base_path)
        .join(analysis)
        .join(format!("{hemi_str}.{statistic}.bin.gz"));

    load_overlay_from_io_formats(&path)
}

/// Load a FreeSurfer curvature overlay.
///
/// # Arguments
/// * `base_path` - Base directory containing the surface files
/// * `hemisphere` - Which hemisphere to load
/// * `overlay_type` - Type of overlay ("curv", "thickness", "sulc", etc.)
///
/// # Returns
/// The loaded statistic data, or an error string if loading fails.
#[cfg(feature = "io-formats-overlay-loader")]
pub fn load_curv_overlay(
    base_path: &str,
    hemisphere: Hemisphere,
    overlay_type: &str,
) -> Result<StatisticData, String> {
    use std::path::PathBuf;

    let hemi_str = match hemisphere {
        Hemisphere::Left => "lh",
        Hemisphere::Right => "rh",
    };

    // FreeSurfer structure: {base_path}/{hemi}.{overlay_type}
    let path = PathBuf::from(base_path).join(format!("{hemi_str}.{overlay_type}"));

    load_overlay_from_io_formats(&path)
}

/// Validate that overlay data is compatible with a geometry.
///
/// Checks that the number of vertices in the overlay matches the geometry.
///
/// # Arguments
/// * `overlay` - The loaded statistic data
/// * `geometry` - The brain geometry to validate against
///
/// # Returns
/// Ok if compatible, or an error describing the mismatch.
#[cfg(feature = "io-formats-overlay-loader")]
pub fn validate_overlay_geometry(
    overlay: &StatisticData,
    geometry: &BrainGeometry,
) -> Result<(), String> {
    if overlay.n_vertices != geometry.vertices.len() {
        return Err(format!(
            "Overlay/geometry vertex count mismatch: overlay has {} vertices, geometry has {}",
            overlay.n_vertices,
            geometry.vertices.len()
        ));
    }
    Ok(())
}

// ============================================================================
// Web-based async loaders using gloo-net (WASM only, behind feature flag)
// ============================================================================
//
// These functions are only available when:
// - The `web-loaders` feature is enabled
// - Compiling for wasm32 target
//
// Enable by adding `web-loaders` to your feature flags in Cargo.toml or
// via command line: `cargo build --features web-loaders`

#[cfg(all(feature = "web-loaders", target_arch = "wasm32"))]
mod web_loaders {
    use super::{BrainGeometry, Hemisphere, StatisticData};

    /// Fetch raw bytes from a URL.
    ///
    /// This is the underlying fetch function used by higher-level loaders.
    /// It handles gzip-compressed responses automatically if the URL ends with `.gz`.
    pub(super) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
        use gloo_net::http::Request;

        let response = Request::get(url)
            .send()
            .await
            .map_err(|e| format!("Network error: Unable to connect to server ({e})"))?;

        if !response.ok() {
            let status = response.status();
            let msg = match status {
                404 => format!("Data not found: The file '{}' does not exist on the server", url),
                403 => format!("Access denied: You don't have permission to access '{}'", url),
                500..=599 => format!("Server error: The server encountered an error while processing your request (HTTP {})", status),
                _ => format!("HTTP error {}: {}", status, response.status_text()),
            };
            return Err(msg);
        }

        let bytes = response.binary().await.map_err(|e| {
            format!(
                "Failed to read response: The server response was incomplete or corrupted ({e})"
            )
        })?;

        // Decompress if gzip-compressed
        if url.ends_with(".gz") {
            use flate2::read::GzDecoder;
            use std::io::Read;

            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| format!("Decompression failed: The file appears to be corrupted or is not a valid gzip archive ({e})"))?;
            Ok(decompressed)
        } else {
            Ok(bytes)
        }
    }

    /// Load a brain surface from a URL.
    ///
    /// Fetches the surface file and parses it using the byte-based API.
    /// Supports FreeSurfer surfaces and GIFTI surfaces.
    ///
    /// # Arguments
    /// * `url` - URL to fetch the surface from
    /// * `hemisphere` - Which hemisphere this surface belongs to
    ///
    /// # Returns
    /// The loaded brain geometry, or an error string if loading fails.
    ///
    /// # Example
    /// ```ignore
    /// let geom = fetch_surface("/data/surfaces/lh.pial", Hemisphere::Left).await?;
    /// ```
    pub async fn fetch_surface(url: &str, hemisphere: Hemisphere) -> Result<BrainGeometry, String> {
        let bytes = fetch_bytes(url).await?;
        let format = io_formats::detect_format_from_bytes(&bytes);

        let source = io_formats::load_surface_bytes(&bytes, format, hemisphere)
            .map_err(|e| {
                // Provide more user-friendly error messages based on error type
                let err_str = e.to_string();
                if err_str.contains("magic") || err_str.contains("Magic") {
                    format!("Invalid surface file: The file does not appear to be a valid FreeSurfer or GIFTI surface. Expected magic bytes not found.")
                } else if err_str.contains("unexpected end") || err_str.contains("premature") {
                    format!("Incomplete surface file: The file appears to be truncated or incomplete.")
                } else {
                    format!("Unable to read surface file: {e}")
                }
            })?;

        match source {
            io_formats::SurfaceSource::FreeSurfer(geom) => Ok(geom),
            io_formats::SurfaceSource::Gifti(geom) => Ok(geom),
        }
    }

    /// Load an overlay from a URL.
    ///
    /// Fetches the overlay file and parses it using the byte-based API.
    /// Supports BLMM statistics, FreeSurfer curvature, GIFTI functional, and NIfTI.
    ///
    /// # Arguments
    /// * `url` - URL to fetch the overlay from
    ///
    /// # Returns
    /// The loaded statistic data, or an error string if loading fails.
    ///
    /// # Example
    /// ```ignore
    /// let stats = fetch_overlay("/data/overlays/lh.curv").await?;
    /// ```
    pub async fn fetch_overlay(url: &str) -> Result<StatisticData, String> {
        let bytes = fetch_bytes(url).await?;
        let format = io_formats::detect_format_from_bytes(&bytes);

        let source = io_formats::load_overlay_bytes(&bytes, format)
            .map_err(|e| {
                // Provide more user-friendly error messages based on error type
                let err_str = e.to_string();
                if err_str.contains("magic") || err_str.contains("Magic") || err_str.contains("BRS") {
                    format!("Invalid statistics file: The file format is not recognized. Expected BLMM statistics (.bin.gz), FreeSurfer curvature, or GIFTI overlay.")
                } else if err_str.contains("unexpected end") || err_str.contains("premature") {
                    format!("Incomplete statistics file: The file appears to be truncated or incomplete.")
                } else if err_str.contains("version") {
                    format!("Unsupported file version: The statistics file uses an unsupported format version.")
                } else {
                    format!("Unable to read statistics file: {e}")
                }
            })?;

        Ok(source.to_statistic_data())
    }

    /// Load both hemispheres' surfaces from a base URL.
    ///
    /// This is a convenience function that fetches both lh and rh surfaces
    /// in parallel and returns them together.
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the surfaces (e.g., "/data/surfaces/")
    /// * `surface_type` - Type of surface (e.g., "pial", "inflated")
    ///
    /// # Returns
    /// A tuple of (left_geometry, right_geometry), or an error if either fails.
    ///
    /// # Example
    /// ```ignore
    /// let (lh, rh) = fetch_both_surfaces("/data/surfaces", "pial").await?;
    /// ```
    pub async fn fetch_both_surfaces(
        base_url: &str,
        surface_type: &str,
    ) -> Result<(BrainGeometry, BrainGeometry), String> {
        let base = base_url.trim_end_matches('/');

        let lh_url = format!("{}/lh.{}", base, surface_type);
        let rh_url = format!("{}/rh.{}", base, surface_type);

        // Fetch both in parallel using futures
        let (lh_result, rh_result) = futures::join!(
            fetch_surface(&lh_url, Hemisphere::Left),
            fetch_surface(&rh_url, Hemisphere::Right)
        );

        let lh = lh_result?;
        let rh = rh_result?;

        Ok((lh, rh))
    }

    /// Load an overlay for a specific hemisphere from a base URL.
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the overlays
    /// * `hemisphere` - Which hemisphere to load
    /// * `overlay_name` - Name/type of overlay (e.g., "curv", "t_stat.bin.gz")
    ///
    /// # Returns
    /// The loaded statistic data, or an error if loading fails.
    ///
    /// # Example
    /// ```ignore
    /// let stats = fetch_hemisphere_overlay("/data/overlays", Hemisphere::Left, "curv").await?;
    /// ```
    pub async fn fetch_hemisphere_overlay(
        base_url: &str,
        hemisphere: Hemisphere,
        overlay_name: &str,
    ) -> Result<StatisticData, String> {
        let base = base_url.trim_end_matches('/');
        let hemi_str = hemisphere.as_str();
        let url = format!("{}/{}.{}", base, hemi_str, overlay_name);

        fetch_overlay(&url).await
    }

    /// Load overlays for both hemispheres from a base URL.
    ///
    /// This fetches both left and right hemisphere overlays in parallel.
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the overlays
    /// * `overlay_name` - Name/type of overlay (e.g., "curv", "t_stat.bin.gz")
    ///
    /// # Returns
    /// A tuple of (left_overlay, right_overlay), or an error if either fails.
    ///
    /// # Example
    /// ```ignore
    /// let (lh_stats, rh_stats) = fetch_both_overlays("/data/overlays", "curv").await?;
    /// ```
    pub async fn fetch_both_overlays(
        base_url: &str,
        overlay_name: &str,
    ) -> Result<(StatisticData, StatisticData), String> {
        let (lh_result, rh_result) = futures::join!(
            fetch_hemisphere_overlay(base_url, Hemisphere::Left, overlay_name),
            fetch_hemisphere_overlay(base_url, Hemisphere::Right, overlay_name)
        );

        let lh = lh_result?;
        let rh = rh_result?;

        Ok((lh, rh))
    }

    /// Load a parcellation (brain atlas) from a URL.
    ///
    /// Fetches the parcellation file and parses it using the byte-based API.
    /// Supports FreeSurfer .annot files and GIFTI .label.gii files.
    ///
    /// # Arguments
    /// * `url` - URL to fetch the parcellation from
    /// * `hemisphere` - Which hemisphere this parcellation belongs to
    ///
    /// # Returns
    /// The loaded parcellation, or an error string if loading fails.
    ///
    /// # Example
    /// ```ignore
    /// let parc = fetch_parcellation("/data/lh.aparc.annot", Hemisphere::Left).await?;
    /// println!("Atlas: {:?}, Regions: {}", parc.atlas_name, parc.n_regions());
    /// ```
    pub async fn fetch_parcellation(
        url: &str,
        hemisphere: io_formats::statistics::Hemisphere,
    ) -> Result<io_formats::parcellation::Parcellation, String> {
        let bytes = fetch_bytes(url).await?;

        // FreeSurfer .annot based on extension
        if url.ends_with(".annot") {
            return io_formats::read_real_annot_bytes(&bytes, hemisphere).map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("unexpected end") || err_str.contains("premature") {
                    "Incomplete parcellation file: The .annot file appears to be truncated."
                        .to_string()
                } else {
                    format!("Unable to read parcellation: {e}")
                }
            });
        }

        // Try GIFTI label (.label.gii) based on content
        let content = String::from_utf8_lossy(&bytes);
        if content.contains("<LabelTable>") || content.contains("NIFTI_INTENT_LABEL") {
            return io_formats::read_gifti_label_bytes(&bytes, hemisphere)
                .map_err(|e| format!("Unable to read GIFTI parcellation: {e}"));
        }

        Err("Unsupported parcellation format: Please use FreeSurfer .annot or GIFTI .label.gii files with LabelTable.".to_string())
    }

    /// Load parcellations for both hemispheres from a base URL.
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the parcellations
    /// * `atlas_name` - Name of the atlas (e.g., "aparc", "aparc.a2009s")
    /// * `format_ext` - File extension ("annot" or "label.gii")
    ///
    /// # Returns
    /// A tuple of (left_parcellation, right_parcellation).
    ///
    /// # Example
    /// ```ignore
    /// let (lh_parc, rh_parc) = fetch_both_parcellations(
    ///     "/data/parcellations",
    ///     "aparc",
    ///     "annot",
    /// ).await?;
    /// ```
    pub async fn fetch_both_parcellations(
        base_url: &str,
        atlas_name: &str,
        format_ext: &str,
    ) -> Result<
        (
            io_formats::parcellation::Parcellation,
            io_formats::parcellation::Parcellation,
        ),
        String,
    > {
        use io_formats::statistics::Hemisphere;

        let base = base_url.trim_end_matches('/');

        let lh_url = format!("{}/lh.{}.{}", base, atlas_name, format_ext);
        let rh_url = format!("{}/rh.{}.{}", base, atlas_name, format_ext);

        let (lh_result, rh_result) = futures::join!(
            fetch_parcellation(&lh_url, Hemisphere::Left),
            fetch_parcellation(&rh_url, Hemisphere::Right)
        );

        Ok((lh_result?, rh_result?))
    }
}

// Re-export web loaders at module level when feature is enabled
#[cfg(all(feature = "web-loaders", target_arch = "wasm32"))]
pub use web_loaders::{
    fetch_both_overlays, fetch_both_parcellations, fetch_both_surfaces, fetch_hemisphere_overlay,
    fetch_overlay, fetch_parcellation, fetch_surface,
};

// ============================================================================
// Example usage (web-loaders feature, WASM only)
// ============================================================================

/// Example: Load a complete brain visualization dataset.
///
/// This demonstrates the typical workflow for loading brain surfaces and
/// overlays from a web server. It:
/// 1. Fetches both hemisphere surfaces in parallel
/// 2. Fetches curvature overlays in parallel
/// 3. Validates that overlays match surfaces
///
/// # Example URL structure
/// ```text
/// /data/
///   surfaces/
///     lh.pial
///     rh.pial
///   overlays/
///     lh.curv
///     rh.curv
/// ```
///
/// # Usage
/// ```ignore
/// use brain_viewer::data::example_load_brain_data;
///
/// let result = example_load_brain_data("/data").await;
/// match result {
///     Ok((surfaces, overlays)) => {
///         let (lh_geom, rh_geom) = surfaces;
///         let (lh_curv, rh_curv) = overlays;
///         // Use the loaded data...
///     }
///     Err(e) => log::error!("Failed to load: {}", e),
/// }
/// ```
#[cfg(all(feature = "web-loaders", target_arch = "wasm32"))]
pub async fn example_load_brain_data(
    base_url: &str,
) -> Result<
    (
        (BrainGeometry, BrainGeometry),
        (StatisticData, StatisticData),
    ),
    String,
> {
    let base = base_url.trim_end_matches('/');

    // Step 1: Load both hemisphere surfaces
    let surfaces_url = format!("{}/surfaces", base);
    let (lh_surface, rh_surface) = fetch_both_surfaces(&surfaces_url, "pial").await?;

    // Step 2: Load curvature overlays
    let overlays_url = format!("{}/overlays", base);
    let (lh_curv, rh_curv) = fetch_both_overlays(&overlays_url, "curv").await?;

    // Step 3: Validate overlay/surface compatibility
    io_formats::ensure_stats_match_surface(&lh_curv, &lh_surface)
        .map_err(|e| format!("Left hemisphere validation failed: {}", e))?;
    io_formats::ensure_stats_match_surface(&rh_curv, &rh_surface)
        .map_err(|e| format!("Right hemisphere validation failed: {}", e))?;

    Ok(((lh_surface, rh_surface), (lh_curv, rh_curv)))
}
