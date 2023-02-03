use glam::{vec2, vec4, Vec2};

use crate::{
    grid::Grid,
    render::{context::RenderContext, quad::Quad, square::TetrominoSquare},
    tetromino::{FallingTetromino, Tetromino},
};

/// Represents the game board. Mainly a wrapper around `Grid` with convenience
/// methods.
pub struct Board {
    grid: Grid<Option<Tetromino>>,
}

impl Board {
    pub const WIDTH: usize = 10;
    pub const HEIGHT: usize = 20;

    /// Creates a new empty board.
    pub fn empty() -> Self {
        Self {
            grid: Grid::filled_with(None, Self::WIDTH, Self::HEIGHT),
        }
    }

    /// Checks wheter a falling tetromino can fit onto the board.
    pub fn can_fit(&self, tetromino: FallingTetromino) -> bool {
        for square in tetromino.squares() {
            // Allow tetrominos to stick out the top of the board to enable immediate
            // rotation.
            // FIXME The current handling of pieces sticking out the top is not ideal.
            // If a piece is placed when sticking out the top, only part of it will get
            // placed. If the player then manages to complete a row, the row is cleared
            // and above rows shifted down, part of the piece will be missing.
            if square.y < 0 && square.x >= 0 && square.x < Self::WIDTH as i32 {
                continue;
            }
            let value = self.grid.get(square.x as usize, square.y as usize);
            // The piece does not fit if it would be outside the bounds of the grid or if
            // some of it's squares are already occupied.
            if let None | Some(Some(_)) = value {
                return false;
            }
        }
        true
    }

    /// Places a falling tetromino onto the board.
    ///
    /// # Panics
    ///
    /// Panics if the tetromino is out of bounds of the board (except at the top).
    pub fn place(&mut self, tetromino: FallingTetromino) {
        for square in tetromino.squares() {
            if square.y < 0 {
                continue;
            }
            self.grid.set(
                square.x as usize,
                square.y as usize,
                Some(tetromino.tetromino),
            );
        }
    }

    /// Clears complete rows and shifts above rows down. Returns the number of
    /// cleared rows.
    pub fn clear_complete(&mut self) -> u8 {
        let mut rows_cleared = 0;

        for y in 0..self.grid.height() {
            // Check if row is complete (all cells fillled).
            let row_complete = self.grid.row_slice(y).iter().all(|c| c.is_some());
            if row_complete {
                rows_cleared += 1;

                // Clear row.
                self.grid.row_slice_mut(y).fill(None);

                // Shift above rows down, clear top row.
                for y in (0..=y).rev() {
                    if y == 0 {
                        self.grid.row_slice_mut(y).fill(None);
                    } else {
                        for x in 0..self.grid.width() {
                            self.grid.set(x, y, *self.grid.get(x, y - 1).unwrap());
                        }
                    }
                }
            }
        }

        rows_cleared
    }

    /// Render the board.
    pub fn render(&self, ctx: &mut RenderContext, offset: Vec2) {
        ctx.quad_renderer.submit(Quad {
            position: offset,
            // TODO This should be calculated from border size and tetromino square size.
            size: vec2(310.0, 610.0),
            fill_color: vec4(0.0, 0.0, 0.0, 0.0),
            border_size: 5.0,
            border_color: vec4(0.8, 0.8, 0.8, 1.0),
        });

        let instances = self
            .grid
            .iter_with_indices()
            .filter_map(|(x, y, sq)| sq.map(|t| (x, y, t)))
            .map(|(x, y, t)| TetrominoSquare {
                position: offset
                    // TODO This should be calculated from border size
                    + Vec2::splat(5.0)
                    + vec2(x as f32, y as f32) * Vec2::splat(TetrominoSquare::SIZE),
                color: t.color(),
            });
        ctx.square_renderer.submit_iter(instances);
    }
}
