//! Colored quad renderer.

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
pub struct Quad {
    pub position: Vec2,
    pub size: Vec2,
    pub fill_color: Vec4,
    pub border_size: f32,
    pub border_color: Vec4,
}

impl Quad {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32,
        4 => Float32x4
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct QuadRenderer {
    proj_matrix_buffer: Buffer,
    proj_matrix_bind_group: BindGroup,
    pipeline: Pipeline,
    instance_buffer: Buffer,
    instances: Vec<Quad>,
}

impl QuadRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, max_instances: u64) -> Self {
        let proj_matrix_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad renderer: proj. matrix buffer"),
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/quad.wgsl"));
        let pipeline = Pipeline::new(
            device,
            &shader,
            config.format,
            &[proj_matrix_bind_group.layout()],
            &[Quad::desc()],
        );

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad renderer: instance buffer"),
            size: max_instances * (std::mem::size_of::<Quad>() as u64),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            proj_matrix_buffer,
            proj_matrix_bind_group,
            pipeline,
            instance_buffer,
            instances: Vec::with_capacity(max_instances as usize),
        }
    }

    pub fn submit(&mut self, quad: Quad) {
        self.instances.push(quad);
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
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instances.len() as u32);

        self.instances.clear();
        Ok(())
    }
}
