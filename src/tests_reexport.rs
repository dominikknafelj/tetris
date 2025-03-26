// This module re-exports functionality from main.rs for use in tests
// It wraps the code from main.rs into small, testable units

use ggez::{Context, GameResult};
use ggez::graphics::Color;
use ggez::event;
use ggez::input::keyboard::KeyCode;
use glam::Vec2;

use crate::tetromino::{Tetromino, TetrominoType};

// Constants from main.rs
pub const GRID_WIDTH: i32 = 10;
pub const GRID_HEIGHT: i32 = 20;
pub const GRID_SIZE: f32 = 30.0;
pub const MARGIN: f32 = 30.0;
pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 600.0;
pub const PREVIEW_BOX_SIZE: f32 = 4.0;
pub const PREVIEW_X: f32 = GRID_SIZE * (GRID_WIDTH as f32 + 3.0) + MARGIN;
pub const PREVIEW_Y: f32 = GRID_SIZE * 2.0 + MARGIN;
pub const MAX_HIGH_SCORES: usize = 10;

// Re-export the keycode_to_char function from main
pub fn keycode_to_char(keycode: KeyCode, shift: bool) -> Option<char> {
    match keycode {
        KeyCode::A => Some(if shift { 'A' } else { 'a' }),
        KeyCode::B => Some(if shift { 'B' } else { 'b' }),
        KeyCode::C => Some(if shift { 'C' } else { 'c' }),
        KeyCode::D => Some(if shift { 'D' } else { 'd' }),
        KeyCode::E => Some(if shift { 'E' } else { 'e' }),
        KeyCode::F => Some(if shift { 'F' } else { 'f' }),
        KeyCode::G => Some(if shift { 'G' } else { 'g' }),
        KeyCode::H => Some(if shift { 'H' } else { 'h' }),
        KeyCode::I => Some(if shift { 'I' } else { 'i' }),
        KeyCode::J => Some(if shift { 'J' } else { 'j' }),
        KeyCode::K => Some(if shift { 'K' } else { 'k' }),
        KeyCode::L => Some(if shift { 'L' } else { 'l' }),
        KeyCode::M => Some(if shift { 'M' } else { 'm' }),
        KeyCode::N => Some(if shift { 'N' } else { 'n' }),
        KeyCode::O => Some(if shift { 'O' } else { 'o' }),
        KeyCode::P => Some(if shift { 'P' } else { 'p' }),
        KeyCode::Q => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::R => Some(if shift { 'R' } else { 'r' }),
        KeyCode::S => Some(if shift { 'S' } else { 's' }),
        KeyCode::T => Some(if shift { 'T' } else { 't' }),
        KeyCode::U => Some(if shift { 'U' } else { 'u' }),
        KeyCode::V => Some(if shift { 'V' } else { 'v' }),
        KeyCode::W => Some(if shift { 'W' } else { 'w' }),
        KeyCode::X => Some(if shift { 'X' } else { 'x' }),
        KeyCode::Y => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::Z => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Key0 => Some('0'),
        KeyCode::Key1 => Some('1'),
        KeyCode::Key2 => Some('2'),
        KeyCode::Key3 => Some('3'),
        KeyCode::Key4 => Some('4'),
        KeyCode::Key5 => Some('5'),
        KeyCode::Key6 => Some('6'),
        KeyCode::Key7 => Some('7'),
        KeyCode::Key8 => Some('8'),
        KeyCode::Key9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some('-'),
        KeyCode::Equals => Some('='),
        KeyCode::LBracket => Some('['),
        KeyCode::RBracket => Some(']'),
        KeyCode::Semicolon => Some(';'),
        KeyCode::Apostrophe => Some('\''),
        KeyCode::Comma => Some(','),
        KeyCode::Period => Some('.'),
        KeyCode::Slash => Some('/'),
        KeyCode::Backslash => Some('\\'),
        _ => None,
    }
}

