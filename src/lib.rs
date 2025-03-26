pub mod tetromino;
pub mod sound_tests;
pub mod test_event;

// Export main types from tetromino module
pub use crate::tetromino::{Tetromino, TetrominoType};

// Export TestState for tests
pub use crate::test_event::TestState;

// Re-export functionality from main.rs for testing
mod tests_reexport;

// Re-export functionality for integration tests
pub use tests_reexport::{keycode_to_char, GameState, GameScreen, HighScores, HighScoreEntry}; 