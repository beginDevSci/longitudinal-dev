//! Core rendering abstractions for the brain viewer.
//!
//! This crate provides GPU-accelerated rendering for brain surface visualization,
//! implemented using wgpu for cross-platform WebGPU support. It is designed to be
//! UI-agnostic and can be integrated with various frontend frameworks.
//!
//! # Architecture
//!
//! The crate is organized into several key modules:
//!
//! - [`wgpu_renderer`]: The main renderer implementation (`WgpuRenderer`)
//! - [`device`]: GPU device and context management (`DeviceContext`)
//! - [`scene`]: Scene graph for multi-surface rendering (`Scene`, `SceneNode`)
//! - [`pipelines`]: WGSL shader pipelines for surface, picking, and markers
//! - [`resources`]: GPU resource management (buffers, textures, colormaps)
//! - [`picking`]: GPU-based vertex picking system
//! - [`orbit`]: Orbit camera controller for 3D navigation
//! - [`traits`]: Backend-agnostic traits and error types
//!
//! # Main Types
//!
//! ## [`WgpuRenderer`](wgpu_renderer::WgpuRenderer)
//!
//! The primary renderer that manages all GPU resources and rendering state.
//! It provides methods for:
//! - Loading and displaying brain surface meshes
//! - Applying statistical overlays with colormaps
//! - GPU-based vertex picking for interaction
//! - Marker rendering for annotations
//! - Camera control via orbit controller
//!
//! ## [`DeviceContext`](device::DeviceContext)
//!
//! Encapsulates the wgpu device, queue, surface, and configuration.
//! Handles WebGPU initialization from an HTML canvas element.
//!
//! ## [`Scene`](scene::Scene)
//!
//! A node-based scene graph supporting multiple surfaces (e.g., left/right
//! hemispheres) with per-surface transforms and visibility control.
//!
//! ## [`Pipelines`](pipelines::Pipelines)
//!
//! Contains all WGSL shader pipelines:
//! - Surface rendering with overlay colormapping
//! - Picking pass for GPU-based vertex selection
//! - Marker rendering for point annotations
//!
//! # Feature Flags
//!
//! This crate is designed for WASM/WebGPU targets. The `DeviceContext`
//! initialization requires `target_arch = "wasm32"` for full functionality.
//!
//! # Example
//!
//! ```ignore
//! use core_render::wgpu_renderer::WgpuRenderer;
//! use core_render::resources::SurfaceId;
//!
//! // Create renderer from canvas
//! let mut renderer = WgpuRenderer::new(canvas).await?;
//!
//! // Load a surface
//! renderer.set_surface(SurfaceId(0), &brain_geometry);
//!
//! // Set overlay data
//! renderer.set_overlay(overlay_id, &values, (min, max), threshold);
//!
//! // Render a frame
//! renderer.render()?;
//!
//! // Pick a vertex at screen coordinates
//! if let Some(pick) = renderer.pick(x, y) {
//!     println!("Hit vertex {} on surface {}", pick.vertex_index, pick.surface_id);
//! }
//! ```

pub mod traits;
pub mod device;
pub mod orbit;
pub mod resources;
pub mod pipelines;
pub mod picking;
pub mod scene;
pub mod wgpu_renderer;
pub mod debug;
pub mod color_source;

pub use debug::DebugView;
pub use color_source::{ColorSource, ParcellationDisplay};
