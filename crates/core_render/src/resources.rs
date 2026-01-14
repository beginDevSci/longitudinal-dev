use std::collections::HashMap;

use io_formats::geometry::BrainGeometry;
use wgpu::util::DeviceExt;

pub type SurfaceId = u32;
pub type OverlayId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColormapKind {
    RdBu,
    Viridis,
    Hot,
    /// Colorblind-friendly perceptually uniform colormap.
    Cividis,
    /// Perceptually uniform sequential colormap (purple to yellow).
    Plasma,
}

pub struct SurfaceBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub normal_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub vertex_id_buffer: wgpu::Buffer,
    /// Per-vertex region labels for parcellation coloring (optional).
    pub label_buffer: Option<wgpu::Buffer>,
    /// Number of vertices (for validation).
    pub vertex_count: u32,
}

pub struct OverlayBuffer {
    pub data_buffer: wgpu::Buffer,
    pub n_vertices: u32,
    pub range: (f32, f32),
}

pub struct ColormapTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

/// Per-vertex ROI mask data.
pub struct RoiMaskBuffer {
    /// Per-vertex mask values (0.0 = not in ROI, 1.0 = in ROI).
    pub mask_buffer: wgpu::Buffer,
    /// Number of vertices.
    pub n_vertices: u32,
}

/// Region color lookup texture for parcellation rendering.
/// Stores RGBA colors indexed by region ID.
pub struct RegionColorTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    /// Maximum region ID stored.
    pub max_region_id: u32,
}

pub struct ResourceManager {
    pub surface_buffers: HashMap<SurfaceId, SurfaceBuffers>,
    pub overlay_buffers: HashMap<OverlayId, OverlayBuffer>,
    pub colormaps: HashMap<ColormapKind, ColormapTexture>,
    /// Region color textures per surface (for parcellation coloring).
    pub region_colors: HashMap<SurfaceId, RegionColorTexture>,
    /// ROI mask buffers per surface.
    pub roi_masks: HashMap<SurfaceId, RoiMaskBuffer>,
}

