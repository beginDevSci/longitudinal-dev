//! ABOUTME: Adapter to integrate core_render::WgpuRenderer with viewer_app's BrainRendererBackend trait.
//! ABOUTME: Handles async picking, event translation, and manages selection/ROI state.

use std::cell::Cell;
use std::collections::VecDeque;

use web_sys::HtmlCanvasElement;

/// Callback context for pending pick (what to do when pick completes).
#[derive(Clone)]
enum PendingPickContext {
    Click { modifiers: ModifierKeys },
    Hover,
    RoiPaint { erase: bool },
}

use crate::data::{BrainGeometry, StatisticData};
use crate::renderer::traits::{BrainRendererBackend, CameraState, EventResult, RendererError};
use crate::types::{
    ColormapType, Hemisphere, LayoutMode, ModifierKeys, MousePosition, VertexInfo, ViewerEvent,
};

use core_render::resources::{ColormapKind, OverlayId, SurfaceId};
use core_render::scene::NodeId;
use core_render::traits::BrainRendererBackend as CoreBrainRendererBackend;
use core_render::wgpu_renderer::WgpuRenderer;
use glam::Vec3;
use interaction::{History, HoverState, RoiManager, RoiStatistics, Selection};
use neuro_surface::{BrainSurface, BrainViewPreset, Parcellation};

/// Surface ID for the left hemisphere.
pub const SURFACE_ID_LEFT: SurfaceId = 0;
/// Surface ID for the right hemisphere.
pub const SURFACE_ID_RIGHT: SurfaceId = 1;

/// Minimum interval between hover picks in milliseconds.
const HOVER_THROTTLE_MS: f64 = 50.0;

/// Time in milliseconds before hover info decays when cursor moves off surface.
#[allow(dead_code)] // Scaffolding for hover decay feature
const HOVER_DECAY_MS: f64 = 200.0;

/// Brush radius in world units for ROI painting.
const DEFAULT_BRUSH_RADIUS: f32 = 5.0;
/// Number of 1-ring expansions to apply to ROI masks for visualization.
const ROI_MASK_DILATION_STEPS: usize = 1;

/// Offset for side-by-side layout (each hemisphere shifted by this amount on X axis).
const LAYOUT_OFFSET_X: f32 = 40.0;

/// Offset for stacked layout (each hemisphere shifted by this amount on Y axis).
const LAYOUT_OFFSET_Y: f32 = 50.0;

/// Wrapper around WgpuRenderer that implements viewer_app's BrainRendererBackend.
pub struct WgpuRendererAdapter {
    renderer: WgpuRenderer,
    running: Cell<bool>,
    /// Legacy single geometry field (for backwards compatibility with set_geometry)
    geometry: Option<BrainGeometry>,
    /// Left hemisphere geometry (for dual-hemisphere picking)
    geometry_left: Option<BrainGeometry>,
    /// Right hemisphere geometry (for dual-hemisphere picking)
    geometry_right: Option<BrainGeometry>,
    statistics: Option<StatisticData>,
    active_volume: u32,
    threshold: Option<f32>,
    symmetric: bool,
    /// Last time a hover pick was performed (for throttling).
    last_hover_pick_time: f64,
    /// Whether mouse is currently dragging (suppress hover during drag).
    is_dragging: bool,
    /// Current layout mode for dual-hemisphere display.
    layout_mode: LayoutMode,
    /// Currently selected vertex info (stored here for easy clearing).
    selected_vertex: Option<VertexInfo>,
    /// Surface ID of the selected vertex.
    selected_surface_id: Option<SurfaceId>,
    /// Multi-select state with history for undo/redo.
    selection_history: History<Selection>,
    /// Current hover state (for throttling and decay).
    #[allow(dead_code)] // Scaffolding for hover info display
    hover_state: HoverState,
    /// Parcellation for left hemisphere (optional).
    parcellation_left: Option<Parcellation>,
    /// Parcellation for right hemisphere (optional).
    parcellation_right: Option<Parcellation>,
    /// Whether region selection mode is enabled.
    region_selection_mode: bool,
    /// ROI manager for user-defined regions.
    roi_manager: RoiManager,
    /// Whether ROI drawing mode is enabled.
    roi_drawing_mode: bool,
    /// Current brush radius for ROI painting.
    brush_radius: f32,
    /// Whether currently painting (mouse down in ROI mode).
    is_painting: bool,
    /// Whether erasing (right-click or modifier while painting).
    is_erasing: bool,
    /// Last screen position used for ROI painting interpolation.
    last_paint_position: Option<MousePosition>,
    /// Queue of screen positions to paint (for interpolated strokes).
    paint_queue: VecDeque<MousePosition>,
    /// Node ID for the selection marker (if any).
    selection_marker_id: Option<NodeId>,
    /// Annotation markers with their vertex info.
    annotations: Vec<(NodeId, VertexInfo)>,
    /// Callback context for pending pick (what to do when pick completes).
    pending_pick_context: Option<PendingPickContext>,
}

// Many methods in this impl are scaffolding for ROI, annotation, and advanced
// selection features whose UI components are not yet implemented.
#[allow(dead_code)]
impl WgpuRendererAdapter {
    /// Create a new adapter from a canvas element.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, RendererError> {
        let renderer = WgpuRenderer::new(canvas)
            .await
            .map_err(|e| RendererError::ContextCreation(format!("{}", e)))?;

