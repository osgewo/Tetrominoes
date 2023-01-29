struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) color: vec4<f32>,
}

// Vertex Shader

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(vert_in: VertexInput, inst_in: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj * vec4<f32>(vert_in.position + inst_in.position, 0.0, 1.0);
    out.color = inst_in.color;
    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
