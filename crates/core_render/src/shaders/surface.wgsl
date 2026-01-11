// =============================================================================
// Surface Shader - Consolidated Bind Groups (max 4 for WebGPU compatibility)
// =============================================================================
//
// Bind Group Layout:
//   Group 0: Camera + Selection uniforms
//   Group 1: Overlay data + params
//   Group 2: Colormap + Parcellation
//   Group 3: ROI mask
// =============================================================================

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    model_offset: vec4<f32>,  // xyz = translation, w = unused (for alignment)
};

struct OverlayUniforms {
    data_min: f32,
    data_max: f32,
    threshold: f32,
    use_threshold: f32,
    // debug_mode: 0 = normal, 1 = normals, 2 = raw overlay, 3 = vertex_id
    debug_mode: u32,
    // color_source: 0 = overlay, 1 = parcellation
    color_source: u32,
    // parcellation_display: 0 = fill, 1 = edges, 2 = fill_and_edges
    parcellation_display: u32,
    // roi_enabled: 0 = off, 1 = on
    roi_enabled: u32,
};

struct SelectionUniforms {
    // x = selected_vertex_id, y = selected_surface_id, z = current_surface_id, w = has_selection (0 or 1)
    selection_info: vec4<u32>,
};

// =============================================================================
// Group 0: Camera + Selection
// =============================================================================
@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(0) @binding(1)
var<uniform> selection: SelectionUniforms;

// =============================================================================
// Group 1: Overlay data + params
// =============================================================================
@group(1) @binding(0)
var<storage, read> overlay_data: array<f32>;

@group(1) @binding(1)
var<uniform> overlay_params: OverlayUniforms;

// =============================================================================
// Group 2: Colormap + Parcellation
// =============================================================================
@group(2) @binding(0)
var colormap_texture: texture_2d<f32>;

@group(2) @binding(1)
var colormap_sampler: sampler;

@group(2) @binding(2)
var<storage, read> parcellation_labels: array<u32>;

@group(2) @binding(3)
var region_color_texture: texture_1d<f32>;

// =============================================================================
// Group 3: ROI mask
// =============================================================================
@group(3) @binding(0)
var<storage, read> roi_mask: array<f32>;

// =============================================================================
// Vertex/Fragment I/O
// =============================================================================
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) vertex_id: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) data_value: f32,
    @location(2) @interpolate(flat) vertex_id: u32,
    @location(3) @interpolate(flat) region_id: u32,
    @location(4) roi_value: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Apply model offset (translation) to position
    let world_pos = in.position + camera.model_offset.xyz;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.normal = normalize(in.normal);
    out.data_value = overlay_data[in.vertex_id];
    out.vertex_id = in.vertex_id;

    // Parcellation label (default to 0 if not available)
    out.region_id = parcellation_labels[in.vertex_id];

    // ROI mask value
    if overlay_params.roi_enabled == 1u {
        out.roi_value = roi_mask[in.vertex_id];
    } else {
        out.roi_value = 0.0;
    }

    return out;
}

// Sample region color from 1D texture
fn get_region_color(region_id: u32) -> vec4<f32> {
    // Use textureLoad for direct integer indexing
    return textureLoad(region_color_texture, i32(region_id), 0);
}

// Apply lighting to a color
fn apply_lighting(color: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let n = normalize(normal);
    let light_dir = normalize(vec3<f32>(0.3, 0.5, 0.8));
    let ndotl = max(dot(n, light_dir), 0.0);
    return color * (0.4 + 0.6 * ndotl);
}

// Apply ROI tint/highlight
fn apply_roi_tint(color: vec3<f32>, roi_value: f32) -> vec3<f32> {
    if roi_value > 0.5 {
        // Override-style tint so ROI stays visible across colormaps
        let roi_color = vec3<f32>(1.0, 0.2, 0.1);
        return mix(color, roi_color, 0.85);
    }
    return color;
}

