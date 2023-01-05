use std::sync::{Arc, Mutex};

use wgpu::{CommandEncoderDescriptor, SurfaceError, TextureViewDescriptor};

use crate::render::{context::RenderContext, example::ExampleRenderer, line::LineRenderer};

pub struct Game {
    render_context: Arc<Mutex<RenderContext>>,
    example_renderer: ExampleRenderer,
    line_renderer: LineRenderer,
}

impl Game {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        Self {
            render_context: render_context.clone(),
            example_renderer: ExampleRenderer::new(render_context.clone()),
            line_renderer: LineRenderer::new(render_context),
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let render_context = self.render_context.lock().unwrap();
        let output = render_context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = render_context
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.example_renderer.render(&mut encoder, &view)?;
        self.line_renderer.render(&mut encoder, &view)?;

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}
