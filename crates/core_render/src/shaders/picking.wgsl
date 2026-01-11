// Picking shader - renders vertex IDs and surface IDs as colors for GPU picking.
//
// Uses the camera uniform for transforms and a picking uniform for surface ID.
// Outputs vertex_id and surface_id as integer colors for readback.

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    model_offset: vec4<f32>,  // xyz = translation, w = unused (for alignment)
};

struct PickingUniforms {
    surface_id: u32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var<uniform> picking: PickingUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(2) vertex_id: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) vertex_id: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Apply model offset (translation) to position
    let world_pos = in.position + camera.model_offset.xyz;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.vertex_id = in.vertex_id;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<u32> {
    // Encode vertex_id in r, surface_id in g, use 1 in alpha to distinguish from background
    return vec4<u32>(in.vertex_id, picking.surface_id, 0u, 1u);
}
