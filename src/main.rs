use std::num::NonZeroU32;

use cgmath::{Matrix4, Point3, Vector3};
use image::GenericImageView;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Extent3d, Face,
    Features, FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat,
    Instance, Limits, LoadOp, MultisampleState, Operations, Origin3d, PipelineLayoutDescriptor,
    PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, SamplerDescriptor, ShaderStages, Surface,
    SurfaceConfiguration, SurfaceError, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => {
                state.resize(*new_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => match state.render() {
            Ok(_) => {}
            err => err.unwrap(),
        },
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once unless manually requested
            window.request_redraw();
        }
        _ => {}
    });
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_proj_mat(&self) -> Matrix4<f32> {
        #[rustfmt::skip]
        const OPENGL_TO_WGPU: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );
        let view = Matrix4::look_at_rh(self.eye, self.target, Vector3::unit_y());
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU * proj * view;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    diffuse_bind_group: BindGroup,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

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
        queue.write_texture(
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
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            aspect: config.width as f32 / config.height as f32,
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

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
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
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
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
            Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
            Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
            Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
            Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
            Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
        ];
        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
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

        Self {
            surface,
            device,
            queue,
            config,
            diffuse_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: INDICES.len() as u32,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.camera.eye -= Vector3::new(0.0, 0.0, -0.01);
        self.camera_uniform.view_proj = self.camera.build_proj_mat().into();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        Ok(())
    }
}
