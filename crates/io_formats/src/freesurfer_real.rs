//! Real FreeSurfer format support and ground-truth test fixtures.
//!
//! This module contains:
//! 1. Ground-truth constants extracted from real FreeSurfer files in demo/geom/
//! 2. The real FreeSurfer surface parser (triangular surface format)
//!
//! Ground-truth values were extracted using Python/struct from the actual files.
//! DO NOT MODIFY the constants unless you re-extract from the actual files.
//!
//! ## Implementation Status
//!
//! | Format | Status | Notes |
//! |--------|--------|-------|
//! | `.pial` / `.white` / `.inflated` | ✅ Implemented | Tested against demo/geom/*.pial |
//! | `.curv` / `.thickness` / `.sulc` | ✅ Implemented | Tested against minimal fixture |
//!
//! ## FreeSurfer Triangular Surface Format
//!
//! The real FreeSurfer surface format uses:
//! - Magic bytes: 0xFF 0xFF 0xFE (3 bytes, big-endian)
//! - Comment: ASCII text terminated by double newline (\n\n)
//! - n_vertices: u32 big-endian
//! - n_faces: u32 big-endian
//! - vertices: n_vertices * 3 * f32 big-endian (x, y, z)
//! - faces: n_faces * 3 * u32 big-endian (v0, v1, v2)
//!
//! ## FreeSurfer Curvature Format (.curv)
//!
//! The real FreeSurfer "new" curvature format uses:
//! - Magic bytes: 0xFF 0xFF 0xFF (3 bytes)
//! - n_vertices: u32 big-endian
//! - n_faces: u32 big-endian (not used for curvature)
//! - vals_per_vertex: u32 big-endian (typically 1)
//! - values: n_vertices * vals_per_vertex * f32 big-endian
//!
//! The "old" format (no magic, raw int16 values) is not supported.

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::statistics::Hemisphere;

use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// ============================================================================
// Ground-Truth Constants for Test Fixtures
// ============================================================================

/// Path to lh.pial relative to io_formats crate root.
pub const LH_PIAL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../demo/geom/lh.pial");

/// Path to rh.pial relative to io_formats crate root.
pub const RH_PIAL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../demo/geom/rh.pial");

/// Expected number of vertices in lh.pial.
pub const LH_PIAL_N_VERTICES: usize = 10242;

/// Expected number of faces in lh.pial.
pub const LH_PIAL_N_FACES: usize = 20480;

/// Expected number of vertices in rh.pial.
pub const RH_PIAL_N_VERTICES: usize = 10242;

/// Expected number of faces in rh.pial.
pub const RH_PIAL_N_FACES: usize = 20480;

/// Sample vertices from lh.pial: (index, [x, y, z]).
/// Indices chosen at: 0, 100, 1000, midpoint, last.
pub const LH_PIAL_VERTEX_SAMPLES: &[(usize, [f32; 3])] = &[
    (0, [-38.735958, -19.343365, 67.220139]),
    (100, [-55.733116, -10.402655, 31.788502]),
    (1000, [-45.843636, 4.994218, 45.378933]),
    (5121, [-48.663830, -59.776836, 14.161862]),
    (10241, [-34.491192, -25.403906, -24.645117]),
];

/// Sample faces from lh.pial: (index, [v0, v1, v2]).
pub const LH_PIAL_FACE_SAMPLES: &[(usize, [u32; 3])] = &[
    (0, [0, 2564, 2562]),
    (100, [52, 2797, 2810]),
    (1000, [546, 5062, 5060]),
    (10240, [6403, 998, 6402]),
    (20479, [10161, 11, 9918]),
];

/// Sample vertices from rh.pial: (index, [x, y, z]).
pub const RH_PIAL_VERTEX_SAMPLES: &[(usize, [f32; 3])] = &[
    (0, [27.947912, -12.046541, 60.970966]),
    (100, [8.274683, -1.470703, 53.339741]),
    (1000, [23.569714, 19.545950, 58.350128]),
    (5121, [11.503923, -67.794327, 19.262442]),
    (10241, [47.221764, -21.873978, -31.368597]),
];

