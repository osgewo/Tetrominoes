#[derive(Clone, Copy, Debug, Pod, Zeroable, VertexBuffer)]
struct Vertex {
    #[vbo(location = 0)]
    position: Vector2<f32>,
    #[vbo(location = 1)]
    tex_coords: Vector2<f32>,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable, VertexBuffer)]
struct Instance {
    #[vbo(location = 2)]
    position: Vector2<f32>,
}

bind_group! DiffuseBindGroup {
    0 => Texture in Fragment,
    1 => Sampler(Filtering) in Fragment,
};

struct DiffuseBindGroup {
    layout: BindGroupLayout,
    group: BindGroup,
}

impl DiffuseBindGroup {
    fn new(binding_0: &TextureView, binding_1: Sampler) -> Self {
        
    }
}

bind_group! CameraBindGroup {
    0 => Buffer(Uniform) in Vertex,
};

fn new() -> ? {
    let pipeline = pipeline! {
        bind_groups: [diffuse_bind_group, camera_bind_group],
        shader: include_wgsl!("shader.wgsl"),
        vertex_buffers: [Vertex, Instance],
    };
}
