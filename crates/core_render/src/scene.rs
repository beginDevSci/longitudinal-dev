//! Scene graph for multi-surface rendering.
//!
//! This module provides a node-based scene structure that supports
//! multiple surfaces (e.g., left/right hemispheres) with visibility control
//! and per-surface transforms.

use glam::Vec3;

use crate::resources::SurfaceId;

/// Unique identifier for a scene node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// Simple 3D transform (translation only for now).
#[derive(Debug, Clone, Copy)]
pub struct Transform3D {
    /// Translation offset.
    pub translation: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
        }
    }
}

impl Transform3D {
    /// Create a new transform with the given translation.
    pub fn from_translation(translation: Vec3) -> Self {
        Self { translation }
    }
}

/// Marker style options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkerStyle {
    /// Size multiplier (1.0 = default size).
    pub size: f32,
    /// Whether this marker is selected/highlighted.
    pub selected: bool,
}

impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            size: 1.0,
            selected: false,
        }
    }
}

/// Content type for a scene node.
#[derive(Debug, Clone, Copy)]
pub enum NodeContent {
    /// A surface mesh node.
    Surface { surface_id: SurfaceId },
    /// A point marker (for annotations, selected vertices, etc.).
    Marker {
        /// World-space position of the marker.
        position: Vec3,
        /// RGB color of the marker (0.0-1.0).
        color: [f32; 3],
        /// Marker style options.
        style: MarkerStyle,
    },
}

/// A node in the scene graph.
#[derive(Debug)]
pub struct SceneNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Content of this node.
    pub content: NodeContent,
    /// Whether this node should be rendered.
    pub visible: bool,
    /// Transform applied to this node.
    pub transform: Transform3D,
}

/// Scene graph managing multiple renderable nodes.
pub struct Scene {
    nodes: Vec<SceneNode>,
    next_id: u32,
}

impl Scene {
    /// Create a new empty scene.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a surface to the scene and return its node ID.
    pub fn add_surface(&mut self, surface_id: SurfaceId) -> NodeId {
        self.add_surface_with_transform(surface_id, Transform3D::default())
    }

    /// Add a surface with a specific transform.
    pub fn add_surface_with_transform(
        &mut self,
        surface_id: SurfaceId,
        transform: Transform3D,
    ) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.push(SceneNode {
            id,
            content: NodeContent::Surface { surface_id },
            visible: true,
            transform,
        });

        id
    }

    /// Remove all nodes from the scene.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Iterate over all visible surfaces with their transforms.
    pub fn iter_surfaces_with_transforms(&self) -> impl Iterator<Item = (SurfaceId, Transform3D)> + '_ {
        self.nodes.iter().filter(|n| n.visible).filter_map(|n| match n.content {
            NodeContent::Surface { surface_id } => Some((surface_id, n.transform)),
            NodeContent::Marker { .. } => None,
        })
    }

    /// Iterate over all visible surface IDs (for backwards compatibility).
    pub fn iter_surfaces(&self) -> impl Iterator<Item = SurfaceId> + '_ {
        self.nodes.iter().filter(|n| n.visible).filter_map(|n| {
            match n.content {
                NodeContent::Surface { surface_id } => Some(surface_id),
                NodeContent::Marker { .. } => None,
            }
        })
    }

    /// Add a marker to the scene and return its node ID.
    pub fn add_marker(&mut self, position: Vec3, color: [f32; 3]) -> NodeId {
        self.add_marker_with_style(position, color, MarkerStyle::default())
    }

    /// Add a marker with custom style to the scene and return its node ID.
    pub fn add_marker_with_style(&mut self, position: Vec3, color: [f32; 3], style: MarkerStyle) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.push(SceneNode {
            id,
            content: NodeContent::Marker { position, color, style },
            visible: true,
            transform: Transform3D::default(),
        });

        id
    }

    /// Update an existing marker's position and color.
    pub fn update_marker(&mut self, node_id: NodeId, position: Vec3, color: [f32; 3]) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            if let NodeContent::Marker { style, .. } = node.content {
                node.content = NodeContent::Marker { position, color, style };
            }
        }
    }

    /// Update an existing marker's style.
    pub fn update_marker_style(&mut self, node_id: NodeId, style: MarkerStyle) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            if let NodeContent::Marker { position, color, .. } = node.content {
                node.content = NodeContent::Marker { position, color, style };
            }
        }
    }

    /// Get the style of a marker.
    pub fn get_marker_style(&self, node_id: NodeId) -> Option<MarkerStyle> {
        self.nodes.iter().find(|n| n.id == node_id).and_then(|n| {
            match n.content {
                NodeContent::Marker { style, .. } => Some(style),
                _ => None,
            }
        })
    }

    /// Remove a marker by its node ID.
    pub fn remove_marker(&mut self, node_id: NodeId) {
        self.nodes.retain(|n| n.id != node_id);
    }

    /// Clear all markers from the scene (keeps surfaces).
    pub fn clear_markers(&mut self) {
        self.nodes.retain(|n| matches!(n.content, NodeContent::Surface { .. }));
    }

    /// Iterate over all visible markers.
    pub fn iter_markers(&self) -> impl Iterator<Item = &SceneNode> + '_ {
        self.nodes.iter().filter(|n| n.visible && matches!(n.content, NodeContent::Marker { .. }))
    }

    /// Get the number of markers in the scene.
    pub fn marker_count(&self) -> usize {
        self.nodes.iter().filter(|n| matches!(n.content, NodeContent::Marker { .. })).count()
    }

    /// Set the transform for a specific node.
    pub fn set_transform(&mut self, node_id: NodeId, transform: Transform3D) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.transform = transform;
        }
    }

    /// Get the transform for a specific surface.
    pub fn get_surface_transform(&self, surface_id: SurfaceId) -> Option<Transform3D> {
        self.nodes.iter().find_map(|n| match n.content {
            NodeContent::Surface { surface_id: sid } if sid == surface_id => Some(n.transform),
            _ => None,
        })
    }

    /// Set the transform for a specific surface by surface ID.
    pub fn set_surface_transform(&mut self, surface_id: SurfaceId, transform: Transform3D) {
        if let Some(node) = self.nodes.iter_mut().find(|n| match n.content {
            NodeContent::Surface { surface_id: sid } => sid == surface_id,
            NodeContent::Marker { .. } => false,
        }) {
            node.transform = transform;
        }
    }

    /// Check if the scene has any surfaces.
    pub fn has_surface(&self) -> bool {
        self.nodes
            .iter()
            .any(|n| matches!(n.content, NodeContent::Surface { .. }))
    }

    /// Set visibility for a specific node.
    pub fn set_visible(&mut self, node_id: NodeId, visible: bool) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.visible = visible;
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
