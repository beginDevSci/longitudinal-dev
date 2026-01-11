use glam::{Mat4, Vec3};
use io_formats::geometry::BrainGeometry;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

use crate::device::DeviceContext;
use crate::orbit::OrbitController;
use crate::picking::PickingSystem;
use crate::pipelines::Pipelines;
use crate::resources::{ColormapKind, OverlayId, ResourceManager, SurfaceId};
use crate::scene::{NodeContent, NodeId, Scene};
use crate::traits::{BrainRendererBackend, CameraState, PickResult, RenderError, ViewerEvent};

// Re-export types for external use
pub use crate::resources::ColormapKind as ColormapKindType;
pub use crate::resources::OverlayId as OverlayIdType;
pub use crate::resources::SurfaceId as SurfaceIdType;

/// Selection state for vertex highlighting.
#[derive(Clone, Copy, Default)]
pub struct SelectionState {
    /// The selected vertex ID (if any).
    pub vertex_id: Option<u32>,
    /// The surface ID containing the selected vertex.
    pub surface_id: Option<SurfaceId>,
}

/// wgpu-based renderer with orbit camera and surface rendering.
///
/// ## Bind Group Layout (4 groups for WebGPU compatibility)
///
/// - Group 0: Camera + Selection uniforms
/// - Group 1: Overlay data + params
/// - Group 2: Colormap + Parcellation (texture, sampler, labels, region colors)
/// - Group 3: ROI mask
pub struct WgpuRenderer {
    ctx: DeviceContext,
    resources: ResourceManager,
    pipelines: Pipelines,
    camera_buffer: wgpu::Buffer,
    selection_uniform_buffer: wgpu::Buffer,
    /// Group 0: Camera + Selection bind group
    camera_selection_bind_group: wgpu::BindGroup,
    orbit: OrbitController,
    surface_id: Option<SurfaceId>,
    // Overlay state
    /// Group 1: Overlay bind group
    overlay_bind_group: Option<wgpu::BindGroup>,
    overlay_id: Option<OverlayId>,
    colormap_kind: ColormapKind,
    overlay_params_buffer: wgpu::Buffer,
    // Picking
    picking: PickingSystem,
    picking_uniform_buffer: wgpu::Buffer,
    /// Separate camera bind group for picking pipeline (camera only, no selection)
    picking_camera_bind_group: wgpu::BindGroup,
    // Selection state
    selection: SelectionState,
    // Scene state
    scene: Scene,
    // Marker rendering
    marker_instance_buffer: Option<wgpu::Buffer>,
    marker_corner_buffer: wgpu::Buffer,
    marker_count: u32,
    // Debug mode
    debug_view: crate::debug::DebugView,
    // Color source mode
    color_source: crate::color_source::ColorSource,
    // Parcellation display mode
    parcellation_display: crate::color_source::ParcellationDisplay,
    /// Group 2: Colormap + Parcellation bind groups per surface
    colormap_parcellation_bind_groups: std::collections::HashMap<SurfaceId, wgpu::BindGroup>,
    /// Group 3: ROI bind groups per surface
    roi_bind_groups: std::collections::HashMap<SurfaceId, wgpu::BindGroup>,
    /// Default Group 2 bind group (empty parcellation + default colormap)
    default_colormap_parcellation_bind_group: wgpu::BindGroup,
    /// Default Group 3: ROI bind group (empty/zeros)
    default_roi_bind_group: wgpu::BindGroup,
    /// Default label buffer for surfaces without parcellation
    default_label_buffer: wgpu::Buffer,
    /// Default region color texture view
    default_region_view: wgpu::TextureView,
    // ROI enabled flag
    roi_enabled: bool,
}

