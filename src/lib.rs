pub mod tetromino;
pub mod sound_tests;
pub mod constants;
pub mod sound_manager;

pub mod test_event;

// Re-export modules
pub mod board;
pub mod score;
pub mod ui;

// Re-export the main types for convenience
pub use board::GameBoard;
pub use score::{HighScores, HighScoreEntry};
pub use tetromino::Tetromino;
pub use ui::GameRenderer;
pub use sound_manager::GameSounds;

// Export the game screen states
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameScreen {
    Title,
    Playing,
    GameOver,
    EnterName,
    HighScores,
}

// Export TetrominoType from tetromino module
pub use crate::tetromino::TetrominoType;

// Export TestState for tests
pub use crate::test_event::TestState;

// Re-export functionality from main.rs for testing
// Making this module public so it can be accessed from integration tests as well
pub mod tests_reexport;

// Re-export functionality for integration tests
// Making this available for all tests, not just unit tests
pub use tests_reexport::{keycode_to_char, GameState}; 