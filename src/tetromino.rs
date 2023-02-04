use glam::{ivec2, vec4, IVec2, Vec4};
use rand::Rng;

/// A tetromino.
#[derive(Clone, Copy, Debug)]
pub enum Tetromino {
    I,
    J,
    L,
    O,
    T,
    Z,
    S,
}

impl Tetromino {
    const VARIANTS: [Tetromino; 7] = [
        Tetromino::I,
        Tetromino::J,
        Tetromino::L,
        Tetromino::O,
        Tetromino::T,
        Tetromino::Z,
        Tetromino::S,
    ];

    /// Returns a random tetromino.
    pub fn random() -> Self {
        Self::VARIANTS[rand::thread_rng().gen_range(0..Self::VARIANTS.len())]
    }

    /// Returns the color this tetromino.
    pub fn color(self) -> Vec4 {
        match self {
            Tetromino::I => vec4(0.2, 0.9, 0.9, 1.0),
            Tetromino::J => vec4(0.2, 0.2, 0.9, 1.0),
            Tetromino::L => vec4(0.9, 0.5, 0.2, 1.0),
            Tetromino::O => vec4(0.9, 0.9, 0.2, 1.0),
            Tetromino::T => vec4(0.9, 0.2, 0.9, 1.0),
            Tetromino::Z => vec4(0.9, 0.2, 0.2, 1.0),
            Tetromino::S => vec4(0.2, 0.9, 0.2, 1.0),
        }
    }

    /// Returns the positions of squares which represent this tetromino on a 4x4
    /// grid.
    // TODO Provide graphical example of tetromino representation.
    pub fn squares(self, rotation: u8) -> [IVec2; 4] {
        match (self, rotation % 4) {
            (Tetromino::I, 0 | 2) => [ivec2(0, 2), ivec2(1, 2), ivec2(2, 2), ivec2(3, 2)],
            (Tetromino::I, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(2, 3)],
            (Tetromino::O, _) => [ivec2(1, 1), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Tetromino::J, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Tetromino::J, 1) => [ivec2(2, 0), ivec2(2, 1), ivec2(1, 2), ivec2(2, 2)],
            (Tetromino::J, 2) => [ivec2(1, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Tetromino::J, 3) => [ivec2(2, 0), ivec2(3, 0), ivec2(2, 1), ivec2(2, 2)],
            (Tetromino::L, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(1, 2)],
            (Tetromino::L, 1) => [ivec2(1, 0), ivec2(2, 0), ivec2(2, 1), ivec2(2, 2)],
            (Tetromino::L, 2) => [ivec2(3, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Tetromino::L, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Tetromino::S, 0 | 2) => [ivec2(2, 1), ivec2(3, 1), ivec2(1, 2), ivec2(2, 2)],
            (Tetromino::S, 1 | 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(3, 2)],
            (Tetromino::Z, 0 | 2) => [ivec2(1, 1), ivec2(2, 1), ivec2(2, 2), ivec2(3, 2)],
            (Tetromino::Z, 1 | 3) => [ivec2(3, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Tetromino::T, 0) => [ivec2(1, 1), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            (Tetromino::T, 1) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(2, 2)],
            (Tetromino::T, 2) => [ivec2(2, 0), ivec2(1, 1), ivec2(2, 1), ivec2(3, 1)],
            (Tetromino::T, 3) => [ivec2(2, 0), ivec2(2, 1), ivec2(3, 1), ivec2(2, 2)],
            _ => unreachable!(),
        }
    }

    /// Returns the width of this tetromino as the number of squares.
    pub fn width(self, rotation: u8) -> u8 {
        match (self, rotation % 4) {
            (Tetromino::I, 0 | 2) => 4,
            (Tetromino::I, 1 | 3) => 1,
            (Tetromino::O, _) => 2,
            (_, 0 | 2) => 3,
            (_, 1 | 3) => 2,
            _ => unreachable!(),
        }
    }
}

/// A falling tetromino.
///
/// Unlike [`Tetromino`], [`FallingTetromino`] has a position and rotation.
#[derive(Clone, Copy, Debug)]
pub struct FallingTetromino {
    position: IVec2,
    rotation: u8,
    pub tetromino: Tetromino,
}

impl FallingTetromino {
    /// The starting position of a falling tetromino.
    const ORIGIN: IVec2 = ivec2(3, -1);

    /// Creates a new falling tetromino positioned at [`Self::ORIGIN`].
    pub fn new_at_origin(tetromino: Tetromino) -> Self {
        Self {
            position: Self::ORIGIN,
            rotation: 0,
            tetromino,
        }
    }

    /// Creates a new falling tetromino with a random shape positioned at
    /// [`Self::ORIGIN`].
    pub fn random_at_origin() -> Self {
        Self {
            position: ivec2(3, -1),
            rotation: 0,
            tetromino: Tetromino::random(),
        }
    }

    /// Returns the positions of squares representing this tetromino.
    pub fn squares(&self) -> [IVec2; 4] {
        let squares = self.tetromino.squares(self.rotation);
        [
            squares[0] + self.position,
            squares[1] + self.position,
            squares[2] + self.position,
            squares[3] + self.position,
        ]
    }

    /// Returns a new rotated instance of this tetromino.
    ///
    /// Rotation is specified in multiples of 90 deg.
    /// (1 = 90 deg. clockwise, -1 = 90 deg. counterclockwise).
    pub fn rotated(self, by: i8) -> FallingTetromino {
        FallingTetromino {
            rotation: self.rotation.wrapping_add_signed(by),
            ..self
        }
    }

    /// Returns a new instance of this tetromino moved by `by`.
    pub fn moved(self, by: IVec2) -> FallingTetromino {
        FallingTetromino {
            position: self.position + by,
            ..self
        }
    }
}
