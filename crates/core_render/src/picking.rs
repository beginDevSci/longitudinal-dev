//! GPU picking system for vertex selection.
//!
//! This module provides infrastructure for GPU-based picking using color ID rendering:
//! - Renders vertex IDs as integer colors to an offscreen texture
//! - Reads back a single pixel at the click position
//! - Decodes the vertex index from the color value

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::pipelines::{Pipelines, PICKING_FORMAT};
use crate::resources::{ResourceManager, SurfaceId};
use crate::traits::PickResult;

/// Represents an in-flight pick operation waiting for GPU readback.
pub struct PendingPick {
    /// Screen coordinates where the pick was requested
    pub screen_x: f32,
    pub screen_y: f32,
    /// Whether the buffer has been mapped (callback fired)
    pub mapped: Arc<AtomicBool>,
}

/// GPU picking system for determining which vertex was clicked.
///
/// Uses an offscreen render target with integer format to encode vertex IDs.
pub struct PickingSystem {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    /// Currently pending pick operation, if any
    pending: Option<PendingPick>,
}

impl PickingSystem {
    /// Size of readback buffer in bytes.
    /// WebGPU requires bytes_per_row to be a multiple of 256, so allocate at least one padded row.
    const READBACK_SIZE: u64 = 256;

    /// Create a new picking system with the given dimensions.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (texture, view) = Self::create_texture(device, width, height);
        let readback_buffer = Self::create_readback_buffer(device);

        Self {
            texture,
            view,
            readback_buffer,
            width,
            height,
            pending: None,
        }
    }

    /// Create the picking render target texture.
    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("picking_texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PICKING_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// Create the buffer used to read back pixel data.
    fn create_readback_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("picking_readback"),
            size: Self::READBACK_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    /// Resize the picking render target.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == self.width && height == self.height {
            return;
        }
        let (texture, view) = Self::create_texture(device, width, height);
        self.texture = texture;
        self.view = view;
        self.width = width;
        self.height = height;
    }

    /// Render the picking pass to the offscreen texture.
    ///
    /// The `picking_bg` bind group should contain the surface_id uniform.
    pub fn render_pick_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipelines: &Pipelines,
        camera_bg: &wgpu::BindGroup,
        picking_bg: &wgpu::BindGroup,
        resources: &ResourceManager,
        surface_id: SurfaceId,
    ) {
        let Some(bufs) = resources.get_surface(surface_id) else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("picking_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Clear to 0 (no vertex = vertex_id 0 with alpha 0)
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        pass.set_pipeline(&pipelines.picking);
        pass.set_bind_group(0, camera_bg, &[]);
        pass.set_bind_group(1, picking_bg, &[]);
        // Picking uses positions (slot 0) and vertex_ids (slot 1 in picking layout)
        pass.set_vertex_buffer(0, bufs.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, bufs.vertex_id_buffer.slice(..));
        pass.set_index_buffer(bufs.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..bufs.index_count, 0, 0..1);
    }

    /// Copy the pixel at (x, y) to the readback buffer.
    pub fn copy_pixel_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, x: u32, y: u32) {
        // Clamp coordinates to valid range
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    // Rgba32Uint = 16 bytes per pixel; pad to 256-byte multiple per WebGPU requirements.
                    bytes_per_row: Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Read the pick result from the readback buffer.
    ///
    /// This must be called after the command buffer containing `copy_pixel_to_buffer`
    /// has been submitted and the GPU work has completed.
    pub fn read_pick_result(&self, device: &wgpu::Device) -> Option<PickResult> {
        let buffer_slice = self.readback_buffer.slice(..);

        // Map the buffer synchronously (blocking)
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        // Poll the device until the buffer is mapped
        device.poll(wgpu::Maintain::Wait);

        // Wait for the mapping to complete
        if rx.recv().ok()?.is_err() {
            return None;
        }

        // Read the data
        let data = buffer_slice.get_mapped_range();
        let values: &[u32] = bytemuck::cast_slice(&data);

        // Rgba32Uint: [vertex_id, surface_id, 0, alpha]
        // alpha = 1 means we hit a vertex, alpha = 0 means background
        let vertex_id = values[0];
        let surface_id = values[1];
        let alpha = values[3];

        drop(data);
        self.readback_buffer.unmap();

        if alpha == 0 {
            // No vertex was hit (background)
            None
        } else {
            Some(PickResult {
                vertex_index: Some(vertex_id),
                surface_id: Some(surface_id),
                position: None, // Filled in by caller
                value: None,    // Filled in by caller
            })
        }
    }

    /// Get the current dimensions of the picking texture.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns true if there's a pick operation in flight.
    pub fn has_pending_pick(&self) -> bool {
        self.pending.is_some()
    }

    /// Start an async pick operation. Returns false if a pick is already pending.
    pub fn start_pick(&mut self, x: u32, y: u32) -> bool {
        if self.pending.is_some() {
            return false; // Already have a pending pick
        }

        // Store pending state
        self.pending = Some(PendingPick {
            screen_x: x as f32,
            screen_y: y as f32,
            mapped: Arc::new(AtomicBool::new(false)),
        });

        true
    }

    /// Submit the buffer mapping request. Call after copy_pixel_to_buffer.
    pub fn submit_pick_mapping(&mut self) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };

        let buffer_slice = self.readback_buffer.slice(..);

        // Clone the Arc for the closure
        let mapped = pending.mapped.clone();

        // Start async mapping (non-blocking)
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            if result.is_ok() {
                mapped.store(true, Ordering::Release);
            }
        });
    }

    /// Poll for pick result. Returns Some if ready, None if still pending or no pick.
    /// Uses non-blocking poll.
    pub fn poll_pick_result(&mut self, device: &wgpu::Device) -> Option<PickResult> {
        let pending = self.pending.as_ref()?;

        // Non-blocking poll
        device.poll(wgpu::Maintain::Poll);

        // Check if mapped
        if !pending.mapped.load(Ordering::Acquire) {
            return None; // Not ready yet
        }

        // Read the result
        let buffer_slice = self.readback_buffer.slice(..);
        let data = buffer_slice.get_mapped_range();
        let values: &[u32] = bytemuck::cast_slice(&data);

        let vertex_id = values[0];
        let surface_id = values[1];
        let alpha = values[3];

        drop(data);
        self.readback_buffer.unmap();

        // Clear pending state
        self.pending = None;

        if alpha == 0 {
            // No vertex was hit (background)
            None
        } else {
            Some(PickResult {
                vertex_index: Some(vertex_id),
                surface_id: Some(surface_id),
                position: None,
                value: None,
            })
        }
    }
}
