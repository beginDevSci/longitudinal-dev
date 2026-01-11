// Marker shader for rendering point markers (annotations, selections, etc.)
// Renders small quads/billboards at marker positions.

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    model_offset: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,  // World-space marker center
    @location(1) corner: vec2<f32>,    // Billboard corner offset (-1 to 1)
    @location(2) color: vec3<f32>,     // Marker color
    @location(3) size: f32,            // Size multiplier
    @location(4) selected: f32,        // 1.0 if selected, 0.0 otherwise
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,        // For circle rendering
    @location(1) color: vec3<f32>,
    @location(2) selected: f32,
};

// Base size of marker in world units
const BASE_MARKER_SIZE: f32 = 3.0;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Apply model offset and project to clip space first
    let world_pos = in.position + camera.model_offset.xyz;
    let clip_pos = camera.view_proj * vec4<f32>(world_pos, 1.0);

    // Calculate screen-space offset for billboard
    // Scale by w to keep consistent screen size, multiply by size factor
    let effective_size = BASE_MARKER_SIZE * in.size;
    let screen_offset = in.corner * effective_size * 0.01 * clip_pos.w;

    out.clip_position = clip_pos + vec4<f32>(screen_offset, 0.0, 0.0);
    out.uv = in.corner;
    out.color = in.color;
    out.selected = in.selected;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Render as a circle with smooth edge
    let dist = length(in.uv);

    // Discard pixels outside the circle
    if dist > 1.0 {
        discard;
    }

    // Smooth edge (anti-aliasing)
    let edge_smoothness = 0.1;
    let alpha = 1.0 - smoothstep(1.0 - edge_smoothness, 1.0, dist);

    // Base color with ring effect
    let ring_start = 0.6;
    let ring_intensity = smoothstep(ring_start, 0.8, dist) * (1.0 - smoothstep(0.8, 1.0, dist));
    var final_color = mix(in.color, vec3<f32>(1.0, 1.0, 1.0), ring_intensity * 0.3);

    // Apply selection highlight
    if in.selected > 0.5 {
        // Pulsing glow effect for selected markers
        // Brighter outline and glow
        let glow_color = vec3<f32>(1.0, 0.9, 0.3); // Gold/yellow

        // Add inner glow
        let inner_glow = smoothstep(0.6, 0.0, dist);
        final_color = mix(final_color, glow_color, inner_glow * 0.4);

        // Brighter outer ring
        let outer_ring = smoothstep(0.5, 0.7, dist) * (1.0 - smoothstep(0.8, 1.0, dist));
        final_color = mix(final_color, glow_color, outer_ring * 0.8);
    }

    return vec4<f32>(final_color, alpha);
}
