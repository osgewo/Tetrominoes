use std::iter::repeat;

pub struct Grid<T> {
    // Row-major representation of the grid.
    raw: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    /// Creates a new `Grid` from a `Vec` of values in row-major order.
    ///
    /// # Panics
    ///
    /// The function will panic if the provided dimensions don't correspond with the length of the
    /// values `Vec`.
    ///
    /// # Example
    ///
    /// The following example creates a 3x4 grid of strings:
    /// ```
    /// let grid = Grid::from_row_major(vec![
    ///     "A1", "B1", "C1",
    ///     "A2", "B2", "C2",
    ///     "A3", "B3", "C3",
    ///     "A4", "B4", "C4",
    /// ], 3, 4);
    /// ```
    pub fn from_row_major(values: Vec<T>, width: usize, height: usize) -> Self {
        assert_eq!(
            values.len(),
            width * height,
            "all rows and columns in a grid must be the same length"
        );
        Self {
            raw: values,
            width,
            height,
        }
    }

    /// Creates a new `Grid` of size `width` by `height` filled with copies of `value`.
    pub fn filled_with(value: T, width: usize, height: usize) -> Self
    where
        T: Clone,
    {
        Self {
            raw: repeat(value).take(width * height).collect(),
            width,
            height,
        }
    }

    /// Returns the value at the specified position.
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if y >= self.height || x >= self.width {
            return None;
        }
        Some(&self.raw[self.index_of(x, y)])
    }

    /// Sets the value at the specified position.
    ///
    /// # Panics
    ///
    /// Panics if the specified `x` or `y` indexes are ouf of bounds.
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        if y >= self.height || x >= self.width {
            panic!("grid index out of bounds");
        }
        let index = self.index_of(x, y);
        self.raw[index] = value;
    }

    /// Returns all values in row-major order.
    pub fn as_row_major(&self) -> &[T] {
        &self.raw
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
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

        assert_eq!(grid.get(2, 1), Some(&"C2"));
        assert_eq!(grid.get(1, 4), None);
        assert_eq!(grid.get(5, 0), None);
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
