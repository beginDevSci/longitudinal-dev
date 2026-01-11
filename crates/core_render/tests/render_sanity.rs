//! Render sanity tests for core_render.
//!
//! These tests verify that the core data structures and logic work correctly
//! without requiring actual GPU rendering (which needs WASM/WebGPU).

use core_render::scene::{Scene, Transform3D};
use core_render::orbit::OrbitController;
use glam::Vec3;

/// Test that Scene can manage multiple surfaces with transforms.
#[test]
fn test_scene_multi_surface() {
    let mut scene = Scene::new();

    // Add two surfaces (simulating left/right hemispheres)
    let _node1 = scene.add_surface(0); // Left hemisphere
    let _node2 = scene.add_surface(1); // Right hemisphere

    assert!(scene.has_surface(), "Scene should have surfaces");

    // Verify surfaces are iterable
    let surfaces: Vec<_> = scene.iter_surfaces().collect();
    assert_eq!(surfaces.len(), 2);
    assert!(surfaces.contains(&0));
    assert!(surfaces.contains(&1));
}

/// Test that Scene transforms work correctly for side-by-side layout.
#[test]
fn test_scene_transforms() {
    let mut scene = Scene::new();

    // Add surfaces with side-by-side transforms
    scene.add_surface(0);
    scene.add_surface(1);

    // Set transforms (typical side-by-side layout)
    scene.set_surface_transform(0, Transform3D::from_translation(Vec3::new(-50.0, 0.0, 0.0)));
    scene.set_surface_transform(1, Transform3D::from_translation(Vec3::new(50.0, 0.0, 0.0)));

    // Verify transforms
    let t0 = scene.get_surface_transform(0).expect("Should have transform");
    let t1 = scene.get_surface_transform(1).expect("Should have transform");

    assert_eq!(t0.translation.x, -50.0);
    assert_eq!(t1.translation.x, 50.0);

    // Verify transforms are returned when iterating
    let surfaces_with_transforms: Vec<_> = scene.iter_surfaces_with_transforms().collect();
    assert_eq!(surfaces_with_transforms.len(), 2);

    // Transforms should have opposite X offsets
    let (_id0, trans0) = surfaces_with_transforms[0];
    let (_id1, trans1) = surfaces_with_transforms[1];
    assert!(trans0.translation.x * trans1.translation.x < 0.0, "Transforms should have opposite X signs");
}

/// Test that markers can be added and removed.
#[test]
fn test_scene_markers() {
    let mut scene = Scene::new();

    // Add markers
    let m1 = scene.add_marker(Vec3::new(10.0, 20.0, 30.0), [1.0, 0.0, 0.0]);
    let m2 = scene.add_marker(Vec3::new(-10.0, -20.0, -30.0), [0.0, 1.0, 0.0]);

    assert_eq!(scene.marker_count(), 2);

    // Verify marker content
    let markers: Vec<_> = scene.iter_markers().collect();
    assert_eq!(markers.len(), 2);

    // Update a marker
    scene.update_marker(m1, Vec3::new(100.0, 0.0, 0.0), [0.0, 0.0, 1.0]);

    // Remove a marker
    scene.remove_marker(m2);
    assert_eq!(scene.marker_count(), 1);

    // Clear all markers
    scene.clear_markers();
    assert_eq!(scene.marker_count(), 0);
}

/// Test orbit controller basic functionality.
#[test]
fn test_orbit_controller() {
    let mut orbit = OrbitController::default();

    // Check initial state
    assert!(orbit.distance > 0.0, "Should have positive distance");

    // Test preset application
    use neuro_surface::BrainViewPreset;
    orbit.set_preset(BrainViewPreset::LateralLeft);

    // Verify view matrix is valid (not NaN/Inf)
    let view = orbit.view_matrix();
    let cols = view.to_cols_array();
    for val in cols {
        assert!(val.is_finite(), "View matrix should have finite values");
    }
}

