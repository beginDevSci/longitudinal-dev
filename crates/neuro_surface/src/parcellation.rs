//! Parcellation (atlas) support for brain surfaces.
//!
//! Parcellations divide the cortical surface into discrete regions (parcels)
//! based on anatomical or functional criteria. This module provides types
//! for representing and querying parcellation data.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Unique identifier for a parcel within a parcellation.
pub type ParcelId = u32;

/// Reserved ID for unlabeled/unknown vertices.
pub const PARCEL_ID_UNKNOWN: ParcelId = 0;

/// Information about a single parcel (brain region).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelInfo {
    /// Unique identifier for this parcel.
    pub id: ParcelId,
    /// Human-readable name of the region.
    pub name: String,
    /// Short abbreviation (e.g., "STG" for Superior Temporal Gyrus).
    pub abbreviation: Option<String>,
    /// Display color as RGBA (0-255 for each component).
    pub color: [u8; 4],
    /// Number of vertices in this parcel.
    pub vertex_count: usize,
}

impl ParcelInfo {
    /// Create a new parcel with the given ID and name.
    pub fn new(id: ParcelId, name: String) -> Self {
        Self {
            id,
            name,
            abbreviation: None,
            color: [128, 128, 128, 255], // Default gray
            vertex_count: 0,
        }
    }

    /// Create a parcel with all fields specified.
    pub fn with_details(
        id: ParcelId,
        name: String,
        abbreviation: Option<String>,
        color: [u8; 4],
    ) -> Self {
        Self {
            id,
            name,
            abbreviation,
            color,
            vertex_count: 0,
        }
    }
}

/// A parcellation scheme for a brain surface.
///
/// Maps each vertex to a parcel ID and provides metadata about each parcel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parcellation {
    /// Name of the parcellation atlas (e.g., "Desikan-Killiany", "Destrieux").
    pub name: String,
    /// Parcel ID for each vertex (length = n_vertices).
    pub labels_per_vertex: Vec<ParcelId>,
    /// Information about each parcel, keyed by parcel ID.
    pub parcels: HashMap<ParcelId, ParcelInfo>,
}

impl Parcellation {
    /// Create a new empty parcellation.
    pub fn new(name: String, n_vertices: usize) -> Self {
        Self {
            name,
            labels_per_vertex: vec![PARCEL_ID_UNKNOWN; n_vertices],
            parcels: HashMap::new(),
        }
    }

    /// Create a parcellation from vertex labels.
    pub fn from_labels(name: String, labels: Vec<ParcelId>) -> Self {
        let mut parcellation = Self {
            name,
            labels_per_vertex: labels,
            parcels: HashMap::new(),
        };
        parcellation.compute_vertex_counts();
        parcellation
    }

    /// Get the parcel ID for a specific vertex.
    pub fn get_parcel_id(&self, vertex_index: usize) -> Option<ParcelId> {
        self.labels_per_vertex.get(vertex_index).copied()
    }

    /// Get parcel info for a specific vertex.
    pub fn get_parcel_info(&self, vertex_index: usize) -> Option<&ParcelInfo> {
        let id = self.get_parcel_id(vertex_index)?;
        self.parcels.get(&id)
    }

    /// Get parcel name for a specific vertex.
    pub fn get_parcel_name(&self, vertex_index: usize) -> Option<&str> {
        self.get_parcel_info(vertex_index).map(|p| p.name.as_str())
    }