/// Sample faces from rh.pial: (index, [v0, v1, v2]).
pub const RH_PIAL_FACE_SAMPLES: &[(usize, [u32; 3])] = &[
    (0, [0, 2564, 2562]),
    (100, [52, 2797, 2810]),
    (1000, [546, 5062, 5060]),
    (10240, [6403, 998, 6402]),
    (20479, [10161, 11, 9918]),
];

/// Tolerance for floating-point coordinate comparisons.
pub const COORD_TOLERANCE: f32 = 1e-4;

// ============================================================================
// Ground-Truth Constants for .curv Test Fixture
// ============================================================================

/// Path to test lh.curv fixture (minimal synthetic file in real FreeSurfer format).
pub const LH_CURV_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/freesurfer_real/lh.curv"
);

/// Expected number of vertices in the test lh.curv fixture.
pub const LH_CURV_N_VERTICES: usize = 100;

/// Sample values from test lh.curv: (index, value).
/// Values are sin(i * 0.1) * 0.5 for easy verification.
pub const LH_CURV_VALUE_SAMPLES: &[(usize, f32)] = &[
    (0, 0.000000),
    (10, 0.420735),
    (50, -0.479462),
    (99, -0.228768),
];

/// Tolerance for curvature value comparisons.
pub const CURV_TOLERANCE: f32 = 1e-5;

// ============================================================================
// Real FreeSurfer Surface Parser
// ============================================================================

/// Magic bytes for FreeSurfer triangular surface format.
const FS_TRIANGLE_MAGIC: [u8; 3] = [0xFF, 0xFF, 0xFE];

/// Magic bytes for FreeSurfer "new" curvature format.
const FS_NEW_CURV_MAGIC: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// Read a real FreeSurfer triangular surface file.
///
/// This implements the actual FreeSurfer binary surface format:
/// - 3-byte magic (0xFF 0xFF 0xFE)
/// - ASCII comment terminated by \n\n
/// - u32 n_vertices (big-endian)
/// - u32 n_faces (big-endian)
/// - vertices: n_vertices * 3 * f32 (big-endian)
/// - faces: n_faces * 3 * u32 (big-endian)
pub fn read_real_surface(path: &Path, hemisphere: Hemisphere) -> Result<BrainGeometry, FormatError> {
    let bytes = std::fs::read(path)?;
    read_real_surface_bytes(&bytes, hemisphere)
}

/// Read a real FreeSurfer triangular surface from in-memory bytes.
///
/// Same format as `read_real_surface` but from a byte slice instead of a file.
pub fn read_real_surface_bytes(bytes: &[u8], hemisphere: Hemisphere) -> Result<BrainGeometry, FormatError> {
    use std::io::Cursor;

    let mut reader = Cursor::new(bytes);

    // Read and verify magic bytes
    let mut magic = [0u8; 3];
    reader.read_exact(&mut magic)?;
    if magic != FS_TRIANGLE_MAGIC {
        return Err(FormatError::InvalidMagic {
            expected: "0xFFFFFE (FreeSurfer triangle)",
            found: format!("0x{:02X}{:02X}{:02X}", magic[0], magic[1], magic[2]),
        });
    }

    // Skip comment until double newline
    let mut prev_byte = 0u8;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        if prev_byte == b'\n' && byte[0] == b'\n' {
            break;
        }
        prev_byte = byte[0];
    }

    // Read vertex and face counts
    let n_vertices = reader.read_u32::<BigEndian>()? as usize;
    let n_faces = reader.read_u32::<BigEndian>()? as usize;

    // Read vertices
    let mut vertices = Vec::with_capacity(n_vertices);
    for _ in 0..n_vertices {
        let x = reader.read_f32::<BigEndian>()?;
        let y = reader.read_f32::<BigEndian>()?;
        let z = reader.read_f32::<BigEndian>()?;
        vertices.push([x, y, z]);
    }

    // Read faces
    let mut indices = Vec::with_capacity(n_faces);
    for _ in 0..n_faces {
        let v0 = reader.read_u32::<BigEndian>()?;
        let v1 = reader.read_u32::<BigEndian>()?;
        let v2 = reader.read_u32::<BigEndian>()?;
        indices.push([v0, v1, v2]);
    }

    // Compute normals from face data
    let normals = compute_vertex_normals(&vertices, &indices);

    Ok(BrainGeometry {
        hemisphere,
        vertices,
        normals,
        indices,
    })
}