impl WgpuRenderer {
    /// Create a new renderer from a canvas element.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            web_sys::console::log_1(&JsValue::from_str("[WgpuRenderer] Starting initialization..."));
        }

        let ctx = DeviceContext::new(canvas)
            .await
            .map_err(|e| RenderError::Message(format!("wgpu init failed: {e:?}")))?;

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            web_sys::console::log_1(&JsValue::from_str("[WgpuRenderer] DeviceContext ready, creating pipelines..."));
        }

        let surface_format = ctx.config.format;
        let pipelines = Pipelines::new(&ctx.device, surface_format);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            web_sys::console::log_1(&JsValue::from_str("[WgpuRenderer] Pipelines created, creating buffers..."));
        }

        // Create camera buffer (mat4x4 + vec4 model_offset = 80 bytes)
        let camera_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_uniform"),
            size: 80, // mat4x4<f32> (64) + vec4<f32> model_offset (16)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create selection uniform buffer (vec4<u32>: vertex_id, surface_id, current_surface_id, has_selection)
        let selection_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selection_uniform"),
            size: 16, // 4 * u32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create Group 0: Camera + Selection bind group
        let camera_selection_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_selection_bg"),
            layout: &pipelines.camera_selection_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: selection_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Create separate camera bind group for picking (camera only, no selection)
        // Picking pipeline uses a simpler layout
        let picking_camera_bgl = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("picking_camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let picking_camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("picking_camera_bg"),
            layout: &picking_camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create overlay params buffer (4 floats + 4 u32s = 32 bytes)
        // Layout: data_min, data_max, threshold, use_threshold, debug_mode, color_source, parcellation_display, roi_enabled
        let overlay_params_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay_uniform"),
            size: 32, // 4 * f32 + 4 * u32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create default parcellation label buffer (single zero)
        let default_label_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_parcellation_labels"),
            contents: bytemuck::cast_slice(&[0u32; 1]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create default region color texture (single texel)
        let default_region_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("default_region_colors"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &default_region_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128u8, 128, 128, 255], // Gray color
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let default_region_view = default_region_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create default colormap texture
        let mut resources = ResourceManager::new();
        let default_colormap = resources.get_or_create_colormap(&ctx.device, &ctx.queue, ColormapKind::RdBu);

        // Create default Group 2: Colormap + Parcellation bind group
        let default_colormap_parcellation_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("default_colormap_parcellation_bg"),
            layout: &pipelines.colormap_parcellation_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_colormap.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_colormap.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: default_label_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&default_region_view),
                },
            ],
        });

        // Create default ROI mask buffer (single zero)
        let default_roi_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_roi_mask"),
            contents: bytemuck::cast_slice(&[0.0f32; 1]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create default Group 3: ROI bind group
        let default_roi_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("default_roi_bg"),
            layout: &pipelines.roi_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: default_roi_buffer.as_entire_binding(),
            }],
        });

        let mut orbit = OrbitController::default();
        orbit.target = Vec3::ZERO;

        // Create picking system with initial canvas size
        let picking = PickingSystem::new(&ctx.device, ctx.config.width, ctx.config.height);

        // Create picking uniform buffer (1 u32 for surface_id, padded to 4 bytes)
        let picking_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("picking_uniform"),
            size: 4, // 1 * u32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Initialize scene state
        let scene = Scene::new();

        // Create marker corner buffer (static quad vertices)
        // Two triangles forming a quad: corners at (-1,-1), (1,-1), (1,1), (-1,1)
        let marker_corners: [[f32; 2]; 6] = [
            [-1.0, -1.0], [1.0, -1.0], [1.0, 1.0],  // First triangle
            [-1.0, -1.0], [1.0, 1.0], [-1.0, 1.0],  // Second triangle
        ];
        let marker_corner_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("marker_corner_buffer"),
            contents: bytemuck::cast_slice(&marker_corners),
            usage: wgpu::BufferUsages::VERTEX,
        });

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            web_sys::console::log_1(&JsValue::from_str("[WgpuRenderer] Initialization complete!"));
        }

        Ok(Self {
            ctx,
            resources,
            pipelines,
            camera_buffer,
            selection_uniform_buffer,
            camera_selection_bind_group,
            picking_camera_bind_group,
            orbit,
            surface_id: None,
            overlay_bind_group: None,
            overlay_id: None,
            colormap_kind: ColormapKind::RdBu,
            overlay_params_buffer,
            picking,
            picking_uniform_buffer,
            selection: SelectionState::default(),
            scene,
            marker_instance_buffer: None,
            marker_corner_buffer,
            marker_count: 0,
            debug_view: crate::debug::DebugView::None,
            color_source: crate::color_source::ColorSource::default(),
            parcellation_display: crate::color_source::ParcellationDisplay::default(),
            colormap_parcellation_bind_groups: std::collections::HashMap::new(),
            roi_bind_groups: std::collections::HashMap::new(),
            default_colormap_parcellation_bind_group,
            default_roi_bind_group,
            default_label_buffer,
            default_region_view,
            roi_enabled: false,
        })
    }

    /// Upload a surface mesh to the GPU.
    pub fn set_surface(&mut self, id: SurfaceId, geom: &BrainGeometry) {
        self.resources.upload_surface(&self.ctx.device, id, geom);
        self.surface_id = Some(id);
        self.scene.add_surface(id);
    }

    /// Upload multiple surfaces to the GPU, clearing any existing surfaces.
    ///
    /// Takes a slice of (SurfaceId, geometry) pairs and uploads each to the GPU.
    /// The scene is cleared first, then all surfaces are added.
    pub fn set_surfaces(&mut self, surfaces: &[(SurfaceId, &BrainGeometry)]) {
        // Clear existing scene
        self.scene.clear();
        self.surface_id = None;

        for (id, geom) in surfaces {
            self.resources.upload_surface(&self.ctx.device, *id, geom);
            self.scene.add_surface(*id);

            // Keep track of first surface for backwards compatibility
            if self.surface_id.is_none() {
                self.surface_id = Some(*id);
            }
        }
    }

    /// Set the transform for a specific surface by ID.
    pub fn set_surface_transform(&mut self, surface_id: SurfaceId, translation: Vec3) {
        use crate::scene::Transform3D;
        self.scene.set_surface_transform(surface_id, Transform3D::from_translation(translation));
    }

    // ==================== Marker Methods ====================

    /// Add a marker at the given position with the specified color.
    /// Returns the node ID for later reference.
    pub fn add_marker(&mut self, position: Vec3, color: [f32; 3]) -> NodeId {
        let node_id = self.scene.add_marker(position, color);
        self.update_marker_buffer();
        node_id
    }

    /// Update an existing marker's position and color.
    pub fn update_marker(&mut self, node_id: NodeId, position: Vec3, color: [f32; 3]) {
        self.scene.update_marker(node_id, position, color);
        self.update_marker_buffer();
    }

    /// Remove a marker by its node ID.
    pub fn remove_marker(&mut self, node_id: NodeId) {
        self.scene.remove_marker(node_id);
        self.update_marker_buffer();
    }

    /// Clear all markers from the scene.
    pub fn clear_markers(&mut self) {
        self.scene.clear_markers();
        self.update_marker_buffer();
    }

    /// Get the number of markers in the scene.
    pub fn marker_count(&self) -> usize {
        self.scene.marker_count()
    }

    /// Update a marker's style (size and selected state).
    pub fn update_marker_style(&mut self, node_id: NodeId, style: crate::scene::MarkerStyle) {
        self.scene.update_marker_style(node_id, style);
        self.update_marker_buffer();
    }

    /// Get the style of a marker.
    pub fn get_marker_style(&self, node_id: NodeId) -> Option<crate::scene::MarkerStyle> {
        self.scene.get_marker_style(node_id)
    }

    /// Set a marker as selected (highlighted).
    pub fn set_marker_selected(&mut self, node_id: NodeId, selected: bool) {
        if let Some(mut style) = self.scene.get_marker_style(node_id) {
            style.selected = selected;
            self.scene.update_marker_style(node_id, style);
            self.update_marker_buffer();
        }
    }

    /// Set a marker's size.
    pub fn set_marker_size(&mut self, node_id: NodeId, size: f32) {
        if let Some(mut style) = self.scene.get_marker_style(node_id) {
            style.size = size;
            self.scene.update_marker_style(node_id, style);
            self.update_marker_buffer();
        }
    }

    /// Update the marker instance buffer with current marker data.
    fn update_marker_buffer(&mut self) {
        // Collect marker data from scene
        // Format: [pos.x, pos.y, pos.z, color.r, color.g, color.b, size, selected]
        let markers: Vec<[f32; 8]> = self.scene.iter_markers().filter_map(|node| {
            match node.content {
                NodeContent::Marker { position, color, style } => {
                    Some([
                        position.x, position.y, position.z,
                        color[0], color[1], color[2],
                        style.size,
                        if style.selected { 1.0 } else { 0.0 },
                    ])
                }
                _ => None,
            }
        }).collect();

        self.marker_count = markers.len() as u32;

        if markers.is_empty() {
            self.marker_instance_buffer = None;
            return;
        }

        // Create/update instance buffer
        self.marker_instance_buffer = Some(self.ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("marker_instance_buffer"),
                contents: bytemuck::cast_slice(&markers),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
    }

    // ==================== Camera Methods ====================

    /// Set the camera to a preset view.
    pub fn set_view_preset(&mut self, preset: neuro_surface::BrainViewPreset) {
        self.orbit.set_preset(preset);
    }

    /// Set overlay data with range and optional threshold.
    pub fn set_overlay(
        &mut self,
        overlay_id: OverlayId,
        values: &[f32],
        range: (f32, f32),
        threshold: Option<f32>,
    ) {
        // Upload overlay data
        self.resources
            .upload_overlay(&self.ctx.device, overlay_id, values, range);
        self.overlay_id = Some(overlay_id);

        let overlay = self.resources.get_overlay(overlay_id).unwrap();

        // Create Group 1: Overlay bind group
        let overlay_bg = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay_bg"),
            layout: &self.pipelines.overlay_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: overlay.data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.overlay_params_buffer.as_entire_binding(),
                },
            ],
        });
        self.overlay_bind_group = Some(overlay_bg);

        // Create/update Group 2 bind groups for all surfaces (colormap + parcellation)
        self.update_colormap_parcellation_bind_groups();

        // Update overlay params uniform
        let (data_min, data_max) = range;
        let threshold_val = threshold.unwrap_or(0.0);
        let use_threshold: u32 = if threshold.is_some() { 1 } else { 0 };

        self.write_overlay_params(data_min, data_max, threshold_val, use_threshold);
    }

    /// Update Group 2 bind groups for all surfaces with current colormap.
    fn update_colormap_parcellation_bind_groups(&mut self) {
        // First, ensure colormap exists (this may create it)
        let _ = self.resources.get_or_create_colormap(
            &self.ctx.device,
            &self.ctx.queue,
            self.colormap_kind,
        );

        // Now collect surface data without holding mutable borrow
        let surface_ids: Vec<SurfaceId> = self.scene.iter_surfaces().collect();

        // Collect what we need for each surface
        struct SurfaceBindGroupData {
            surface_id: SurfaceId,
            has_labels: bool,
            has_region_colors: bool,
        }

        let surface_data: Vec<SurfaceBindGroupData> = surface_ids
            .iter()
            .map(|&sid| {
                let has_labels = self.resources.get_surface(sid)
                    .map(|s| s.label_buffer.is_some())
                    .unwrap_or(false);
                let has_region_colors = self.resources.get_region_colors(sid).is_some();
                SurfaceBindGroupData {
                    surface_id: sid,
                    has_labels,
                    has_region_colors,
                }
            })
            .collect();

        // Now get references for bind group creation
        // (We need to re-borrow since data collection above released the borrow)
        let cmap = self.resources.get_colormap(self.colormap_kind).unwrap();
        let cmap_view = &cmap.view;
        let cmap_sampler = &cmap.sampler;

        for data in surface_data {
            let (label_buffer, region_view): (&wgpu::Buffer, &wgpu::TextureView) =
                if data.has_labels && data.has_region_colors {
                    let surface = self.resources.get_surface(data.surface_id).unwrap();
                    let label_buf = surface.label_buffer.as_ref().unwrap();
                    let region_colors = self.resources.get_region_colors(data.surface_id).unwrap();
                    (label_buf, &region_colors.view)
                } else {
                    (&self.default_label_buffer, &self.default_region_view)
                };

            let bind_group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("colormap_parcellation_bg"),
                layout: &self.pipelines.colormap_parcellation_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(cmap_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(cmap_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: label_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(region_view),
                    },
                ],
            });
            self.colormap_parcellation_bind_groups.insert(data.surface_id, bind_group);
        }

        // Also update the default bind group
        self.default_colormap_parcellation_bind_group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("default_colormap_parcellation_bg"),
            layout: &self.pipelines.colormap_parcellation_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(cmap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(cmap_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.default_label_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.default_region_view),
                },
            ],
        });
    }

    /// Internal helper to write overlay params uniform buffer.
    fn write_overlay_params(&self, data_min: f32, data_max: f32, threshold: f32, use_threshold: u32) {
        // Pack: 4 floats (min, max, threshold, use_threshold) + 4 u32s
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct OverlayParams {
            data_min: f32,
            data_max: f32,
            threshold: f32,
            use_threshold: f32, // Cast to f32 for alignment
            debug_mode: u32,
            color_source: u32,
            parcellation_display: u32,
            roi_enabled: u32,
        }

        let params = OverlayParams {
            data_min,
            data_max,
            threshold,
            use_threshold: use_threshold as f32,
            debug_mode: self.debug_view.as_u32(),
            color_source: self.color_source.as_u32(),
            parcellation_display: self.parcellation_display.as_u32(),
            roi_enabled: if self.roi_enabled { 1 } else { 0 },
        };

        let bytes = bytemuck::bytes_of(&params);
        self.ctx
            .queue
            .write_buffer(&self.overlay_params_buffer, 0, bytes);
    }

    /// Change the colormap (requires re-creating colormap bind group).
    pub fn set_colormap(&mut self, kind: ColormapKind) {
        self.colormap_kind = kind;

        // If we have an overlay, recreate the colormap + parcellation bind groups
        if self.overlay_id.is_some() {
            self.update_colormap_parcellation_bind_groups();
        }
    }

    /// Update just the threshold without re-uploading overlay data.
    pub fn set_threshold(&mut self, threshold: Option<f32>) {
        if let Some(overlay_id) = self.overlay_id {
            if let Some(overlay) = self.resources.get_overlay(overlay_id) {
                let (data_min, data_max) = overlay.range;
                let threshold_val = threshold.unwrap_or(0.0);
                let use_threshold: u32 = if threshold.is_some() { 1 } else { 0 };

                self.write_overlay_params(data_min, data_max, threshold_val, use_threshold);
            }
        }
    }

    /// Set the debug view mode.
    ///
    /// Debug modes allow visualizing different aspects of the rendering:
    /// - `None`: Normal rendering with overlay colormapping
    /// - `Normals`: Display surface normals as RGB colors
    /// - `RawOverlay`: Display raw overlay values as grayscale
    /// - `VertexId`: Display vertex IDs as pseudo-random colors
    pub fn set_debug_view(&mut self, debug_view: crate::debug::DebugView) {
        self.debug_view = debug_view;

        // Update the overlay params to include the new debug mode
        if let Some(overlay_id) = self.overlay_id {
            if let Some(overlay) = self.resources.get_overlay(overlay_id) {
                let (data_min, data_max) = overlay.range;
                // Keep current threshold settings (we'd need to store them to avoid this issue)
                self.write_overlay_params(data_min, data_max, 0.0, 0);
            }
        }
    }

    /// Get the current debug view mode.
    pub fn debug_view(&self) -> crate::debug::DebugView {
        self.debug_view
    }

    // ==================== Color Source Methods ====================

    /// Set the color source mode (overlay vs parcellation).
    pub fn set_color_source(&mut self, source: crate::color_source::ColorSource) {
        self.color_source = source;
        self.refresh_overlay_params();
    }

    /// Get the current color source mode.
    pub fn color_source(&self) -> crate::color_source::ColorSource {
        self.color_source
    }

    /// Set the parcellation display mode (fill, edges, fill_and_edges).
    pub fn set_parcellation_display(&mut self, display: crate::color_source::ParcellationDisplay) {
        self.parcellation_display = display;
        self.refresh_overlay_params();
    }

    /// Get the current parcellation display mode.
    pub fn parcellation_display(&self) -> crate::color_source::ParcellationDisplay {
        self.parcellation_display
    }

    // ==================== Parcellation Methods ====================

    /// Upload parcellation data for a surface.
    ///
    /// This sets up the per-vertex labels and region color lookup for parcellation rendering.
    ///
    /// # Arguments
    /// * `surface_id` - The surface to apply parcellation to
    /// * `labels` - Per-vertex region IDs (must match surface vertex count)
    /// * `region_colors` - RGBA colors indexed by region ID
    pub fn set_parcellation(
        &mut self,
        surface_id: SurfaceId,
        labels: &[u32],
        region_colors: &[[f32; 4]],
    ) {
        // Upload labels to resource manager
        self.resources.upload_parcellation_labels(&self.ctx.device, surface_id, labels);

        // Upload region colors
        self.resources.upload_region_colors(
            &self.ctx.device,
            &self.ctx.queue,
            surface_id,
            region_colors,
        );

        // Rebuild the colormap + parcellation bind groups
        self.update_colormap_parcellation_bind_groups();
    }

    /// Check if a surface has parcellation data.
    pub fn has_parcellation(&self, surface_id: SurfaceId) -> bool {
        if let Some(surface) = self.resources.get_surface(surface_id) {
            surface.label_buffer.is_some()
        } else {
            false
        }
    }

    /// Clear parcellation data for a surface.
    pub fn clear_parcellation(&mut self, surface_id: SurfaceId) {
        // Note: We can't easily remove the label buffer from resources,
        // but we can rebuild the bind groups without it
        self.colormap_parcellation_bind_groups.remove(&surface_id);
    }

    // ==================== ROI Methods ====================

    /// Set ROI mask for a surface.
    ///
    /// The mask is a per-vertex float where 1.0 means the vertex is in the ROI.
    /// Vertices in the ROI will be tinted with the ROI color.
    pub fn set_roi_mask(&mut self, surface_id: SurfaceId, mask: &[f32]) {
        self.resources.upload_roi_mask(&self.ctx.device, surface_id, mask);

        // Create bind group
        if let Some(roi_mask) = self.resources.get_roi_mask(surface_id) {
            let bind_group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("roi_bg"),
                layout: &self.pipelines.roi_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: roi_mask.mask_buffer.as_entire_binding(),
                }],
            });
            self.roi_bind_groups.insert(surface_id, bind_group);
        }

        self.roi_enabled = true;
        self.refresh_overlay_params();
    }

    /// Clear ROI mask for a surface.
    pub fn clear_roi_mask(&mut self, surface_id: SurfaceId) {
        self.resources.clear_roi_mask(surface_id);
        self.roi_bind_groups.remove(&surface_id);

        // Disable ROI if no surfaces have masks
        if self.roi_bind_groups.is_empty() {
            self.roi_enabled = false;
            self.refresh_overlay_params();
        }
    }

    /// Enable or disable ROI visualization.
    pub fn set_roi_enabled(&mut self, enabled: bool) {
        self.roi_enabled = enabled;
        self.refresh_overlay_params();
    }

    /// Check if ROI visualization is enabled.
    pub fn roi_enabled(&self) -> bool {
        self.roi_enabled
    }

    /// Helper to refresh overlay params without changing threshold.
    fn refresh_overlay_params(&self) {
        if let Some(overlay_id) = self.overlay_id {
            if let Some(overlay) = self.resources.get_overlay(overlay_id) {
                let (data_min, data_max) = overlay.range;
                self.write_overlay_params(data_min, data_max, 0.0, 0);
            }
        } else {
            // Write default params even without overlay
            self.write_overlay_params(0.0, 1.0, 0.0, 0);
        }
    }

    /// Set the selected vertex for visual highlighting.
    ///
    /// Pass `None` for both parameters to clear the selection.
    pub fn set_selected_vertex(&mut self, vertex_id: Option<u32>, surface_id: Option<SurfaceId>) {
        self.selection = SelectionState { vertex_id, surface_id };
    }

    /// Clear the current vertex selection.
    pub fn clear_selection(&mut self) {
        self.selection = SelectionState::default();
    }

    /// Update the selection uniform buffer for the current surface being rendered.
    fn update_selection_uniform(&mut self, current_surface_id: SurfaceId) {
        let (selected_vertex_id, selected_surface_id, has_selection) = match self.selection.vertex_id {
            Some(vid) => (vid, self.selection.surface_id.unwrap_or(0), 1u32),
            None => (0u32, 0u32, 0u32),
        };

        let packed: [u32; 4] = [
            selected_vertex_id,
            selected_surface_id,
            current_surface_id,
            has_selection,
        ];
        let bytes = bytemuck::bytes_of(&packed);
        self.ctx
            .queue
            .write_buffer(&self.selection_uniform_buffer, 0, bytes);
    }

    fn update_camera_with_offset(&mut self, model_offset: Vec3) {
        let view = self.orbit.view_matrix();
        let aspect = self.ctx.config.width as f32 / self.ctx.config.height.max(1) as f32;
        let proj = Mat4::perspective_rh(45f32.to_radians(), aspect.max(0.1), 1.0, 1000.0);
        let view_proj = proj * view;

        // Pack view_proj matrix and model_offset into a single buffer
        let m = view_proj.to_cols_array_2d();
        let offset = [model_offset.x, model_offset.y, model_offset.z, 0.0f32];

        // Write view_proj (64 bytes) then model_offset (16 bytes)
        let matrix_bytes = bytemuck::bytes_of(&m);
        let offset_bytes = bytemuck::bytes_of(&offset);

        self.ctx.queue.write_buffer(&self.camera_buffer, 0, matrix_bytes);
        self.ctx.queue.write_buffer(&self.camera_buffer, 64, offset_bytes);
    }

    fn render_internal(&mut self) -> Result<(), RenderError> {
        // Early return if no surface has been loaded
        if !self.scene.has_surface() {
            return Ok(());
        }

        let output = self
            .ctx
            .surface
            .get_current_texture()
            .map_err(|e| RenderError::Message(format!("get_current_texture failed: {e:?}")))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Collect surfaces with transforms (need to collect to avoid borrow conflict)
        let surfaces: Vec<_> = self.scene.iter_surfaces_with_transforms().collect();

        // Only draw if we have overlay bind group
        if self.overlay_bind_group.is_none() {
            output.present();
            return Ok(());
        }

        let mut encoder =
            self.ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("main_encoder"),
                });

        // Draw each surface with its transform
        for (i, (surf_id, transform)) in surfaces.iter().enumerate() {
            // Update camera uniform with this surface's transform
            self.update_camera_with_offset(transform.translation);

            // Update selection uniform with current surface ID
            self.update_selection_uniform(*surf_id);

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear only on first surface
                        load: if i == 0 {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.15,
                                a: 1.0,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipelines.surface);

            // Group 0: Camera + Selection
            pass.set_bind_group(0, &self.camera_selection_bind_group, &[]);

            // Group 1: Overlay
            pass.set_bind_group(1, self.overlay_bind_group.as_ref().unwrap(), &[]);

            // Group 2: Colormap + Parcellation
            let colormap_parcellation_bg = self.colormap_parcellation_bind_groups
                .get(surf_id)
                .unwrap_or(&self.default_colormap_parcellation_bind_group);
            pass.set_bind_group(2, colormap_parcellation_bg, &[]);

            // Group 3: ROI
            let roi_bg = self.roi_bind_groups
                .get(surf_id)
                .unwrap_or(&self.default_roi_bind_group);
            pass.set_bind_group(3, roi_bg, &[]);

            if let Some(bufs) = self.resources.get_surface(*surf_id) {
                pass.set_vertex_buffer(0, bufs.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, bufs.normal_buffer.slice(..));
                pass.set_vertex_buffer(2, bufs.vertex_id_buffer.slice(..));
                pass.set_index_buffer(bufs.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..bufs.index_count, 0, 0..1);
            }
        }

        // Render markers (if any)
        if self.marker_count > 0 && self.marker_instance_buffer.is_some() {
            // Update camera with no offset for markers (they use world-space positions)
            self.update_camera_with_offset(Vec3::ZERO);

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("marker_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,  // Don't clear, draw on top
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipelines.marker);
            pass.set_bind_group(0, &self.picking_camera_bind_group, &[]); // Use camera-only bind group
            // Safe to unwrap since we checked is_some() above
            pass.set_vertex_buffer(0, self.marker_instance_buffer.as_ref().unwrap().slice(..));
            pass.set_vertex_buffer(1, self.marker_corner_buffer.slice(..));
            // 6 vertices per quad (2 triangles), one instance per marker
            pass.draw(0..6, 0..self.marker_count);
        }

        self.ctx.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

