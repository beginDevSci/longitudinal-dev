use serde::{Deserialize, Serialize};

/// Surface identifier for multi-surface scenes.
pub type SurfaceId = u32;

/// A selected vertex with its surface context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SelectedVertex {
    /// Surface ID the vertex belongs to.
    pub surface_id: SurfaceId,
    /// Vertex index within the surface.
    pub vertex_index: u32,
}

impl SelectedVertex {
    pub fn new(surface_id: SurfaceId, vertex_index: u32) -> Self {
        Self { surface_id, vertex_index }
    }
}

/// Vertex and region selection state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    /// Primary selected vertex (the last clicked one, used for highlighting).
    pub primary: Option<SelectedVertex>,
    /// All selected vertices (including primary).
    pub vertices: Vec<SelectedVertex>,
    /// Selected region identifiers (e.g., parcel names).
    pub regions: Vec<String>,
}

impl Selection {
    /// Create a new empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all selections.
    pub fn clear(&mut self) {
        self.primary = None;
        self.vertices.clear();
        self.regions.clear();
    }

    /// Check if a vertex is selected.
    pub fn contains_vertex(&self, surface_id: SurfaceId, vertex_index: u32) -> bool {
        self.vertices.iter().any(|v| v.surface_id == surface_id && v.vertex_index == vertex_index)
    }

    /// Set a single vertex as the only selection (single-select mode).
    pub fn set_single(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        let sv = SelectedVertex::new(surface_id, vertex_index);
        self.vertices.clear();
        self.vertices.push(sv);
        self.primary = Some(sv);
        self.regions.clear();
    }

    /// Add a vertex to the selection (multi-select mode).
    /// If already selected, does nothing.
    pub fn add_vertex(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        let sv = SelectedVertex::new(surface_id, vertex_index);
        if !self.vertices.contains(&sv) {
            self.vertices.push(sv);
        }
        self.primary = Some(sv);
    }

    /// Remove a vertex from the selection.
    /// If it was the primary, primary becomes the last remaining vertex (or None).
    pub fn remove_vertex(&mut self, surface_id: SurfaceId, vertex_index: u32) {
        let sv = SelectedVertex::new(surface_id, vertex_index);
        self.vertices.retain(|v| *v != sv);
        if self.primary == Some(sv) {
            self.primary = self.vertices.last().copied();
        }
    }

    /// Toggle a vertex: add if not present, remove if present.
    /// Returns true if vertex was added, false if removed.
    pub fn toggle_vertex(&mut self, surface_id: SurfaceId, vertex_index: u32) -> bool {
        if self.contains_vertex(surface_id, vertex_index) {
            self.remove_vertex(surface_id, vertex_index);
            false
        } else {
            self.add_vertex(surface_id, vertex_index);
            true
        }
    }

    /// Get the count of selected vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Check if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.regions.is_empty()
    }

    /// Add a region to the selection.
    pub fn add_region(&mut self, region: String) {
        if !self.regions.contains(&region) {
            self.regions.push(region);
        }
    }

    /// Remove a region from the selection.
    pub fn remove_region(&mut self, region: &str) {
        self.regions.retain(|r| r != region);
    }
}