/// Sample a colormap at position t (0.0 to 1.0) returning RGBA bytes
fn sample_colormap(kind: ColormapKind, t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    match kind {
        ColormapKind::RdBu => {
            // Red (t=0) -> white (t=0.5) -> blue (t=1)
            // Matches matplotlib RdBu convention
            let (r, g, b) = if t < 0.5 {
                // Red to white
                let s = t * 2.0;
                (1.0, s, s)
            } else {
                // White to blue
                let s = (t - 0.5) * 2.0;
                (1.0 - s, 1.0 - s, 1.0)
            };
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
        }
        ColormapKind::Viridis => {
            // Approximate viridis: dark blue/purple -> teal -> yellow
            let (r, g, b) = if t < 0.25 {
                let s = t * 4.0;
                (0.267 + s * 0.05, 0.004 + s * 0.15, 0.329 + s * 0.15)
            } else if t < 0.5 {
                let s = (t - 0.25) * 4.0;
                (0.317 - s * 0.1, 0.154 + s * 0.25, 0.479 - s * 0.05)
            } else if t < 0.75 {
                let s = (t - 0.5) * 4.0;
                (0.217 + s * 0.25, 0.404 + s * 0.2, 0.429 - s * 0.15)
            } else {
                let s = (t - 0.75) * 4.0;
                (0.467 + s * 0.52, 0.604 + s * 0.35, 0.279 - s * 0.1)
            };
            [
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]
        }
        ColormapKind::Hot => {
            // Black -> red -> yellow -> white
            let (r, g, b) = if t < 0.33 {
                let s = t * 3.0;
                (s, 0.0, 0.0)
            } else if t < 0.66 {
                let s = (t - 0.33) * 3.0;
                (1.0, s, 0.0)
            } else {
                let s = (t - 0.66) * 3.0;
                (1.0, 1.0, s)
            };
            [
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]
        }
        ColormapKind::Cividis => {
            // Cividis: colorblind-friendly, dark blue -> yellow
            // Approximate the cividis colormap
            let (r, g, b) = if t < 0.25 {
                let s = t * 4.0;
                (0.0 + s * 0.15, 0.135 + s * 0.08, 0.304 + s * 0.1)
            } else if t < 0.5 {
                let s = (t - 0.25) * 4.0;
                (0.15 + s * 0.2, 0.215 + s * 0.15, 0.404 - s * 0.05)
            } else if t < 0.75 {
                let s = (t - 0.5) * 4.0;
                (0.35 + s * 0.3, 0.365 + s * 0.15, 0.354 - s * 0.1)
            } else {
                let s = (t - 0.75) * 4.0;
                (0.65 + s * 0.34, 0.515 + s * 0.35, 0.254 - s * 0.05)
            };
            [
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]
        }
        ColormapKind::Plasma => {
            // Plasma: purple -> pink -> orange -> yellow
            let (r, g, b) = if t < 0.25 {
                let s = t * 4.0;
                (0.05 + s * 0.35, 0.03 + s * 0.05, 0.53 + s * 0.1)
            } else if t < 0.5 {
                let s = (t - 0.25) * 4.0;
                (0.4 + s * 0.35, 0.08 + s * 0.1, 0.63 - s * 0.15)
            } else if t < 0.75 {
                let s = (t - 0.5) * 4.0;
                (0.75 + s * 0.2, 0.18 + s * 0.35, 0.48 - s * 0.25)
            } else {
                let s = (t - 0.75) * 4.0;
                (0.95 + s * 0.05, 0.53 + s * 0.4, 0.23 - s * 0.1)
            };
            [
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]
        }
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            surface_buffers: HashMap::new(),
            overlay_buffers: HashMap::new(),
            colormaps: HashMap::new(),
            region_colors: HashMap::new(),
            roi_masks: HashMap::new(),
        }
    }

    pub fn upload_surface(&mut self, device: &wgpu::Device, id: SurfaceId, geom: &BrainGeometry) {
        let vertex_count = geom.vertices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface_vertices"),
            contents: bytemuck::cast_slice(&geom.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let normal_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface_normals"),
            contents: bytemuck::cast_slice(&geom.normals),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface_indices"),
            contents: bytemuck::cast_slice(&geom.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = (geom.indices.len() as u32) * 3;

        let vertex_ids: Vec<u32> = (0..vertex_count).collect();
        let vertex_id_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface_vertex_ids"),
            contents: bytemuck::cast_slice(&vertex_ids),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.surface_buffers.insert(
            id,
            SurfaceBuffers {
                vertex_buffer,
                normal_buffer,
                index_buffer,
                index_count,
                vertex_id_buffer,
                label_buffer: None, // Set via upload_parcellation_labels
                vertex_count,
            },
        );
    }

    /// Upload parcellation labels for a surface.
    ///
    /// Labels are per-vertex region IDs that index into the region color texture.
    pub fn upload_parcellation_labels(
        &mut self,
        device: &wgpu::Device,
        surface_id: SurfaceId,
        labels: &[u32],
    ) {
        let label_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("parcellation_labels"),
            contents: bytemuck::cast_slice(labels),
            usage: wgpu::BufferUsages::STORAGE,
        });

        if let Some(surface) = self.surface_buffers.get_mut(&surface_id) {
            surface.label_buffer = Some(label_buffer);
        }
    }

    /// Upload region colors for parcellation rendering.
    ///
    /// Creates a 1D texture where each texel is the RGBA color for a region ID.
    /// The texture is sized to fit the maximum region ID.
    pub fn upload_region_colors(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_id: SurfaceId,
        region_colors: &[[f32; 4]],
    ) {
        let max_region_id = region_colors.len() as u32;
        // Use power of 2 size for better GPU compatibility
        let texture_size = max_region_id.next_power_of_two().max(1);

        // Convert to RGBA8
        let mut data = vec![0u8; (texture_size * 4) as usize];
        for (i, color) in region_colors.iter().enumerate() {
            let offset = i * 4;
            data[offset] = (color[0] * 255.0).clamp(0.0, 255.0) as u8;
            data[offset + 1] = (color[1] * 255.0).clamp(0.0, 255.0) as u8;
            data[offset + 2] = (color[2] * 255.0).clamp(0.0, 255.0) as u8;
            data[offset + 3] = (color[3] * 255.0).clamp(0.0, 255.0) as u8;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("region_color_texture"),
            size: wgpu::Extent3d {
                width: texture_size,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8Unorm, // Not sRGB for direct color lookup
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture_size * 4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: texture_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.region_colors.insert(
            surface_id,
            RegionColorTexture {
                texture,
                view,
                max_region_id,
            },
        );
    }

    /// Get the region color texture for a surface.
    pub fn get_region_colors(&self, surface_id: SurfaceId) -> Option<&RegionColorTexture> {
        self.region_colors.get(&surface_id)
    }

    /// Upload ROI mask data for a surface.
    ///
    /// The mask is a per-vertex float where 1.0 means the vertex is in the ROI.
    pub fn upload_roi_mask(
        &mut self,
        device: &wgpu::Device,
        surface_id: SurfaceId,
        mask: &[f32],
    ) {
        let mask_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("roi_mask"),
            contents: bytemuck::cast_slice(mask),
            usage: wgpu::BufferUsages::STORAGE,
        });

        self.roi_masks.insert(
            surface_id,
            RoiMaskBuffer {
                mask_buffer,
                n_vertices: mask.len() as u32,
            },
        );
    }

    /// Get the ROI mask buffer for a surface.
    pub fn get_roi_mask(&self, surface_id: SurfaceId) -> Option<&RoiMaskBuffer> {
        self.roi_masks.get(&surface_id)
    }

    /// Clear ROI mask for a surface.
    pub fn clear_roi_mask(&mut self, surface_id: SurfaceId) {
        self.roi_masks.remove(&surface_id);
    }

    pub fn get_surface(&self, id: SurfaceId) -> Option<&SurfaceBuffers> {
        self.surface_buffers.get(&id)
    }

    /// Upload overlay data (per-vertex scalar values) to the GPU
    pub fn upload_overlay(
        &mut self,
        device: &wgpu::Device,
        id: OverlayId,
        values: &[f32],
        range: (f32, f32),
    ) {
        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("overlay_data"),
            contents: bytemuck::cast_slice(values),
            usage: wgpu::BufferUsages::STORAGE,
        });

        self.overlay_buffers.insert(
            id,
            OverlayBuffer {
                data_buffer,
                n_vertices: values.len() as u32,
                range,
            },
        );
    }

    pub fn get_overlay(&self, id: OverlayId) -> Option<&OverlayBuffer> {
        self.overlay_buffers.get(&id)
    }

    /// Get or create a colormap texture
    pub fn get_or_create_colormap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        kind: ColormapKind,
    ) -> &ColormapTexture {
        if !self.colormaps.contains_key(&kind) {
            let cmap = Self::create_colormap_texture(device, queue, kind);
            self.colormaps.insert(kind, cmap);
        }
        self.colormaps.get(&kind).unwrap()
    }

    /// Get an existing colormap (immutable borrow).
    pub fn get_colormap(&self, kind: ColormapKind) -> Option<&ColormapTexture> {
        self.colormaps.get(&kind)
    }

    /// Create a 1D colormap texture (256 x 1)
    fn create_colormap_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        kind: ColormapKind,
    ) -> ColormapTexture {
        const N: u32 = 256;

        // Generate colormap samples
        let mut data = Vec::with_capacity((N * 4) as usize);
        for i in 0..N {
            let t = i as f32 / (N - 1) as f32;
            let rgba = sample_colormap(kind, t);
            data.extend_from_slice(&rgba);
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("colormap_texture"),
            size: wgpu::Extent3d {
                width: N,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(N * 4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: N,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("colormap_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        ColormapTexture {
            texture,
            view,
            sampler,
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
