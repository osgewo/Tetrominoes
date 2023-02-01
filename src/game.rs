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
    score: u32,
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
            score: 0,
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

    /// Drops the falling tetromino and places it immediately.
    fn drop(&mut self) {
        while self.try_move(ivec2(0, 1)) {}
        self.finalize();
    }

    /// Places the falling tetromino and spawns a new one.
    fn finalize(&mut self) {
        self.board.place(self.falling_tetromino);
        let rows_cleared = self.board.clear_complete();
        self.score += Self::calc_score(rows_cleared);
        println!("Score: {}", self.score);

        self.falling_tetromino = Tetromino::random_at_origin();
        if !self.board.can_fit(self.falling_tetromino) {
            // TODO Game over screen.
            println!("Game over! Score: {}", self.score);
        }
    }

    /// Moves the falling tetromino down. If it can't be moved down it's placed.
    fn fall(&mut self) {
        if !self.try_move(ivec2(0, 1)) {
            self.finalize();
        }
    }

    /// Calculates the score for a given number of cleared rows.
    ///
    /// # Panics
    ///
    /// Panics if the number of cleared rows is greater than 4.
    fn calc_score(rows_cleared: u8) -> u32 {
        match rows_cleared {
            0 => 0,
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => panic!("it should not be possible to clear more than 4 rows at once"),
        }
    }

    /// Creates the projection matrix for a given surface size.
    fn build_proj_mat(width: u32, height: u32) -> Mat4 {
        return Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, 0.0, 1.0)
            * Mat4::from_scale(Vec3::new(30.0, 30.0, 1.0));
    }

    /// Renders the game.
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let ctx = &mut *self.render_context.lock().unwrap();
        let output = ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = ctx
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
        self.quad_renderer.render(
            &mut render_pass,
            &ctx.queue,
            Self::build_proj_mat(ctx.config.width, ctx.config.height),
            &instances,
        )?;

        drop(render_pass);

        // Text
        ctx.glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (350.0, 30.0),
            text: vec![wgpu_glyph::Text::new("Next")
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(40.0)],
            bounds: (ctx.config.width as f32, ctx.config.height as f32),
            ..Default::default()
        });
        ctx.glyph_brush
            .draw_queued(
                &ctx.device,
                &mut ctx.staging_belt,
                &mut encoder,
                &view,
                ctx.config.width,
                ctx.config.height,
            )
            .unwrap();

        ctx.staging_belt.finish();
        ctx.queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }
}