        Ok(Self {
            renderer,
            running: Cell::new(true),
            geometry: None,
            geometry_left: None,
            geometry_right: None,
            statistics: None,
            active_volume: 0,
            threshold: None,
            symmetric: false,
            last_hover_pick_time: 0.0,
            is_dragging: false,
            layout_mode: LayoutMode::Single,
            selected_vertex: None,
            selected_surface_id: None,
            selection_history: History::new(Selection::new()),
            hover_state: HoverState::default(),
            parcellation_left: None,
            parcellation_right: None,
            region_selection_mode: false,
            roi_manager: RoiManager::new(),
            roi_drawing_mode: false,
            brush_radius: DEFAULT_BRUSH_RADIUS,
            is_painting: false,
            is_erasing: false,
            last_paint_position: None,
            paint_queue: VecDeque::new(),
            selection_marker_id: None,
            annotations: Vec::new(),
            pending_pick_context: None,
        })
    }

    /// Set surface geometry on the renderer.
    #[allow(dead_code)] // Scaffolding for single-surface API
    pub fn set_surface(&mut self, geom: &BrainGeometry) {
        self.geometry = Some(geom.clone());
        self.renderer.set_surface(SURFACE_ID_LEFT, geom);
    }

    /// Set brain surfaces from a BrainSurface containing optional left/right hemispheres.
    ///
    /// This uploads each available hemisphere to the GPU as a separate surface
    /// and updates the scene to render all surfaces.
    pub fn set_brain_surfaces(&mut self, brain_surface: &BrainSurface) {
        // Build list of surfaces to upload
        let mut surfaces: Vec<(SurfaceId, &BrainGeometry)> = Vec::new();

        // Store and upload left hemisphere
        if let Some(ref left) = brain_surface.left {
            self.geometry_left = Some(left.geometry.clone());
            surfaces.push((SURFACE_ID_LEFT, &left.geometry));
        } else {
            self.geometry_left = None;
        }

        // Store and upload right hemisphere
        if let Some(ref right) = brain_surface.right {
            self.geometry_right = Some(right.geometry.clone());
            surfaces.push((SURFACE_ID_RIGHT, &right.geometry));
        } else {
            self.geometry_right = None;
        }

        // Upload all surfaces to the renderer
        self.renderer.set_surfaces(&surfaces);

        // For backwards compatibility with set_geometry, also set the legacy geometry field
        self.geometry = self
            .geometry_left
            .clone()
            .or_else(|| self.geometry_right.clone());
    }

    /// Set overlay data from statistics.
    pub fn set_overlay_from_stats(
        &mut self,
        stats: &StatisticData,
        volume_idx: usize,
        threshold: Option<f32>,
    ) {
        self.statistics = Some(stats.clone());
        self.active_volume = volume_idx as u32;
        self.threshold = threshold;

        // Validate volume index
        if volume_idx >= stats.n_volumes {
            log::error!(
                "set_overlay_from_stats: volume_idx {} >= n_volumes {}",
                volume_idx,
                stats.n_volumes
            );
        }

        // Get overlay values for the specified volume
        let values = match stats.volume_slice(volume_idx) {
            Some(slice) => {
                if slice.len() != stats.n_vertices {
                    log::error!(
                        "Volume slice length {} != n_vertices {}",
                        slice.len(),
                        stats.n_vertices
                    );
                }
                slice
            }
            None => {
                log::error!(
                    "Failed to get volume slice {}, using first n_vertices",
                    volume_idx
                );
                &stats.values[..stats.n_vertices]
            }
        };

        // Get base range for this volume
        let base_range = stats
            .volume_ranges
            .get(volume_idx)
            .cloned()
            .unwrap_or((stats.global_min, stats.global_max));

        // Apply symmetric scaling if enabled
        let range = self.compute_range(base_range);

        self.renderer
            .set_overlay(0 as OverlayId, values, range, threshold);
    }

    /// Show only the selected hemisphere by re-uploading the matching surface.
    pub fn show_hemisphere(&mut self, hemi: Hemisphere) {
        match hemi {
            Hemisphere::Left => {
                if let Some(ref left) = self.geometry_left {
                    self.renderer.set_surfaces(&[(SURFACE_ID_LEFT, left)]);
                } else if let Some(ref geom) = self.geometry {
                    self.renderer.set_surfaces(&[(SURFACE_ID_LEFT, geom)]);
                }
            }
            Hemisphere::Right => {
                if let Some(ref right) = self.geometry_right {
                    self.renderer.set_surfaces(&[(SURFACE_ID_RIGHT, right)]);
                } else if let Some(ref geom) = self.geometry {
                    self.renderer.set_surfaces(&[(SURFACE_ID_RIGHT, geom)]);
                }
            }
        }

        // Re-apply overlay for the active volume after surface swap.
        if let Some(stats) = self.statistics.clone() {
            self.set_overlay_from_stats(&stats, self.active_volume as usize, self.threshold);
        }

        // Re-apply layout transforms so offsets remain consistent.
        self.apply_layout_transforms();
    }

    /// Set the layout mode for dual-hemisphere display.
    ///
    /// - `Single`: Both hemispheres centered (overlapping)
    /// - `SideBySide`: Hemispheres separated on X axis
    #[allow(dead_code)] // Called via trait, keeping for direct API
    pub fn set_layout(&mut self, mode: LayoutMode) {
        if self.layout_mode == mode {
            return;
        }
        self.layout_mode = mode;
        self.apply_layout_transforms();
    }

    /// Apply transforms based on the current layout mode.
    fn apply_layout_transforms(&mut self) {
        match self.layout_mode {
            LayoutMode::Single => {
                // Both hemispheres centered (no offset)
                self.renderer
                    .set_surface_transform(SURFACE_ID_LEFT, Vec3::ZERO);
                self.renderer
                    .set_surface_transform(SURFACE_ID_RIGHT, Vec3::ZERO);
            }
            LayoutMode::SideBySide => {
                // Left hemisphere on left side (-X), right hemisphere on right side (+X)
                self.renderer
                    .set_surface_transform(SURFACE_ID_LEFT, Vec3::new(-LAYOUT_OFFSET_X, 0.0, 0.0));
                self.renderer
                    .set_surface_transform(SURFACE_ID_RIGHT, Vec3::new(LAYOUT_OFFSET_X, 0.0, 0.0));
            }
            LayoutMode::Stacked => {
                // Left hemisphere on top (+Y), right hemisphere on bottom (-Y)
                self.renderer
                    .set_surface_transform(SURFACE_ID_LEFT, Vec3::new(0.0, LAYOUT_OFFSET_Y, 0.0));
                self.renderer
                    .set_surface_transform(SURFACE_ID_RIGHT, Vec3::new(0.0, -LAYOUT_OFFSET_Y, 0.0));
            }
        }
    }

    /// Update the overlay for the current statistics.
    fn update_overlay(&mut self) {
        if let Some(ref stats) = self.statistics {
            let vol_idx = self.active_volume as usize;

            // Validate volume index
            if vol_idx >= stats.n_volumes {
                log::error!(
                    "Volume index {} out of bounds (n_volumes={}), using volume 0",
                    vol_idx,
                    stats.n_volumes
                );
            }

            let values = match stats.volume_slice(vol_idx) {
                Some(slice) => {
                    // Validate slice length
                    if slice.len() != stats.n_vertices {
                        log::error!(
                            "Volume slice length {} does not match n_vertices {}",
                            slice.len(),
                            stats.n_vertices
                        );
                    }
                    slice
                }
                None => {
                    log::error!(
                        "Failed to get volume slice for index {}, falling back to first n_vertices",
                        vol_idx
                    );
                    &stats.values[..stats.n_vertices]
                }
            };

            // Get base range for this volume
            let base_range = stats
                .volume_ranges
                .get(vol_idx)
                .cloned()
                .unwrap_or((stats.global_min, stats.global_max));

            // Apply symmetric scaling if enabled
            let range = if self.symmetric {
                let max_abs = base_range.0.abs().max(base_range.1.abs());
                (-max_abs, max_abs)
            } else {
                base_range
            };

            self.renderer
                .set_overlay(0 as OverlayId, values, range, self.threshold);
        }
    }

    /// Compute the effective range, applying symmetric scaling if enabled.
    fn compute_range(&self, base_range: (f32, f32)) -> (f32, f32) {
        if self.symmetric {
            let max_abs = base_range.0.abs().max(base_range.1.abs());
            (-max_abs, max_abs)
        } else {
            base_range
        }
    }

    /// Perform a pick at the given screen coordinates and return vertex info.
    #[allow(dead_code)] // Synchronous picking API, async path preferred
    fn pick_vertex(&mut self, x: f32, y: f32) -> Option<VertexInfo> {
        let pick_result = self.renderer.pick(x, y)?;
        let idx = pick_result.vertex_index?;
        let surface_id = pick_result.surface_id.unwrap_or(SURFACE_ID_LEFT);

        // Choose the correct geometry based on surface_id
        let geom = match pick_result.surface_id {
            Some(id) if id == SURFACE_ID_LEFT => self.geometry_left.as_ref(),
            Some(id) if id == SURFACE_ID_RIGHT => self.geometry_right.as_ref(),
            _ => self
                .geometry_left
                .as_ref()
                .or(self.geometry_right.as_ref())
                .or(self.geometry.as_ref()),
        };

        // Look up vertex position from the correct geometry
        let pos = geom
            .and_then(|g| g.vertices.get(idx as usize))
            .copied()
            .unwrap_or([0.0, 0.0, 0.0]);

        let value = self
            .statistics
            .as_ref()
            .and_then(|s| s.get(self.active_volume as usize, idx as usize))
            .unwrap_or(f32::NAN);

        Some(VertexInfo {
            index: idx,
            position: pos,
            value,
            surface_id: Some(surface_id),
        })
    }

    /// Build a VertexInfo from a PickResult (for async pick results).
    fn build_vertex_info(
        &self,
        pick_result: &core_render::traits::PickResult,
    ) -> Option<VertexInfo> {
        let idx = pick_result.vertex_index?;
        let surface_id = pick_result.surface_id.unwrap_or(SURFACE_ID_LEFT);

        // Choose the correct geometry based on surface_id
        let geom = match pick_result.surface_id {
            Some(id) if id == SURFACE_ID_LEFT => self.geometry_left.as_ref(),
            Some(id) if id == SURFACE_ID_RIGHT => self.geometry_right.as_ref(),
            _ => self
                .geometry_left
                .as_ref()
                .or(self.geometry_right.as_ref())
                .or(self.geometry.as_ref()),
        };

        // Look up vertex position from the correct geometry
        let pos = geom
            .and_then(|g| g.vertices.get(idx as usize))
            .copied()
            .unwrap_or([0.0, 0.0, 0.0]);

        let value = self
            .statistics
            .as_ref()
            .and_then(|s| s.get(self.active_volume as usize, idx as usize))
            .unwrap_or(f32::NAN);

        Some(VertexInfo {
            index: idx,
            position: pos,
            value,
            surface_id: Some(surface_id),
        })
    }

    /// Poll for pending pick results and handle them.
    /// Returns EventResult with any clicked/hovered vertex info.
    fn poll_pending_picks(&mut self) -> EventResult {
        let mut result = EventResult::default();

        if let Some(pick_result) = self.renderer.poll_pick() {
            if let Some(context) = self.pending_pick_context.take() {
                match context {
                    PendingPickContext::Click { modifiers } => {
                        if let Some(vertex_info) = self.build_vertex_info(&pick_result) {
                            if self.region_selection_mode {
                                self.select_region(&vertex_info, &modifiers);
                            } else {
                                self.handle_selection(&vertex_info, &modifiers);
                            }
                            result.clicked = Some(vertex_info);
                        }
                    }
                    PendingPickContext::Hover => {
                        result.hovered = self.build_vertex_info(&pick_result);
                    }
                    PendingPickContext::RoiPaint { erase } => {
                        if let Some(vertex_info) = self.build_vertex_info(&pick_result) {
                            self.paint_brush(&vertex_info, erase);
                        }
                        if self.is_painting {
                            self.request_next_paint_pick(erase);
                        } else {
                            self.reset_paint_queue();
                        }
                    }
                }
            }
        }

        result
    }

    /// Perform hover pick with decay logic.
    /// Returns the hovered vertex, keeping the last hover for a short time after cursor moves off.
    #[allow(dead_code)] // Scaffolding for hover decay feature
    fn hover_pick_with_decay(&mut self, x: f32, y: f32) -> Option<VertexInfo> {
        let now = Self::now_ms();

        if let Some(info) = self.pick_vertex(x, y) {
            // Got a hit, update hover state
            self.hover_state.set_hit(
                info.index,
                info.surface_id.unwrap_or(SURFACE_ID_LEFT),
                None,
                now,
            );
            Some(info)
        } else {
            // No hit - check if we should return the decayed hover based on HoverState
            if !self.hover_state.should_decay(now, HOVER_DECAY_MS) {
                if let (Some(vertex), Some(surface_id)) =
                    (self.hover_state.vertex, self.hover_state.surface_id)
                {
                    // Reconstruct minimal VertexInfo from cached state
                    if let Some(info) = self.pick_vertex(x, y) {
                        return Some(info);
                    } else {
                        return Some(VertexInfo {
                            index: vertex,
                            position: [0.0, 0.0, 0.0],
                            value: f32::NAN,
                            surface_id: Some(surface_id),
                        });
                    }
                }
            }
            // Decay expired, clear hover
            self.hover_state.clear();
            None
        }
    }

    /// Handle selection based on click with modifier keys.
    fn handle_selection(&mut self, info: &VertexInfo, modifiers: &ModifierKeys) {
        let surface_id = info.surface_id.unwrap_or(SURFACE_ID_LEFT);

        if modifiers.shift {
            // Multi-select: toggle this vertex
            let mut new_selection = self.selection_history.present().clone();
            new_selection.toggle_vertex(surface_id, info.index);
            self.selection_history.apply(new_selection);
        } else {
            // Single-select: replace selection
            let mut new_selection = Selection::new();
            new_selection.set_single(surface_id, info.index);
            self.selection_history.apply(new_selection);
        }

        // Update renderer selection state for visual highlighting
        // (GPU highlights only the primary selection for now)
        if let Some(primary) = self.selection_history.present().primary {
            self.renderer
                .set_selected_vertex(Some(primary.vertex_index), Some(primary.surface_id));
        } else {
            self.renderer.clear_selection();
        }

        self.selected_vertex = Some(info.clone());
        self.selected_surface_id = Some(surface_id);

        // Update the selection marker
        self.update_selection_marker();
    }

    /// Undo the last selection change.
    pub fn undo_selection(&mut self) -> bool {
        if self.selection_history.undo() {
            self.sync_selection_to_renderer();
            true
        } else {
            false
        }
    }

    /// Redo the last undone selection change.
    pub fn redo_selection(&mut self) -> bool {
        if self.selection_history.redo() {
            self.sync_selection_to_renderer();
            true
        } else {
            false
        }
    }

    /// Sync selection state to the renderer for visual highlighting.
    fn sync_selection_to_renderer(&mut self) {
        if let Some(primary) = self.selection_history.present().primary {
            self.renderer
                .set_selected_vertex(Some(primary.vertex_index), Some(primary.surface_id));
        } else {
            self.renderer.clear_selection();
        }
    }

    /// Get the current selection.
    #[allow(dead_code)] // Scaffolding for selection history queries
    pub fn selection(&self) -> &Selection {
        self.selection_history.present()
    }

    /// Set parcellation data for a hemisphere.
    pub fn set_parcellation(&mut self, surface_id: SurfaceId, parcellation: Parcellation) {
        if surface_id == SURFACE_ID_LEFT {
            self.parcellation_left = Some(parcellation);
        } else if surface_id == SURFACE_ID_RIGHT {
            self.parcellation_right = Some(parcellation);
        }
    }

    /// Get parcellation for a surface.
    #[allow(dead_code)] // Scaffolding for parcellation queries
    pub fn get_parcellation(&self, surface_id: SurfaceId) -> Option<&Parcellation> {
        if surface_id == SURFACE_ID_LEFT {
            self.parcellation_left.as_ref()
        } else if surface_id == SURFACE_ID_RIGHT {
            self.parcellation_right.as_ref()
        } else {
            None
        }
    }

    /// Enable or disable region selection mode.
    pub fn set_region_selection_mode(&mut self, enabled: bool) {
        self.region_selection_mode = enabled;
    }

    /// Check if region selection mode is enabled.
    #[allow(dead_code)] // Scaffolding for region selection UI
    pub fn is_region_selection_mode(&self) -> bool {
        self.region_selection_mode
    }

    /// Select all vertices in a region (parcel) containing the given vertex.
    fn select_region(&mut self, info: &VertexInfo, modifiers: &ModifierKeys) {
        let surface_id = info.surface_id.unwrap_or(SURFACE_ID_LEFT);

        // Get parcellation for this surface
        let parcellation = if surface_id == SURFACE_ID_LEFT {
            self.parcellation_left.as_ref()
        } else {
            self.parcellation_right.as_ref()
        };

        if let Some(parc) = parcellation {
            if let Some(lookup) = parc.lookup_parcel(info.index as usize) {
                let mut new_selection = if modifiers.shift {
                    // Multi-select: add to existing selection
                    self.selection_history.present().clone()
                } else {
                    // Single-select: replace selection
                    Selection::new()
                };

                // Add all vertices in the region
                for &vertex_idx in &lookup.vertices {
                    if !new_selection.contains_vertex(surface_id, vertex_idx) {
                        new_selection.add_vertex(surface_id, vertex_idx);
                    }
                }

                // Set primary to the clicked vertex
                new_selection.primary =
                    Some(interaction::SelectedVertex::new(surface_id, info.index));

                // Add region name to selection
                if let Some(ref info) = lookup.info {
                    if !new_selection.regions.contains(&info.name) {
                        new_selection.regions.push(info.name.clone());
                    }
                }

                self.selection_history.apply(new_selection);
                self.sync_selection_to_renderer();
            }
        } else {
            // No parcellation available, fall back to single vertex selection
            self.handle_selection(info, modifiers);
        }
    }

    /// Get region info for a vertex (if parcellation is available).
    pub fn get_region_info(&self, surface_id: SurfaceId, vertex_index: u32) -> Option<String> {
        let parc = if surface_id == SURFACE_ID_LEFT {
            self.parcellation_left.as_ref()
        } else {
            self.parcellation_right.as_ref()
        }?;

        parc.get_parcel_name(vertex_index as usize)
            .map(|s| s.to_string())
    }

    /// Set the selected vertex for visual highlighting.
    #[allow(dead_code)] // Scaffolding for programmatic selection
    pub fn set_selected_vertex(
        &mut self,
        vertex_info: Option<VertexInfo>,
        surface_id: Option<SurfaceId>,
    ) {
        self.selected_vertex = vertex_info.clone();
        self.selected_surface_id = surface_id;

        match vertex_info {
            Some(info) => {
                self.renderer
                    .set_selected_vertex(Some(info.index), surface_id);
            }
            None => {
                self.renderer.clear_selection();
            }
        }
    }

    /// Clear the current vertex selection.
    pub fn clear_selection(&mut self) {
        self.selected_vertex = None;
        self.selected_surface_id = None;
        self.renderer.clear_selection();

        // Remove selection marker
        if let Some(marker_id) = self.selection_marker_id.take() {
            self.renderer.remove_marker(marker_id);
        }
    }

    // ==================== ROI Drawing Methods ====================

    /// Enable or disable ROI drawing mode.
    pub fn set_roi_drawing_mode(&mut self, enabled: bool) {
        self.roi_drawing_mode = enabled;
        if enabled && self.roi_manager.current_roi.is_none() {
            // Automatically start a new ROI when entering drawing mode
            self.roi_manager.start_new_roi_default();
        }
    }

    /// Check if ROI drawing mode is enabled.
    #[allow(dead_code)] // Scaffolding for ROI mode queries
    pub fn is_roi_drawing_mode(&self) -> bool {
        self.roi_drawing_mode
    }

    /// Start a new ROI with the given name.
    #[allow(dead_code)] // Scaffolding for ROI creation UI
    pub fn start_new_roi(&mut self, name: String) {
        self.roi_manager.start_new_roi(name);
    }

    /// Save the current ROI and return its ID.
    #[allow(dead_code)] // Scaffolding for ROI save UI
    pub fn save_current_roi(&mut self) -> Option<u32> {
        self.roi_manager.save_current()
    }

    /// Discard the current ROI without saving.
    #[allow(dead_code)] // Scaffolding for ROI discard UI
    pub fn discard_current_roi(&mut self) {
        self.roi_manager.discard_current();
    }

    /// Clear all vertices from the current ROI.
    #[allow(dead_code)] // Scaffolding for ROI clear UI
    pub fn clear_current_roi(&mut self) {
        self.roi_manager.clear_current();
    }

    /// Delete a saved ROI by ID.
    #[allow(dead_code)] // Scaffolding for ROI delete UI
    pub fn delete_roi(&mut self, id: u32) -> bool {
        self.roi_manager.delete_roi(id)
    }

    /// Rename a saved ROI.
    #[allow(dead_code)] // Scaffolding for ROI rename UI
    pub fn rename_roi(&mut self, id: u32, new_name: String) -> bool {
        self.roi_manager.rename_roi(id, new_name)
    }

    /// Get the ROI manager for read access.
    pub fn roi_manager(&self) -> &RoiManager {
        &self.roi_manager
    }

    /// Get the number of saved ROIs.
    pub fn roi_count(&self) -> usize {
        self.roi_manager.roi_count()
    }

    /// Set the brush radius for ROI painting.
    pub fn set_brush_radius(&mut self, radius: f32) {
        self.brush_radius = radius.clamp(0.25, 3.0);
    }

    /// Get the current brush radius.
    pub fn brush_radius(&self) -> f32 {
        self.brush_radius
    }

    fn reset_paint_queue(&mut self) {
        self.paint_queue.clear();
        self.last_paint_position = None;
    }

    fn enqueue_paint_positions(&mut self, from: MousePosition, to: MousePosition) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dist_sq = dx * dx + dy * dy;
        let step = 6.0f32;

        if dist_sq <= step * step {
            self.paint_queue.push_back(to);
            return;
        }

        let dist = dist_sq.sqrt();
        let steps = (dist / step).ceil() as usize;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.paint_queue.push_back(MousePosition {
                x: from.x + dx * t,
                y: from.y + dy * t,
            });
        }
    }

    fn request_next_paint_pick(&mut self, erase: bool) {
        if self.renderer.has_pending_pick() {
            return;
        }

        if let Some(pos) = self.paint_queue.pop_front() {
            if self.renderer.request_pick(pos.x, pos.y) {
                self.pending_pick_context = Some(PendingPickContext::RoiPaint { erase });
            }
        }
    }

    /// Paint vertices within brush radius of the given vertex.
    fn paint_brush(&mut self, center_vertex: &VertexInfo, erase: bool) {
        let surface_id = center_vertex.surface_id.unwrap_or(SURFACE_ID_LEFT);

        // Get the geometry for this surface
        let geom = if surface_id == SURFACE_ID_LEFT {
            self.geometry_left.as_ref()
        } else {
            self.geometry_right.as_ref()
        };

        let Some(geom) = geom else { return };

        let center_pos = center_vertex.position;

        // Find all vertices within brush radius
        let brush_sq = self.brush_radius * self.brush_radius;
        for (idx, vertex_pos) in geom.vertices.iter().enumerate() {
            let dx = vertex_pos[0] - center_pos[0];
            let dy = vertex_pos[1] - center_pos[1];
            let dz = vertex_pos[2] - center_pos[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq <= brush_sq {
                if erase {
                    self.roi_manager
                        .remove_vertex_from_current(surface_id, idx as u32);
                } else {
                    self.roi_manager
                        .add_vertex_to_current(surface_id, idx as u32);
                }
            }
        }
    }

    /// Compute statistics for the current ROI using loaded overlay data.
    pub fn compute_current_roi_statistics(&self) -> Option<RoiStatistics> {
        let roi = self.roi_manager.current_roi.as_ref()?;
        let stats = self.statistics.as_ref()?;

        // Collect values from all vertices in the ROI
        let mut values = Vec::new();

        for vertex in &roi.vertices {
            if let Some(val) = stats.get(self.active_volume as usize, vertex.vertex_index as usize)
            {
                values.push(val);
            }
        }

        if values.is_empty() {
            return Some(RoiStatistics::default());
        }

        Some(RoiStatistics::from_values(&values))
    }

    /// Compute statistics for a saved ROI by ID.
    pub fn compute_roi_statistics(&self, roi_id: u32) -> Option<RoiStatistics> {
        let roi = self.roi_manager.get_roi(roi_id)?;
        let stats = self.statistics.as_ref()?;

        let mut values = Vec::new();

        for vertex in &roi.vertices {
            if let Some(val) = stats.get(self.active_volume as usize, vertex.vertex_index as usize)
            {
                values.push(val);
            }
        }

        if values.is_empty() {
            return Some(RoiStatistics::default());
        }

        Some(RoiStatistics::from_values(&values))
    }

    /// Check if a vertex is in any ROI (saved or current).
    pub fn is_vertex_in_roi(&self, surface_id: SurfaceId, vertex_index: u32) -> bool {
        self.roi_manager
            .is_vertex_in_any_roi(surface_id, vertex_index)
    }

    /// Get all saved ROIs as (id, name) pairs.
    pub fn list_rois(&self) -> Vec<(u32, String)> {
        self.roi_manager
            .rois
            .iter()
            .map(|r| (r.id, r.name.clone()))
            .collect()
    }

    /// Get the current ROI vertex count.
    pub fn current_roi_vertex_count(&self) -> usize {
        self.roi_manager
            .current_roi
            .as_ref()
            .map(|r| r.vertex_count())
            .unwrap_or(0)
    }

    /// Create a small demo ROI from the first available hemisphere for UI previews.
    pub fn create_sample_roi(&mut self) -> Option<RoiStatistics> {
        // Prefer left geometry, fall back to right or legacy geometry.
        let (surface_id, total) = if let Some(ref left) = self.geometry_left {
            (SURFACE_ID_LEFT, left.vertices.len())
        } else if let Some(ref right) = self.geometry_right {
            (SURFACE_ID_RIGHT, right.vertices.len())
        } else if let Some(ref geom) = self.geometry {
            (SURFACE_ID_LEFT, geom.vertices.len())
        } else {
            return None;
        };

        self.start_new_roi("Sample ROI".to_string());

        // Add a small, regular subsample of vertices to keep it lightweight.
        if total == 0 {
            return None;
        }
        let step = (total / 200).max(1); // target at most ~200 verts
        for idx in (0..total).step_by(step) {
            self.roi_manager
                .add_vertex_to_current(surface_id, idx as u32);
        }

        // Compute stats on the current ROI with active overlay.
        let stats = self.compute_current_roi_statistics()?;
        Some(stats)
    }

    // ==================== Camera Preset Methods ====================

    /// Set the camera to a preset view.
    pub fn set_view_preset(&mut self, preset: BrainViewPreset) {
        // Compute a reasonable target/zoom based on loaded geometry.
        let mut state = self.renderer.camera_state();
        let (theta, phi) = preset.orbit_angles();
        state.azimuth = theta;
        state.elevation = phi;

        if let Some((_center, radius)) = self.scene_bounds() {
            let desired_distance = radius.max(1.0) * 2.0;
            state.distance = state.distance.max(desired_distance);
        }

        log::info!(
            "Applied view preset {:?} (azimuth {:.3}, elevation {:.3}, distance {:.2})",
            preset,
            state.azimuth,
            state.elevation,
            state.distance
        );
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &format!(
                "Applied view preset {:?} (azimuth {:.3}, elevation {:.3}, distance {:.2})",
                preset, state.azimuth, state.elevation, state.distance
            )
            .into(),
        );
        self.renderer.set_camera_state(state);
    }

    /// Compute scene bounding box and radius from loaded geometry.
    fn scene_bounds(&self) -> Option<(glam::Vec3, f32)> {
        let mut mins = glam::Vec3::splat(f32::INFINITY);
        let mut maxs = glam::Vec3::splat(f32::NEG_INFINITY);
        let mut have_any = false;

        let accumulate = |geom: &BrainGeometry,
                          mins: &mut glam::Vec3,
                          maxs: &mut glam::Vec3,
                          have_any: &mut bool| {
            for v in &geom.vertices {
                let p = glam::Vec3::new(v[0], v[1], v[2]);
                *mins = mins.min(p);
                *maxs = maxs.max(p);
            }
            *have_any = true;
        };

        if let Some(ref left) = self.geometry_left {
            accumulate(left, &mut mins, &mut maxs, &mut have_any);
        }
        if let Some(ref right) = self.geometry_right {
            accumulate(right, &mut mins, &mut maxs, &mut have_any);
        }
        // Fallback to legacy single geometry if present.
        if !have_any {
            if let Some(ref geom) = self.geometry {
                accumulate(geom, &mut mins, &mut maxs, &mut have_any);
            }
        }

        if !have_any {
            return None;
        }

        let center = (mins + maxs) * 0.5;
        let extents = maxs - mins;
        let radius = extents.length() * 0.5;
        Some((center, radius))
    }

    // ==================== Marker & Annotation Methods ====================

    /// Selection marker color (gold/yellow).
    const SELECTION_MARKER_COLOR: [f32; 3] = [1.0, 0.8, 0.0];
    /// Annotation marker color (cyan/teal).
    const ANNOTATION_MARKER_COLOR: [f32; 3] = [0.0, 0.8, 0.9];

    /// Update the selection marker to highlight the current selection.
    fn update_selection_marker(&mut self) {
        // Remove old selection marker if present
        if let Some(marker_id) = self.selection_marker_id.take() {
            self.renderer.remove_marker(marker_id);
        }

        // Add new marker if there's a selection
        if let Some(ref info) = self.selected_vertex {
            let position = Vec3::new(info.position[0], info.position[1], info.position[2]);
            let marker_id = self
                .renderer
                .add_marker(position, Self::SELECTION_MARKER_COLOR);
            self.selection_marker_id = Some(marker_id);
        }
    }

    /// Add an annotation at the current selection.
    /// Returns true if annotation was added, false if no selection.
    pub fn add_annotation_at_selection(&mut self) -> bool {
        if let Some(info) = self.selected_vertex.clone() {
            let position = Vec3::new(info.position[0], info.position[1], info.position[2]);
            let marker_id = self
                .renderer
                .add_marker(position, Self::ANNOTATION_MARKER_COLOR);
            self.annotations.push((marker_id, info));
            true
        } else {
            false
        }
    }

    /// Add an annotation at a specific vertex.
    pub fn add_annotation(&mut self, info: VertexInfo) {
        let position = Vec3::new(info.position[0], info.position[1], info.position[2]);
        let marker_id = self
            .renderer
            .add_marker(position, Self::ANNOTATION_MARKER_COLOR);
        self.annotations.push((marker_id, info));
    }

    /// Remove an annotation by index.
    pub fn remove_annotation(&mut self, index: usize) -> Option<VertexInfo> {
        if index < self.annotations.len() {
            let (marker_id, info) = self.annotations.remove(index);
            self.renderer.remove_marker(marker_id);
            Some(info)
        } else {
            None
        }
    }

    /// Clear all annotations.
    pub fn clear_annotations(&mut self) {
        for (marker_id, _) in self.annotations.drain(..) {
            self.renderer.remove_marker(marker_id);
        }
    }

    /// Get the list of annotations.
    pub fn annotations(&self) -> &[(NodeId, VertexInfo)] {
        &self.annotations
    }

    /// Get the number of annotations.
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }

    /// Get the current time in milliseconds.
    fn now_ms() -> f64 {
        // Use js_sys::Date for timing as it doesn't require extra feature flags
        js_sys::Date::now()
    }
}

