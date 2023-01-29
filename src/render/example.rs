use glam::{Mat4, Vec2, Vec3};
use image::GenericImageView;
use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoder, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor,
    Extent3d, Face, Features, FilterMode, FragmentState, FrontFace, ImageCopyTexture,
    ImageDataLayout, IndexFormat, Limits, LoadOp, MultisampleState, Operations, Origin3d,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType, SamplerDescriptor,
    ShaderStages, Surface, SurfaceConfiguration, SurfaceError, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexState,
    VertexStepMode,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::context::RenderContext;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Float32x2];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: Vec2,
}

impl Instance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![2 => Float32x2];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

struct Camera {
    // eye: Point3<f32>,
    // target: Point3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_proj_mat(&self) -> Mat4 {
        #[rustfmt::skip]
        // const OPENGL_TO_WGPU: Mat4 = Mat4::new(
        //     1.0, 0.0, 0.0, 0.0,
        //     0.0, 1.0, 0.0, 0.0,
        //     0.0, 0.0, 0.5, 0.0,
        //     0.0, 0.0, 0.5, 1.0,
        // );
        // let view = Matrix4::look_at_rh(self.eye, self.target, Vector3::unit_y());
        let scale = Mat4::from_scale(Vec3::new(1.0, self.aspect, 1.0));
        // // let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        // let proj = cgmath::ortho(-1.0, 100.0, -1.0, 100.0, 0.1, 100.0);
        return Mat4::from_translation(Vec3::new(-1.0, -1.0, 0.0))
            * scale
            * Mat4::from_scale(Vec3::new(2.0 / 500.0, 2.0 / 500.0, 2.0 / 500.0))
            * Mat4::from_translation(Vec3::new(0.5, 0.5, 0.0));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct ExampleRenderer {
    render_context: Arc<Mutex<RenderContext>>,
    diffuse_bind_group: BindGroup,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl ExampleRenderer {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        let locked_context = render_context.lock().unwrap();
        let device = &locked_context.device;

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let diffuse_dimensions = diffuse_image.dimensions();
        let texture_size = Extent3d {
            width: diffuse_dimensions.0,
            height: diffuse_dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&TextureDescriptor {
            label: Some("happy-tree.png"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        locked_context.queue.write_texture(
            ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &diffuse_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * texture_size.width),
                rows_per_image: NonZeroU32::new(texture_size.height),
            },
            texture_size,
        );
        let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &diffuse_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_sampler),
                },
            ],
        });

        let camera = Camera {
            aspect: locked_context.config.width as f32 / locked_context.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_uniform = CameraUniform {
            view_proj: camera.build_proj_mat().into(),
        };
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(include_wgsl!("shader/example.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&diffuse_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
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
                topology: PrimitiveTopology::TriangleList,
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
            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] },
        ];
        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 2, 3,
        ];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let number = 500;
        let instances = (0..number)
            .map(|x| {
                (0..number).map(move |y| Instance {
                    position: Vector2::new(x as f32, y as f32),
                })
            })
            .flatten()
            .collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instances buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        drop(locked_context);

        Self {
            render_context,
            diffuse_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: INDICES.len() as u32,
            instances,
            instance_buffer,
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
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as u32);
        // self.camera.eye -= Vector3::new(0.0, 0.0, -0.01);
        // self.camera_uniform.view_proj = self.camera.build_proj_mat().into();
        // self.queue.write_buffer(
        //     &self.camera_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera_uniform]),
        // );

        Ok(())
    }
}
