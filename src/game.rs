use std::sync::{Arc, Mutex};

use glam::{ivec2, vec2, vec4, IVec2, Mat4};
use wgpu::{CommandEncoderDescriptor, SurfaceError, TextureViewDescriptor};
use winit::event::{ElementState, KeyboardInput};

use crate::{
    board::Board,
    render::{
        context::RenderContext,
        quad::{self, QuadRenderer},
        square::{self, SquareRenderer},
    },
    tetromino::{Shape, Tetromino},
};

pub struct Game {
    render_context: Arc<Mutex<RenderContext>>,
    square_renderer: SquareRenderer,
    quad_renderer: QuadRenderer,
    board: Board,
    falling_tetromino: Tetromino,
    next_shape: Shape,
    ticks_elapsed: usize,
    score: u32,
    level: u32,
    rows_cleared: u32,
}

impl Game {
    pub fn new(render_context: Arc<Mutex<RenderContext>>) -> Self {
        let ctx = render_context.lock().unwrap();
        Self {
            render_context: render_context.clone(),
            square_renderer: SquareRenderer::new(
                &ctx,
                4 * (7 + Board::WIDTH * Board::HEIGHT) as u64,
            ),
            quad_renderer: QuadRenderer::new(&ctx, 100),
            board: Board::empty(),
            falling_tetromino: Tetromino::random_at_origin(),
            next_shape: Shape::random(),
            ticks_elapsed: 0,
            score: 0,
            level: 0,
            rows_cleared: 0,
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
        self.rows_cleared += rows_cleared as u32;
        self.score += Self::calc_score(rows_cleared);

        self.falling_tetromino = Tetromino::new_at_origin(self.next_shape);
        self.next_shape = Shape::random();
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
        Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, 0.0, 1.0)
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

        let proj_matrix = Self::build_proj_mat(ctx.config.width, ctx.config.height);

        self.quad_renderer.render(
            &mut render_pass,
            &ctx.queue,
            proj_matrix,
            &[
                quad::Instance {
                    position: vec2(20.0, 20.0),
                    size: vec2(310.0, 610.0),
                    fill_color: vec4(0.0, 0.0, 0.0, 0.0),
                    border_size: 5.0,
                    border_color: vec4(0.8, 0.8, 0.8, 1.0),
                },
                quad::Instance {
                    position: vec2(350.0, 20.0),
                    size: vec2(210.0, 150.0),
                    fill_color: vec4(0.0, 0.0, 0.0, 0.0),
                    border_size: 5.0,
                    border_color: vec4(0.8, 0.8, 0.8, 1.0),
                },
                quad::Instance {
                    position: vec2(350.0, 190.0),
                    size: vec2(210.0, 80.0),
                    fill_color: vec4(0.0, 0.0, 0.0, 0.0),
                    border_size: 5.0,
                    border_color: vec4(0.8, 0.8, 0.8, 1.0),
                },
                quad::Instance {
                    position: vec2(350.0, 290.0),
                    size: vec2(210.0, 80.0),
                    fill_color: vec4(0.0, 0.0, 0.0, 0.0),
                    border_size: 5.0,
                    border_color: vec4(0.8, 0.8, 0.8, 1.0),
                },
                quad::Instance {
                    position: vec2(350.0, 390.0),
                    size: vec2(210.0, 80.0),
                    fill_color: vec4(0.0, 0.0, 0.0, 0.0),
                    border_size: 5.0,
                    border_color: vec4(0.8, 0.8, 0.8, 1.0),
                },
            ],
        )?;

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
        let instances = grid.chain(falling).map(|(pos, color)| square::Instance {
            position: vec2(25.0, 25.0) + pos.as_vec2() * vec2(30.0, 30.0),
            color,
        });
        let next_squares = self.next_shape.squares(0);
        let next = next_squares.iter().map(|&sq| (sq, self.next_shape.color()));
        let next_instances = next.map(|(pos, color)| square::Instance {
            position: vec2(
                455.0 - (((self.next_shape.width(0) % 2) as f32 * 0.5 + 2.0) * 30.0),
                50.0,
            ) + pos.as_vec2() * vec2(30.0, 30.0),
            color,
        });
        let all_instances = instances.chain(next_instances).collect::<Vec<_>>();
        self.square_renderer
            .render(&mut render_pass, &ctx.queue, proj_matrix, &all_instances)?;

        drop(render_pass);

        // Text
        ctx.glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (455.0, 30.0),
            text: vec![wgpu_glyph::Text::new("NEXT")
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(30.0)],
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: wgpu_glyph::Layout::default_wrap().h_align(wgpu_glyph::HorizontalAlign::Center),
        });
        ctx.glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (455.0, 200.0),
            text: vec![wgpu_glyph::Text::new(&format!("SCORE\n{}", self.score))
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(30.0)],
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: wgpu_glyph::Layout::default_wrap().h_align(wgpu_glyph::HorizontalAlign::Center),
        });
        ctx.glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (455.0, 300.0),
            text: vec![wgpu_glyph::Text::new(&format!("LEVEL\n{}", self.level))
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(30.0)],
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: wgpu_glyph::Layout::default_wrap().h_align(wgpu_glyph::HorizontalAlign::Center),
        });
        ctx.glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (455.0, 400.0),
            text: vec![
                wgpu_glyph::Text::new(&format!("LINES\n{}", self.rows_cleared))
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(30.0),
            ],
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: wgpu_glyph::Layout::default_wrap().h_align(wgpu_glyph::HorizontalAlign::Center),
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