/// Convert viewer_app ColormapType to core_render ColormapKind.
/// These enums are kept in sync; this is the single place for the mapping.
fn convert_colormap(colormap: ColormapType) -> ColormapKind {
    match colormap {
        ColormapType::RdBu => ColormapKind::RdBu,
        ColormapType::Viridis => ColormapKind::Viridis,
        ColormapType::Hot => ColormapKind::Hot,
        ColormapType::Cividis => ColormapKind::Cividis,
        ColormapType::Plasma => ColormapKind::Plasma,
    }
}

/// Convert viewer_app MouseButton to core_render MouseButton.
fn convert_mouse_button(button: crate::types::MouseButton) -> core_render::traits::MouseButton {
    match button {
        crate::types::MouseButton::Left => core_render::traits::MouseButton::Left,
        crate::types::MouseButton::Middle => core_render::traits::MouseButton::Middle,
        crate::types::MouseButton::Right => core_render::traits::MouseButton::Right,
    }
}

/// Convert viewer_app MousePosition to core_render MousePosition.
fn convert_mouse_position(pos: crate::types::MousePosition) -> core_render::traits::MousePosition {
    core_render::traits::MousePosition { x: pos.x, y: pos.y }
}

/// Convert viewer_app ViewerEvent to core_render ViewerEvent.
fn convert_event(event: &ViewerEvent) -> core_render::traits::ViewerEvent {
    match event {
        ViewerEvent::MouseDown {
            position, button, ..
        } => core_render::traits::ViewerEvent::MouseDown {
            position: convert_mouse_position(*position),
            button: convert_mouse_button(*button),
        },
        ViewerEvent::MouseUp {
            position, button, ..
        } => core_render::traits::ViewerEvent::MouseUp {
            position: convert_mouse_position(*position),
            button: convert_mouse_button(*button),
        },
        ViewerEvent::MouseMove { position, delta } => core_render::traits::ViewerEvent::MouseMove {
            position: convert_mouse_position(*position),
            delta: *delta,
        },
        ViewerEvent::Wheel { delta_y } => {
            core_render::traits::ViewerEvent::Wheel { delta_y: *delta_y }
        }
        ViewerEvent::Click { position, .. } => core_render::traits::ViewerEvent::Click {
            position: convert_mouse_position(*position),
        },
        ViewerEvent::Resize { width, height } => core_render::traits::ViewerEvent::Resize {
            width: *width,
            height: *height,
        },
        ViewerEvent::KeyDown { key, .. } => {
            core_render::traits::ViewerEvent::KeyDown { key: key.clone() }
        }
    }
}

