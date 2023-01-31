use std::sync::{Arc, Mutex};

use glam::{ivec2, IVec2, Mat4, Vec3, Vec4};
use rand::Rng;
use wgpu::{CommandEncoderDescriptor, SurfaceError, TextureViewDescriptor};
use winit::event::{ElementState, KeyboardInput};

use crate::{
    board::Board,
    grid::Grid,
    render::{
        context::RenderContext,
        quad::{self, QuadRenderer},
    },
};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    I,
    J,
    L,
    O,
    T,
    Z,
    S,
}

impl Shape {
    const VARIANTS: [Shape; 7] = [
        Shape::I,
        Shape::J,
        Shape::L,
        Shape::O,
        Shape::T,
        Shape::Z,
        Shape::S,
    ];

    fn random() -> Self {
        Self::VARIANTS[rand::thread_rng().gen_range(0..Self::VARIANTS.len())]
    }

    fn color(&self) -> Vec4 {
        match self {
            Shape::I => Vec4::new(0.2, 0.9, 0.9, 1.0),
            Shape::J => Vec4::new(0.2, 0.2, 0.9, 1.0),
            Shape::L => Vec4::new(0.9, 0.5, 0.2, 1.0),
            Shape::O => Vec4::new(0.9, 0.9, 0.2, 1.0),
            Shape::T => Vec4::new(0.9, 0.2, 0.9, 1.0),
            Shape::Z => Vec4::new(0.9, 0.2, 0.2, 1.0),
            Shape::S => Vec4::new(0.2, 0.9, 0.2, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tetromino {
    position: IVec2,
    rotation: u8,
    pub shape: Shape,
}

impl Tetromino {
    pub fn random_at_origin() -> Self {
        Self {
            position: ivec2(3, -1),
            rotation: 0,
            shape: Shape::random(),
        }
    }

    pub fn squares(&self) -> [IVec2; 4] {
        let mut sqs = match (self.shape, self.rotation % 4) {
            (Shape::I, 0 | 2) => [ivec2(0, 2), ivec2(1, 2), ivec2(2, 2), ivec2(3, 2)],
            (Shape::I, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(2, 3)],
            (Shape::O, _) => [ivec2(1, 1), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::J, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Shape::J, 1) => [ivec2(2, 0), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::J, 2) => [ivec2(1, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::J, 3) => [ivec2(2, 0), ivec2(3, 0), ivec2(2, 1), ivec2(2, 2)],
            (Shape::L, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(1, 2)],
            (Shape::L, 1) => [ivec2(1, 0), ivec2(2, 0), ivec2(2, 1), ivec2(2, 2)],
            (Shape::L, 2) => [ivec2(3, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::L, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Shape::S, 0 | 2) => [ivec2(2, 1), ivec2(3, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::S, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Shape::Z, 0 | 2) => [ivec2(1, 1), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Shape::Z, 1 | 3) => [ivec2(3, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Shape::T, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Shape::T, 1) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(2, 2)],
            (Shape::T, 2) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::T, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            _ => unreachable!(),
        };
        for s in sqs.iter_mut() {
            *s += self.position;
        }
        sqs
    }

    fn rotated(&self, by: i8) -> Tetromino {
        Tetromino {
            rotation: self.rotation.wrapping_add_signed(by),
            ..*self
        }
    }

    fn moved(&self, by: IVec2) -> Tetromino {
        Tetromino {
            position: self.position + by,
            ..*self
        }
    }
}

pub struct Game {
    render_context: Arc<Mutex<RenderContext>>,
    quad_renderer: QuadRenderer,
    board: Board,
    falling_tetromino: Tetromino,
    ticks_elapsed: usize,
}

impl Game {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        let grid = Grid::filled_with(Some(Shape::J), 10, 20);

        Self {
            render_context: render_context.clone(),
            quad_renderer: QuadRenderer::new(
                render_context.clone(),
                4 * (7 + grid.width() * grid.height()) as u64,
            ),
            board: Board::empty(),
            falling_tetromino: Tetromino::random_at_origin(),
            ticks_elapsed: 0,
        }
    }

    pub fn keyboard_input(&mut self, input: KeyboardInput) {
        match (input.scancode, input.state) {
            (44, ElementState::Pressed) => {
                self.try_rotate(-1);
            }
            (45, ElementState::Pressed) => {
                self.try_rotate(1);
            }
            (37, ElementState::Pressed) => {
                self.try_move(ivec2(-1, 0));
            }
            (39, ElementState::Pressed) => {
                self.try_move(ivec2(1, 0));
            }
            (38, ElementState::Pressed) => {
                self.try_move(ivec2(0, 1));
            }
            (57, ElementState::Pressed) => {
                self.drop();
            }
            (scancode, ElementState::Pressed) => println!("{scancode}"),
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        self.ticks_elapsed += 1;
        if self.ticks_elapsed == 60 {
            self.ticks_elapsed = 0;
            self.fall();
        }
    }

    fn try_rotate(&mut self, by: i8) {
        let rotated = self.falling_tetromino.rotated(by);
        if self.board.can_fit(rotated) {
            self.falling_tetromino = rotated;
        }
    }

    fn try_move(&mut self, by: IVec2) -> bool {
        let moved = self.falling_tetromino.moved(by);
        if self.board.can_fit(moved) {
            self.falling_tetromino = moved;

            // Reset tick counter after successfully moving down.
            if by.y > 0 {
                self.ticks_elapsed = 0;
            }

            true
        } else {
            false
        }
    }

    fn drop(&mut self) {
        while self.try_move(ivec2(0, 1)) {}
        self.finalize();
    }

    fn finalize(&mut self) {
        self.board.place(self.falling_tetromino);

        self.falling_tetromino = Tetromino::random_at_origin();
        // TODO Check for fit (game over)
    }

    fn fall(&mut self) {
        if !self.try_move(ivec2(0, 1)) {
            self.finalize();
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
        ) * Mat4::from_scale(Vec3::new(30.0, 30.0, 1.0));
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
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
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let queue = &render_context.queue;

        let squares = self.falling_tetromino.squares();
        let falling = squares
            .iter()
            .map(|&sq| (sq, self.falling_tetromino.shape.color()));
        let grid = self
            .board
            .grid
            .as_row_major()
            .iter()
            .enumerate()
            .filter_map(|(i, sq)| {
                sq.as_ref().map(|sq| {
                    (
                        ivec2(
                            (i % self.board.grid.width()) as i32,
                            (i / self.board.grid.width()) as i32,
                        ),
                        sq.color(),
                    )
                })
            });
        let instances = grid
            .chain(falling)
            .map(|(pos, color)| quad::Instance {
                position: pos.as_vec2(),
                color,
            })
            .collect::<Vec<_>>();
        self.quad_renderer.write_instances(queue, &instances);
        self.quad_renderer.render(
            &mut render_pass,
            queue,
            Self::build_proj_mat(&render_context),
        )?;

        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}
