//! Region of Interest (ROI) definitions and tools.
//!
//! ROIs allow users to define custom regions on brain surfaces for
//! analysis and visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Surface identifier for multi-surface scenes.
pub type SurfaceId = u32;

/// A vertex within an ROI, identified by surface and index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoiVertex {
    /// Surface ID the vertex belongs to.
    pub surface_id: SurfaceId,
    /// Vertex index within the surface.
    pub vertex_index: u32,
}

impl RoiVertex {
    pub fn new(surface_id: SurfaceId, vertex_index: u32) -> Self {
        Self { surface_id, vertex_index }
    }
}

/// A region of interest (ROI) definition.
///
/// ROIs are user-defined regions on brain surfaces that can be used
/// for statistical analysis, visualization, or data export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiDefinition {
    /// Unique identifier for this ROI.
    pub id: u32,
    /// Human-readable name for the ROI.
    pub name: String,
    /// Vertices included in this ROI.
    pub vertices: HashSet<RoiVertex>,
    /// Display color as RGBA (0-255 for each component).
    pub color: [u8; 4],
    /// Whether this ROI is currently visible.
    pub visible: bool,
}

impl RoiDefinition {
    /// Create a new empty ROI with the given ID and name.
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            vertices: HashSet::new(),
            color: [255, 215, 0, 180], // Default gold with some transparency
            visible: true,
        }
    }

    /// Create an ROI with a specific color.
    pub fn with_color(id: u32, name: String, color: [u8; 4]) -> Self {
        Self {
            id,
            name,
            vertices: HashSet::new(),
            color,
            visible: true,
        }
    }

    /// Add a vertex to the ROI.
    pub fn add_vertex(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        self.vertices.insert(RoiVertex::new(surface_id, vertex_index));
    }

    /// Remove a vertex from the ROI.
    pub fn remove_vertex(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        self.vertices.remove(&RoiVertex::new(surface_id, vertex_index));
    }

    /// Check if a vertex is in this ROI.
    pub fn contains(&self, surface_id: SurfaceId, vertex_index: u32) -> bool {
        self.vertices.contains(&RoiVertex::new(surface_id, vertex_index))
    }

    /// Get the number of vertices in this ROI.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Check if the ROI is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Clear all vertices from the ROI.
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// Get all vertices for a specific surface.
    pub fn vertices_for_surface(&self, surface_id: SurfaceId) -> Vec<u32> {
        self.vertices
            .iter()
            .filter_map(|v| {
                if v.surface_id == surface_id {
                    Some(v.vertex_index)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Add vertices from an iterator.
    pub fn add_vertices(&mut self, vertices: impl IntoIterator<Item = (SurfaceId, u32)>) {
        for (surface_id, vertex_index) in vertices {
            self.add_vertex(surface_id, vertex_index);
        }
    }

    /// Merge another ROI into this one.
    pub fn merge(&mut self, other: &RoiDefinition) {
        self.vertices.extend(other.vertices.iter().copied());
    }
}

/// Statistics computed over an ROI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoiStatistics {
    /// Number of vertices in the ROI.
    pub vertex_count: usize,
    /// Mean value within the ROI.
    pub mean: f32,
    /// Standard deviation of values within the ROI.
    pub std_dev: f32,
    /// Minimum value within the ROI.
    pub min: f32,
    /// Maximum value within the ROI.
    pub max: f32,
    /// Number of NaN values in the ROI.
    pub nan_count: usize,
}

impl RoiStatistics {
    /// Compute statistics from a list of values.
    pub fn from_values(values: &[f32]) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        let mut sum = 0.0f64;
        let mut sum_sq = 0.0f64;
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        let mut count = 0usize;
        let mut nan_count = 0usize;

        for &v in values {
            if v.is_nan() {
                nan_count += 1;
                continue;
            }
            if !v.is_finite() {
                continue;
            }

            sum += v as f64;
            sum_sq += (v as f64) * (v as f64);
            min = min.min(v);
            max = max.max(v);
            count += 1;
        }

        if count == 0 {
            return Self {
                vertex_count: values.len(),
                nan_count,
                ..Default::default()
            };
        }

        let mean = (sum / count as f64) as f32;
        let variance = (sum_sq / count as f64) - (mean as f64 * mean as f64);
        let std_dev = (variance.max(0.0) as f32).sqrt();

        Self {
            vertex_count: values.len(),
            mean,
            std_dev,
            min,
            max,
            nan_count,
        }
    }
}

/// Manager for multiple ROIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoiManager {
    /// List of saved ROIs.
    pub rois: Vec<RoiDefinition>,
    /// The currently active/editing ROI (not yet saved).
    pub current_roi: Option<RoiDefinition>,
    /// Next available ROI ID.
    next_id: u32,
}

impl RoiManager {
    /// Create a new empty ROI manager.
    pub fn new() -> Self {
        Self {
            rois: Vec::new(),
            current_roi: None,
            next_id: 1,
        }
    }

