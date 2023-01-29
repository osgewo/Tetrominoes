use std::sync::{Arc, Mutex};

use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::{CommandEncoderDescriptor, SurfaceError, TextureViewDescriptor};

use crate::{
    grid::Grid,
    render::{
        context::RenderContext,
        line::LineRenderer,
        quad::{self, QuadRenderer},
    },
};

enum Tile {
    Blocked,
    Open,
}

impl Tile {
    fn color(&self) -> Vec4 {
        match self {
            Tile::Blocked => Vec4::new(1.0, 0.0, 0.0, 1.0),
            Tile::Open => Vec4::new(0.0, 1.0, 0.0, 1.0),
        }
    }
}

pub struct Game {
    render_context: Arc<Mutex<RenderContext>>,
    quad_renderer: QuadRenderer,
    line_renderer: LineRenderer,
    _grid: Grid<Tile>,
}

impl Game {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        #[rustfmt::skip]
        let grid = Grid::from_row_major(vec![
            Tile::Open,    Tile::Blocked, Tile::Open,    Tile::Open,
            Tile::Open,    Tile::Open,    Tile::Open,    Tile::Blocked,
            Tile::Blocked, Tile::Open,    Tile::Open,    Tile::Open,
            Tile::Open,    Tile::Open,    Tile::Blocked, Tile::Blocked,
        ], 4, 4);

        let instances = grid
            .as_row_major()
            .iter()
            .enumerate()
            .map(|(i, tile)| quad::Instance {
                position: Vec2::new((i % 4) as f32, (i / 4) as f32),
                color: tile.color(),
            })
            .collect::<Vec<_>>();

        Self {
            render_context: render_context.clone(),
            quad_renderer: QuadRenderer::new(render_context.clone(), &instances),
            line_renderer: LineRenderer::new(render_context),
            _grid: grid,
        }
    }

    fn build_proj_mat(render_context: &RenderContext) -> Mat4 {
        return Mat4::orthographic_lh(
            0.0,
            render_context.config.width as f32,
            render_context.config.height as f32,
            0.0,
            0.0,
            1.0,
        ) * Mat4::from_scale(Vec3::new(50.0, 50.0, 1.0));
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

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
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

        let queue = &render_context.queue;

        self.quad_renderer.render(
            &mut render_pass,
            queue,
            Self::build_proj_mat(&render_context),
        )?;
        self.line_renderer.render(&mut render_pass)?;

        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}