/// Read a real FreeSurfer curvature file (.curv, .thickness, .sulc, etc.).
///
/// This implements the FreeSurfer "new" curvature format:
/// - Magic bytes: 0xFF 0xFF 0xFF (3 bytes)
/// - n_vertices: u32 big-endian
/// - n_faces: u32 big-endian (not used for curvature data)
/// - vals_per_vertex: u32 big-endian (typically 1)
/// - values: n_vertices * vals_per_vertex * f32 big-endian
///
/// Note: The "old" format (no magic, just raw int16 values) is not supported.
pub fn read_real_curv(path: &Path) -> Result<Vec<f32>, FormatError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and verify magic bytes
    let mut magic = [0u8; 3];
    reader.read_exact(&mut magic)?;
    if magic != FS_NEW_CURV_MAGIC {
        return Err(FormatError::InvalidMagic {
            expected: "0xFFFFFF (FreeSurfer new curv)",
            found: format!("0x{:02X}{:02X}{:02X}", magic[0], magic[1], magic[2]),
        });
    }

    // Read header
    let n_vertices = reader.read_u32::<BigEndian>()? as usize;
    let _n_faces = reader.read_u32::<BigEndian>()?; // Not used for curv data
    let vals_per_vertex = reader.read_u32::<BigEndian>()? as usize;

    // Typically vals_per_vertex is 1, but handle multiple values per vertex
    let total_values = n_vertices * vals_per_vertex;
    let mut values = Vec::with_capacity(total_values);
    for _ in 0..total_values {
        let v = reader.read_f32::<BigEndian>()?;
        values.push(v);
    }

    // If multiple values per vertex, just return them all flattened
    // (caller can reshape if needed based on vals_per_vertex)
    Ok(values)
}

/// Read a real FreeSurfer curvature from in-memory bytes.
///
/// Same format as `read_real_curv` but from a byte slice instead of a file.
pub fn read_real_curv_from_bytes(bytes: &[u8]) -> Result<Vec<f32>, FormatError> {
    use std::io::Cursor;

    let mut reader = Cursor::new(bytes);

    // Read and verify magic bytes
    let mut magic = [0u8; 3];
    reader.read_exact(&mut magic)?;
    if magic != FS_NEW_CURV_MAGIC {
        return Err(FormatError::InvalidMagic {
            expected: "0xFFFFFF (FreeSurfer new curv)",
            found: format!("0x{:02X}{:02X}{:02X}", magic[0], magic[1], magic[2]),
        });
    }

    // Read header
    let n_vertices = reader.read_u32::<BigEndian>()? as usize;
    let _n_faces = reader.read_u32::<BigEndian>()?; // Not used for curv data
    let vals_per_vertex = reader.read_u32::<BigEndian>()? as usize;

    // Typically vals_per_vertex is 1, but handle multiple values per vertex
    let total_values = n_vertices * vals_per_vertex;
    let mut values = Vec::with_capacity(total_values);
    for _ in 0..total_values {
        let v = reader.read_f32::<BigEndian>()?;
        values.push(v);
    }

    Ok(values)
}

// ============================================================================
// FreeSurfer Annotation (.annot) Parser
// ============================================================================

use crate::parcellation::{Parcellation, Region};