    /// Start a new ROI with the given name.
    pub fn start_new_roi(&mut self, name: String) {
        self.current_roi = Some(RoiDefinition::new(self.next_id, name));
    }

    /// Start a new ROI with a default name.
    pub fn start_new_roi_default(&mut self) {
        let name = format!("ROI {}", self.next_id);
        self.start_new_roi(name);
    }

    /// Add a vertex to the current ROI.
    pub fn add_vertex_to_current(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        if let Some(ref mut roi) = self.current_roi {
            roi.add_vertex(surface_id, vertex_index);
        }
    }

    /// Remove a vertex from the current ROI.
    pub fn remove_vertex_from_current(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        if let Some(ref mut roi) = self.current_roi {
            roi.remove_vertex(surface_id, vertex_index);
        }
    }

    /// Save the current ROI to the list.
    pub fn save_current(&mut self) -> Option<u32> {
        if let Some(roi) = self.current_roi.take() {
            if roi.is_empty() {
                return None;
            }
            let id = roi.id;
            self.rois.push(roi);
            self.next_id += 1;
            Some(id)
        } else {
            None
        }
    }

    /// Clear the current ROI without saving.
    pub fn discard_current(&mut self) {
        self.current_roi = None;
    }

    /// Clear all vertices from the current ROI.
    pub fn clear_current(&mut self) {
        if let Some(ref mut roi) = self.current_roi {
            roi.clear();
        }
    }

    /// Get a saved ROI by ID.
    pub fn get_roi(&self, id: u32) -> Option<&RoiDefinition> {
        self.rois.iter().find(|r| r.id == id)
    }

    /// Get a mutable reference to a saved ROI by ID.
    pub fn get_roi_mut(&mut self, id: u32) -> Option<&mut RoiDefinition> {
        self.rois.iter_mut().find(|r| r.id == id)
    }

    /// Delete a saved ROI by ID.
    pub fn delete_roi(&mut self, id: u32) -> bool {
        if let Some(idx) = self.rois.iter().position(|r| r.id == id) {
            self.rois.remove(idx);
            true
        } else {
            false
        }
    }

    /// Rename a saved ROI.
    pub fn rename_roi(&mut self, id: u32, new_name: String) -> bool {
        if let Some(roi) = self.get_roi_mut(id) {
            roi.name = new_name;
            true
        } else {
            false
        }
    }

    /// Get the number of saved ROIs.
    pub fn roi_count(&self) -> usize {
        self.rois.len()
    }

    /// Check if a vertex is in any ROI (saved or current).
    pub fn is_vertex_in_any_roi(&self, surface_id: SurfaceId, vertex_index: u32) -> bool {
        if let Some(ref roi) = self.current_roi {
            if roi.contains(surface_id, vertex_index) {
                return true;
            }
        }
        self.rois.iter().any(|r| r.contains(surface_id, vertex_index))
    }

    /// Get all ROI IDs that contain a vertex.
    pub fn rois_containing_vertex(&self, surface_id: SurfaceId, vertex_index: u32) -> Vec<u32> {
        self.rois
            .iter()
            .filter_map(|r| {
                if r.contains(surface_id, vertex_index) {
                    Some(r.id)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roi_definition() {
        let mut roi = RoiDefinition::new(1, "Test ROI".into());
        assert!(roi.is_empty());

        roi.add_vertex(0, 100);
        roi.add_vertex(0, 101);
        roi.add_vertex(1, 50);

        assert_eq!(roi.vertex_count(), 3);
        assert!(roi.contains(0, 100));
        assert!(!roi.contains(0, 99));

        roi.remove_vertex(0, 100);
        assert_eq!(roi.vertex_count(), 2);
        assert!(!roi.contains(0, 100));
    }

    #[test]
    fn test_roi_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = RoiStatistics::from_values(&values);

        assert_eq!(stats.vertex_count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.nan_count, 0);
    }

    #[test]
    fn test_roi_statistics_with_nan() {
        let values = vec![1.0, f32::NAN, 3.0, f32::NAN, 5.0];
        let stats = RoiStatistics::from_values(&values);

        assert_eq!(stats.vertex_count, 5);
        assert_eq!(stats.nan_count, 2);
        assert!((stats.mean - 3.0).abs() < 0.001); // mean of 1, 3, 5
    }

    #[test]
    fn test_roi_manager() {
        let mut manager = RoiManager::new();

        manager.start_new_roi("Test ROI 1".into());
        manager.add_vertex_to_current(0, 1);
        manager.add_vertex_to_current(0, 2);
        manager.add_vertex_to_current(0, 3);

        let id = manager.save_current().unwrap();
        assert_eq!(id, 1);
        assert_eq!(manager.roi_count(), 1);

        let roi = manager.get_roi(1).unwrap();
        assert_eq!(roi.vertex_count(), 3);
        assert_eq!(roi.name, "Test ROI 1");

        manager.rename_roi(1, "Renamed ROI".into());
        assert_eq!(manager.get_roi(1).unwrap().name, "Renamed ROI");

        manager.delete_roi(1);
        assert_eq!(manager.roi_count(), 0);
    }
}
