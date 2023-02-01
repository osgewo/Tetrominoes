struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) vert_pos: vec2<f32>,
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
    out.clip_position = view_proj * vec4<f32>(vert_in.position * 30.0 + inst_in.position, 0.0, 1.0);
    out.color = inst_in.color;
    out.vert_pos = vert_in.position;
    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x: f32;
    if in.vert_pos.y < 0.2 && in.vert_pos.x >= in.vert_pos.y && 1.0 - in.vert_pos.x >= in.vert_pos.y {
        // Top
        x = 1.2;
    } else if in.vert_pos.y > 0.8 && in.vert_pos.x < in.vert_pos.y && 1.0 - in.vert_pos.x < in.vert_pos.y {
        // Bottom
        x = 0.6;
    } else if in.vert_pos.x > 0.8 && in.vert_pos.y < in.vert_pos.x {
        // Right
        x = 0.9;
    } else if in.vert_pos.x < 0.2 {
        // Left
        x = 0.8;
    } else {
        // Middle
        x = 1.0;
    }
    return vec4<f32>(in.color.r * x, in.color.g * x, in.color.b * x, in.color.a);
}
