//! Round-trip and consistency tests for io_formats.
//!
//! These tests verify that:
//! - Reading from path vs bytes produces identical results
//! - Format detection is consistent between path and bytes APIs
//! - Parcellation structures are consistent across formats

use io_formats::{
    detect_format, detect_format_from_bytes, load_overlay_bytes, load_surface, load_surface_bytes,
    FileFormat, SurfaceSource,
};
use io_formats::statistics::Hemisphere;

// ============================================================================
// Format Detection Consistency Tests
// ============================================================================

#[test]
fn format_detection_consistent_freesurfer_surface() {
    use io_formats::freesurfer_real::LH_PIAL_PATH;
    let path = std::path::Path::new(LH_PIAL_PATH);
    if !path.exists() {
        return;
    }

    // Detect from path
    let format_from_path = detect_format(path);

    // Detect from bytes
    let bytes = std::fs::read(path).unwrap();
    let format_from_bytes = detect_format_from_bytes(&bytes);

    assert_eq!(
        format_from_path, format_from_bytes,
        "Format detection should be consistent between path and bytes"
    );
    assert_eq!(format_from_path, FileFormat::FreeSurferSurface);
}

#[test]
fn format_detection_consistent_freesurfer_curv() {
    use io_formats::freesurfer_real::LH_CURV_PATH;
    let path = std::path::Path::new(LH_CURV_PATH);
    if !path.exists() {
        return;
    }

    let format_from_path = detect_format(path);
    let bytes = std::fs::read(path).unwrap();
    let format_from_bytes = detect_format_from_bytes(&bytes);

    assert_eq!(format_from_path, format_from_bytes);
    assert_eq!(format_from_path, FileFormat::FreeSurferCurv);
}

#[test]
fn format_detection_consistent_gifti() {
    use io_formats::gifti_real::GIFTI_SURFACE_ASCII_PATH;
    let path = std::path::Path::new(GIFTI_SURFACE_ASCII_PATH);
    if !path.exists() {
        return;
    }

    let format_from_path = detect_format(path);
    let bytes = std::fs::read(path).unwrap();
    let format_from_bytes = detect_format_from_bytes(&bytes);

    assert_eq!(format_from_path, format_from_bytes);
    assert_eq!(format_from_path, FileFormat::Gifti);
}

#[test]
fn format_detection_consistent_nifti() {
    use io_formats::nifti_real::NIFTI_3D_FLOAT32_PATH;
    let path = std::path::Path::new(NIFTI_3D_FLOAT32_PATH);
    if !path.exists() {
        return;
    }

    let format_from_path = detect_format(path);
    let bytes = std::fs::read(path).unwrap();
    let format_from_bytes = detect_format_from_bytes(&bytes);

    assert_eq!(format_from_path, format_from_bytes);
    assert_eq!(format_from_path, FileFormat::Nifti);
}

// ============================================================================
// Surface Loading Consistency Tests
// ============================================================================

#[test]
fn surface_loading_consistent_freesurfer() {
    use io_formats::freesurfer_real::LH_PIAL_PATH;
    let path = std::path::Path::new(LH_PIAL_PATH);
    if !path.exists() {
        return;
    }

    // Load from path
    let source_from_path = load_surface(path).unwrap();

    // Load from bytes
    let bytes = std::fs::read(path).unwrap();
    let format = detect_format_from_bytes(&bytes);
    let source_from_bytes = load_surface_bytes(&bytes, format, Hemisphere::Left).unwrap();

    // Compare geometry
    let (SurfaceSource::FreeSurfer(geom1), SurfaceSource::FreeSurfer(geom2)) =
        (source_from_path, source_from_bytes)
    else {
        panic!("Expected FreeSurfer surfaces");
    };

    assert_eq!(
        geom1.vertices.len(),
        geom2.vertices.len(),
        "Vertex count should match"
    );
    assert_eq!(
        geom1.indices.len(),
        geom2.indices.len(),
        "Triangle count should match"
    );
    assert_eq!(
        geom1.normals.len(),
        geom2.normals.len(),
        "Normal count should match"
    );

    // Verify first few vertices match exactly
    for i in 0..10.min(geom1.vertices.len()) {
        for j in 0..3 {
            assert!(
                (geom1.vertices[i][j] - geom2.vertices[i][j]).abs() < 1e-6,
                "Vertex {}[{}] mismatch: {} vs {}",
                i,
                j,
                geom1.vertices[i][j],
                geom2.vertices[i][j]
            );
        }
    }

    // Verify first few indices match exactly
    for i in 0..10.min(geom1.indices.len()) {
        assert_eq!(
            geom1.indices[i], geom2.indices[i],
            "Triangle {} mismatch",
            i
        );
    }
}

