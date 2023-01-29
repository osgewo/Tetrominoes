pub struct Grid<T> {
    // Row-major representation of the grid.
    raw: Vec<T>,
    row_len: usize,
    col_len: usize,
}

impl<T> Grid<T> {
    /// Creates a new `Grid` from a `Vec` of values in row-major order.
    ///
    /// # Panics
    /// The function will panic if the provided dimensions don't correspond with the length of the
    /// values `Vec`.
    ///
    /// # Example
    /// The following example creates a 3x4 grid of strings:
    /// ```
    /// let grid = Grid::from_row_major(vec![
    ///     "A1", "B1", "C1",
    ///     "A2", "B2", "C2",
    ///     "A3", "B3", "C3",
    ///     "A4", "B4", "C4",
    /// ], 3, 4);
    /// ```
    pub fn from_row_major(values: Vec<T>, row_len: usize, col_len: usize) -> Self {
        assert_eq!(
            values.len(),
            row_len * col_len,
            "all rows and columns in a grid must be the same length"
        );
        Self {
            raw: values,
            row_len,
            col_len,
        }
    }

    /// Returns the value at the specified position.
    pub fn get(&self, row: usize, column: usize) -> Option<&T> {
        if row >= self.row_len || column >= self.col_len {
            return None;
        }
        Some(&self.raw[self.index_of(row, column)])
    }

    pub fn as_row_major(&self) -> &Vec<T> {
        &self.raw
    }

    pub fn row_len(&self) -> usize {
        self.row_len
    }

    pub fn col_len(&self) -> usize {
        self.col_len
    }

    fn index_of(&self, row: usize, column: usize) -> usize {
        row * self.row_len + column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid() {
        #[rustfmt::skip]
        let grid = Grid::from_row_major(vec![
            "A1", "B1", "C1",
            "A2", "B2", "C2",
            "A3", "B3", "C3",
            "A4", "B4", "C4",
        ], 3, 4);

        assert_eq!(grid.get(1, 2), Some(&"C2"));
        assert_eq!(grid.get(4, 1), None);
        assert_eq!(grid.get(0, 5), None);
    }

    #[test]
    #[should_panic]
    fn grid_bad() {
        // Should panic:
        #[rustfmt::skip]
        Grid::from_row_major(vec![
            "A1", "B1", "C1",
            "A2", "B2", "C2",
        ], 2, 2);
    }
}
