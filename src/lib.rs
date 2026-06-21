pub mod tetromino;
pub mod constants;
pub mod sound;

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
pub use sound::GameSounds;

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

// For integration tests, re-export the specific types defined in tests_reexport
// instead of the crate's own types to avoid type mismatches
pub use tests_reexport::{GameScreen as TestGameScreen, HighScores as TestHighScores, GameState, keycode_to_char}; 