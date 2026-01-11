//! Integration tests for byte-based loading APIs.
//!
//! These tests verify that `load_surface_bytes`, `load_overlay_bytes`, and
//! `load_overlay_from_bytes` work correctly with real fixture data.

use io_formats::{
    detect_format_from_bytes, load_overlay_bytes, load_overlay_from_bytes, load_surface_bytes,
    FileFormat, OverlaySource, SurfaceSource,
};
use io_formats::statistics::Hemisphere;

// ============================================================================
// Surface Loading Tests
// ============================================================================

#[test]
fn load_surface_bytes_freesurfer_lh() {
    use io_formats::freesurfer_real::LH_PIAL_PATH;
    let path = std::path::Path::new(LH_PIAL_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", LH_PIAL_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);
    assert_eq!(format, FileFormat::FreeSurferSurface);

    let source = load_surface_bytes(&bytes, format, Hemisphere::Left)
        .expect("Failed to load surface");

    let SurfaceSource::FreeSurfer(geom) = source else {
        panic!("Expected FreeSurfer source");
    };

    assert_eq!(geom.hemisphere, Hemisphere::Left);
    assert!(geom.vertices.len() > 1000, "Expected many vertices");
    assert!(geom.indices.len() > 1000, "Expected many triangles");
    assert_eq!(
        geom.normals.len(),
        geom.vertices.len(),
        "Normals count should match vertices"
    );

    // Verify normals are normalized
    for normal in &geom.normals {
        let len = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        assert!(
            (len - 1.0).abs() < 0.01,
            "Normal should be unit length, got {}",
            len
        );
    }
}

#[test]
fn load_surface_bytes_freesurfer_rh() {
    use io_formats::freesurfer_real::RH_PIAL_PATH;
    let path = std::path::Path::new(RH_PIAL_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", RH_PIAL_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);

    let source = load_surface_bytes(&bytes, format, Hemisphere::Right)
        .expect("Failed to load surface");

    let SurfaceSource::FreeSurfer(geom) = source else {
        panic!("Expected FreeSurfer source");
    };

    assert_eq!(geom.hemisphere, Hemisphere::Right);
    assert!(geom.vertices.len() > 1000, "Expected many vertices");
}

#[test]
fn load_surface_bytes_gifti_ascii() {
    use io_formats::gifti_real::GIFTI_SURFACE_ASCII_PATH;
    let path = std::path::Path::new(GIFTI_SURFACE_ASCII_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", GIFTI_SURFACE_ASCII_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);
    assert_eq!(format, FileFormat::Gifti);

    let source = load_surface_bytes(&bytes, format, Hemisphere::Left)
        .expect("Failed to load GIFTI surface");

    let SurfaceSource::Gifti(geom) = source else {
        panic!("Expected GIFTI source");
    };

    // Ground truth from gifti_real.rs
    assert_eq!(geom.vertices.len(), 8, "Expected 8 vertices (cube)");
    assert_eq!(geom.indices.len(), 12, "Expected 12 triangles");

    // Verify first vertex
    let v0 = geom.vertices[0];
    assert!((v0[0] - (-1.0)).abs() < 0.001);
    assert!((v0[1] - (-1.0)).abs() < 0.001);
    assert!((v0[2] - 1.0).abs() < 0.001);
}

#[test]
fn load_surface_bytes_gifti_b64() {
    use io_formats::gifti_real::GIFTI_SURFACE_B64_PATH;
    let path = std::path::Path::new(GIFTI_SURFACE_B64_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", GIFTI_SURFACE_B64_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);

    let source = load_surface_bytes(&bytes, format, Hemisphere::Right)
        .expect("Failed to load GIFTI B64 surface");

    let SurfaceSource::Gifti(geom) = source else {
        panic!("Expected GIFTI source");
    };

    assert_eq!(geom.vertices.len(), 8);
    assert_eq!(geom.indices.len(), 12);
}

// ============================================================================
// Overlay Loading Tests
// ============================================================================

#[test]
fn load_overlay_bytes_freesurfer_curv() {
    use io_formats::freesurfer_real::LH_CURV_PATH;
    let path = std::path::Path::new(LH_CURV_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", LH_CURV_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);
    assert_eq!(format, FileFormat::FreeSurferCurv);

    let source = load_overlay_bytes(&bytes, format).expect("Failed to load curv");

    let OverlaySource::FreeSurferCurv(values) = source else {
        panic!("Expected FreeSurferCurv source");
    };

    // Test fixture has 100 vertices
    assert_eq!(values.len(), 100, "Expected 100 curvature values in fixture");

    // Curvature values should be in a reasonable range (test file has positive values)
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(min.is_finite(), "Should have finite min");
    assert!(max.is_finite(), "Should have finite max");
    assert!(max > min, "Max should be greater than min");
}

#[test]
fn load_overlay_bytes_gifti_func() {
    use io_formats::gifti_real::{
        GIFTI_FUNC_N_TIMEPOINTS, GIFTI_FUNC_N_VERTICES, GIFTI_FUNC_PATH,
    };
    let path = std::path::Path::new(GIFTI_FUNC_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", GIFTI_FUNC_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);
    assert_eq!(format, FileFormat::Gifti);

    let source = load_overlay_bytes(&bytes, format).expect("Failed to load GIFTI func");

    let OverlaySource::GiftiFunc(volumes) = source else {
        panic!("Expected GiftiFunc source");
    };

    assert_eq!(volumes.len(), GIFTI_FUNC_N_TIMEPOINTS);
    for vol in &volumes {
        assert_eq!(vol.len(), GIFTI_FUNC_N_VERTICES);
    }
}

#[test]
fn load_overlay_bytes_nifti() {
    use io_formats::nifti_real::NIFTI_3D_FLOAT32_PATH;
    let path = std::path::Path::new(NIFTI_3D_FLOAT32_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", NIFTI_3D_FLOAT32_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");
    let format = detect_format_from_bytes(&bytes);
    assert_eq!(format, FileFormat::Nifti);

    let source = load_overlay_bytes(&bytes, format).expect("Failed to load NIfTI");

    let OverlaySource::Nifti(nifti) = source else {
        panic!("Expected Nifti source");
    };

    assert!(!nifti.data.is_empty(), "NIfTI data should not be empty");
    // Verify dimensions are set
    let total = nifti.dims[0] as usize
        * nifti.dims[1].max(1) as usize
        * nifti.dims[2].max(1) as usize;
    assert!(total > 0, "NIfTI should have positive dimensions");
}

// ============================================================================
// Auto-detect Loading Tests (load_overlay_from_bytes)
// ============================================================================

#[test]
fn load_overlay_from_bytes_auto_detect_curv() {
    use io_formats::freesurfer_real::LH_CURV_PATH;
    let path = std::path::Path::new(LH_CURV_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", LH_CURV_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");

    // Use auto-detect path
    let source = load_overlay_from_bytes(&bytes).expect("Failed to auto-detect curv");

    let OverlaySource::FreeSurferCurv(values) = source else {
        panic!("Expected FreeSurferCurv from auto-detect");
    };

    // Test fixture has 100 vertices
    assert_eq!(values.len(), 100);
}

#[test]
fn load_overlay_from_bytes_auto_detect_nifti() {
    use io_formats::nifti_real::NIFTI_3D_FLOAT32_PATH;
    let path = std::path::Path::new(NIFTI_3D_FLOAT32_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", NIFTI_3D_FLOAT32_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");

    let source = load_overlay_from_bytes(&bytes).expect("Failed to auto-detect NIfTI");

    assert!(
        matches!(source, OverlaySource::Nifti(_)),
        "Expected Nifti from auto-detect"
    );
}

#[test]
fn load_overlay_from_bytes_auto_detect_gifti() {
    use io_formats::gifti_real::GIFTI_FUNC_PATH;
    let path = std::path::Path::new(GIFTI_FUNC_PATH);
    if !path.exists() {
        eprintln!("Skipping test: {} not found", GIFTI_FUNC_PATH);
        return;
    }

    let bytes = std::fs::read(path).expect("Failed to read fixture");

    let source = load_overlay_from_bytes(&bytes).expect("Failed to auto-detect GIFTI");

    assert!(
        matches!(source, OverlaySource::GiftiFunc(_)),
        "Expected GiftiFunc from auto-detect"
    );
}

#[test]
fn load_overlay_from_bytes_fails_unknown_format() {
    let bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let result = load_overlay_from_bytes(&bytes);
    assert!(result.is_err(), "Should fail on unknown format");
}

// ============================================================================
// OverlaySource::to_statistic_data conversion tests
// ============================================================================

#[test]
fn overlay_to_statistic_data_curv() {
    use io_formats::freesurfer_real::LH_CURV_PATH;
    let path = std::path::Path::new(LH_CURV_PATH);
    if !path.exists() {
        return;
    }

    let bytes = std::fs::read(path).unwrap();
    let source = load_overlay_from_bytes(&bytes).unwrap();

    let stats = source.to_statistic_data();
    assert_eq!(stats.n_volumes, 1);
    assert!(stats.n_vertices > 0);
    assert!(stats.global_min < stats.global_max);
    assert_eq!(stats.volume_ranges.len(), 1);
}

#[test]
fn overlay_to_statistic_data_gifti_func() {
    use io_formats::gifti_real::{GIFTI_FUNC_N_TIMEPOINTS, GIFTI_FUNC_PATH};
    let path = std::path::Path::new(GIFTI_FUNC_PATH);
    if !path.exists() {
        return;
    }

    let bytes = std::fs::read(path).unwrap();
    let source = load_overlay_from_bytes(&bytes).unwrap();

    let stats = source.to_statistic_data();
    assert_eq!(stats.n_volumes, GIFTI_FUNC_N_TIMEPOINTS);
    assert_eq!(stats.volume_ranges.len(), GIFTI_FUNC_N_TIMEPOINTS);
}
