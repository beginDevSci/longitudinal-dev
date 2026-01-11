//! Synthetic GIFTI-like formats for testing.
//!
//! # Purpose
//!
//! This module defines a simplified JSON-based format that mimics GIFTI
//! structure. Real GIFTI files are XML-based; this module uses JSON for
//! easier test fixture creation.
//!
//! # When to use
//!
//! - **Tests**: Create JSON fixtures with the structure below
//! - **Production**: Use `gifti_real` module for actual GIFTI XML files
//!
//! # JSON structure
//!
//! ```json
//! {
//!   "vertices": [[x, y, z], ...],
//!   "faces": [[i, j, k], ...],
//!   "arrays": [[...], ...]
//! }
//! ```
//!
//! The loader in `lib.rs` auto-detects real vs synthetic based on XML detection.

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::statistics::Hemisphere;

use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize)]
struct GiftiJson {
    vertices: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    #[serde(default)]
    arrays: Vec<Vec<f32>>,
}

pub fn read_gifti_surface(path: &Path) -> Result<BrainGeometry, FormatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let parsed: GiftiJson = serde_json::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let hemisphere = Hemisphere::Left;
    let normals = vec![[0.0_f32, 0.0, 0.0]; parsed.vertices.len()];

    Ok(BrainGeometry {
        hemisphere,
        vertices: parsed.vertices,
        normals,
        indices: parsed.faces,
    })
}

pub fn read_gifti_func(path: &Path) -> Result<Vec<Vec<f32>>, FormatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let parsed: GiftiJson = serde_json::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(parsed.arrays)
}

