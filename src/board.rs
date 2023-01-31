use crate::{
    game::{Shape, Tetromino},
    grid::Grid,
};

pub struct Board {
    pub grid: Grid<Option<Shape>>,
}

impl Board {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 20;

    pub fn empty() -> Self {
        Self {
            grid: Grid::filled_with(None, Self::WIDTH, Self::HEIGHT),
        }
    }

    pub fn can_fit(&self, tetromino: Tetromino) -> bool {
        for square in tetromino.squares() {
            // Allow tetrominos to stick out the top of the board to enable immediate rotation.
            if square.y < 0 && square.x >= 0 && square.x < Self::WIDTH as i32 {
                continue;
            }
            let value = self.grid.get(square.x as usize, square.y as usize);
            if let None | Some(Some(_)) = value {
                return false;
            }
        }
        true
    }

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

    fn clear_complete(&mut self) {
        let mut contiguous_cleared = 0;
        for row in 0..self.grid.height() {
            let row_complete = (0..self.grid.width())
                .map(|i| self.grid.get(i, row).unwrap())
                .all(|sq| sq.is_some());
            if row_complete {
                contiguous_cleared += 1;

                for x in 0..self.grid.width() {
                    self.grid.set(x, row, None);
                }
                // Move down
                for row in (0..=row).rev() {
                    for x in 0..self.grid.width() {
                        if row == 0 {
                            self.grid.set(x, row, None);
                        } else {
                            self.grid.set(x, row, *self.grid.get(x, row - 1).unwrap());
                        }
                    }
                }
            } else if contiguous_cleared < 4 {
                contiguous_cleared = 0;
            }
        }

        if contiguous_cleared >= 4 {
            println!("TETRIS!");
        }
    }
}