    /// Get all vertices that belong to a given parcel.
    pub fn get_vertices_in_parcel(&self, parcel_id: ParcelId) -> Vec<u32> {
        self.labels_per_vertex
            .iter()
            .enumerate()
            .filter_map(|(idx, &id)| {
                if id == parcel_id {
                    Some(idx as u32)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of unique parcels (excluding unknown).
    pub fn parcel_count(&self) -> usize {
        self.parcels.len()
    }

    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.labels_per_vertex.len()
    }

    /// Add or update parcel info.
    pub fn set_parcel_info(&mut self, info: ParcelInfo) {
        self.parcels.insert(info.id, info);
    }

    /// Compute vertex counts for each parcel.
    pub fn compute_vertex_counts(&mut self) {
        // Reset counts
        for parcel in self.parcels.values_mut() {
            parcel.vertex_count = 0;
        }

        // Count vertices per parcel
        let mut counts: HashMap<ParcelId, usize> = HashMap::new();
        for &id in &self.labels_per_vertex {
            *counts.entry(id).or_insert(0) += 1;
        }

        // Update parcel info
        for (&id, &count) in &counts {
            if let Some(parcel) = self.parcels.get_mut(&id) {
                parcel.vertex_count = count;
            } else {
                // Create placeholder parcel info if not already present
                let info = ParcelInfo {
                    id,
                    name: format!("Region {}", id),
                    abbreviation: None,
                    color: [128, 128, 128, 255],
                    vertex_count: count,
                };
                self.parcels.insert(id, info);
            }
        }
    }

    /// Get list of all parcel IDs (sorted).
    pub fn parcel_ids(&self) -> Vec<ParcelId> {
        let mut ids: Vec<_> = self.parcels.keys().copied().collect();
        ids.sort();
        ids
    }

    /// Get list of all parcel names (sorted by ID).
    pub fn parcel_names(&self) -> Vec<&str> {
        let mut items: Vec<_> = self.parcels.iter().collect();
        items.sort_by_key(|(id, _)| *id);
        items.into_iter().map(|(_, info)| info.name.as_str()).collect()
    }

    /// Build a `Parcellation` from an `io_formats` parcellation.
    ///
    /// This is a convenience bridge so viewer code can load parcellations via
    /// `io_formats` and convert them into the richer `neuro_surface` structure.
    pub fn from_io_formats(src: &io_formats::parcellation::Parcellation) -> Self {
        let name = src
            .atlas_name
            .clone()
            .unwrap_or_else(|| "Unknown atlas".to_string());

        // Convert per-vertex labels.
        let labels: Vec<ParcelId> = src.labels.iter().map(|&id| id as ParcelId).collect();

        // Convert region definitions into ParcelInfo map.
        let mut parcels = HashMap::new();
        for region in &src.regions {
            let info = ParcelInfo {
                id: region.id as ParcelId,
                name: region.name.clone(),
                abbreviation: None,
                color: region.rgba,
                vertex_count: 0,
            };
            parcels.insert(info.id, info);
        }

        let mut p = Parcellation {
            name,
            labels_per_vertex: labels,
            parcels,
        };
        p.compute_vertex_counts();
        p
    }
}

/// Result of looking up a parcel by vertex.
#[derive(Debug, Clone)]
pub struct ParcelLookup {
    /// The parcel ID.
    pub parcel_id: ParcelId,
    /// The parcel info, if available.
    pub info: Option<ParcelInfo>,
    /// All vertex indices in this parcel.
    pub vertices: Vec<u32>,
}

impl Parcellation {
    /// Look up the parcel containing a vertex and return all vertices in that parcel.
    pub fn lookup_parcel(&self, vertex_index: usize) -> Option<ParcelLookup> {
        let parcel_id = self.get_parcel_id(vertex_index)?;
        let info = self.parcels.get(&parcel_id).cloned();
        let vertices = self.get_vertices_in_parcel(parcel_id);

        Some(ParcelLookup {
            parcel_id,
            info,
            vertices,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parcellation_creation() {
        let mut parc = Parcellation::new("Test Atlas".into(), 100);
        assert_eq!(parc.vertex_count(), 100);
        assert_eq!(parc.parcel_count(), 0);

        // Set some labels
        for i in 0..50 {
            parc.labels_per_vertex[i] = 1;
        }
        for i in 50..100 {
            parc.labels_per_vertex[i] = 2;
        }

        // Add parcel info
        parc.set_parcel_info(ParcelInfo::new(1, "Region A".into()));
        parc.set_parcel_info(ParcelInfo::new(2, "Region B".into()));

        parc.compute_vertex_counts();

        assert_eq!(parc.parcel_count(), 2);
        assert_eq!(parc.parcels.get(&1).unwrap().vertex_count, 50);
        assert_eq!(parc.parcels.get(&2).unwrap().vertex_count, 50);
    }

    #[test]
    fn test_parcel_lookup() {
        let labels = vec![1, 1, 1, 2, 2, 3];
        let mut parc = Parcellation::from_labels("Test".into(), labels);
        parc.set_parcel_info(ParcelInfo::new(1, "Region A".into()));
        parc.set_parcel_info(ParcelInfo::new(2, "Region B".into()));
        parc.set_parcel_info(ParcelInfo::new(3, "Region C".into()));

        let lookup = parc.lookup_parcel(0).unwrap();
        assert_eq!(lookup.parcel_id, 1);
        assert_eq!(lookup.vertices, vec![0, 1, 2]);

        let lookup = parc.lookup_parcel(4).unwrap();
        assert_eq!(lookup.parcel_id, 2);
        assert_eq!(lookup.vertices, vec![3, 4]);
    }

    #[test]
    fn test_get_parcel_name() {
        let labels = vec![1, 1, 2, 2];
        let mut parc = Parcellation::from_labels("Test".into(), labels);
        parc.set_parcel_info(ParcelInfo::new(1, "Superior Temporal Gyrus".into()));
        parc.set_parcel_info(ParcelInfo::new(2, "Inferior Frontal Gyrus".into()));

        assert_eq!(parc.get_parcel_name(0), Some("Superior Temporal Gyrus"));
        assert_eq!(parc.get_parcel_name(2), Some("Inferior Frontal Gyrus"));
    }
}