// Simplified enum of game screens
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameScreen {
    Title,
    Playing,
    GameOver,
    EnterName,
    HighScores,
}

// Simplified high score entry
#[derive(Debug, Clone)]
pub struct HighScoreEntry {
    pub name: String,
    pub score: u32,
}

// Simplified high scores
#[derive(Debug, Clone)]
pub struct HighScores {
    pub entries: Vec<HighScoreEntry>,
}

impl HighScores {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_score(&mut self, name: String, score: u32) -> bool {
        // Add a score to the high scores list
        let entry = HighScoreEntry { name, score };
        
        // Check if entries is empty to avoid panic in the or condition
        if self.entries.is_empty() {
            self.entries.push(entry);
            return true;
        }
        
        // If list is not full, or score is better than the lowest score
        if self.entries.len() < MAX_HIGH_SCORES || score > self.entries.last().unwrap().score {
            self.entries.push(entry);
            self.entries.sort_by(|a, b| b.score.cmp(&a.score)); // Sort by descending score
            
            if self.entries.len() > MAX_HIGH_SCORES {
                self.entries.truncate(MAX_HIGH_SCORES);
            }
            
            true
        } else {
            false
        }
    }
    
    pub fn would_qualify(&self, score: u32) -> bool {
        // Check if a score would qualify for the high score list
        if self.entries.len() < MAX_HIGH_SCORES {
            return true;
        }
        
        // If we have a full list, check if score is better than the lowest
        if let Some(last) = self.entries.last() {
            score > last.score
        } else {
            true
        }
    }
}

// Simplified game sounds
#[derive(Debug)]
pub struct GameSounds {
    pub background_playing: bool,
}

impl GameSounds {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            background_playing: false,
        })
    }
    
    // Mock version for testing without a context
    pub fn new_mock() -> Self {
        Self {
            background_playing: false,
        }
    }
}

// GameState struct with minimal functionality for testing
#[derive(Debug)]
pub struct GameState {
    pub screen: GameScreen,
    pub board: Vec<Vec<Color>>,
    pub current_piece: Option<Tetromino>,
    pub next_piece: Tetromino,
    pub drop_timer: f64,
    pub sounds: GameSounds,
    pub blink_timer: f64,
    pub show_text: bool,
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub high_scores: HighScores,
    pub current_name: String,
    pub cursor_blink_timer: f64,
    pub show_cursor: bool,
    pub paused: bool,
}

impl GameState {
    // Create a new GameState with context (for integration tests that have a context)
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut board = vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        
        let current_piece = Some(Tetromino::random());
        let next_piece = Tetromino::random();
        
        let sounds = GameSounds::new(ctx)?;
        