// Apply selection highlight
fn apply_selection_highlight(color: vec3<f32>, is_selected: bool) -> vec3<f32> {
    if is_selected {
        let highlight_color = vec3<f32>(1.0, 0.8, 0.0);
        return mix(color, highlight_color, 0.7);
    }
    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let value = in.data_value;

    // Check if this vertex is selected
    let has_selection = selection.selection_info.w == 1u;
    let selected_vertex_id = selection.selection_info.x;
    let selected_surface_id = selection.selection_info.y;
    let current_surface_id = selection.selection_info.z;
    let is_selected = has_selection
        && in.vertex_id == selected_vertex_id
        && current_surface_id == selected_surface_id;

    // Handle debug modes first
    let debug_mode = overlay_params.debug_mode;

    // Debug mode 1: Normals as color
    if debug_mode == 1u {
        let n = normalize(in.normal);
        let normal_color = (n + 1.0) * 0.5;
        var result = normal_color;
        result = apply_roi_tint(result, in.roi_value);
        result = apply_selection_highlight(result, is_selected);
        return vec4<f32>(result, 1.0);
    }

    // Debug mode 2: Raw overlay grayscale
    if debug_mode == 2u {
        let range = overlay_params.data_max - overlay_params.data_min;
        var t: f32;
        if range > 0.0 {
            t = clamp((value - overlay_params.data_min) / range, 0.0, 1.0);
        } else {
            t = 0.5;
        }
        // Handle NaN
        if value != value {
            var result = vec3<f32>(1.0, 0.0, 1.0);  // Magenta for NaN
            result = apply_selection_highlight(result, is_selected);
            return vec4<f32>(result, 1.0);
        }
        var result = vec3<f32>(t, t, t);
        result = apply_roi_tint(result, in.roi_value);
        result = apply_selection_highlight(result, is_selected);
        return vec4<f32>(result, 1.0);
    }

    // Debug mode 3: Vertex ID as color
    if debug_mode == 3u {
        let id = in.vertex_id;
        let r = f32((id * 73u + 31u) % 256u) / 255.0;
        let g = f32((id * 127u + 97u) % 256u) / 255.0;
        let b = f32((id * 211u + 173u) % 256u) / 255.0;
        var result = vec3<f32>(r, g, b);
        result = apply_roi_tint(result, in.roi_value);
        result = apply_selection_highlight(result, is_selected);
        return vec4<f32>(result, 1.0);
    }

    // Normal rendering modes (debug_mode == 0)
    var base_color: vec3<f32>;
    var is_edge = false;

    // Determine color source
    if overlay_params.color_source == 1u {
        // Parcellation mode
        let region_color = get_region_color(in.region_id);
        base_color = region_color.rgb;

        // Edge detection for parcellation boundaries
        // Note: For proper edge detection, we'd need neighbor information.
        // This simplified version darkens low-alpha regions as "unknown"
        if region_color.a < 0.5 {
            base_color = vec3<f32>(0.3, 0.3, 0.3); // Gray for unknown regions
        }

        // Apply parcellation display mode
        let parc_display = overlay_params.parcellation_display;
        if parc_display == 1u {
            // Edges only mode: darken fill, show only boundaries
            // Without neighbor info, we approximate by showing outline effect
            base_color = base_color * 0.3;
        }
        // For fill_and_edges (2), we'd overlay edges on fill - needs boundary detection
    } else {
        // Overlay mode (default)
        let is_nan = value != value;
        let use_threshold = overlay_params.use_threshold > 0.5;
        let below_threshold = abs(value) < overlay_params.threshold;
        let suppress_overlay = is_nan || (use_threshold && below_threshold);

        // Normalize value to [0, 1] for colormap lookup
        let range = overlay_params.data_max - overlay_params.data_min;
        var t: f32;
        if range > 0.0 {
            t = clamp((value - overlay_params.data_min) / range, 0.0, 1.0);
        } else {
            t = 0.5;
        }

        // Sample colormap texture using textureSampleLevel to avoid uniform control flow requirement
        // Level 0.0 is fine since our colormap is a simple 1D/2D texture without mipmaps
        let sampled = textureSampleLevel(colormap_texture, colormap_sampler, vec2<f32>(t, 0.5), 0.0).rgb;

        // Choose between gray and sampled color without data-dependent control flow
        let gray = vec3<f32>(0.3, 0.3, 0.3);
        base_color = select(gray, sampled, !suppress_overlay);
    }

    // Apply lighting
    var lit_color = apply_lighting(base_color, in.normal);

    // Apply ROI tint
    lit_color = apply_roi_tint(lit_color, in.roi_value);

    // Apply selection highlight
    lit_color = apply_selection_highlight(lit_color, is_selected);

    return vec4<f32>(lit_color, 1.0);
}
