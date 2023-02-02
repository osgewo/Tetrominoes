//! Renderer for tetromino squares.

use crate::render::bind_group::{BindGroup, Entry};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, RenderPass, SurfaceConfiguration, SurfaceError,
};

use super::pipeline::Pipeline;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
// TODO Consider generating vertices in shader instead.
struct Vertex {
    position: Vec2,
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TetrominoSquare {
    pub position: Vec2,
    pub color: Vec4,
}

impl TetrominoSquare {
    /// The size of a tetromino square in pixels.
    pub const SIZE: f32 = 30.0;

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct SquareRenderer {
    proj_matrix_buffer: Buffer,
    proj_matrix_bind_group: BindGroup,
    pipeline: Pipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instance_buffer: Buffer,
    instances: Vec<TetrominoSquare>,
}

impl SquareRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, max_instances: u64) -> Self {
        let proj_matrix_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("square renderer: proj. matrix buffer"),
            contents: bytemuck::cast_slice(&[Mat4::IDENTITY]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let proj_matrix_bind_group = BindGroup::new(
            device,
            &[Entry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                resource: proj_matrix_buffer.as_entire_binding(),
            }],
        );

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/square.wgsl"));
        let pipeline = Pipeline::new(
            device,
            &shader,
            config.format,
            &[proj_matrix_bind_group.layout()],
            &[Vertex::desc(), TetrominoSquare::desc()],
        );

        #[rustfmt::skip]
        const VERTICES: &[Vertex] = &[
            Vertex { position: Vec2::new(1.0, 0.0) },
            Vertex { position: Vec2::new(0.0, 0.0) },
            Vertex { position: Vec2::new(0.0, 1.0) },
            Vertex { position: Vec2::new(1.0, 1.0) },
        ];
        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 2, 3,
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("square renderer: instance buffer"),
            size: max_instances * (std::mem::size_of::<TetrominoSquare>() as u64),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            proj_matrix_buffer,
            proj_matrix_bind_group,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: INDICES.len() as u32,
            instance_buffer,
            instances: Vec::with_capacity(max_instances as usize),
        }
    }

    pub fn submit_iter(&mut self, squares: impl Iterator<Item = TetrominoSquare>) {
        self.instances.extend(squares);
    }

    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut RenderPass<'a>,
        queue: &Queue,
        proj_matrix: Mat4,
    ) -> Result<(), SurfaceError> {
        queue.write_buffer(
            &self.proj_matrix_buffer,
            0,
            bytemuck::cast_slice(&[proj_matrix]),
        );

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.proj_matrix_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as u32);

        self.instances.clear();

        Ok(())
    }
}
