//! Parcellation data structures for brain region annotations.
//!
//! This module defines structures for representing brain parcellations (annotations),
//! which map vertices to discrete regions with names and colors. The structures are
//! format-agnostic and can be populated from:
//!
//! - FreeSurfer `.annot` files
//! - GIFTI `.label.gii` files
//!
//! ## Example
//!
//! ```ignore
//! use io_formats::parcellation::Parcellation;
//!
//! let parcellation = load_freesurfer_annot("lh.aparc.annot")?;
//! for region in &parcellation.regions {
//!     println!("{}: RGBA {:?}", region.name, region.rgba);
//! }
//! let label_at_vertex_0 = parcellation.labels[0];
//! ```

use serde::{Deserialize, Serialize};

use crate::statistics::Hemisphere;

/// A brain region in a parcellation atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier for this region (matches labels_per_vertex values).
    pub id: u32,
    /// Human-readable name of the region (e.g., "precentral", "superiorfrontal").
    pub name: String,
    /// RGBA color for visualization [R, G, B, A] in 0-255 range.
    pub rgba: [u8; 4],
}

impl Region {
    /// Create a new region with the given id, name, and color.
    pub fn new(id: u32, name: impl Into<String>, rgba: [u8; 4]) -> Self {
        Self {
            id,
            name: name.into(),
            rgba,
        }
    }

    /// Get the region color as normalized floats [0.0, 1.0].
    pub fn rgba_f32(&self) -> [f32; 4] {
        [
            self.rgba[0] as f32 / 255.0,
            self.rgba[1] as f32 / 255.0,
            self.rgba[2] as f32 / 255.0,
            self.rgba[3] as f32 / 255.0,
        ]
    }
}

/// A complete brain parcellation (annotation).
///
/// This structure represents a mapping of brain surface vertices to discrete
/// anatomical regions, along with metadata about each region.
#[derive(Debug, Clone)]
pub struct Parcellation {
    /// Which hemisphere this parcellation belongs to.
    pub hemisphere: Hemisphere,

    /// Per-vertex region labels. Length must match the surface vertex count.
    /// Each value is a region ID that can be looked up in `regions`.
    /// A value of 0 typically indicates "unknown" or "medial wall".
    pub labels: Vec<u32>,

    /// List of all regions in this parcellation.
    /// The `id` field of each Region matches values in `labels`.
    pub regions: Vec<Region>,

    /// Optional name of the parcellation atlas (e.g., "aparc", "aparc.a2009s").
    pub atlas_name: Option<String>,
}

impl Parcellation {
    /// Get the number of vertices in this parcellation.
    pub fn n_vertices(&self) -> usize {
        self.labels.len()
    }

    /// Get the number of regions in this parcellation.
    pub fn n_regions(&self) -> usize {
        self.regions.len()
    }

    /// Look up a region by its ID.
    pub fn get_region(&self, id: u32) -> Option<&Region> {
        self.regions.iter().find(|r| r.id == id)
    }

    /// Get the region for a specific vertex.
    pub fn get_vertex_region(&self, vertex_idx: usize) -> Option<&Region> {
        self.labels.get(vertex_idx).and_then(|&id| self.get_region(id))
    }

    /// Get the region name for a specific vertex.
    pub fn get_vertex_region_name(&self, vertex_idx: usize) -> Option<&str> {
        self.get_vertex_region(vertex_idx).map(|r| r.name.as_str())
    }

    /// Count how many vertices are assigned to each region.
    pub fn region_vertex_counts(&self) -> std::collections::HashMap<u32, usize> {
        let mut counts = std::collections::HashMap::new();
        for &label in &self.labels {
            *counts.entry(label).or_insert(0) += 1;
        }
        counts
    }

    /// Get all unique region IDs present in the labels.
    pub fn unique_labels(&self) -> Vec<u32> {
        let mut labels: Vec<u32> = self.labels.iter().cloned().collect();
        labels.sort_unstable();
        labels.dedup();
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_parcellation() -> Parcellation {
        Parcellation {
            hemisphere: Hemisphere::Left,
            labels: vec![0, 1, 1, 2, 0, 1, 2, 2],
            regions: vec![
                Region::new(0, "unknown", [25, 5, 25, 255]),
                Region::new(1, "precentral", [60, 20, 220, 255]),
                Region::new(2, "postcentral", [220, 20, 20, 255]),
            ],
            atlas_name: Some("test".to_string()),
        }
    }

    #[test]
    fn test_parcellation_basics() {
        let p = make_test_parcellation();
        assert_eq!(p.n_vertices(), 8);
        assert_eq!(p.n_regions(), 3);
    }

    #[test]
    fn test_get_region() {
        let p = make_test_parcellation();
        let r = p.get_region(1).unwrap();
        assert_eq!(r.name, "precentral");
        assert_eq!(r.rgba, [60, 20, 220, 255]);
    }

    #[test]
    fn test_get_vertex_region() {
        let p = make_test_parcellation();
        assert_eq!(p.get_vertex_region_name(0), Some("unknown"));
        assert_eq!(p.get_vertex_region_name(1), Some("precentral"));
        assert_eq!(p.get_vertex_region_name(3), Some("postcentral"));
    }

    #[test]
    fn test_region_vertex_counts() {
        let p = make_test_parcellation();
        let counts = p.region_vertex_counts();
        assert_eq!(counts.get(&0), Some(&2));
        assert_eq!(counts.get(&1), Some(&3));
        assert_eq!(counts.get(&2), Some(&3));
    }

    #[test]
    fn test_rgba_f32() {
        let r = Region::new(0, "test", [255, 128, 0, 255]);
        let f = r.rgba_f32();
        assert!((f[0] - 1.0).abs() < 0.001);
        assert!((f[1] - 0.502).abs() < 0.01);
        assert!((f[2] - 0.0).abs() < 0.001);
        assert!((f[3] - 1.0).abs() < 0.001);
    }
}
