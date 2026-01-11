//! Synthetic FreeSurfer-like formats for testing.
//!
//! # Purpose
//!
//! This module defines simplified, stable test formats that mimic FreeSurfer
//! file structure but use custom magic bytes (FSS1, FSC1). These are **not**
//! the real FreeSurfer on-disk formats.
//!
//! # When to use
//!
//! - **Tests**: Use `write_test_surface` / `write_test_curv` to create fixtures
//! - **Production**: Use `freesurfer_real` module for actual FreeSurfer files
//!
//! # Magic bytes
//!
//! - Surface: `FSS1` (vs real FreeSurfer: `0xFFFFFE`)
//! - Curvature: `FSC1` (vs real FreeSurfer: `0xFFFFFF`)
//!
//! The loader in `lib.rs` auto-detects which parser to use based on magic bytes.

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::statistics::Hemisphere;

use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Read a (test) FreeSurfer-like binary surface file into a `BrainGeometry`.
///
/// This uses a compact binary layout tailored for tests:
/// - u32 magic = b"FSS1" (big-endian)
/// - u32 n_vertices (big-endian)
/// - u32 n_faces (big-endian)
/// - vertices: n_vertices * (f32 x, y, z) (big-endian)
/// - faces: n_faces * (u32 i, j, k) (big-endian)
pub fn read_surface(path: &Path, hemisphere: Hemisphere) -> Result<BrainGeometry, FormatError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let mut cursor = std::io::Cursor::new(&buf);

    let magic = cursor.read_u32::<BigEndian>()?;
    if magic != u32::from_be_bytes(*b"FSS1") {
        return Err(FormatError::InvalidMagic {
            expected: "FSS1",
            found: format!("{magic:#010x}"),
        });
    }

    let n_vertices = cursor.read_u32::<BigEndian>()? as usize;
    let n_faces = cursor.read_u32::<BigEndian>()? as usize;

    // Each vertex: 3 * f32, each face: 3 * u32.
    let header_bytes = 4 + 4 + 4;
    let expected_size = header_bytes + n_vertices * 3 * 4 + n_faces * 3 * 4;
    if buf.len() != expected_size {
        return Err(FormatError::SizeMismatch {
            expected: expected_size,
            actual: buf.len(),
        });
    }

    let mut vertices = Vec::with_capacity(n_vertices);
    for _ in 0..n_vertices {
        let x = cursor.read_f32::<BigEndian>()?;
        let y = cursor.read_f32::<BigEndian>()?;
        let z = cursor.read_f32::<BigEndian>()?;
        vertices.push([x, y, z]);
    }

    // No normals stored in this simple test format; fill with zeroes.
    let mut normals = Vec::with_capacity(n_vertices);
    for _ in 0..n_vertices {
        normals.push([0.0, 0.0, 0.0]);
    }

    let mut indices = Vec::with_capacity(n_faces);
    for _ in 0..n_faces {
        let i = cursor.read_u32::<BigEndian>()?;
        let j = cursor.read_u32::<BigEndian>()?;
        let k = cursor.read_u32::<BigEndian>()?;
        indices.push([i, j, k]);
    }

    Ok(BrainGeometry {
        hemisphere,
        vertices,
        normals,
        indices,
    })
}

/// Read a (test) FreeSurfer-like curvature file into scalar values.
///
/// Layout:
/// - u32 magic = b"FSC1" (big-endian)
/// - u32 n_vertices (big-endian)
/// - n_vertices * f32 values (big-endian)
pub fn read_curv(path: &Path) -> Result<Vec<f32>, FormatError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let mut cursor = std::io::Cursor::new(&buf);

    let magic = cursor.read_u32::<BigEndian>()?;
    if magic != u32::from_be_bytes(*b"FSC1") {
        return Err(FormatError::InvalidMagic {
            expected: "FSC1",
            found: format!("{magic:#010x}"),
        });
    }

    let n_vertices = cursor.read_u32::<BigEndian>()? as usize;

    let header_bytes = 4 + 4;
    let expected_size = header_bytes + n_vertices * 4;
    if buf.len() != expected_size {
        return Err(FormatError::SizeMismatch {
            expected: expected_size,
            actual: buf.len(),
        });
    }

    let mut values = Vec::with_capacity(n_vertices);
    for _ in 0..n_vertices {
        let v = cursor.read_f32::<BigEndian>()?;
        values.push(v);
    }

    Ok(values)
}

/// Placeholder for a future real FreeSurfer surface reader.
///
/// For now this delegates to the synthetic `read_surface` so that tests
/// can exercise a stable API surface without depending on the true on-disk
/// format. The implementation can be swapped out later.
pub fn read_fs_surface(
    path: &Path,
    hemisphere: Hemisphere,
) -> Result<BrainGeometry, FormatError> {
    read_surface(path, hemisphere)
}

/// Placeholder for a future real FreeSurfer curvature reader.
///
/// Currently delegates to the synthetic `read_curv` helper.
pub fn read_fs_curv(path: &Path) -> Result<Vec<f32>, FormatError> {
    read_curv(path)
}

#[cfg(test)]
pub(crate) fn write_test_surface(
    path: &Path,
    vertices: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Result<(), FormatError> {
    use byteorder::WriteBytesExt;
    use std::fs;
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buf = Vec::new();
    buf.write_u32::<BigEndian>(u32::from_be_bytes(*b"FSS1"))?;
    buf.write_u32::<BigEndian>(vertices.len() as u32)?;
    buf.write_u32::<BigEndian>(faces.len() as u32)?;

    for [x, y, z] in vertices {
        buf.write_f32::<BigEndian>(*x)?;
        buf.write_f32::<BigEndian>(*y)?;
        buf.write_f32::<BigEndian>(*z)?;
    }

    for [i, j, k] in faces {
        buf.write_u32::<BigEndian>(*i)?;
        buf.write_u32::<BigEndian>(*j)?;
        buf.write_u32::<BigEndian>(*k)?;
    }

    let mut file = File::create(path)?;
    file.write_all(&buf)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn write_test_curv(path: &Path, values: &[f32]) -> Result<(), FormatError> {
    use byteorder::WriteBytesExt;
    use std::fs;
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buf = Vec::new();
    buf.write_u32::<BigEndian>(u32::from_be_bytes(*b"FSC1"))?;
    buf.write_u32::<BigEndian>(values.len() as u32)?;
    for v in values {
        buf.write_f32::<BigEndian>(*v)?;
    }

    let mut file = File::create(path)?;
    file.write_all(&buf)?;
    Ok(())
}
