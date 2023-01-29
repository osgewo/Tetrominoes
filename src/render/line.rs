use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BlendState, Buffer, BufferAddress, BufferUsages, ColorTargetState,
    ColorWrites, Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPipeline, SurfaceError, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use super::context::RenderContext;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: Vec2,
    color: Vec3,
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Float32x3];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct LineRenderer {
    #[allow(dead_code)]
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl LineRenderer {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        let locked_context = render_context.lock().unwrap();
        let device = &locked_context.device;

        let shader = device.create_shader_module(include_wgsl!("shader/line.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line-pipeline-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: locked_context.config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        #[rustfmt::skip]
        const VERTICES: &[Vertex] = &[
            Vertex { position: Vec2::new(-0.5, 0.0), color: Vec3::new(0.0, 0.0, 0.0) },
            Vertex { position: Vec2::new( 0.5, 0.0), color: Vec3::new(0.0, 0.0, 0.0) },
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("line-vertex-buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        drop(locked_context);

        Self {
            pipeline,
            vertex_buffer,
            vertex_count: VERTICES.len() as u32,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) -> Result<(), SurfaceError> {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);

        Ok(())
    }
}