impl WgpuRenderer {
    /// Perform a pick operation at the given screen coordinates.
    ///
    /// Renders each visible surface to the picking buffer with its surface ID,
    /// reads back the pixel at (x, y), and returns the vertex index and surface ID if hit.
    pub fn pick(&mut self, screen_x: f32, screen_y: f32) -> Option<PickResult> {
        // Convert screen coords to pixel coords
        let x = screen_x as u32;
        let y = screen_y as u32;

        // Need at least one surface to pick from
        if !self.scene.has_surface() {
            return None;
        }

        // Collect surfaces with transforms to pick from
        let surfaces: Vec<_> = self.scene.iter_surfaces_with_transforms().collect();

        // Create command encoder for pick pass
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pick_encoder"),
            });

        // Render pick pass for each surface with its transform
        // Note: Later surfaces will overwrite earlier ones at the same pixel (painter's algorithm)
        // For proper depth handling, we'd need a depth buffer in the picking pass
        for (surf_id, transform) in surfaces {
            // Update camera uniform with this surface's transform
            self.update_camera_with_offset(transform.translation);

            // Update the picking uniform buffer with the current surface ID
            let surf_id_bytes = bytemuck::bytes_of(&surf_id);
            self.ctx
                .queue
                .write_buffer(&self.picking_uniform_buffer, 0, surf_id_bytes);

            // Create the picking bind group for this surface
            let picking_bg = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("picking_bg"),
                layout: &self.pipelines.picking_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.picking_uniform_buffer.as_entire_binding(),
                }],
            });

            // Render pick pass for this surface
            self.picking.render_pick_pass(
                &mut encoder,
                &self.pipelines,
                &self.picking_camera_bind_group,
                &picking_bg,
                &self.resources,
                surf_id,
            );
        }

        // Copy the pixel at (x, y) to readback buffer
        self.picking.copy_pixel_to_buffer(&mut encoder, x, y);

        // Submit and wait for GPU
        self.ctx.queue.submit(Some(encoder.finish()));

        // Read the result
        self.picking.read_pick_result(&self.ctx.device)
    }

    /// Request a pick at screen coordinates. Non-blocking.
    /// Returns false if a pick is already in progress or no surfaces loaded.
    pub fn request_pick(&mut self, screen_x: f32, screen_y: f32) -> bool {
        if !self.scene.has_surface() {
            return false;
        }

        let x = screen_x as u32;
        let y = screen_y as u32;

        if !self.picking.start_pick(x, y) {
            return false; // Already pending
        }

        // Render pick pass (same as current pick() method)
        let surfaces: Vec<_> = self.scene.iter_surfaces_with_transforms().collect();

        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("pick_encoder"),
        });

        for (surf_id, transform) in surfaces {
            self.update_camera_with_offset(transform.translation);

            let surf_id_bytes = bytemuck::bytes_of(&surf_id);
            self.ctx.queue.write_buffer(&self.picking_uniform_buffer, 0, surf_id_bytes);

            let picking_bg = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("picking_bg"),
                layout: &self.pipelines.picking_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.picking_uniform_buffer.as_entire_binding(),
                }],
            });

            self.picking.render_pick_pass(
                &mut encoder,
                &self.pipelines,
                &self.picking_camera_bind_group,
                &picking_bg,
                &self.resources,
                surf_id,
            );
        }

        self.picking.copy_pixel_to_buffer(&mut encoder, x, y);
        self.ctx.queue.submit(Some(encoder.finish()));

        // Start async buffer mapping
        self.picking.submit_pick_mapping();

        true
    }

    /// Poll for pick result. Call each frame.
    pub fn poll_pick(&mut self) -> Option<PickResult> {
        self.picking.poll_pick_result(&self.ctx.device)
    }

    /// Check if a pick is currently in progress.
    pub fn has_pending_pick(&self) -> bool {
        self.picking.has_pending_pick()
    }
}

impl BrainRendererBackend for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        self.ctx.config.width = width;
        self.ctx.config.height = height;
        self.ctx
            .surface
            .configure(&self.ctx.device, &self.ctx.config);

        // Resize picking system to match
        self.picking.resize(&self.ctx.device, width, height);
    }

    fn handle_event(&mut self, event: ViewerEvent) -> bool {
        self.orbit.handle_event(&event)
    }

    fn render(&mut self) -> Result<(), RenderError> {
        self.render_internal()
    }

    fn camera_state(&self) -> CameraState {
        CameraState {
            distance: self.orbit.distance,
            azimuth: self.orbit.theta,
            elevation: self.orbit.phi,
        }
    }

    fn set_camera_state(&mut self, state: CameraState) {
        self.orbit.distance = state.distance;
        self.orbit.theta = state.azimuth;
        self.orbit.phi = state.elevation;
    }
}
