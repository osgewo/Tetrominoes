use std::ops::Deref;

use wgpu::{BindGroupLayout, Device, ShaderModule, TextureFormat, VertexBufferLayout};

pub struct Pipeline {
    inner: wgpu::RenderPipeline,
    layout: wgpu::PipelineLayout,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        shader: &ShaderModule,
        texture_format: TextureFormat,
        bind_group_layouts: &[&BindGroupLayout],
        vertex_buffer_layouts: &[VertexBufferLayout],
    ) -> Self {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            inner: pipeline,
            layout,
        }
    }

    pub fn layout(&self) -> &wgpu::PipelineLayout {
        &self.layout
    }
}

impl Deref for Pipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
