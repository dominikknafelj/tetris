// Game constants
pub const GRID_SIZE: f32 = 60.0;      // Size of each grid cell in pixels (doubled from 30.0)
pub const GRID_WIDTH: i32 = 10;       // Width of the game board in cells
pub const GRID_HEIGHT: i32 = 20;      // Height of the game board in cells
pub const MARGIN: f32 = 40.0;         // Margin between game field and window borders (doubled from 20.0)
pub const BORDER_WIDTH: f32 = 4.0;    // Width of the game field border (doubled from 2.0)
pub const PREVIEW_BOX_SIZE: f32 = 6.0;  // Size of the preview box in grid cells
pub const SCREEN_WIDTH: f32 = GRID_SIZE * (GRID_WIDTH as f32 + PREVIEW_BOX_SIZE + 3.0) + 2.0 * MARGIN;   // Total screen width including preview and margins
pub const SCREEN_HEIGHT: f32 = GRID_SIZE * GRID_HEIGHT as f32 + 2.0 * MARGIN; // Total screen height including margins
pub const DROP_TIME: f64 = 1.0;       // Time in seconds between automatic piece movements
pub const PREVIEW_X: f32 = GRID_SIZE * (GRID_WIDTH as f32 + 3.0) + MARGIN; // X position of preview box, with extra spacing
pub const PREVIEW_Y: f32 = GRID_SIZE * 2.0 + MARGIN;  // Y position of preview box

// 8-bit aesthetic constants
#[allow(dead_code)]
pub const PIXEL_SIZE: f32 = 6.0;      // Size of a "pixel" in our 8-bit style
#[allow(dead_code)]
pub const BLOCK_PIXELS: i32 = 8;      // Number of "pixels" per tetris block (squared)
pub const GRID_LINE_WIDTH: f32 = 2.0; // Width of grid lines
pub const BLOCK_PADDING: f32 = 4.0;   // Padding inside blocks to create a pixelated effect

// Scoring constants
pub const SCORE_SINGLE: u32 = 100;    // Points for clearing 1 line
pub const SCORE_DOUBLE: u32 = 300;    // Points for clearing 2 lines
pub const SCORE_TRIPLE: u32 = 500;    // Points for clearing 3 lines
pub const SCORE_TETRIS: u32 = 800;    // Points for clearing 4 lines
pub const SCORE_DROP: u32 = 1;        // Points per cell for dropping a piece
pub const MAX_HIGH_SCORES: usize = 10; // Maximum number of high scores to store
pub const HIGH_SCORES_FILE: &str = "high_scores.json";

// Game timing constants
pub const BLINK_TIME: f64 = 0.5;      // Time in seconds for UI blinking effects
pub const CURSOR_BLINK_TIME: f64 = 0.3; // Time in seconds for text cursor blinking

// Drop speed calculation helpers
pub const BASE_DROP_TIME: f64 = 1.0;  // Base time between drops at level 1
pub const MIN_DROP_TIME: f64 = 0.05;  // Minimum time between drops at max level