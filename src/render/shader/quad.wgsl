struct InstanceInput {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) fill_color: vec4<f32>,
    @location(3) border_size: f32,
    @location(4) border_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) size: vec2<f32>,
    @location(1) vert_pos: vec2<f32>,
    @location(2) fill_color: vec4<f32>,
    @location(3) border_size: f32,
    @location(4) border_color: vec4<f32>,
};

// Vertex Shader

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vert_index: u32, inst_in: InstanceInput) -> VertexOutput {
    var vert_pos: vec2<f32>;
    switch vert_index {
        case 0u, 3u {
            vert_pos = vec2<f32>(inst_in.size.x, 0.0);
        }
        case 1u {
            vert_pos = vec2<f32>(0.0, 0.0);
        }
        case 2u, 4u {
            vert_pos = vec2<f32>(0.0, inst_in.size.y);
        }
        case 5u {
            vert_pos = inst_in.size;
        }
        default { }
    }


    var out: VertexOutput;
    out.clip_position = view_proj * vec4<f32>(inst_in.position + vert_pos, 0.0, 1.0);
    out.vert_pos = vert_pos;
    out.size = inst_in.size;
    out.fill_color = inst_in.fill_color;
    out.border_size = inst_in.border_size;
    out.border_color = inst_in.border_color;
    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.vert_pos.y < in.border_size
        || in.vert_pos.x < in.border_size
        || in.vert_pos.y > in.size.y - in.border_size
        || in.vert_pos.x > in.size.x - in.border_size
    {
        return in.border_color;
    } else {
        // TODO Handle alpha properly.
        if in.fill_color.a == 0.0 {
            discard;
        }
        return in.fill_color;
    }
}
