use std::sync::{Arc, Mutex};

use glam::{ivec2, IVec2, Mat4, Vec3};
use wgpu::{CommandEncoderDescriptor, SurfaceError, TextureViewDescriptor};
use winit::event::{ElementState, KeyboardInput};

use crate::{
    board::Board,
    render::{
        context::RenderContext,
        square::{self, SquareRenderer},
    },
    tetromino::Tetromino,
};

pub struct Game {
    render_context: Arc<Mutex<RenderContext>>,
    quad_renderer: SquareRenderer,
    board: Board,
    falling_tetromino: Tetromino,
    ticks_elapsed: usize,
}

impl Game {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        Self {
            render_context: render_context.clone(),
            quad_renderer: SquareRenderer::new(
                render_context.clone(),
                4 * (7 + Board::WIDTH * Board::HEIGHT) as u64,
            ),
            board: Board::empty(),
            falling_tetromino: Tetromino::random_at_origin(),
            ticks_elapsed: 0,
        }
    }

    pub fn keyboard_input(&mut self, input: KeyboardInput) {
        match (input.scancode, input.state) {
            // Rotate counterclockwise. [Q] / [Z] / [I]
            (16 | 44 | 23, ElementState::Pressed) => {
                self.try_rotate(-1);
            }
            // Rotate clockwise. [E] / [X] / [P]
            (18 | 45 | 25, ElementState::Pressed) => {
                self.try_rotate(1);
            }
            // Move left. [A] / [Left] / [K]
            (30 | 57419 | 37, ElementState::Pressed) => {
                self.try_move(ivec2(-1, 0));
            }
            // Move right. [D] / [Right] / [;]
            (32 | 57421 | 39, ElementState::Pressed) => {
                self.try_move(ivec2(1, 0));
            }
            // Move down. [S] / [Down] / [L]
            (31 | 57424 | 38, ElementState::Pressed) => {
                self.try_move(ivec2(0, 1));
            }
            // Drop. [Space]
            (57, ElementState::Pressed) => {
                self.drop();
            }
            // TODO Remove once everything else is finished.
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

    /// Rotates the falling tetromino if possible.
    fn try_rotate(&mut self, by: i8) {
        let rotated = self.falling_tetromino.rotated(by);
        if self.board.can_fit(rotated) {
            self.falling_tetromino = rotated;
        }
    }

    /// Moves the falling tetromino if possible.
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

    /// Drops the falling tetromino and place it immediately.
    fn drop(&mut self) {
        while self.try_move(ivec2(0, 1)) {}
        self.finalize();
    }

    /// Places the falling tetromino and spawns a new one.
    fn finalize(&mut self) {
        self.board.place(self.falling_tetromino);
        self.falling_tetromino = Tetromino::random_at_origin();
        // TODO Count score.
        // TODO Check for fit (game over).
    }

    /// Moves the falling tetromino down. If it can't be moved down it's placed.
    fn fall(&mut self) {
        if !self.try_move(ivec2(0, 1)) {
            self.finalize();
        }
    }

    /// Creates the projection matrix for a given surface size.
    fn build_proj_mat(width: u32, height: u32) -> Mat4 {
        return Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, 0.0, 1.0)
            * Mat4::from_scale(Vec3::new(30.0, 30.0, 1.0));
    }

    /// Renders the game.
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
            .map(|(pos, color)| square::Instance {
                position: pos.as_vec2(),
                color,
            })
            .collect::<Vec<_>>();
        self.quad_renderer.write_instances(queue, &instances);
        self.quad_renderer.render(
            &mut render_pass,
            queue,
            Self::build_proj_mat(render_context.config.width, render_context.config.height),
        )?;

        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}
