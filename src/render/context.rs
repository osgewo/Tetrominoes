use wgpu::{
    util::StagingBelt, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureUsages,
};
use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};
use winit::{dpi::PhysicalSize, window::Window};

/// Groups together all wgpu objects neccessary for rendering.
pub struct RenderContext {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub staging_belt: StagingBelt,
    pub glyph_brush: GlyphBrush<()>,
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

        Self {
            surface,
            device,
            queue,
            config,
            staging_belt: StagingBelt::new(1024),
            glyph_brush,
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
}