/// Test orbit controller responds to view presets correctly.
#[test]
fn test_orbit_presets() {
    use neuro_surface::BrainViewPreset;

    let mut orbit = OrbitController::default();

    // Test each preset produces different camera positions
    let mut angles = Vec::new();

    for preset in BrainViewPreset::all() {
        orbit.set_preset(*preset);
        angles.push((orbit.theta, orbit.phi));
    }

    // Verify we get different angles for different presets
    // (Some presets may share angles, but not all)
    let unique_angles: std::collections::HashSet<_> = angles.iter()
        .map(|(t, p)| ((*t * 1000.0) as i32, (*p * 1000.0) as i32))
        .collect();

    assert!(unique_angles.len() > 2, "Should have multiple distinct camera positions");
}

/// Test that empty scene returns None for transforms.
#[test]
fn test_empty_scene() {
    let scene = Scene::new();

    assert!(!scene.has_surface());
    assert_eq!(scene.iter_surfaces().count(), 0);
    assert_eq!(scene.marker_count(), 0);
    assert!(scene.get_surface_transform(0).is_none());
}

/// Test scene visibility control.
#[test]
fn test_scene_visibility() {
    let mut scene = Scene::new();

    let node = scene.add_surface(0);

    // Initially visible
    let visible_count = scene.iter_surfaces().count();
    assert_eq!(visible_count, 1);

    // Hide the surface
    scene.set_visible(node, false);
    let visible_count = scene.iter_surfaces().count();
    assert_eq!(visible_count, 0);

    // Show again
    scene.set_visible(node, true);
    let visible_count = scene.iter_surfaces().count();
    assert_eq!(visible_count, 1);
}

/// Test building a synthetic BrainGeometry.
#[test]
fn test_synthetic_brain_geometry() {
    use io_formats::geometry::BrainGeometry;
    use io_formats::statistics::Hemisphere;

    // Create a minimal valid geometry (a single triangle)
    let geom = BrainGeometry {
        hemisphere: Hemisphere::Left,
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        normals: vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        indices: vec![[0, 1, 2]],
    };

    assert_eq!(geom.vertices.len(), 3);
    assert_eq!(geom.normals.len(), 3);
    assert_eq!(geom.indices.len(), 1);
    assert_eq!(geom.hemisphere, Hemisphere::Left);
}

/// Test building synthetic StatisticData.
#[test]
fn test_synthetic_statistic_data() {
    use io_formats::statistics::StatisticData;

    // Create synthetic overlay data
    let values = vec![0.5, -0.3, 0.8, f32::NAN, 1.2];
    let n_vertices = values.len();

    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut nan_count = 0u32;

    for &v in &values {
        if v.is_nan() {
            nan_count += 1;
        } else if v.is_finite() {
            min = min.min(v);
            max = max.max(v);
        }
    }

    let stats = StatisticData {
        values: values.clone(),
        n_vertices,
        n_volumes: 1,
        global_min: min,
        global_max: max,
        volume_ranges: vec![(min, max)],
        nan_count,
    };

    assert_eq!(stats.n_vertices, 5);
    assert_eq!(stats.nan_count, 1);
    assert_eq!(stats.global_min, -0.3);
    assert_eq!(stats.global_max, 1.2);
}

// ============================================================================
// Layout Tests
// ============================================================================

/// Layout offset constant (matches viewer_app::renderer::wgpu_adapter).
const LAYOUT_OFFSET_X: f32 = 50.0;
const LAYOUT_OFFSET_Y: f32 = 50.0;

/// Surface IDs for left and right hemispheres.
const SURFACE_ID_LEFT: u32 = 0;
const SURFACE_ID_RIGHT: u32 = 1;