#[test]
fn surface_loading_consistent_gifti() {
    use io_formats::gifti_real::GIFTI_SURFACE_ASCII_PATH;
    let path = std::path::Path::new(GIFTI_SURFACE_ASCII_PATH);
    if !path.exists() {
        return;
    }

    // Load from path
    let source_from_path = load_surface(path).unwrap();

    // Load from bytes
    let bytes = std::fs::read(path).unwrap();
    let format = detect_format_from_bytes(&bytes);
    let source_from_bytes = load_surface_bytes(&bytes, format, Hemisphere::Left).unwrap();

    let (SurfaceSource::Gifti(geom1), SurfaceSource::Gifti(geom2)) =
        (source_from_path, source_from_bytes)
    else {
        panic!("Expected GIFTI surfaces");
    };

    assert_eq!(geom1.vertices.len(), geom2.vertices.len());
    assert_eq!(geom1.indices.len(), geom2.indices.len());

    // Verify vertices match
    for i in 0..geom1.vertices.len() {
        for j in 0..3 {
            assert!(
                (geom1.vertices[i][j] - geom2.vertices[i][j]).abs() < 1e-6,
                "Vertex {}[{}] mismatch",
                i,
                j
            );
        }
    }
}

// ============================================================================
// Overlay Loading Consistency Tests
// ============================================================================

#[test]
fn overlay_loading_consistent_freesurfer_curv() {
    use io_formats::freesurfer_real::LH_CURV_PATH;
    use io_formats::{load_overlay, OverlaySource};

    let path = std::path::Path::new(LH_CURV_PATH);
    if !path.exists() {
        return;
    }

    // Load from path
    let source_from_path = load_overlay(path).unwrap();

    // Load from bytes
    let bytes = std::fs::read(path).unwrap();
    let format = detect_format_from_bytes(&bytes);
    let source_from_bytes = load_overlay_bytes(&bytes, format).unwrap();

    let (OverlaySource::FreeSurferCurv(v1), OverlaySource::FreeSurferCurv(v2)) =
        (source_from_path, source_from_bytes)
    else {
        panic!("Expected FreeSurferCurv overlays");
    };

    assert_eq!(v1.len(), v2.len(), "Value count should match");

    // Verify values match exactly
    for i in 0..v1.len() {
        assert!(
            (v1[i] - v2[i]).abs() < 1e-6,
            "Value {} mismatch: {} vs {}",
            i,
            v1[i],
            v2[i]
        );
    }
}

#[test]
fn overlay_loading_consistent_gifti_func() {
    use io_formats::gifti_real::GIFTI_FUNC_PATH;
    use io_formats::{load_overlay, OverlaySource};

    let path = std::path::Path::new(GIFTI_FUNC_PATH);
    if !path.exists() {
        return;
    }

    let source_from_path = load_overlay(path).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let format = detect_format_from_bytes(&bytes);
    let source_from_bytes = load_overlay_bytes(&bytes, format).unwrap();

    let (OverlaySource::GiftiFunc(v1), OverlaySource::GiftiFunc(v2)) =
        (source_from_path, source_from_bytes)
    else {
        panic!("Expected GiftiFunc overlays");
    };

    assert_eq!(v1.len(), v2.len(), "Volume count should match");

    for (vol_idx, (vol1, vol2)) in v1.iter().zip(v2.iter()).enumerate() {
        assert_eq!(
            vol1.len(),
            vol2.len(),
            "Volume {} vertex count should match",
            vol_idx
        );

        for i in 0..vol1.len() {
            assert!(
                (vol1[i] - vol2[i]).abs() < 1e-6,
                "Volume {} value {} mismatch",
                vol_idx,
                i
            );
        }
    }
}

// ============================================================================
// Parcellation Consistency Tests
// ============================================================================

#[test]
fn parcellation_annot_consistency() {
    use io_formats::freesurfer_real::LH_ANNOT_PATH;
    use io_formats::{read_real_annot, read_real_annot_bytes};

    let path = std::path::Path::new(LH_ANNOT_PATH);
    if !path.exists() {
        return;
    }

    // Load from path
    let parc_from_path = read_real_annot(path, Hemisphere::Left).unwrap();

    // Load from bytes
    let bytes = std::fs::read(path).unwrap();
    let parc_from_bytes = read_real_annot_bytes(&bytes, Hemisphere::Left).unwrap();

    // Labels should match exactly
    assert_eq!(
        parc_from_path.labels.len(),
        parc_from_bytes.labels.len(),
        "Label count should match"
    );
    for i in 0..parc_from_path.labels.len() {
        assert_eq!(
            parc_from_path.labels[i], parc_from_bytes.labels[i],
            "Label {} mismatch",
            i
        );
    }

    // Regions should match
    assert_eq!(
        parc_from_path.regions.len(),
        parc_from_bytes.regions.len(),
        "Region count should match"
    );
    for (r1, r2) in parc_from_path.regions.iter().zip(parc_from_bytes.regions.iter()) {
        assert_eq!(r1.id, r2.id, "Region ID mismatch");
        assert_eq!(r1.name, r2.name, "Region name mismatch");
        assert_eq!(r1.rgba, r2.rgba, "Region color mismatch");
    }
}

