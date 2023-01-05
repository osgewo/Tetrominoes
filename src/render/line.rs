use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BlendState, Buffer, BufferAddress, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, Face, FragmentState, FrontFace, LoadOp, MultisampleState,
    Operations, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, SurfaceError, TextureView, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use super::context::RenderContext;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: Vector2<f32>,
    color: Vector3<f32>,
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
    render_context: Arc<Mutex<RenderContext>>,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl LineRenderer {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        let locked_context = render_context.lock().unwrap();
        let device = &locked_context.device;

        let shader = device.create_shader_module(include_wgsl!("line.wgsl"));
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
            Vertex { position: Vector2::new(-0.5, 0.0), color: Vector3::new(255.0, 0.0, 0.0) },
            Vertex { position: Vector2::new( 0.5, 0.0), color: Vector3::new(255.0, 0.0, 0.0) },
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("line-vertex-buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        drop(locked_context);

        Self {
            render_context,
            pipeline,
            vertex_buffer,
            vertex_count: VERTICES.len() as u32,
        }
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
    ) -> Result<(), SurfaceError> {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);

        Ok(())
    }
}