/// Test side-by-side layout applies correct transforms.
#[test]
fn test_layout_side_by_side() {
    let mut scene = Scene::new();

    // Add two surfaces
    scene.add_surface(SURFACE_ID_LEFT);
    scene.add_surface(SURFACE_ID_RIGHT);

    // Apply side-by-side layout transforms
    scene.set_surface_transform(
        SURFACE_ID_LEFT,
        Transform3D::from_translation(Vec3::new(-LAYOUT_OFFSET_X, 0.0, 0.0)),
    );
    scene.set_surface_transform(
        SURFACE_ID_RIGHT,
        Transform3D::from_translation(Vec3::new(LAYOUT_OFFSET_X, 0.0, 0.0)),
    );

    // Verify transforms
    let left_transform = scene.get_surface_transform(SURFACE_ID_LEFT).unwrap();
    let right_transform = scene.get_surface_transform(SURFACE_ID_RIGHT).unwrap();

    // Left should have negative X offset
    assert_eq!(left_transform.translation.x, -50.0);
    assert_eq!(left_transform.translation.y, 0.0);
    assert_eq!(left_transform.translation.z, 0.0);

    // Right should have positive X offset
    assert_eq!(right_transform.translation.x, 50.0);
    assert_eq!(right_transform.translation.y, 0.0);
    assert_eq!(right_transform.translation.z, 0.0);

    // Transforms should have opposite X signs
    assert!(left_transform.translation.x < 0.0);
    assert!(right_transform.translation.x > 0.0);

    // Transforms should be separated by minimum offset
    let separation = right_transform.translation.x - left_transform.translation.x;
    assert!(separation >= 2.0 * LAYOUT_OFFSET_X - 1.0); // Allow small tolerance
}

/// Test stacked layout applies correct transforms.
#[test]
fn test_layout_stacked() {
    let mut scene = Scene::new();

    scene.add_surface(SURFACE_ID_LEFT);
    scene.add_surface(SURFACE_ID_RIGHT);

    // Apply stacked layout transforms
    scene.set_surface_transform(
        SURFACE_ID_LEFT,
        Transform3D::from_translation(Vec3::new(0.0, LAYOUT_OFFSET_Y, 0.0)),
    );
    scene.set_surface_transform(
        SURFACE_ID_RIGHT,
        Transform3D::from_translation(Vec3::new(0.0, -LAYOUT_OFFSET_Y, 0.0)),
    );

    let left_transform = scene.get_surface_transform(SURFACE_ID_LEFT).unwrap();
    let right_transform = scene.get_surface_transform(SURFACE_ID_RIGHT).unwrap();

    // Left should be on top (positive Y)
    assert_eq!(left_transform.translation.y, 50.0);

    // Right should be on bottom (negative Y)
    assert_eq!(right_transform.translation.y, -50.0);

    // X coordinates should be zero
    assert_eq!(left_transform.translation.x, 0.0);
    assert_eq!(right_transform.translation.x, 0.0);
}

/// Test single layout applies zero transforms.
#[test]
fn test_layout_single() {
    let mut scene = Scene::new();

    scene.add_surface(SURFACE_ID_LEFT);
    scene.add_surface(SURFACE_ID_RIGHT);

    // Apply single layout (centered, overlapping)
    scene.set_surface_transform(SURFACE_ID_LEFT, Transform3D::from_translation(Vec3::ZERO));
    scene.set_surface_transform(SURFACE_ID_RIGHT, Transform3D::from_translation(Vec3::ZERO));

    let left_transform = scene.get_surface_transform(SURFACE_ID_LEFT).unwrap();
    let right_transform = scene.get_surface_transform(SURFACE_ID_RIGHT).unwrap();

    // Both should be at origin
    assert_eq!(left_transform.translation, Vec3::ZERO);
    assert_eq!(right_transform.translation, Vec3::ZERO);
}

/// Test layout transform iteration.
#[test]
fn test_layout_transform_iteration() {
    let mut scene = Scene::new();

    scene.add_surface(SURFACE_ID_LEFT);
    scene.add_surface(SURFACE_ID_RIGHT);

    // Apply side-by-side layout
    scene.set_surface_transform(
        SURFACE_ID_LEFT,
        Transform3D::from_translation(Vec3::new(-50.0, 0.0, 0.0)),
    );
    scene.set_surface_transform(
        SURFACE_ID_RIGHT,
        Transform3D::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    );

    // Iterate and verify
    let transforms: Vec<_> = scene.iter_surfaces_with_transforms().collect();
    assert_eq!(transforms.len(), 2);

    // Find left and right by surface ID
    let left = transforms.iter().find(|(id, _)| *id == SURFACE_ID_LEFT).unwrap();
    let right = transforms.iter().find(|(id, _)| *id == SURFACE_ID_RIGHT).unwrap();

    assert_eq!(left.1.translation.x, -50.0);
    assert_eq!(right.1.translation.x, 50.0);
}
