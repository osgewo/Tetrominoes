struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) position: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

// Vertex Shader

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(vert_in: VertexInput, inst_in: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(vert_in.position + inst_in.position, 0.0, 1.0);
    out.tex_coords = vert_in.tex_coords;
    return out;
}

// Fragment Shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
