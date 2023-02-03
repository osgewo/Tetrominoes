use std::{
    iter::{repeat, Enumerate},
    slice::Iter,
};

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
    /// Panics if the provided dimensions don't correspond with the length of the
    /// values `Vec`.
    ///
    /// # Examples
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

    /// Creates a new `Grid` of size `width` by `height` filled with copies of
    /// `value`.
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
    /// Panics if the specified `x` or `y` indices are ouf of bounds.
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

    /// Returns the height of the grid.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the width of the grid.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns a slice over row `y`.
    ///
    /// # Panics
    ///
    /// Panics if the specified index `y` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let grid = Grid::from_row_major(vec![
    ///     "A", "B",
    ///     "C", "D"
    /// ], 2, 2);
    /// assert_eq!(grid.row_slice(1), &["C", "D"]);
    /// ```
    pub fn row_slice(&self, y: usize) -> &[T] {
        if y > self.height - 1 {
            panic!("row index out of bounds");
        }
        &self.raw[y * self.width..(y + 1) * self.width]
    }

    /// Returns a mutable slice over row `y`.
    ///
    /// # Panics
    ///
    /// Panics if the specified index `y` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut grid = Grid::from_row_major(vec![
    ///     "A", "B",
    ///     "C", "D"
    /// ], 2, 2);
    /// grid.row_slice_mut(1)[0] = "X";
    /// assert_eq!(grid.row_slice(1), &["X", "D"]);
    /// ```
    pub fn row_slice_mut(&mut self, y: usize) -> &mut [T] {
        if y > self.height - 1 {
            panic!("row index out of bounds");
        }
        &mut self.raw[y * self.width..(y + 1) * self.width]
    }

    /// Returns an iterator over all values with `x` and `y` indices.
    ///
    /// # Examples
    ///
    /// ```
    /// let grid = Grid::from_row_major(vec!["A", "B", "C", "D"], 2, 2);
    /// let mut iter = grid.iter_with_indices();
    ///
    /// assert_eq!(iter.next(), Some((0, 0, &"A")));
    /// assert_eq!(iter.next(), Some((1, 0, &"B")));
    /// assert_eq!(iter.next(), Some((0, 1, &"C")));
    /// assert_eq!(iter.next(), Some((1, 1, &"D")));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_with_indices(&self) -> IterWithIndices<T> {
        IterWithIndices {
            grid: self,
            iter: self.raw.iter().enumerate(),
        }
    }

    /// Converts `x` and `y` indices into an index into the underlying `Vec`.
    fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

/// Iterator over all [`Grid`] values with `x` and `y` indices.
pub struct IterWithIndices<'a, T> {
    grid: &'a Grid<T>,
    iter: Enumerate<Iter<'a, T>>,
}

impl<'a, T> Iterator for IterWithIndices<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((i, value)) => Some((i % self.grid.width(), i / self.grid.width(), value)),
            None => None,
        }
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

    #[test]
    fn iter_with_indices() {
        let grid = Grid::from_row_major(vec!["A", "B", "C", "D"], 2, 2);
        let mut iter = grid.iter_with_indices();

        assert_eq!(iter.next(), Some((0, 0, &"A")));
        assert_eq!(iter.next(), Some((1, 0, &"B")));
        assert_eq!(iter.next(), Some((0, 1, &"C")));
        assert_eq!(iter.next(), Some((1, 1, &"D")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row_slice() {
        let mut grid = Grid::from_row_major(vec!["A", "B", "C", "D"], 2, 2);
        grid.row_slice_mut(1)[0] = "X";
        assert_eq!(grid.row_slice(1), &["X", "D"]);
    }

    #[test]
    #[should_panic]
    fn row_slice_panic() {
        let grid = Grid::from_row_major(vec!["A", "B", "C", "D"], 2, 2);
        // Should panic:
        grid.row_slice(2);
    }
}