impl BrainRendererBackend for WgpuRendererAdapter {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn new(
        _gl_context: web_sys::WebGl2RenderingContext,
        _geometry: crate::data::BrainGeometry,
        _statistics: crate::data::StatisticData,
    ) -> Result<Self, RendererError>
    where
        Self: Sized,
    {
        // This constructor is not used for wgpu - use WgpuRendererAdapter::new() instead
        Err(RendererError::Other(
            "Use WgpuRendererAdapter::new(canvas) instead".into(),
        ))
    }

    fn set_geometry(&mut self, geometry: BrainGeometry) {
        self.geometry = Some(geometry.clone());
        self.renderer.set_surface(0 as SurfaceId, &geometry);
    }

    fn set_statistics(&mut self, statistics: crate::data::StatisticData) {
        self.statistics = Some(statistics);
        self.update_overlay();
    }

    fn set_volume(&mut self, idx: u32) {
        self.active_volume = idx;
        self.update_overlay();
    }

    fn set_threshold(&mut self, threshold: Option<f32>) {
        self.threshold = threshold;
        self.renderer.set_threshold(threshold);
    }

    fn set_colormap(&mut self, colormap: ColormapType) {
        self.renderer.set_colormap(convert_colormap(colormap));
    }

    fn set_symmetric(&mut self, symmetric: bool) {
        self.symmetric = symmetric;
        // Re-upload overlay with adjusted range
        self.update_overlay();
    }

