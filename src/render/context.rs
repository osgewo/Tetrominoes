use glam::Mat4;
use wgpu::{
    util::StagingBelt, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    SurfaceError, TextureUsages,
};
use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};
use winit::{dpi::PhysicalSize, window::Window};

use crate::board::Board;

use super::{quad::QuadRenderer, square::SquareRenderer};

/// Groups together all wgpu objects neccessary for rendering.
pub struct RenderContext {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub staging_belt: StagingBelt,
    pub glyph_brush: GlyphBrush<()>,
    pub square_renderer: SquareRenderer,
    pub quad_renderer: QuadRenderer,
}

impl RenderContext {
    /// Creates a new rendering context on the given window with default settings.
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(Backends::all());
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

        let font = FontArc::try_from_slice(include_bytes!("font/RobotoFlex-Regular.ttf")).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, config.format);

        let square_renderer = SquareRenderer::new(
            &device,
            &config,
            4 * (7 + Board::WIDTH * Board::HEIGHT) as u64,
        );
        let quad_renderer = QuadRenderer::new(&device, &config, 16);

        Self {
            surface,
            device,
            queue,
            config,
            staging_belt: StagingBelt::new(1024),
            glyph_brush,
            square_renderer,
            quad_renderer,
        }
    }

    /// Modifies the config and configures the surface for an updated window size.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Creates all resources neccessary to render a frame and calls the `render`
    /// methods of all renderers.
    pub fn render_frame(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.02,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let proj_matrix = self.build_proj_mat();

        self.square_renderer
            .render(&mut render_pass, &self.queue, proj_matrix)?;

        self.quad_renderer
            .render(&mut render_pass, &self.queue, proj_matrix)?;

        drop(render_pass);

        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                self.config.width,
                self.config.height,
            )
            .unwrap();

        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }

    /// Creates the projection matrix.
    fn build_proj_mat(&self) -> Mat4 {
        Mat4::orthographic_lh(
            0.0,
            self.config.width as f32,
            self.config.height as f32,
            0.0,
            0.0,
            1.0,
        )
    }
}