/// Read a real FreeSurfer annotation (.annot) file.
///
/// The FreeSurfer annotation format contains:
/// 1. Per-vertex labels (encoded as RGBA-based annotation values)
/// 2. A color table mapping label IDs to region names and colors
///
/// ## Format Details
///
/// The .annot format is binary, big-endian:
/// - n_vertices: i32
/// - For each vertex: vertex_number (i32), annotation (i32)
/// - has_ctab: i32 (non-zero if color table present)
/// - Color table (if present):
///   - version: i32 (-2 for new format)
///   - n_entries: i32
///   - orig_tab_length: i32
///   - orig_tab: chars (original table name)
///   - For each entry:
///     - structure_length: i32
///     - structure_name: chars
///     - r, g, b, a: i32 (0-255 each)
///
/// The annotation value encodes RGBA: annotation = R + G*256 + B*65536 + A*16777216
///
/// # Arguments
/// * `path` - Path to the .annot file
/// * `hemisphere` - Which hemisphere this annotation belongs to
///
/// # Returns
/// A Parcellation containing per-vertex labels and region definitions.
pub fn read_real_annot(path: &Path, hemisphere: Hemisphere) -> Result<Parcellation, FormatError> {
    let bytes = std::fs::read(path)?;
    let mut parcellation = read_real_annot_bytes(&bytes, hemisphere)?;

    // Extract atlas name from file path (e.g., "lh.aparc.annot" -> "aparc")
    parcellation.atlas_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            if let Some(pos) = s.find('.') {
                s[pos + 1..].to_string()
            } else {
                s.to_string()
            }
        });

    Ok(parcellation)
}

