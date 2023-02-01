use glam::{ivec2, vec4, IVec2, Vec4};
use rand::Rng;

/// Shape of tetromino.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    I,
    J,
    L,
    O,
    T,
    Z,
    S,
}

impl Shape {
    const VARIANTS: [Shape; 7] = [
        Shape::I,
        Shape::J,
        Shape::L,
        Shape::O,
        Shape::T,
        Shape::Z,
        Shape::S,
    ];

    /// Returns a random shape.
    pub fn random() -> Self {
        Self::VARIANTS[rand::thread_rng().gen_range(0..Self::VARIANTS.len())]
    }

    /// Returns the color a the shape.
    pub fn color(self) -> Vec4 {
        match self {
            Shape::I => vec4(0.2, 0.9, 0.9, 1.0),
            Shape::J => vec4(0.2, 0.2, 0.9, 1.0),
            Shape::L => vec4(0.9, 0.5, 0.2, 1.0),
            Shape::O => vec4(0.9, 0.9, 0.2, 1.0),
            Shape::T => vec4(0.9, 0.2, 0.9, 1.0),
            Shape::Z => vec4(0.9, 0.2, 0.2, 1.0),
            Shape::S => vec4(0.2, 0.9, 0.2, 1.0),
        }
    }

    /// Returns the positions of squares which represent this shape in a 4x4 grid.
    pub fn squares(self, rotation: u8) -> [IVec2; 4] {
        match (self, rotation % 4) {
            (Shape::I, 0 | 2) => [ivec2(0, 2), ivec2(1, 2), ivec2(2, 2), ivec2(3, 2)],
            (Shape::I, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(2, 3)],
            (Shape::O, _) => [ivec2(1, 1), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::J, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Shape::J, 1) => [ivec2(2, 0), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::J, 2) => [ivec2(1, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::J, 3) => [ivec2(2, 0), ivec2(3, 0), ivec2(2, 1), ivec2(2, 2)],
            (Shape::L, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(1, 2)],
            (Shape::L, 1) => [ivec2(1, 0), ivec2(2, 0), ivec2(2, 1), ivec2(2, 2)],
            (Shape::L, 2) => [ivec2(3, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::L, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Shape::S, 0 | 2) => [ivec2(2, 1), ivec2(3, 1), ivec2(1, 2), ivec2(2, 2)],
            (Shape::S, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Shape::Z, 0 | 2) => [ivec2(1, 1), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Shape::Z, 1 | 3) => [ivec2(3, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Shape::T, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Shape::T, 1) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(2, 2)],
            (Shape::T, 2) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Shape::T, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            _ => unreachable!(),
        }
    }

    /// Returns the width of this shape as the number of squares.
    pub fn width(self, rotation: u8) -> u8 {
        match (self, rotation % 4) {
            (Shape::I, 0 | 2) => 4,
            (Shape::I, 1 | 3) => 1,
            (Shape::O, _) => 2,
            (_, 0 | 2) => 3,
            (_, 1 | 3) => 2,
            _ => unreachable!(),
        }
    }

    /// Returns the height of this shape as the number of squares.
    pub fn height(self, rotation: u8) -> u8 {
        match (self, rotation % 4) {
            (Shape::I, 0 | 2) => 1,
            (Shape::I, 1 | 3) => 4,
            (Shape::O, _) => 2,
            (_, 0 | 2) => 2,
            (_, 1 | 3) => 3,
            _ => unreachable!(),
        }
    }
}

/// Tetromino in space (including rotation and position).
#[derive(Clone, Copy, Debug)]
pub struct Tetromino {
    position: IVec2,
    rotation: u8,
    pub shape: Shape,
}

impl Tetromino {
    const ORIGIN: IVec2 = ivec2(3, -1);

    /// Creates a new tetromino with a given shape positioned at origin (3, -1).
    pub fn new_at_origin(shape: Shape) -> Self {
        Self {
            position: Self::ORIGIN,
            rotation: 0,
            shape,
        }
    }

    /// Creates a new tetromino with a random shape positioned at origin (3, -1).
    pub fn random_at_origin() -> Self {
        Self {
            position: ivec2(3, -1),
            rotation: 0,
            shape: Shape::random(),
        }
    }

    /// Returns the positions of squares representing this tetromino.
    pub fn squares(&self) -> [IVec2; 4] {
        let mut squares = self.shape.squares(self.rotation);
        // Offset relative positions by the position of the tetromino.
        // TODO Rewrite this.
        for s in squares.iter_mut() {
            *s += self.position;
        }
        squares
    }

    /// Returns a new rotated instance of this tetromino.
    ///
    /// Rotation is specified in multiples of 45 deg.
    /// (1 = 45 deg. clockwise, -1 = 45 deg. counterclockwise).
    pub fn rotated(self, by: i8) -> Tetromino {
        Tetromino {
            rotation: self.rotation.wrapping_add_signed(by),
            ..self
        }
    }

    /// Returns a new instance of this tetromino moved by `by`.
    pub fn moved(self, by: IVec2) -> Tetromino {
        Tetromino {
            position: self.position + by,
            ..self
        }
    }

    /// Returns a new instance of this tetromino at the specified position.
    pub fn at(self, position: IVec2) -> Tetromino {
        Tetromino { position, ..self }
    }
}