    fn set_layout(&mut self, layout: LayoutMode) {
        if self.layout_mode == layout {
            return;
        }
        self.layout_mode = layout;
        self.apply_layout_transforms();
    }

    fn set_view_preset(&mut self, preset: BrainViewPreset) {
        // Compute a reasonable target/zoom based on loaded geometry.
        let mut state = self.renderer.camera_state();
        let (theta, phi) = preset.orbit_angles();
        state.azimuth = theta;
        state.elevation = phi;

        if let Some((_center, radius)) = self.scene_bounds() {
            let desired_distance = radius.max(1.0) * 2.0;
            state.distance = state.distance.max(desired_distance);
        }

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &format!(
                "Applied view preset {:?} (azimuth {:.3}, elevation {:.3}, distance {:.2})",
                preset, state.azimuth, state.elevation, state.distance
            )
            .into(),
        );
        log::info!(
            "Applied view preset {:?} (azimuth {:.3}, elevation {:.3}, distance {:.2})",
            preset,
            state.azimuth,
            state.elevation,
            state.distance
        );
        self.renderer.set_camera_state(state);
    }

    fn handle_event(&mut self, event: ViewerEvent) -> EventResult {
        let result = EventResult::default();

        match &event {
            // Handle click events for selection (skip if in ROI drawing mode)
            // Uses async picking to avoid blocking the main thread
            ViewerEvent::Click {
                position,
                modifiers,
            } => {
                if !self.roi_drawing_mode && !self.renderer.has_pending_pick() {
                    if self.renderer.request_pick(position.x, position.y) {
                        self.pending_pick_context = Some(PendingPickContext::Click {
                            modifiers: modifiers.clone(),
                        });
                    }
                }
            }

            // Track drag state and handle ROI painting on mouse down
            ViewerEvent::MouseDown {
                position,
                button,
                modifiers,
            } => {
                if self.roi_drawing_mode {
                    // In ROI mode, left-click paints, right-click or alt+click erases
                    let erase = matches!(button, crate::types::MouseButton::Right) || modifiers.alt;
                    self.is_painting = true;
                    self.is_erasing = erase;

                    self.reset_paint_queue();
                    self.paint_queue.push_back(*position);
                    self.last_paint_position = Some(*position);
                    self.request_next_paint_pick(erase);
                } else {
                    self.is_dragging = true;
                }
            }
            ViewerEvent::MouseUp { .. } => {
                self.is_dragging = false;
                self.is_painting = false;
                self.is_erasing = false;
                self.reset_paint_queue();
            }

            // Handle hover picking with throttling and ROI painting while dragging
            ViewerEvent::MouseMove { position, .. } => {
                if self.is_painting {
                    if let Some(last) = self.last_paint_position {
                        self.enqueue_paint_positions(last, *position);
                    } else {
                        self.paint_queue.push_back(*position);
                    }
                    self.last_paint_position = Some(*position);
                    self.request_next_paint_pick(self.is_erasing);
                } else if !self.is_dragging {
                    // Hover picking with throttling
                    let now = Self::now_ms();
                    if now - self.last_hover_pick_time >= HOVER_THROTTLE_MS {
                        if !self.renderer.has_pending_pick() {
                            if self.renderer.request_pick(position.x, position.y) {
                                self.pending_pick_context = Some(PendingPickContext::Hover);
                                self.last_hover_pick_time = now;
                            }
                        }
                    }
                }
            }

            // Handle keyboard shortcuts
            ViewerEvent::KeyDown { key, modifiers } => {
                match key.as_str() {
                    "Escape" => {
                        // Clear selection
                        self.clear_selection();
                        let mut empty = Selection::new();
                        empty.clear();
                        self.selection_history.apply(empty);
                    }
                    "z" | "Z" if modifiers.ctrl || modifiers.meta => {
                        if modifiers.shift {
                            // Ctrl+Shift+Z = Redo
                            self.redo_selection();
                        } else {
                            // Ctrl+Z = Undo
                            self.undo_selection();
                        }
                    }
                    "y" | "Y" if modifiers.ctrl || modifiers.meta => {
                        // Ctrl+Y = Redo
                        self.redo_selection();
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        // Forward event to renderer for camera control (skip when painting ROI)
        if !self.is_painting {
            let core_event = convert_event(&event);
            self.renderer.handle_event(core_event);
        }

        result
    }

    fn has_pending_pick(&self) -> bool {
        self.renderer.has_pending_pick()
    }

    fn poll_completed_pick(&mut self) -> EventResult {
        self.poll_pending_picks()
    }

    fn render(&mut self) {
        // Note: pick results are now polled externally via poll_completed_pick()
        // to propagate results to the UI layer
        if let Err(e) = self.renderer.render() {
            log::error!("wgpu render error: {}", e);
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    fn camera_state(&self) -> CameraState {
        let state = self.renderer.camera_state();
        // Convert from core_render's orbit-based state to viewer_app's eye/target state
        // This is a simplified conversion
        CameraState {
            eye: [state.distance, 0.0, 0.0], // Placeholder
            target: [0.0, 0.0, 0.0],
            up: [0.0, 0.0, 1.0],
            fov_degrees: 45.0,
        }
    }

    fn set_camera_state(&mut self, state: CameraState) {
        // Convert from eye/target to orbit parameters
        let dx = state.eye[0] - state.target[0];
        let dy = state.eye[1] - state.target[1];
        let dz = state.eye[2] - state.target[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let azimuth = dy.atan2(dx);
        let elevation = (dz / distance).asin();

        self.renderer
            .set_camera_state(core_render::traits::CameraState {
                distance,
                azimuth,
                elevation,
            });
    }

    fn is_running(&self) -> bool {
        self.running.get()
    }

    fn stop(&mut self) {
        self.running.set(false);
    }
}

// Debug view API (separate impl block for clarity)
impl WgpuRendererAdapter {
    /// Set the debug view mode.
    ///
    /// Debug modes allow visualizing different aspects of the rendering:
    /// - `None`: Normal rendering with overlay colormapping
    /// - `Normals`: Display surface normals as RGB colors
    /// - `RawOverlay`: Display raw overlay values as grayscale
    /// - `VertexId`: Display vertex IDs as pseudo-random colors
    #[allow(dead_code)] // Scaffolding for debug view UI
    pub fn set_debug_view(&mut self, debug_view: core_render::DebugView) {
        self.renderer.set_debug_view(debug_view);
    }

    /// Get the current debug view mode.
    #[allow(dead_code)] // Scaffolding for debug view UI
    pub fn debug_view(&self) -> core_render::DebugView {
        self.renderer.debug_view()
    }
}

// Color source and parcellation rendering API
impl WgpuRendererAdapter {
    /// Set the color source mode (overlay vs parcellation).
    pub fn set_color_source(&mut self, source: core_render::ColorSource) {
        self.renderer.set_color_source(source);
    }

    /// Get the current color source mode.
    #[allow(dead_code)] // Scaffolding for color source UI query
    pub fn color_source(&self) -> core_render::ColorSource {
        self.renderer.color_source()
    }

    /// Set the parcellation display mode (fill, edges, fill_and_edges).
    pub fn set_parcellation_display(&mut self, display: core_render::ParcellationDisplay) {
        self.renderer.set_parcellation_display(display);
    }

    /// Get the current parcellation display mode.
    #[allow(dead_code)] // Scaffolding for parcellation display UI query
    pub fn parcellation_display(&self) -> core_render::ParcellationDisplay {
        self.renderer.parcellation_display()
    }

    /// Upload parcellation colors to the GPU for a surface.
    ///
    /// This extracts labels and colors from the Parcellation and uploads them
    /// for GPU-based parcellation rendering.
    pub fn set_parcellation_colors(&mut self, surface_id: SurfaceId, parcellation: &Parcellation) {
        // Extract per-vertex labels
        let labels: Vec<u32> = parcellation
            .labels_per_vertex
            .iter()
            .map(|&l| l as u32)
            .collect();

        // Build region color lookup (indexed by parcel ID)
        let max_id = parcellation.parcels.keys().copied().max().unwrap_or(0) as usize;
        let mut region_colors = vec![[0.5f32, 0.5, 0.5, 1.0]; max_id + 1];

        for (&id, info) in &parcellation.parcels {
            let idx = id as usize;
            if idx < region_colors.len() {
                region_colors[idx] = [
                    info.color[0] as f32 / 255.0,
                    info.color[1] as f32 / 255.0,
                    info.color[2] as f32 / 255.0,
                    info.color[3] as f32 / 255.0,
                ];
            }
        }

        self.renderer
            .set_parcellation(surface_id, &labels, &region_colors);
    }

    /// Upload parcellation colors for both hemispheres if available.
    pub fn set_parcellation_colors_from_stored(&mut self) {
        if let Some(ref parc) = self.parcellation_left.clone() {
            self.set_parcellation_colors(SURFACE_ID_LEFT, parc);
        }
        if let Some(ref parc) = self.parcellation_right.clone() {
            self.set_parcellation_colors(SURFACE_ID_RIGHT, parc);
        }
    }

    /// Check if a surface has parcellation data uploaded to GPU.
    #[allow(dead_code)] // Scaffolding for parcellation state queries
    pub fn has_parcellation_colors(&self, surface_id: SurfaceId) -> bool {
        self.renderer.has_parcellation(surface_id)
    }
}

// ROI mask rendering API
impl WgpuRendererAdapter {
    /// Push the current ROI to the GPU as a per-vertex mask.
    ///
    /// This creates a per-vertex float mask (0.0 = not in ROI, 1.0 = in ROI)
    /// and uploads it for GPU-based ROI visualization.
    pub fn push_roi_mask_to_gpu(&mut self, surface_id: SurfaceId) {
        let geom = if surface_id == SURFACE_ID_LEFT {
            self.geometry_left.as_ref()
        } else {
            self.geometry_right.as_ref()
        };

        let Some(geom) = geom else {
            return;
        };

        let vertex_count = geom.vertices.len();
        if vertex_count == 0 {
            return;
        }

        // Build mask from current ROI
        let mut mask_bool = vec![false; vertex_count];

        if let Some(ref roi) = self.roi_manager.current_roi {
            for vertex in &roi.vertices {
                if vertex.surface_id == surface_id && (vertex.vertex_index as usize) < vertex_count
                {
                    mask_bool[vertex.vertex_index as usize] = true;
                }
            }
        }

        for _ in 0..ROI_MASK_DILATION_STEPS {
            let mut expanded = mask_bool.clone();
            for tri in &geom.indices {
                let a = tri[0] as usize;
                let b = tri[1] as usize;
                let c = tri[2] as usize;
                if a < vertex_count && b < vertex_count && c < vertex_count {
                    if mask_bool[a] || mask_bool[b] || mask_bool[c] {
                        expanded[a] = true;
                        expanded[b] = true;
                        expanded[c] = true;
                    }
                }
            }
            mask_bool = expanded;
        }

        let mut mask = vec![0.0f32; vertex_count];
        for (idx, value) in mask_bool.iter().enumerate() {
            if *value {
                mask[idx] = 1.0;
            }
        }

        self.renderer.set_roi_mask(surface_id, &mask);
    }

    /// Push all ROI masks to the GPU for both hemispheres.
    pub fn push_all_roi_masks_to_gpu(&mut self) {
        self.push_roi_mask_to_gpu(SURFACE_ID_LEFT);
        self.push_roi_mask_to_gpu(SURFACE_ID_RIGHT);
    }

    /// Clear ROI mask for a surface.
    #[allow(dead_code)] // Scaffolding for ROI clear/reset
    pub fn clear_roi_mask_on_gpu(&mut self, surface_id: SurfaceId) {
        self.renderer.clear_roi_mask(surface_id);
    }

    /// Enable or disable ROI visualization on GPU.
    pub fn set_roi_visualization_enabled(&mut self, enabled: bool) {
        self.renderer.set_roi_enabled(enabled);
    }

    /// Check if ROI visualization is enabled.
    #[allow(dead_code)] // Scaffolding for ROI state queries
    pub fn is_roi_visualization_enabled(&self) -> bool {
        self.renderer.roi_enabled()
    }
}

// Marker styling API - scaffolding for annotation/marker UI features
#[allow(dead_code)]
impl WgpuRendererAdapter {
    /// Set a marker as selected (highlighted with glow effect).
    pub fn set_marker_selected(&mut self, node_id: NodeId, selected: bool) {
        self.renderer.set_marker_selected(node_id, selected);
    }

    /// Set a marker's size multiplier.
    pub fn set_marker_size(&mut self, node_id: NodeId, size: f32) {
        self.renderer.set_marker_size(node_id, size);
    }

    /// Update a marker's full style (size and selected state).
    pub fn update_marker_style(&mut self, node_id: NodeId, style: core_render::scene::MarkerStyle) {
        self.renderer.update_marker_style(node_id, style);
    }

    /// Get a marker's current style.
    pub fn get_marker_style(&self, node_id: NodeId) -> Option<core_render::scene::MarkerStyle> {
        self.renderer.get_marker_style(node_id)
    }

    /// Select an annotation by index (highlights with glow).
    pub fn select_annotation(&mut self, index: usize) {
        // Deselect all first
        for (marker_id, _) in &self.annotations {
            self.renderer.set_marker_selected(*marker_id, false);
        }

        // Select the specified one
        if let Some((marker_id, _)) = self.annotations.get(index) {
            self.renderer.set_marker_selected(*marker_id, true);
        }
    }

    /// Deselect all annotations.
    pub fn deselect_all_annotations(&mut self) {
        for (marker_id, _) in &self.annotations {
            self.renderer.set_marker_selected(*marker_id, false);
        }
    }
}