/// Read a FreeSurfer annotation from in-memory bytes.
pub fn read_real_annot_bytes(bytes: &[u8], hemisphere: Hemisphere) -> Result<Parcellation, FormatError> {
    use std::io::Cursor;

    let mut reader = Cursor::new(bytes);

    // Read number of vertices
    let n_vertices_i32 = reader.read_i32::<BigEndian>()?;
    if n_vertices_i32 <= 0 {
        return Err(FormatError::SizeMismatch {
            expected: 1,
            actual: 0,
        });
    }
    // Sanity check: FreeSurfer brain surfaces typically have ~160k vertices per hemisphere
    // Allow up to 10 million to be safe, but reject clearly corrupt values
    const MAX_REASONABLE_VERTICES: i32 = 10_000_000;
    if n_vertices_i32 > MAX_REASONABLE_VERTICES {
        return Err(FormatError::SizeMismatch {
            expected: MAX_REASONABLE_VERTICES as usize,
            actual: n_vertices_i32 as usize,
        });
    }
    let n_vertices = n_vertices_i32 as usize;

    // Read per-vertex annotations
    // Format: vertex_number (i32), annotation (i32) for each vertex
    let mut raw_annotations = vec![0i32; n_vertices];
    for _ in 0..n_vertices {
        let vertex_num = reader.read_i32::<BigEndian>()? as usize;
        let annotation = reader.read_i32::<BigEndian>()?;
        if vertex_num < n_vertices {
            raw_annotations[vertex_num] = annotation;
        }
    }

    // Check if color table is present
    let has_ctab = reader.read_i32::<BigEndian>()?;

    let mut regions = Vec::new();
    let mut annotation_to_id: std::collections::HashMap<i32, u32> = std::collections::HashMap::new();

    if has_ctab != 0 {
        // Read color table
        let version = reader.read_i32::<BigEndian>()?;

        if version == -2 {
            // New format (version -2)
            let _n_entries = reader.read_i32::<BigEndian>()? as usize;
            let orig_tab_length_i32 = reader.read_i32::<BigEndian>()?;
            if orig_tab_length_i32 < 0 {
                return Err(FormatError::SizeMismatch {
                    expected: 0,
                    actual: bytes.len(),
                });
            }
            let orig_tab_length = orig_tab_length_i32 as usize;

            // Skip original table name
            let mut orig_tab = vec![0u8; orig_tab_length];
            reader.read_exact(&mut orig_tab)?;

            // Read entries
            let num_entries_to_read_i32 = reader.read_i32::<BigEndian>()?;
            if num_entries_to_read_i32 < 0 {
                return Err(FormatError::SizeMismatch {
                    expected: 0,
                    actual: bytes.len(),
                });
            }
            let num_entries_to_read = num_entries_to_read_i32 as usize;

            for _ in 0..num_entries_to_read {
                let structure_id = reader.read_i32::<BigEndian>()? as u32;
                let name_length_i32 = reader.read_i32::<BigEndian>()?;
                if name_length_i32 < 0 {
                    return Err(FormatError::SizeMismatch {
                        expected: 0,
                        actual: bytes.len(),
                    });
                }
                let name_length = name_length_i32 as usize;

                let mut name_bytes = vec![0u8; name_length];
                reader.read_exact(&mut name_bytes)?;
                // Remove null terminator if present
                let name = String::from_utf8_lossy(&name_bytes)
                    .trim_end_matches('\0')
                    .to_string();

                let r = reader.read_i32::<BigEndian>()? as u8;
                let g = reader.read_i32::<BigEndian>()? as u8;
                let b = reader.read_i32::<BigEndian>()? as u8;
                let a = reader.read_i32::<BigEndian>()? as u8;

                // Compute the annotation value for this entry
                let annot_value = (r as i32)
                    + ((g as i32) << 8)
                    + ((b as i32) << 16)
                    + ((a as i32) << 24);

                annotation_to_id.insert(annot_value, structure_id);

                regions.push(Region::new(structure_id, name, [r, g, b, if a == 0 { 255 } else { a }]));
            }
        } else {
            // Old format - version is actually the number of entries
            let n_entries = version as usize;

            for _ in 0..n_entries {
                let name_length_i32 = reader.read_i32::<BigEndian>()?;
                if name_length_i32 < 0 {
                    return Err(FormatError::SizeMismatch {
                        expected: 0,
                        actual: bytes.len(),
                    });
                }
                let name_length = name_length_i32 as usize;
                let mut name_bytes = vec![0u8; name_length];
                reader.read_exact(&mut name_bytes)?;
                let name = String::from_utf8_lossy(&name_bytes)
                    .trim_end_matches('\0')
                    .to_string();

                let r = reader.read_i32::<BigEndian>()? as u8;
                let g = reader.read_i32::<BigEndian>()? as u8;
                let b = reader.read_i32::<BigEndian>()? as u8;
                let a = reader.read_i32::<BigEndian>()? as u8;

                let structure_id = regions.len() as u32;
                let annot_value = (r as i32)
                    + ((g as i32) << 8)
                    + ((b as i32) << 16)
                    + ((a as i32) << 24);

                annotation_to_id.insert(annot_value, structure_id);
                regions.push(Region::new(structure_id, name, [r, g, b, if a == 0 { 255 } else { a }]));
            }
        }
    }

    // Convert raw annotations to region IDs
    let labels: Vec<u32> = raw_annotations
        .iter()
        .map(|&annot| *annotation_to_id.get(&annot).unwrap_or(&0))
        .collect();

    Ok(Parcellation {
        hemisphere,
        labels,
        regions,
        atlas_name: None, // Set by caller if path is available
    })
}

/// Check if a file is a valid FreeSurfer annotation file.
///
/// This is a heuristic check - annotation files don't have a magic number,
/// so we check if the file extension is .annot.
pub fn is_real_annot(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("annot")
}

// ============================================================================
// Annotation Test Fixtures
// ============================================================================

/// Path to lh.aparc.annot test fixture.
pub const LH_ANNOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/freesurfer_real/lh.aparc.annot"
);

/// Expected number of vertices in the annotation fixture.
pub const LH_ANNOT_N_VERTICES: usize = 100;