        Ok(Self {
            screen: GameScreen::Playing,
            board,
            current_piece,
            next_piece,
            drop_timer: 0.0,
            sounds,
            blink_timer: 0.0,
            show_text: true,
            score: 0,
            level: 1,
            lines_cleared: 0,
            high_scores: HighScores::new(),
            current_name: String::new(),
            cursor_blink_timer: 0.0,
            show_cursor: true,
            paused: false,
        })
    }
    
    // Create a GameState without context (for unit tests)
    pub fn new_test() -> Self {
        let mut board = vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        
        let current_piece = Some(Tetromino::random());
        let next_piece = Tetromino::random();
        
        let sounds = GameSounds::new_mock();
        
        Self {
            screen: GameScreen::Playing,
            board,
            current_piece,
            next_piece,
            drop_timer: 0.0,
            sounds,
            blink_timer: 0.0,
            show_text: true,
            score: 0,
            level: 1,
            lines_cleared: 0,
            high_scores: HighScores::new(),
            current_name: String::new(),
            cursor_blink_timer: 0.0,
            show_cursor: true,
            paused: false,
        }
    }
    
    pub fn check_collision(&self, piece: &Tetromino) -> bool {
        let piece_width = piece.shape[0].len() as i32;
        let piece_height = piece.shape.len() as i32;
        let piece_x = piece.position.x.round() as i32;
        let piece_y = piece.position.y.round() as i32;
        
        for y in 0..piece_height {
            for x in 0..piece_width {
                if !piece.shape[y as usize][x as usize] {
                    continue; // Skip empty cells
                }
                
                let board_x = piece_x + x;
                let board_y = piece_y + y;
                
                // Check if out of bounds
                if board_x < 0 || board_x >= GRID_WIDTH || board_y >= GRID_HEIGHT {
                    return true;
                }
                
                // Check if collides with existing block
                if board_y >= 0 && board_y < GRID_HEIGHT && 
                   self.board[board_y as usize][board_x as usize] != Color::BLACK {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub fn clear_lines(&mut self, ctx: &mut Context) -> u32 {
        let mut lines_cleared = 0;
        let mut y = GRID_HEIGHT as usize - 1;
        
        while y >= 0 && y < GRID_HEIGHT as usize {
            let is_line_complete = self.board[y].iter().all(|&cell| cell != Color::BLACK);
            
            if is_line_complete {
                // Remove the completed line
                for y2 in (1..=y).rev() {
                    for x in 0..GRID_WIDTH as usize {
                        self.board[y2][x] = self.board[y2 - 1][x];
                    }
                }
                
                // Clear the top line
                for x in 0..GRID_WIDTH as usize {
                    self.board[0][x] = Color::BLACK;
                }
                
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }
        
        if lines_cleared > 0 {
            self.update_score(lines_cleared);
            self.lines_cleared += lines_cleared;
            
            // Update level based on lines cleared
            self.level = (self.lines_cleared / 10) + 1;
        }
        
        lines_cleared
    }
    
    // Version for unit tests that doesn't require a context
    pub fn clear_lines_test(&mut self) -> u32 {
        let mut lines_cleared = 0;
        
        // Process lines from bottom to top
        for row in (0..GRID_HEIGHT as usize).rev() {
            // Check if the current row is complete
            let is_line_complete = self.board[row].iter().all(|&cell| cell != Color::BLACK);
            
            if is_line_complete {
                println!("Found complete line at row {}", row);
                
                // Shift all rows above this one down by one
                for y in (1..=row).rev() {
                    println!("Shifting row {} down", y-1);
                    for x in 0..GRID_WIDTH as usize {
                        self.board[y][x] = self.board[y - 1][x];
                    }
                }
                
                // Clear the top row
                println!("Clearing top row");
                for x in 0..GRID_WIDTH as usize {
                    self.board[0][x] = Color::BLACK;
                }
                
                lines_cleared += 1;
            }
        }
        
        if lines_cleared > 0 {
            self.update_score(lines_cleared);
            self.lines_cleared += lines_cleared;
            
            // Update level based on lines cleared
            self.level = (self.lines_cleared / 10) + 1;
        }
        
        lines_cleared
    }
    
    pub fn update_score(&mut self, lines: u32) {
        // Standard scoring system:
        // 1 line = 40 * level
        // 2 lines = 100 * level
        // 3 lines = 300 * level
        // 4 lines = 1200 * level
        let points = match lines {
            1 => 40 * self.level,
            2 => 100 * self.level,
            3 => 300 * self.level,
            4 => 1200 * self.level,
            _ => 0,
        };
        
        self.score += points;
    }
    
    pub fn drop_speed(&self) -> f64 {
        // Each level increases speed
        // Formula: base_interval / (1 + level_factor * (level - 1))
        let base_interval = 1.0;
        let level_factor = 0.1;
        
        base_interval / (1.0 + level_factor * (self.level - 1) as f64)
    }
    
    // Check if the current score would qualify for the high score list
    pub fn check_high_score(&self) -> bool {
        self.high_scores.would_qualify(self.score)
    }
    
    // Add the current score to the high scores
    pub fn add_high_score(&mut self) -> bool {
        self.high_scores.add_score(self.current_name.clone(), self.score)
    }
} 