#[test]
fn parcellation_gifti_label_consistency() {
    use io_formats::gifti_real::GIFTI_LABEL_PATH;
    use io_formats::{read_gifti_label, read_gifti_label_bytes};

    let path = std::path::Path::new(GIFTI_LABEL_PATH);
    if !path.exists() {
        return;
    }

    let parc_from_path = read_gifti_label(path).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let parc_from_bytes = read_gifti_label_bytes(&bytes, Hemisphere::Left).unwrap();

    // Labels should match exactly
    assert_eq!(parc_from_path.labels.len(), parc_from_bytes.labels.len());
    for i in 0..parc_from_path.labels.len() {
        assert_eq!(parc_from_path.labels[i], parc_from_bytes.labels[i]);
    }

    // Regions should match
    assert_eq!(parc_from_path.regions.len(), parc_from_bytes.regions.len());
    for (r1, r2) in parc_from_path.regions.iter().zip(parc_from_bytes.regions.iter()) {
        assert_eq!(r1.id, r2.id);
        assert_eq!(r1.name, r2.name);
        assert_eq!(r1.rgba, r2.rgba);
    }
}

#[test]
fn parcellation_formats_produce_same_structure() {
    use io_formats::freesurfer_real::LH_ANNOT_PATH;
    use io_formats::gifti_real::GIFTI_LABEL_PATH;
    use io_formats::{read_gifti_label, read_real_annot};

    let annot_path = std::path::Path::new(LH_ANNOT_PATH);
    let label_path = std::path::Path::new(GIFTI_LABEL_PATH);

    if !annot_path.exists() || !label_path.exists() {
        return;
    }

    let annot = read_real_annot(annot_path, Hemisphere::Left).unwrap();
    let label = read_gifti_label(label_path).unwrap();

    // Both fixtures have 100 vertices and 5 regions (by design)
    assert_eq!(
        annot.labels.len(),
        label.labels.len(),
        "Both formats should have same vertex count"
    );
    assert_eq!(
        annot.regions.len(),
        label.regions.len(),
        "Both formats should have same region count"
    );

    // Both should produce valid Parcellation structures with all required fields
    assert!(annot.n_vertices() > 0);
    assert!(annot.n_regions() > 0);
    assert!(label.n_vertices() > 0);
    assert!(label.n_regions() > 0);

    // Both should support region lookup
    assert!(annot.get_region(0).is_some());
    assert!(label.get_region(0).is_some());

    // Both should have 'unknown' as region 0
    assert_eq!(annot.get_region(0).unwrap().name, "unknown");
    assert_eq!(label.get_region(0).unwrap().name, "unknown");
}

// ============================================================================
// GIFTI Encoding Consistency Tests
// ============================================================================

#[test]
fn gifti_ascii_vs_b64_surface_equivalent() {
    use io_formats::gifti_real::{GIFTI_SURFACE_ASCII_PATH, GIFTI_SURFACE_B64_PATH};
    use io_formats::gifti_real::{read_real_gifti_surface};

    let ascii_path = std::path::Path::new(GIFTI_SURFACE_ASCII_PATH);
    let b64_path = std::path::Path::new(GIFTI_SURFACE_B64_PATH);

    if !ascii_path.exists() || !b64_path.exists() {
        return;
    }

    let ascii_geom = read_real_gifti_surface(ascii_path).unwrap();
    let b64_geom = read_real_gifti_surface(b64_path).unwrap();

    // Vertex counts should match
    assert_eq!(ascii_geom.vertices.len(), b64_geom.vertices.len());
    assert_eq!(ascii_geom.indices.len(), b64_geom.indices.len());

    // All vertices should be identical (within floating point tolerance)
    for i in 0..ascii_geom.vertices.len() {
        for j in 0..3 {
            assert!(
                (ascii_geom.vertices[i][j] - b64_geom.vertices[i][j]).abs() < 1e-5,
                "Vertex {}[{}] mismatch between ASCII and B64",
                i,
                j
            );
        }
    }

    // All indices should be identical
    for i in 0..ascii_geom.indices.len() {
        assert_eq!(
            ascii_geom.indices[i], b64_geom.indices[i],
            "Triangle {} mismatch between ASCII and B64",
            i
        );
    }
}

#[test]
fn gifti_ascii_vs_b64_shape_equivalent() {
    use io_formats::gifti_real::{GIFTI_SHAPE_ASCII_PATH, GIFTI_SHAPE_B64_PATH};
    use io_formats::gifti_real::read_real_gifti_func;

    let ascii_path = std::path::Path::new(GIFTI_SHAPE_ASCII_PATH);
    let b64_path = std::path::Path::new(GIFTI_SHAPE_B64_PATH);

    if !ascii_path.exists() || !b64_path.exists() {
        return;
    }

    let ascii_volumes = read_real_gifti_func(ascii_path).unwrap();
    let b64_volumes = read_real_gifti_func(b64_path).unwrap();

    assert_eq!(ascii_volumes.len(), b64_volumes.len());

    for (vol_idx, (ascii_vol, b64_vol)) in ascii_volumes.iter().zip(b64_volumes.iter()).enumerate() {
        assert_eq!(ascii_vol.len(), b64_vol.len());

        for i in 0..ascii_vol.len() {
            assert!(
                (ascii_vol[i] - b64_vol[i]).abs() < 1e-5,
                "Volume {} value {} mismatch between ASCII and B64",
                vol_idx,
                i
            );
        }
    }
}