/// Expected number of regions in the annotation fixture.
pub const LH_ANNOT_N_REGIONS: usize = 5;

/// Sample vertex labels: (vertex_index, expected_region_id).
pub const LH_ANNOT_LABEL_SAMPLES: &[(usize, u32)] = &[
    (0, 1),   // First vertex
    (25, 2),  // Quarter way
    (50, 3),  // Halfway
    (75, 4),  // Three quarters
    (99, 0),  // Last vertex
];

/// Sample region definitions: (id, name, [r, g, b]).
pub const LH_ANNOT_REGION_SAMPLES: &[(u32, &str, [u8; 3])] = &[
    (0, "unknown", [25, 5, 25]),
    (1, "bankssts", [25, 100, 40]),
    (2, "caudalanteriorcingulate", [125, 100, 160]),
];

/// Compute vertex normals by averaging face normals.
fn compute_vertex_normals(vertices: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];

    // Accumulate face normals at each vertex
    for [i0, i1, i2] in faces {
        let v0 = vertices[*i0 as usize];
        let v1 = vertices[*i1 as usize];
        let v2 = vertices[*i2 as usize];

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product (face normal, not normalized - area-weighted)
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        // Add to each vertex of this face
        for idx in [*i0 as usize, *i1 as usize, *i2 as usize] {
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }

    // Normalize all vertex normals
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-10 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    fn coords_approx_eq(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        approx_eq(a[0], b[0], tol) && approx_eq(a[1], b[1], tol) && approx_eq(a[2], b[2], tol)
    }

    #[test]
    fn test_read_real_lh_pial() {
        let path = PathBuf::from(LH_PIAL_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", LH_PIAL_PATH);
            return;
        }

        let geom = read_real_surface(&path, Hemisphere::Left).expect("Failed to read lh.pial");

        // Verify counts
        assert_eq!(geom.vertices.len(), LH_PIAL_N_VERTICES);
        assert_eq!(geom.indices.len(), LH_PIAL_N_FACES);
        assert_eq!(geom.normals.len(), LH_PIAL_N_VERTICES);

        // Verify sample vertices
        for (idx, expected_coords) in LH_PIAL_VERTEX_SAMPLES {
            let actual = geom.vertices[*idx];
            assert!(
                coords_approx_eq(actual, *expected_coords, COORD_TOLERANCE),
                "Vertex {} mismatch: expected {:?}, got {:?}",
                idx,
                expected_coords,
                actual
            );
        }

        // Verify sample faces
        for (idx, expected_face) in LH_PIAL_FACE_SAMPLES {
            let actual = geom.indices[*idx];
            assert_eq!(
                actual, *expected_face,
                "Face {} mismatch: expected {:?}, got {:?}",
                idx, expected_face, actual
            );
        }
    }

    #[test]
    fn test_read_real_rh_pial() {
        let path = PathBuf::from(RH_PIAL_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", RH_PIAL_PATH);
            return;
        }

        let geom = read_real_surface(&path, Hemisphere::Right).expect("Failed to read rh.pial");

        // Verify counts
        assert_eq!(geom.vertices.len(), RH_PIAL_N_VERTICES);
        assert_eq!(geom.indices.len(), RH_PIAL_N_FACES);

        // Verify sample vertices
        for (idx, expected_coords) in RH_PIAL_VERTEX_SAMPLES {
            let actual = geom.vertices[*idx];
            assert!(
                coords_approx_eq(actual, *expected_coords, COORD_TOLERANCE),
                "Vertex {} mismatch: expected {:?}, got {:?}",
                idx,
                expected_coords,
                actual
            );
        }

        // Verify sample faces
        for (idx, expected_face) in RH_PIAL_FACE_SAMPLES {
            let actual = geom.indices[*idx];
            assert_eq!(
                actual, *expected_face,
                "Face {} mismatch: expected {:?}, got {:?}",
                idx, expected_face, actual
            );
        }
    }

    #[test]
    fn test_normals_are_normalized() {
        let path = PathBuf::from(LH_PIAL_PATH);
        if !path.exists() {
            return;
        }

        let geom = read_real_surface(&path, Hemisphere::Left).unwrap();

        // Check that all normals have unit length (within tolerance)
        for (i, n) in geom.normals.iter().enumerate() {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 0.01,
                "Normal {} not normalized: length = {}",
                i,
                len
            );
        }
    }

    #[test]
    fn test_read_real_curv() {
        let path = PathBuf::from(LH_CURV_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", LH_CURV_PATH);
            return;
        }

        let values = read_real_curv(&path).expect("Failed to read lh.curv");

        // Verify count
        assert_eq!(
            values.len(),
            LH_CURV_N_VERTICES,
            "Vertex count mismatch: expected {}, got {}",
            LH_CURV_N_VERTICES,
            values.len()
        );

        // Verify sample values
        for (idx, expected_value) in LH_CURV_VALUE_SAMPLES {
            let actual = values[*idx];
            assert!(
                approx_eq(actual, *expected_value, CURV_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                idx,
                expected_value,
                actual
            );
        }
    }

    #[test]
    fn test_curv_invalid_magic() {
        // Try to read a surface file as curv - should fail with magic error
        let path = PathBuf::from(LH_PIAL_PATH);
        if !path.exists() {
            return;
        }

        let result = read_real_curv(&path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string().contains("magic"),
                "Expected magic error, got: {}",
                e
            );
        }
    }

    #[test]
    fn test_read_real_annot() {
        let path = PathBuf::from(LH_ANNOT_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", LH_ANNOT_PATH);
            return;
        }

        let parcellation = read_real_annot(&path, Hemisphere::Left)
            .expect("Failed to read lh.aparc.annot");

        // Verify vertex count
        assert_eq!(
            parcellation.n_vertices(),
            LH_ANNOT_N_VERTICES,
            "Vertex count mismatch"
        );

        // Verify region count
        assert_eq!(
            parcellation.n_regions(),
            LH_ANNOT_N_REGIONS,
            "Region count mismatch"
        );

        // Verify sample vertex labels
        for (idx, expected_label) in LH_ANNOT_LABEL_SAMPLES {
            let actual = parcellation.labels[*idx];
            assert_eq!(
                actual, *expected_label,
                "Label at vertex {} mismatch: expected {}, got {}",
                idx, expected_label, actual
            );
        }

        // Verify sample region definitions
        for (id, expected_name, expected_rgb) in LH_ANNOT_REGION_SAMPLES {
            let region = parcellation.get_region(*id)
                .expect(&format!("Region {} not found", id));
            assert_eq!(
                region.name, *expected_name,
                "Region {} name mismatch", id
            );
            assert_eq!(
                &region.rgba[..3], expected_rgb,
                "Region {} color mismatch", id
            );
        }

        // Verify hemisphere
        assert_eq!(parcellation.hemisphere, Hemisphere::Left);
    }

    #[test]
    fn test_annot_region_lookup() {
        let path = PathBuf::from(LH_ANNOT_PATH);
        if !path.exists() {
            return;
        }

        let parcellation = read_real_annot(&path, Hemisphere::Left).unwrap();

        // Test get_vertex_region_name
        assert_eq!(parcellation.get_vertex_region_name(0), Some("bankssts"));
        assert_eq!(parcellation.get_vertex_region_name(30), Some("caudalanteriorcingulate"));
        assert_eq!(parcellation.get_vertex_region_name(99), Some("unknown"));
    }

    #[test]
    fn test_is_real_annot() {
        assert!(is_real_annot(std::path::Path::new("lh.aparc.annot")));
        assert!(is_real_annot(std::path::Path::new("rh.aparc.a2009s.annot")));
        assert!(!is_real_annot(std::path::Path::new("lh.pial")));
        assert!(!is_real_annot(std::path::Path::new("lh.curv")));
    }
}
