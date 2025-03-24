use ggez::graphics::Color;
use glam::Vec2;
use rand::Rng;

/// Represents the different types of Tetris pieces
/// Each variant corresponds to a standard Tetris piece shape
#[derive(Clone, Copy)]
pub enum TetrominoType {
    I, // Long piece
    O, // Square piece
    T, // T-shaped piece
    S, // S-shaped piece
    Z, // Z-shaped piece
    J, // J-shaped piece
    L, // L-shaped piece
}

/// Represents a Tetris piece with its shape, color, and position
/// The shape is stored as a 2D vector of booleans where true represents a filled cell
#[derive(Clone)]
pub struct Tetromino {
    pub shape: Vec<Vec<bool>>,  // 2D grid representing the piece's shape
    pub color: Color,           // Color of the piece
    pub position: Vec2,         // Current position on the game board
}

impl Tetromino {
    /// Creates a new Tetromino piece of the specified type
    /// Each piece type has its own predefined shape and color
    pub fn new(tetromino_type: TetrominoType) -> Self {
        let (shape, color) = match tetromino_type {
            TetrominoType::I => (
                vec![
                    vec![true, true, true, true],  // I piece is a single row of 4 blocks
                ],
                Color::CYAN,
            ),
            TetrominoType::O => (
                vec![
                    vec![true, true],              // O piece is a 2x2 square
                    vec![true, true],
                ],
                Color::YELLOW,
            ),
            TetrominoType::T => (
                vec![
                    vec![false, true, false],      // T piece has a T shape
                    vec![true, true, true],
                ],
                Color::MAGENTA,
            ),
            TetrominoType::S => (
                vec![
                    vec![false, true, true],       // S piece has an S shape
                    vec![true, true, false],
                ],
                Color::GREEN,
            ),
            TetrominoType::Z => (
                vec![
                    vec![true, true, false],       // Z piece has a Z shape
                    vec![false, true, true],
                ],
                Color::RED,
            ),
            TetrominoType::J => (
                vec![
                    vec![true, false, false],      // J piece has a J shape
                    vec![true, true, true],
                ],
                Color::BLUE,
            ),
            TetrominoType::L => (
                vec![
                    vec![false, false, true],      // L piece has an L shape
                    vec![true, true, true],
                ],
                Color::from_rgb(255, 165, 0), // Orange
            ),
        };

        Self {
            shape,
            color,
            position: Vec2::new(3.0, 0.0),  // Start position: middle top of the board
        }
    }

    /// Creates a random Tetromino piece
    /// Used for spawning new pieces during gameplay
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let types = [
            TetrominoType::I,
            TetrominoType::O,
            TetrominoType::T,
            TetrominoType::S,
            TetrominoType::Z,
            TetrominoType::J,
            TetrominoType::L,
        ];
        Self::new(types[rng.gen_range(0..types.len())])
    }

    /// Rotates the piece 90 degrees clockwise
    /// This is done by transposing the shape matrix and reversing each row
    pub fn rotate(&mut self) {
        let rows = self.shape.len();
        let cols = self.shape[0].len();
        let mut new_shape = vec![vec![false; rows]; cols];

        for y in 0..rows {
            for x in 0..cols {
                new_shape[x][rows - 1 - y] = self.shape[y][x];
            }
        }

        self.shape = new_shape;
    }

    /// Moves the piece one unit to the left
    pub fn move_left(&mut self) {
        self.position.x -= 1.0;
    }

    /// Moves the piece one unit to the right
    pub fn move_right(&mut self) {
        self.position.x += 1.0;
    }

    /// Moves the piece one unit down
    pub fn move_down(&mut self) {
        self.position.y += 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tetromino_creation() {
        // Test I piece
        let i_piece = Tetromino::new(TetrominoType::I);
        assert_eq!(i_piece.shape.len(), 1);
        assert_eq!(i_piece.shape[0].len(), 4);
        assert!(i_piece.shape[0].iter().all(|&cell| cell));
        assert_eq!(i_piece.color, Color::CYAN);

        // Test O piece
        let o_piece = Tetromino::new(TetrominoType::O);
        assert_eq!(o_piece.shape.len(), 2);
        assert_eq!(o_piece.shape[0].len(), 2);
        assert!(o_piece.shape.iter().all(|row| row.iter().all(|&cell| cell)));
        assert_eq!(o_piece.color, Color::YELLOW);
    }

    #[test]
    fn test_tetromino_movement() {
        let mut piece = Tetromino::new(TetrominoType::I);
        let initial_pos = piece.position;

        // Test left movement
        piece.move_left();
        assert_eq!(piece.position.x, initial_pos.x - 1.0);
        assert_eq!(piece.position.y, initial_pos.y);

        // Test right movement
        piece.move_right();
        assert_eq!(piece.position.x, initial_pos.x);
        assert_eq!(piece.position.y, initial_pos.y);

        // Test down movement
        piece.move_down();
        assert_eq!(piece.position.x, initial_pos.x);
        assert_eq!(piece.position.y, initial_pos.y + 1.0);
    }

    #[test]
    fn test_tetromino_rotation() {
        let mut piece = Tetromino::new(TetrominoType::I);
        let original_shape = piece.shape.clone();

        // Test rotation
        piece.rotate();
        assert_eq!(piece.shape.len(), 4);  // I piece becomes vertical
        assert_eq!(piece.shape[0].len(), 1);

        // Test multiple rotations
        piece.rotate();
        piece.rotate();
        piece.rotate();
        assert_eq!(piece.shape, original_shape);  // Should be back to original shape
    }

    #[test]
    fn test_random_tetromino() {
        // Test that random pieces are valid
        for _ in 0..100 {
            let piece = Tetromino::random();
            assert!(!piece.shape.is_empty());
            assert!(!piece.shape[0].is_empty());
            assert!(piece.shape.iter().all(|row| !row.is_empty()));
        }
    }
} 