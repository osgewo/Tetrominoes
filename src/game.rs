use glam::{ivec2, vec2, vec4, IVec2, Vec2};
use wgpu::SurfaceError;
use wgpu_glyph::{HorizontalAlign, Layout, Section, Text};
use winit::event::{ElementState, KeyboardInput};

use crate::{
    board::Board,
    game_over::GameOver,
    render::{context::RenderContext, quad::Quad, square::TetrominoSquare},
    scene::{Action, Scene},
    tetromino::{FallingTetromino, Tetromino},
};

/// An in-progress game.
pub struct Game {
    board: Board,
    falling_tetromino: FallingTetromino,
    next_tetromino: Tetromino,
    ticks_elapsed: usize,
    score: u32,
    level: u32,
    rows_cleared: u32,
    lost: bool,
}

impl Game {
    /// Starts a new game starting at level 0.
    pub fn new() -> Self {
        Self {
            board: Board::empty(),
            falling_tetromino: FallingTetromino::random_at_origin(),
            next_tetromino: Tetromino::random(),
            ticks_elapsed: 0,
            score: 0,
            level: 0,
            rows_cleared: 0,
            lost: false,
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
        self.score += calc_score(rows_cleared);

        self.falling_tetromino = FallingTetromino::new_at_origin(self.next_tetromino);
        self.next_tetromino = Tetromino::random();

        if !self.board.can_fit(self.falling_tetromino) {
            self.lost = true;
        }
    }

    /// Moves the falling tetromino down one square. If it can't be moved down it's
    /// placed.
    fn fall(&mut self) {
        if !self.try_move(ivec2(0, 1)) {
            self.finalize();
        }
    }
}

impl Scene for Game {
    /// Handles keyboard input.
    fn keyboard_input(&mut self, input: KeyboardInput) -> Action {
        match (input.scancode, input.state) {
            // Exit [Esc] (temporary)
            (1, ElementState::Pressed) => {
                return Action::Exit;
            }
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
        Action::Continue
    }

    /// Updates the game logic. Should be called 60 times per second.
    fn tick(&mut self) -> Action {
        if self.lost {
            // TODO Use overlay instead.
            return Action::SwitchScene(Box::new(GameOver::new(self.score)));
        }

        self.ticks_elapsed += 1;
        if self.ticks_elapsed == 60 {
            self.ticks_elapsed = 0;
            self.fall();
        }
        Action::Continue
    }

    /// Renders the game.
    fn render(&mut self, ctx: &mut RenderContext) -> Result<(), SurfaceError> {
        self.board.render(ctx, vec2(20.0, 20.0));
        self.render_falling(ctx, vec2(25.0, 25.0));
        self.render_next(ctx, vec2(350.0, 20.0), vec2(210.0, 150.0));

        render_boxed_text(
            ctx,
            vec2(350.0, 190.0),
            vec2(210.0, 80.0),
            &format!("SCORE\n{}", self.score),
        );
        render_boxed_text(
            ctx,
            vec2(350.0, 290.0),
            vec2(210.0, 80.0),
            &format!("LEVEL\n{}", self.level),
        );
        render_boxed_text(
            ctx,
            vec2(350.0, 390.0),
            vec2(210.0, 80.0),
            &format!("LINES\n{}", self.rows_cleared),
        );

        ctx.render_frame()
    }
}

impl Game {
    /// Renders the falling tetromino.
    fn render_falling(&self, ctx: &mut RenderContext, offset: Vec2) {
        let squares = self.falling_tetromino.squares();
        let instances = squares
            .iter()
            .filter(|pos| pos.y >= 0)
            .map(|&pos| TetrominoSquare {
                position: offset + pos.as_vec2() * Vec2::splat(TetrominoSquare::SIZE),
                color: self.falling_tetromino.tetromino.color(),
            });
        ctx.square_renderer.submit_iter(instances);
    }

    /// Renders the next tetromino.
    fn render_next(&self, ctx: &mut RenderContext, position: Vec2, size: Vec2) {
        render_boxed_text(ctx, position, size, "NEXT");

        let center = vec2(position.x + size.x / 2.0, position.y + 30.0);

        // How many squares to offset the tetromino so that it's centered (-2.0 or -2.5)
        let offset = -((self.next_tetromino.width(0) % 2) as f32 * 0.5 + 2.0);

        let next_squares = self.next_tetromino.squares(0);
        let instances = next_squares.iter().map(|&pos| TetrominoSquare {
            position: center
                + (vec2(offset, 0.0) + pos.as_vec2()) * Vec2::splat(TetrominoSquare::SIZE),
            color: self.next_tetromino.color(),
        });
        ctx.square_renderer.submit_iter(instances);
    }
}

/// Renders an outline with text in the top-center.
fn render_boxed_text(ctx: &mut RenderContext, position: Vec2, size: Vec2, text: &str) {
    ctx.quad_renderer.submit(Quad {
        position,
        size,
        fill_color: vec4(0.0, 0.0, 0.0, 0.0),
        border_size: 5.0,
        border_color: vec4(0.8, 0.8, 0.8, 1.0),
    });
    ctx.glyph_brush.queue(Section {
        screen_position: (position.x + size.x / 2.0, position.y + 10.0),
        text: vec![Text::new(text)
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(30.0)],
        bounds: (f32::INFINITY, f32::INFINITY),
        layout: Layout::default_wrap().h_align(HorizontalAlign::Center),
    });
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
