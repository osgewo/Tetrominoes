use crate::{
    grid::Grid,
    tetromino::{Shape, Tetromino},
};

/// Represents the game board. Mainly a wrapper around `Grid` with convenience
/// functions.
pub struct Board {
    pub grid: Grid<Option<Shape>>,
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

    /// Checks wheter a tetromino can fit onto the board.
    pub fn can_fit(&self, tetromino: Tetromino) -> bool {
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

    /// Places a tetromino onto the board.
    ///
    /// # Panics
    ///
    /// Panics if the tetromino is out of bounds of the board. (Except at the top.)
    pub fn place(&mut self, tetromino: Tetromino) {
        for square in tetromino.squares() {
            if square.y < 0 {
                continue;
            }
            self.grid
                .set(square.x as usize, square.y as usize, Some(tetromino.shape));
        }
        self.clear_complete();
    }

    /// Clears complete rows and shifts above rows down. Returns the number of rows
    /// cleared and whether they were contiguous.
    fn clear_complete(&mut self) -> (u8, bool) {
        // TODO Rewrite this function to use new convenience functions on `Grid`

        let mut rows_cleared = 0;
        let mut contiguous_rows_cleared = 0;

        for row in 0..self.grid.height() {
            // Check if the row is complete (all squares filled).
            let row_complete = (0..self.grid.width())
                .map(|i| self.grid.get(i, row).unwrap())
                .all(|sq| sq.is_some());
            if row_complete {
                rows_cleared += 1;
                contiguous_rows_cleared += 1;

                // Clear row.
                for x in 0..self.grid.width() {
                    self.grid.set(x, row, None);
                }

                // Shift above rows down, clear top row.
                for row in (0..=row).rev() {
                    for x in 0..self.grid.width() {
                        if row == 0 {
                            self.grid.set(x, row, None);
                        } else {
                            self.grid.set(x, row, *self.grid.get(x, row - 1).unwrap());
                        }
                    }
                }
            } else {
                contiguous_rows_cleared = 0;
            }
        }

        (rows_cleared, contiguous_rows_cleared == 4)
    }
}
