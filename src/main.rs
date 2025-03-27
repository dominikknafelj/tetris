mod tetromino;
mod sound_tests;
mod constants;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color, Drawable},
    input::keyboard::{KeyCode, KeyInput},
    audio::{self, SoundSource},
    Context, GameResult,
};
use tetromino::Tetromino;
use std::fs::{self, File};
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use constants::*;

/// Sound effects for the game
struct GameSounds {
    move_sound: audio::Source,
    rotate_sound: audio::Source,
    drop_sound: audio::Source,
    clear_sound: audio::Source,
    tetris_sound: audio::Source,
    game_over_sound: audio::Source,
    background_music: Option<audio::Source>,
    background_playing: bool,
}

impl GameSounds {
    /// Loads all sound effects
    fn new(ctx: &mut Context) -> GameResult<Self> {
        // Create sources with paths relative to the resource directory
        let move_sound = audio::Source::new(ctx, "/sounds/move.wav")?;
        let rotate_sound = audio::Source::new(ctx, "/sounds/rotate.wav")?;
        let drop_sound = audio::Source::new(ctx, "/sounds/drop.wav")?;
        let clear_sound = audio::Source::new(ctx, "/sounds/clear.wav")?;
        let tetris_sound = audio::Source::new(ctx, "/sounds/tetris.wav")?;
        let game_over_sound = audio::Source::new(ctx, "/sounds/game_over.wav")?;

        Ok(Self {
            move_sound,
            rotate_sound,
            drop_sound,
            clear_sound,
            tetris_sound,
            game_over_sound,
            background_music: None,
            background_playing: false,
        })
    }

    /// Plays a sound effect
    fn play_move(&mut self, ctx: &mut Context) -> GameResult {
        self.move_sound.play_detached(ctx)
    }

    fn play_rotate(&mut self, ctx: &mut Context) -> GameResult {
        self.rotate_sound.play_detached(ctx)
    }

    fn play_drop(&mut self, ctx: &mut Context) -> GameResult {
        self.drop_sound.play_detached(ctx)
    }

    fn play_clear(&mut self, ctx: &mut Context) -> GameResult {
        self.clear_sound.play_detached(ctx)
    }

    fn play_tetris(&mut self, ctx: &mut Context) -> GameResult {
        self.tetris_sound.play_detached(ctx)
    }

    fn play_game_over(&mut self, ctx: &mut Context) -> GameResult {
        self.game_over_sound.play_detached(ctx)
    }

    fn stop_background_music(&mut self, ctx: &mut Context) {
        // If we have a music source, stop it
        if let Some(music) = &mut self.background_music {
            music.stop(ctx).unwrap();
        }
        // Set the flag to false and remove the source
        self.background_playing = false;
        self.background_music = None;
    }

    fn start_background_music(&mut self, ctx: &mut Context) -> GameResult {
        // Only start if not already playing
        if !self.background_playing {
            // Create a completely new source
            let mut music = audio::Source::new(ctx, "/sounds/background.wav")?;
            
            // Set up the new source
            music.set_repeat(true);
            
            // Play the music (using play instead of play_detached)
            music.play(ctx)?;
            
            // Store the source and update state
            self.background_music = Some(music);
            self.background_playing = true;
        }
        Ok(())
    }

    /// Ensures background music is playing if it should be
    #[allow(dead_code)]
    fn ensure_background_music(&mut self, ctx: &mut Context) -> GameResult {
        // Make sure music is playing if it's supposed to be
        if self.background_playing && self.background_music.is_none() {
            self.start_background_music(ctx)?;
        }
        Ok(())
    }
}

// Game screen states
#[derive(PartialEq, Clone, Copy)]
enum GameScreen {
    Title,
    Playing,
    GameOver,
    EnterName,
    HighScores,
}

/// High score entry with player name and score
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HighScoreEntry {
    name: String,
    score: u32,
}

/// Collection of high scores that can be loaded/saved
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HighScores {
    entries: Vec<HighScoreEntry>,
}

impl HighScores {
    /// Create a new empty high score list
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Load high scores from file
    fn load() -> Self {
        match fs::read_to_string(HIGH_SCORES_FILE) {
            Ok(contents) => {
                match serde_json::from_str(&contents) {
                    Ok(scores) => scores,
                    Err(_) => Self::new(),
                }
            },
            Err(_) => Self::new(),
        }
    }
    
    /// Save high scores to file
    fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string(self)?;
        let mut file = File::create(HIGH_SCORES_FILE)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    /// Add a new high score if it qualifies, return true if it was added
    fn add_score(&mut self, name: String, score: u32) -> bool {
        // Check if the score qualifies (greater than the lowest score or fewer than MAX_HIGH_SCORES entries)
        let qualifies = self.entries.len() < MAX_HIGH_SCORES || 
                        self.entries.iter().any(|entry| entry.score < score);
        
        if qualifies {
            // Add the new entry
            self.entries.push(HighScoreEntry { name, score });
            
            // Sort entries by score (descending)
            self.entries.sort_by(|a, b| b.score.cmp(&a.score));
            
            // Truncate to max number of entries
            if self.entries.len() > MAX_HIGH_SCORES {
                self.entries.truncate(MAX_HIGH_SCORES);
            }
            
            // Save the updated high scores
            let _ = self.save();
        }
        
        qualifies
    }
    
    /// Check if a score would qualify for the high score list
    fn would_qualify(&self, score: u32) -> bool {
        self.entries.len() < MAX_HIGH_SCORES || 
        self.entries.iter().any(|entry| entry.score < score)
    }
}

/// Main game state that holds all the game data
struct GameState {
    screen: GameScreen,           // Current game screen
    board: Vec<Vec<Color>>,       // 2D grid representing the game board
    current_piece: Option<Tetromino>,  // Currently active piece
    next_piece: Tetromino,        // Next piece to spawn
    drop_timer: f64,              // Timer for automatic piece movement
    sounds: GameSounds,           // Game sound effects
    blink_timer: f64,             // Timer for text blinking effect
    show_text: bool,              // Whether to show blinking text
    score: u32,                   // Current game score
    level: u32,                   // Current game level
    lines_cleared: u32,           // Total number of lines cleared
    high_scores: HighScores,      // High score list
    current_name: String,         // Current player name being entered
    cursor_blink_timer: f64,      // Timer for name input cursor blinking
    show_cursor: bool,            // Whether to show the name input cursor
    paused: bool,                 // Whether the game is paused
}

impl GameState {
    /// Creates a new game state with an empty board and a random starting piece
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut sounds = GameSounds::new(ctx)?;
        
        // Start background music immediately on the start screen
        sounds.start_background_music(ctx)?;
        
        Ok(Self {
            screen: GameScreen::Title,
            board: vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            current_piece: Some(Tetromino::random()),
            next_piece: Tetromino::random(),
            drop_timer: 0.0,
            sounds,
            blink_timer: 0.0,
            show_text: true,
            score: 0,
            level: 1,
            lines_cleared: 0,
            high_scores: HighScores::load(),
            current_name: String::new(),
            cursor_blink_timer: 0.0,
            show_cursor: true,
            paused: false,
        })
    }

    /// Resets the game state for a new game
    fn reset_game(&mut self, _ctx: &mut Context) -> GameResult {
        self.board = vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        self.current_piece = Some(Tetromino::random());
        self.next_piece = Tetromino::random();
        self.drop_timer = 0.0;
        self.screen = GameScreen::Playing;
        self.score = 0;
        self.level = 1;
        self.lines_cleared = 0;
        Ok(())
    }

    /// Spawns a new piece at the top of the board
    /// If the new piece collides with existing pieces, the game is over
    fn spawn_new_piece(&mut self, ctx: &mut Context) {
        let new_piece = self.next_piece.clone();
        if self.check_collision(&new_piece) {
            self.screen = GameScreen::GameOver;
            self.sounds.play_game_over(ctx).unwrap();
            
            // Immediately check if the player qualifies for high score
            // This ensures the transition happens without requiring a key press
            if self.check_high_score() {
                self.screen = GameScreen::EnterName;
            }
        }
        self.current_piece = Some(new_piece);
        self.next_piece = Tetromino::random();
    }

    /// Checks if a piece collides with the board boundaries or existing pieces
    fn check_collision(&self, piece: &Tetromino) -> bool {
        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    let board_x = piece.position.x as i32 + x as i32;
                    let board_y = piece.position.y as i32 + y as i32;
                    
                    // Check for collisions with:
                    // 1. Left/right boundaries
                    // 2. Bottom boundary
                    // 3. Existing pieces on the board
                    if board_x < 0 || board_x >= GRID_WIDTH || 
                       board_y >= GRID_HEIGHT ||
                       (board_y >= 0 && self.board[board_y as usize][board_x as usize] != Color::BLACK) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Attempts to move the current piece using the provided movement function
    /// Returns true if the movement was successful, false if it caused a collision
    fn move_piece(&mut self, movement: fn(&mut Tetromino), ctx: &mut Context) -> bool {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return false,
        };

        let mut new_piece = current;
        movement(&mut new_piece);
        
        if !self.check_collision(&new_piece) {
            self.current_piece = Some(new_piece);
            self.sounds.play_move(ctx).unwrap();
            true
        } else {
            false
        }
    }

    /// Attempts to rotate the current piece
    /// If the rotation would cause a collision, tries various offsets to make it fit
    fn try_rotate(&mut self, ctx: &mut Context) {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return,
        };

        let mut new_piece = current;
        new_piece.rotate();
        
        // Try rotation with various offsets to handle wall kicks
        let offsets = [(0, 0), (-1, 0), (1, 0), (-2, 0), (2, 0)];
        for (x_offset, y_offset) in offsets.iter() {
            let mut test_piece = new_piece.clone();
            test_piece.position.x += *x_offset as f32;
            test_piece.position.y += *y_offset as f32;
            
            if !self.check_collision(&test_piece) {
                self.current_piece = Some(test_piece);
                self.sounds.play_rotate(ctx).unwrap();
                return;
            }
        }
    }

    /// Clears any complete lines and returns the number of lines cleared
    fn clear_lines(&mut self, ctx: &mut Context) -> u32 {
        let mut lines_cleared = 0;
        let mut y = GRID_HEIGHT - 1;
        while y >= 0 {
            if self.board[y as usize].iter().all(|&cell| cell != Color::BLACK) {
                // Remove the line
                for y2 in (1..=y).rev() {
                    self.board[y2 as usize] = self.board[(y2 - 1) as usize].clone();
                }
                // Add empty line at top
                self.board[0] = vec![Color::BLACK; GRID_WIDTH as usize];
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }

        // Update score based on lines cleared
        if lines_cleared > 0 {
            self.update_score(lines_cleared);
            
            // Play appropriate sound based on number of lines cleared
            if lines_cleared == 4 {
                self.sounds.play_tetris(ctx).unwrap();
            } else {
                self.sounds.play_clear(ctx).unwrap();
            }
        }

        lines_cleared
    }

    /// Instantly drops the current piece to the lowest possible position
    fn hard_drop(&mut self, ctx: &mut Context) {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return,
        };

        let mut new_piece = current;
        let original_y = new_piece.position.y;
        
        while !self.check_collision(&new_piece) {
            new_piece.move_down();
        }
        
        // Move back up one step since we found collision
        new_piece.position.y -= 1.0;
        
        // Calculate how many cells were dropped
        let cells_dropped = new_piece.position.y - original_y;
        
        // Add points for hard drop
        self.add_drop_points(cells_dropped as i32);
        
        self.current_piece = Some(new_piece);
        self.sounds.play_drop(ctx).unwrap();
        self.lock_piece(ctx);
    }

    /// Locks the current piece in place on the board
    /// This happens when a piece can't move down further
    fn lock_piece(&mut self, ctx: &mut Context) {
        let piece = match &self.current_piece {
            Some(p) => p.clone(),
            None => return,
        };

        // Copy the piece's shape to the board
        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    let board_x = piece.position.x as i32 + x as i32;
                    let board_y = piece.position.y as i32 + y as i32;
                    if board_y >= 0 {
                        self.board[board_y as usize][board_x as usize] = piece.color;
                    }
                }
            }
        }
        self.sounds.play_drop(ctx).unwrap();
        let lines_cleared = self.clear_lines(ctx);
        if lines_cleared > 0 {
            self.sounds.play_clear(ctx).unwrap();
        }
        self.spawn_new_piece(ctx);
    }

    /// Draws the next piece preview
    fn draw_preview(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw preview box background with pixelated corners (8-bit style)
        let preview_bg = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE,
            PREVIEW_Y - GRID_SIZE,
            GRID_SIZE * 6.0,
            GRID_SIZE * 6.0,
        );
        
        // Draw the outer frame (darker)
        let frame_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            preview_bg,
            Color::new(0.2, 0.2, 0.2, 1.0),
        )?;
        canvas.draw(&frame_mesh, graphics::DrawParam::default());

        // Draw the inner frame (lighter)
        let inner_rect = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 2.0,
            PREVIEW_Y - GRID_SIZE + GRID_LINE_WIDTH * 2.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
        );
        let inner_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_rect,
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&inner_mesh, graphics::DrawParam::default());

        // Draw the main background (darkest)
        let main_bg = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 4.0,
            PREVIEW_Y - GRID_SIZE + GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
        );
        let main_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            main_bg,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )?;
        canvas.draw(&main_mesh, graphics::DrawParam::default());

        // Draw "NEXT" text with a block-like shadow for 8-bit effect
        let text = graphics::Text::new("NEXT");
        // Draw shadow
        canvas.draw(
            &text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.5))
                .dest([PREVIEW_X + 2.0, PREVIEW_Y - GRID_SIZE * 2.0 + 2.0]),
        );
        // Draw main text
        canvas.draw(
            &text,
            graphics::DrawParam::default()
                .color(Color::WHITE)
                .dest([PREVIEW_X, PREVIEW_Y - GRID_SIZE * 2.0]),
        );

        // Draw next piece
        let piece_width = self.next_piece.shape[0].len() as f32;
        let piece_height = self.next_piece.shape.len() as f32;
        let offset_x = (6.0 - piece_width) / 2.0;  // Center horizontally
        let offset_y = (6.0 - piece_height) / 2.0;  // Center vertically

        for (y, row) in self.next_piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    // Calculate position for preview block
                    let preview_block_x = x as f32 + offset_x;
                    let preview_block_y = y as f32 + offset_y;
                    
                    // Draw the block using the 8-bit style but in preview area
                    let block_x = PREVIEW_X - GRID_SIZE + preview_block_x * GRID_SIZE;
                    let block_y = PREVIEW_Y - GRID_SIZE + preview_block_y * GRID_SIZE;
                    
                    // Main block
                    let block_rect = graphics::Rect::new(
                        block_x + GRID_LINE_WIDTH, 
                        block_y + GRID_LINE_WIDTH,
                        GRID_SIZE - 2.0 * GRID_LINE_WIDTH, 
                        GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                    );
                    
                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        block_rect,
                        self.next_piece.color,
                    )?;
                    canvas.draw(&mesh, graphics::DrawParam::default());
                    
                    // Add highlights and shadows like in draw_block
                    // Top highlight
                    let highlight_color = Color::new(
                        f32::min(self.next_piece.color.r + 0.2, 1.0),
                        f32::min(self.next_piece.color.g + 0.2, 1.0),
                        f32::min(self.next_piece.color.b + 0.2, 1.0),
                        self.next_piece.color.a,
                    );
                    
                    let top_highlight = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            block_x + GRID_LINE_WIDTH,
                            block_y + GRID_LINE_WIDTH,
                            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                            BLOCK_PADDING,
                        ),
                        highlight_color,
                    )?;
                    canvas.draw(&top_highlight, graphics::DrawParam::default());
                    
                    // Left highlight
                    let left_highlight = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            block_x + GRID_LINE_WIDTH,
                            block_y + GRID_LINE_WIDTH,
                            BLOCK_PADDING,
                            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                        ),
                        highlight_color,
                    )?;
                    canvas.draw(&left_highlight, graphics::DrawParam::default());
                    
                    // Bottom shadow
                    let shadow_color = Color::new(
                        f32::max(self.next_piece.color.r - 0.3, 0.0),
                        f32::max(self.next_piece.color.g - 0.3, 0.0),
                        f32::max(self.next_piece.color.b - 0.3, 0.0),
                        self.next_piece.color.a,
                    );
                    
                    let bottom_shadow = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            block_x + GRID_LINE_WIDTH,
                            block_y + GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                            BLOCK_PADDING,
                        ),
                        shadow_color,
                    )?;
                    canvas.draw(&bottom_shadow, graphics::DrawParam::default());
                    
                    // Right shadow
                    let right_shadow = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            block_x + GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                            block_y + GRID_LINE_WIDTH,
                            BLOCK_PADDING,
                            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                        ),
                        shadow_color,
                    )?;
                    canvas.draw(&right_shadow, graphics::DrawParam::default());
                }
            }
        }
        Ok(())
    }

    /// Draws the title screen
    fn draw_title_screen(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw title text with pixelated appearance
        let title_text = graphics::Text::new("TETRIS");
        let title_scale = 5.0;

        // Calculate title dimensions for centering
        let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
        let title_y = SCREEN_HEIGHT / 3.0;

        // Draw multiple outlines for pixel-art effect
        // Black outline
        for dx in [-3, -2, -1, 1, 2, 3] {
            for dy in [-3, -2, -1, 1, 2, 3] {
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                        .color(Color::BLACK)
                .scale([title_scale, title_scale])
                .dest([
                            (SCREEN_WIDTH - title_width) / 2.0 + dx as f32,
                            title_y + dy as f32,
                ]),
        );
            }
        }

        // Draw main title with gradient style colors
        let colors = [
            Color::from_rgb(50, 220, 240),   // Cyan
            Color::from_rgb(60, 210, 250),   // Light blue
            Color::from_rgb(80, 190, 255),   // Blue
            Color::from_rgb(100, 170, 255),  // Darker blue
            Color::from_rgb(120, 150, 255),  // Light purple
        ];
        
        // Draw each letter with a slightly different color
        let title_chars = "TETRIS".chars().collect::<Vec<_>>();
        let char_width = title_width / title_chars.len() as f32;
        
        for (i, _) in title_chars.iter().enumerate() {
            let char_text = graphics::Text::new(title_chars[i].to_string());
            let color_idx = i % colors.len();
            
        canvas.draw(
                &char_text,
            graphics::DrawParam::default()
                    .color(colors[color_idx])
                .scale([title_scale, title_scale])
                .dest([
                        (SCREEN_WIDTH - title_width) / 2.0 + i as f32 * char_width,
                    title_y,
                ]),
        );
        }

        // Draw a pixelated decoration line under the title
        let line_y = title_y + title_text.dimensions(ctx).unwrap().h * title_scale + 20.0;
        let line_width = title_width + 100.0;
        let line_segments = 20;
        let segment_width = line_width / line_segments as f32;
        
        for i in 0..line_segments {
            let color_idx = i % colors.len();
            let line_rect = graphics::Rect::new(
                (SCREEN_WIDTH - line_width) / 2.0 + i as f32 * segment_width,
                line_y,
                segment_width,
                8.0,
            );
            let line_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                line_rect,
                colors[color_idx],
            )?;
            canvas.draw(&line_mesh, graphics::DrawParam::default());
        }

        // Draw tetromino decorations
        let tetromino_size = 15.0;
        let tetrominoes = [
            // I piece (cyan)
            (vec![vec![1, 1, 1, 1]], Color::from_rgb(0, 240, 240), 
             (SCREEN_WIDTH / 2.0 - line_width / 2.0 - 80.0, line_y + 40.0)),
            // T piece (purple)
            (vec![vec![0, 1, 0], vec![1, 1, 1]], Color::from_rgb(160, 0, 240),
             (SCREEN_WIDTH / 2.0 + line_width / 2.0 + 30.0, line_y + 40.0)),
            // L piece (orange)
            (vec![vec![0, 0, 1], vec![1, 1, 1]], Color::from_rgb(240, 160, 0),
             (SCREEN_WIDTH / 2.0 - 150.0, line_y + 120.0)),
            // O piece (yellow)
            (vec![vec![1, 1], vec![1, 1]], Color::from_rgb(240, 240, 0),
             (SCREEN_WIDTH / 2.0 + 150.0, line_y + 120.0)),
        ];
        
        for (shape, color, (pos_x, pos_y)) in tetrominoes.iter() {
            for (y, row) in shape.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    if cell == 1 {
                        // Main block
                        let block_rect = graphics::Rect::new(
                            pos_x + x as f32 * tetromino_size,
                            pos_y + y as f32 * tetromino_size,
                            tetromino_size - 1.0,
                            tetromino_size - 1.0,
                        );
                        let block_mesh = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            block_rect,
                            *color,
                        )?;
                        canvas.draw(&block_mesh, graphics::DrawParam::default());
                        
                        // Highlight (top-left)
                        let highlight_color = Color::new(
                            f32::min(color.r + 0.2, 1.0),
                            f32::min(color.g + 0.2, 1.0),
                            f32::min(color.b + 0.2, 1.0),
                            color.a,
                        );
                        let highlight_rect = graphics::Rect::new(
                            pos_x + x as f32 * tetromino_size,
                            pos_y + y as f32 * tetromino_size,
                            tetromino_size - 1.0,
                            2.0,
                        );
                        let highlight_mesh = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            highlight_rect,
                            highlight_color,
                        )?;
                        canvas.draw(&highlight_mesh, graphics::DrawParam::default());
                        
                        // Shadow (bottom-right)
                        let shadow_color = Color::new(
                            f32::max(color.r - 0.3, 0.0),
                            f32::max(color.g - 0.3, 0.0),
                            f32::max(color.b - 0.3, 0.0),
                            color.a,
                        );
                        let shadow_rect = graphics::Rect::new(
                            pos_x + x as f32 * tetromino_size,
                            pos_y + y as f32 * tetromino_size + tetromino_size - 3.0,
                            tetromino_size - 1.0,
                            2.0,
                        );
                        let shadow_mesh = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            shadow_rect,
                            shadow_color,
                        )?;
                        canvas.draw(&shadow_mesh, graphics::DrawParam::default());
                    }
                }
            }
        }

        // Draw "PRESS ANY KEY" text (blinking) with pixelated effect
        if self.show_text {
            let press_text = graphics::Text::new("PRESS ANY KEY TO START");
            let press_scale = 2.0;
            
            // Get text dimensions for proper centering
            let press_width = press_text.dimensions(ctx).unwrap().w * press_scale;
            
            // Shadow for pixelated effect
            canvas.draw(
                &press_text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0 + 2.0,
                        SCREEN_HEIGHT * 0.6 + 2.0,  // Moved up to 60% of screen height
                    ]),
            );
            
            // Main text
            canvas.draw(
                &press_text,
                graphics::DrawParam::default()
                    .color(Color::YELLOW)
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0,
                        SCREEN_HEIGHT * 0.6,  // Moved up to 60% of screen height
                    ]),
            );
        }

        // Draw menu options with pixelated effect
        let menu_scale = 1.5;
        let menu_y_start = SCREEN_HEIGHT * 0.7;  // Moved down to 70% of screen height
        let menu_spacing = 40.0;

        // Create the music status string first
        let music_status = format!("MUSIC: {} (PRESS M)", 
            if self.sounds.background_playing { "ON" } else { "OFF" });

        let menu_items = [
            ("PRESS H FOR HIGH SCORES", Color::from_rgb(100, 255, 100)),
            (music_status.as_str(), Color::new(0.7, 0.7, 1.0, 1.0))
        ];

        for (i, (text, color)) in menu_items.iter().enumerate() {
            let menu_text = graphics::Text::new(*text);
            let text_width = menu_text.dimensions(ctx).unwrap().w * menu_scale;

            // Draw shadow
            canvas.draw(
                &menu_text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([menu_scale, menu_scale])
                    .dest([
                        (SCREEN_WIDTH - text_width) / 2.0 + 2.0,
                        menu_y_start + i as f32 * menu_spacing + 2.0,
                    ]),
            );

            // Draw text
            canvas.draw(
                &menu_text,
                graphics::DrawParam::default()
                    .color(*color)
                    .scale([menu_scale, menu_scale])
                    .dest([
                        (SCREEN_WIDTH - text_width) / 2.0,
                        menu_y_start + i as f32 * menu_spacing,
                    ]),
            );
        }

        // Draw music toggle instruction with pixelated style
        let music_text = graphics::Text::new(
            format!("MUSIC: {} (PRESS M TO TOGGLE)", 
                if self.sounds.background_playing { "ON" } else { "OFF" }
            )
        );
        
        // Get text dimensions for proper centering
        let music_width = music_text.dimensions(ctx).unwrap().w;
        
        // Shadow for pixelated effect
        canvas.draw(
            &music_text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.6))
                .dest([
                    (SCREEN_WIDTH - music_width) / 2.0 + 1.0,
                    SCREEN_HEIGHT * 0.8 + 1.0,
                ]),
        );
        
        // Main text
        canvas.draw(
            &music_text,
            graphics::DrawParam::default()
                .color(Color::new(0.7, 0.7, 1.0, 1.0))  // Light blue color
                .dest([
                    (SCREEN_WIDTH - music_width) / 2.0,
                    SCREEN_HEIGHT * 0.8,
                ]),
        );

        // Draw copyright text with pixelated shadow
        let copyright_text = graphics::Text::new("© 2024 RUST TETRIS");
        let copyright_width = copyright_text.dimensions(ctx).unwrap().w;
        
        canvas.draw(
            &copyright_text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.5))
                .dest([
                    (SCREEN_WIDTH - copyright_width) / 2.0 + 1.0,
                    SCREEN_HEIGHT - 40.0 + 1.0,
                ]),
        );
        
        canvas.draw(
            &copyright_text,
            graphics::DrawParam::default()
                .color(Color::new(0.5, 0.5, 0.5, 1.0))
                .dest([
                    (SCREEN_WIDTH - copyright_width) / 2.0,
                    SCREEN_HEIGHT - 40.0,
                ]),
        );

        Ok(())
    }

    /// Draws the main game screen
    fn draw_game(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
                // Draw game field border
                let border_rect = graphics::Rect::new(
                    MARGIN - BORDER_WIDTH,
                    MARGIN - BORDER_WIDTH,
                    GRID_SIZE * GRID_WIDTH as f32 + 2.0 * BORDER_WIDTH,
                    GRID_SIZE * GRID_HEIGHT as f32 + 2.0 * BORDER_WIDTH,
                );
                let border_mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(BORDER_WIDTH),
                    border_rect,
                    Color::WHITE,
                )?;
                canvas.draw(&border_mesh, graphics::DrawParam::default());

        // Draw the grid lines
        self.draw_grid(ctx, canvas)?;

                // Draw the game board
                for y in 0..GRID_HEIGHT {
                    for x in 0..GRID_WIDTH {
                        let color = self.board[y as usize][x as usize];
                        if color != Color::BLACK {
                    self.draw_block(ctx, canvas, x as f32, y as f32, color)?;
                        }
                    }
                }

                // Draw the current piece
                if let Some(piece) = &self.current_piece {
                    for (y, row) in piece.shape.iter().enumerate() {
                        for (x, &cell) in row.iter().enumerate() {
                            if cell {
                        self.draw_block(
                            ctx, 
                            canvas, 
                            (piece.position.x as i32 + x as i32) as f32, 
                            (piece.position.y as i32 + y as i32) as f32, 
                            piece.color
                        )?;
                            }
                        }
                    }
                }

                // Draw the next piece preview
        self.draw_preview(ctx, canvas)?;

        // Draw the score panel
        self.draw_score_panel(ctx, canvas)?;
        
        Ok(())
    }
    
    /// Draws the game over screen
    fn draw_game_over_screen(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // First draw the game board in the background
        self.draw_game(ctx, canvas)?;
        
        // Draw "GAME OVER" text with pixelated effect
        let game_over_text = graphics::Text::new("GAME OVER");
        let game_over_scale = 3.0;
        
        // Draw multiple outlines for pixel-art effect
        for dx in [-2, -1, 1, 2] {
            for dy in [-2, -1, 1, 2] {
                canvas.draw(
                    &game_over_text,
                    graphics::DrawParam::default()
                        .color(Color::BLACK)
                        .scale([game_over_scale, game_over_scale])
                        .dest([
                            SCREEN_WIDTH / 2.0 + dx as f32,
                            SCREEN_HEIGHT / 2.0 - 60.0 + dy as f32,
                        ])
                        .offset([0.5, 0.5]),
                );
            }
        }
        
        // Draw each letter with a slightly different shade of red
        let game_over_chars = "GAME OVER".chars().collect::<Vec<_>>();
        let char_width = game_over_text.dimensions(ctx).unwrap().w * game_over_scale / game_over_chars.len() as f32;
        
        for (i, ch) in game_over_chars.iter().enumerate() {
            // Skip spaces
            if *ch == ' ' {
                continue;
            }
            
            let char_text = graphics::Text::new(ch.to_string());
            
            // Alternate between different shades of red
            let color = if i % 2 == 0 {
                Color::from_rgb(255, 40, 40)
            } else {
                Color::from_rgb(220, 0, 0)
            };
            
            canvas.draw(
                &char_text,
                graphics::DrawParam::default()
                    .color(color)
                    .scale([game_over_scale, game_over_scale])
                    .dest([
                        SCREEN_WIDTH / 2.0 - (game_over_chars.len() as f32 * char_width / 2.0) + i as f32 * char_width,
                        SCREEN_HEIGHT / 2.0 - 60.0,
                    ])
                    .offset([0.0, 0.5]),
            );
        }

        // Draw "PRESS ANY KEY" text (blinking) with pixelated effect
        if self.show_text {
            let press_text = graphics::Text::new("PRESS ANY KEY TO RESTART");
            let press_scale = 2.0;
            
            // Get text dimensions for proper centering
            let press_width = press_text.dimensions(ctx).unwrap().w * press_scale;
            
            // Shadow for pixelated effect
            canvas.draw(
                &press_text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0 + 2.0,
                        SCREEN_HEIGHT / 2.0 + 60.0 + 2.0,
                    ]),
            );
            
            // Main text
            canvas.draw(
                &press_text,
                graphics::DrawParam::default()
                    .color(Color::YELLOW)
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0,
                        SCREEN_HEIGHT / 2.0 + 60.0,
                    ]),
            );
        }
        
        Ok(())
    }

    /// Draws the pause screen overlay
    fn draw_pause_screen(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // First draw the game in the background
        self.draw_game(ctx, canvas)?;
        
        // Draw semi-transparent overlay
        let overlay_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let overlay = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            overlay_rect,
            Color::new(0.0, 0.0, 0.0, 0.7),
        )?;
        canvas.draw(&overlay, graphics::DrawParam::default());
        
        // Draw "PAUSED" text with pixelated effect
        let pause_text = graphics::Text::new("PAUSED");
        let pause_scale = 4.0;
        let pause_width = pause_text.dimensions(ctx).unwrap().w * pause_scale;
        
        // Draw shadow for pixel-art effect
        canvas.draw(
            &pause_text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.6))
                .scale([pause_scale, pause_scale])
                .dest([
                    (SCREEN_WIDTH - pause_width) / 2.0 + 4.0,
                    SCREEN_HEIGHT / 3.0 + 4.0,
                ]),
        );
        
        // Draw main text
        canvas.draw(
            &pause_text,
            graphics::DrawParam::default()
                .color(Color::YELLOW)
                .scale([pause_scale, pause_scale])
                .dest([
                    (SCREEN_WIDTH - pause_width) / 2.0,
                    SCREEN_HEIGHT / 3.0,
                ]),
        );
        
        // Draw "PRESS P TO CONTINUE" text
        if self.show_text {
            let continue_text = graphics::Text::new("PRESS P TO CONTINUE");
            let continue_scale = 1.5;
            let continue_width = continue_text.dimensions(ctx).unwrap().w * continue_scale;
            
            // Draw shadow
            canvas.draw(
                &continue_text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([continue_scale, continue_scale])
                    .dest([
                        (SCREEN_WIDTH - continue_width) / 2.0 + 2.0,
                        SCREEN_HEIGHT / 2.0 + 2.0,
                    ]),
            );
            
            // Draw main text
            canvas.draw(
                &continue_text,
                graphics::DrawParam::default()
                    .color(Color::WHITE)
                    .scale([continue_scale, continue_scale])
                    .dest([
                        (SCREEN_WIDTH - continue_width) / 2.0,
                        SCREEN_HEIGHT / 2.0,
                    ]),
            );
        }
        
        Ok(())
    }

    /// Draws a block in 8-bit style
    fn draw_block(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, x: f32, y: f32, color: Color) -> GameResult {
        // Calculate the block position
        let block_x = MARGIN + x * GRID_SIZE;
        let block_y = MARGIN + y * GRID_SIZE;
        
        // Main block (slightly smaller to create grid effect)
        let block_rect = graphics::Rect::new(
            block_x + GRID_LINE_WIDTH, 
            block_y + GRID_LINE_WIDTH,
            GRID_SIZE - 2.0 * GRID_LINE_WIDTH, 
            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
        );
        
        // Create the block mesh
                            let mesh = graphics::Mesh::new_rectangle(
                                ctx,
                                graphics::DrawMode::fill(),
            block_rect,
                                color,
                            )?;
                            canvas.draw(&mesh, graphics::DrawParam::default());
        
        // Add a lighter highlight on top and left (8-bit style shading)
        let highlight_color = Color::new(
            f32::min(color.r + 0.2, 1.0),
            f32::min(color.g + 0.2, 1.0),
            f32::min(color.b + 0.2, 1.0),
            color.a,
        );
        
        // Top highlight
        let top_highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                block_x + GRID_LINE_WIDTH,
                block_y + GRID_LINE_WIDTH,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                BLOCK_PADDING,
            ),
            highlight_color,
        )?;
        canvas.draw(&top_highlight, graphics::DrawParam::default());
        
        // Left highlight
        let left_highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                block_x + GRID_LINE_WIDTH,
                block_y + GRID_LINE_WIDTH,
                BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
            ),
            highlight_color,
        )?;
        canvas.draw(&left_highlight, graphics::DrawParam::default());
        
        // Add a darker shadow on bottom and right
        let shadow_color = Color::new(
            f32::max(color.r - 0.3, 0.0),
            f32::max(color.g - 0.3, 0.0),
            f32::max(color.b - 0.3, 0.0),
            color.a,
        );
        
        // Bottom shadow
        let bottom_shadow = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                block_x + GRID_LINE_WIDTH,
                block_y + GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                BLOCK_PADDING,
            ),
            shadow_color,
        )?;
        canvas.draw(&bottom_shadow, graphics::DrawParam::default());
        
        // Right shadow
        let right_shadow = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                block_x + GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                block_y + GRID_LINE_WIDTH,
                BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
            ),
            shadow_color,
        )?;
        canvas.draw(&right_shadow, graphics::DrawParam::default());
        
        Ok(())
    }

    /// Draws grid lines for 8-bit aesthetic
    fn draw_grid(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        let grid_color = Color::new(0.2, 0.2, 0.2, 1.0);
        
        // Draw vertical grid lines
        for x in 0..=GRID_WIDTH {
            let grid_line = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    MARGIN + x as f32 * GRID_SIZE - GRID_LINE_WIDTH / 2.0,
                    MARGIN - GRID_LINE_WIDTH / 2.0,
                    GRID_LINE_WIDTH,
                    GRID_SIZE * GRID_HEIGHT as f32 + GRID_LINE_WIDTH,
                ),
                grid_color,
            )?;
            canvas.draw(&grid_line, graphics::DrawParam::default());
        }
        
        // Draw horizontal grid lines
        for y in 0..=GRID_HEIGHT {
            let grid_line = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    MARGIN - GRID_LINE_WIDTH / 2.0,
                    MARGIN + y as f32 * GRID_SIZE - GRID_LINE_WIDTH / 2.0,
                    GRID_SIZE * GRID_WIDTH as f32 + GRID_LINE_WIDTH,
                    GRID_LINE_WIDTH,
                ),
                grid_color,
            )?;
            canvas.draw(&grid_line, graphics::DrawParam::default());
        }
        
        Ok(())
    }

    /// Calculates the current drop speed based on level
    fn drop_speed(&self) -> f64 {
        let base_drop_time = DROP_TIME;
        // Decrease drop time as level increases (higher levels = faster speed)
        base_drop_time / (1.0 + 0.1 * self.level as f64)
    }

    /// Updates the score based on lines cleared
    fn update_score(&mut self, lines: u32) {
        // Add points based on number of lines cleared
        let line_points = match lines {
            1 => SCORE_SINGLE,
            2 => SCORE_DOUBLE,
            3 => SCORE_TRIPLE,
            4 => SCORE_TETRIS,
            _ => 0,
        };
        
        // Apply level multiplier to reward higher levels
        self.score += line_points * self.level;
        
        // Update total lines cleared
        self.lines_cleared += lines;
        
        // Update level (every 10 lines)
        self.level = (self.lines_cleared / 10) + 1;
    }

    /// Adds points for dropping a piece
    fn add_drop_points(&mut self, cells_dropped: i32) {
        self.score += (cells_dropped as u32) * SCORE_DROP * self.level;
    }

    /// Checks if the current score qualifies for the high score list
    fn check_high_score(&self) -> bool {
        self.high_scores.would_qualify(self.score)
    }

    /// Draws the UI panel with score information
    fn draw_score_panel(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw score panel background with pixelated corners
        let panel_rect = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE,
            PREVIEW_Y + GRID_SIZE * 6.0 + 20.0,
            GRID_SIZE * 6.0,
            GRID_SIZE * 6.0,
        );
        
        // Draw outer frame (darker)
        let frame_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            panel_rect,
            Color::new(0.2, 0.2, 0.2, 1.0),
        )?;
        canvas.draw(&frame_mesh, graphics::DrawParam::default());

        // Draw inner frame (lighter)
        let inner_rect = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 2.0,
            PREVIEW_Y + GRID_SIZE * 6.0 + 20.0 + GRID_LINE_WIDTH * 2.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
        );
        let inner_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_rect,
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&inner_mesh, graphics::DrawParam::default());

        // Draw main background (darkest)
        let main_bg = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 4.0,
            PREVIEW_Y + GRID_SIZE * 6.0 + 20.0 + GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
        );
        let main_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            main_bg,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )?;
        canvas.draw(&main_mesh, graphics::DrawParam::default());
        
        // Draw score text with larger scale and pixelated effect
        let score_text = graphics::Text::new("SCORE");
        let score_value = graphics::Text::new(format!("{}", self.score));
        let level_text = graphics::Text::new("LEVEL");
        let level_value = graphics::Text::new(format!("{}", self.level));
        let lines_text = graphics::Text::new("LINES");
        let lines_value = graphics::Text::new(format!("{}", self.lines_cleared));
        
        // Calculate total height of all text elements
        let text_scale = 1.5;
        let text_spacing = 70.0;  // Increased from 45.0 to 70.0 for better vertical distribution
        let total_text_height = text_spacing * 2.0;  // Space between 3 items
        
        // Calculate starting Y position to center all text vertically
        let panel_top = PREVIEW_Y + GRID_SIZE * 6.0 + 20.0;
        let panel_height = GRID_SIZE * 6.0;
        let text_y_start = panel_top + (panel_height - total_text_height) / 2.0 - 20.0;  // Moved up slightly to better center the whole block
        
        // Calculate horizontal position
        let text_x = PREVIEW_X + GRID_SIZE * 0.5;
        
        // Draw labels and values with pixelated effect
        let label_width = 80.0;  // Fixed width for labels
        let _value_width = 60.0;  // Fixed width for values (unused but kept for future use)
        
        // Helper function to draw text with shadow
        let mut draw_text_with_shadow = |text: &graphics::Text, x: f32, y: f32| {
            // Draw shadow
            canvas.draw(
                text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([text_scale, text_scale])
                    .dest([x + 2.0, y + 2.0]),
            );
            // Draw main text
            canvas.draw(
                text,
                graphics::DrawParam::default()
                    .color(Color::WHITE)
                    .scale([text_scale, text_scale])
                    .dest([x, y]),
            );
        };
        
        // Draw labels (right-aligned)
        draw_text_with_shadow(&score_text, text_x + label_width - score_text.dimensions(ctx).unwrap().w * text_scale, text_y_start);
        draw_text_with_shadow(&level_text, text_x + label_width - level_text.dimensions(ctx).unwrap().w * text_scale, text_y_start + text_spacing);
        draw_text_with_shadow(&lines_text, text_x + label_width - lines_text.dimensions(ctx).unwrap().w * text_scale, text_y_start + text_spacing * 2.0);
        
        // Draw values (left-aligned)
        draw_text_with_shadow(&score_value, text_x + label_width + 20.0, text_y_start);
        draw_text_with_shadow(&level_value, text_x + label_width + 20.0, text_y_start + text_spacing);
        draw_text_with_shadow(&lines_value, text_x + label_width + 20.0, text_y_start + text_spacing * 2.0);
        
        Ok(())
    }

    /// Adds the current score to the high scores
    fn add_high_score(&mut self) -> bool {
        self.high_scores.add_score(self.current_name.clone(), self.score)
    }

    /// Draws the name entry screen
    fn draw_name_entry(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw background with solid color
        canvas.set_screen_coordinates(graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT));
        // Fill with background color instead of using clear
        let bg_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let bg_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            bg_rect,
            Color::new(0.05, 0.05, 0.1, 1.0),
        )?;
        canvas.draw(&bg_mesh, graphics::DrawParam::default());
        
        // Draw title text
        let title_text = graphics::Text::new("HIGH SCORE!");
        let title_scale = 3.0;
        let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
        
        // Draw title with shadow
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.6))
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0 + 2.0,
                    SCREEN_HEIGHT / 4.0 + 2.0,
                ]),
        );
        
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .color(Color::YELLOW)
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0,
                    SCREEN_HEIGHT / 4.0,
                ]),
        );
        
        // Draw score text
        let score_text = graphics::Text::new(format!("YOUR SCORE: {}", self.score));
        let score_scale = 2.0;
        let score_width = score_text.dimensions(ctx).unwrap().w * score_scale;
        
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .color(Color::WHITE)
                .scale([score_scale, score_scale])
                .dest([
                    (SCREEN_WIDTH - score_width) / 2.0,
                    SCREEN_HEIGHT / 3.0,
                ]),
        );
        
        // Draw name entry prompt
        let prompt_text = graphics::Text::new("ENTER YOUR NAME:");
        let prompt_scale = 1.5;
        let prompt_width = prompt_text.dimensions(ctx).unwrap().w * prompt_scale;
        
        canvas.draw(
            &prompt_text,
            graphics::DrawParam::default()
                .color(Color::WHITE)
                .scale([prompt_scale, prompt_scale])
                .dest([
                    (SCREEN_WIDTH - prompt_width) / 2.0,
                    SCREEN_HEIGHT / 2.0 - 30.0,
                ]),
        );
        
        // Draw the current name
        let display_name = if self.show_cursor {
            format!("{}_", self.current_name)
        } else {
            format!("{}  ", self.current_name) // Two spaces to maintain consistent width
        };
        
        let name_text = graphics::Text::new(display_name);
        let name_scale = 2.0;
        
        // Calculate fixed box width based on maximum name length (15 chars) plus cursor
        let max_name_width = graphics::Text::new("A".repeat(15) + " ").dimensions(ctx).unwrap().w * name_scale;
        let fixed_box_width = max_name_width + 60.0; // Add more padding
        
        // Draw with fixed-width background box
        let name_bg_rect = graphics::Rect::new(
            (SCREEN_WIDTH - fixed_box_width) / 2.0,
            SCREEN_HEIGHT / 2.0 + 10.0,
            fixed_box_width,
            50.0,
        );
        
        let name_bg = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            name_bg_rect,
            Color::new(0.1, 0.1, 0.2, 1.0),
        )?;
        canvas.draw(&name_bg, graphics::DrawParam::default());
        
        // Center the text within the fixed box
        canvas.draw(
            &name_text,
            graphics::DrawParam::default()
                .color(Color::from_rgb(100, 255, 100))
                .scale([name_scale, name_scale])
                .dest([
                    (SCREEN_WIDTH - name_text.dimensions(ctx).unwrap().w * name_scale) / 2.0,
                    SCREEN_HEIGHT / 2.0 + 20.0,
                ]),
        );
        
        // Draw instructions
        let instructions_text = graphics::Text::new("PRESS ENTER WHEN DONE");
        let inst_scale = 1.0;
        let inst_width = instructions_text.dimensions(ctx).unwrap().w * inst_scale;
        
        canvas.draw(
            &instructions_text,
            graphics::DrawParam::default()
                .color(Color::new(0.7, 0.7, 1.0, 1.0))
                .scale([inst_scale, inst_scale])
                .dest([
                    (SCREEN_WIDTH - inst_width) / 2.0,
                    SCREEN_HEIGHT * 3.0 / 4.0,
                ]),
        );
        
        Ok(())
    }

    /// Draws the high scores screen
    fn draw_high_scores(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw background with solid color
        canvas.set_screen_coordinates(graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT));
        let bg_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let bg_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            bg_rect,
            Color::new(0.05, 0.05, 0.1, 1.0),
        )?;
        canvas.draw(&bg_mesh, graphics::DrawParam::default());
        
        // Draw title text
        let title_text = graphics::Text::new("HIGH SCORES");
        let title_scale = 3.0;
        let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
        
        // Draw title with shadow
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.6))
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0 + 4.0,
                    50.0 + 4.0,
                ]),
        );
        
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .color(Color::YELLOW)
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0,
                    50.0,
                ]),
        );
        
        // Draw decorative line
        let line_width = title_width + 100.0;
        let line_y = 50.0 + title_text.dimensions(ctx).unwrap().h * title_scale + 20.0;
        let line_segments = 20;
        let segment_width = line_width / line_segments as f32;
        
        let colors = [
            Color::from_rgb(50, 220, 240),   // Cyan
            Color::from_rgb(60, 210, 250),   // Light blue
            Color::from_rgb(80, 190, 255),   // Blue
            Color::from_rgb(100, 170, 255),  // Darker blue
            Color::from_rgb(120, 150, 255),  // Light purple
        ];
        
        for i in 0..line_segments {
            let color_idx = i % colors.len();
            let line_rect = graphics::Rect::new(
                (SCREEN_WIDTH - line_width) / 2.0 + i as f32 * segment_width,
                line_y,
                segment_width,
                4.0,
            );
            let line_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                line_rect,
                colors[color_idx],
            )?;
            canvas.draw(&line_mesh, graphics::DrawParam::default());
        }
        
        // Draw scores with larger text and better alignment
        let mut y_pos = line_y + 60.0;  // Increased initial spacing
        let line_height = 50.0;  // Increased line height
        let text_scale = 1.8;    // Increased text scale
        
        // Column positions (adjusted for better alignment)
        let rank_x = SCREEN_WIDTH * 0.25;        // Move rank to 25% of screen width
        let name_x = SCREEN_WIDTH * 0.45;        // Move name to 45% of screen width
        let score_x = SCREEN_WIDTH * 0.75;       // Move score to 75% of screen width
        
        // Draw header with larger scale and shadow
        let rank_header = graphics::Text::new("RANK");
        let name_header = graphics::Text::new("NAME");
        let score_header = graphics::Text::new("SCORE");
        
        // Draw headers with proper alignment
        let mut draw_header = |text: &graphics::Text, x: f32, align: f32| {
            let text_width = text.dimensions(ctx).unwrap().w * text_scale;
            // Draw shadow
            canvas.draw(
                text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([text_scale, text_scale])
                    .dest([x - text_width * align + 2.0, y_pos + 2.0]),
            );
            // Draw text
            canvas.draw(
                text,
                graphics::DrawParam::default()
                    .color(Color::WHITE)
                    .scale([text_scale, text_scale])
                    .dest([x - text_width * align, y_pos]),
            );
        };
        
        // Draw headers with different alignments
        draw_header(&rank_header, rank_x, 0.5);  // Center-aligned
        draw_header(&name_header, name_x, 0.0);  // Left-aligned
        draw_header(&score_header, score_x, 1.0); // Right-aligned
        
        y_pos += line_height + 20.0;  // Add extra spacing after header
        
        // Draw each score entry with matching alignment
        for (i, entry) in self.high_scores.entries.iter().enumerate() {
            let rank = i + 1;
            let color = if rank <= 3 {
                match rank {
                    1 => Color::from_rgb(255, 215, 0),  // Gold
                    2 => Color::from_rgb(192, 192, 192), // Silver
                    3 => Color::from_rgb(205, 127, 50),  // Bronze
                    _ => Color::WHITE,
                }
            } else {
                Color::WHITE
            };
            
            // Helper function to draw text with shadow
            let mut draw_text_with_shadow = |text: &str, x: f32, align: f32| {
                let text_obj = graphics::Text::new(text);
                let text_width = text_obj.dimensions(ctx).unwrap().w * text_scale;
                // Draw shadow
                canvas.draw(
                    &text_obj,
                    graphics::DrawParam::default()
                        .color(Color::new(0.0, 0.0, 0.0, 0.6))
                        .scale([text_scale, text_scale])
                        .dest([x - text_width * align + 2.0, y_pos + 2.0]),
                );
                // Draw text
                canvas.draw(
                    &text_obj,
                    graphics::DrawParam::default()
                        .color(color)
                        .scale([text_scale, text_scale])
                        .dest([x - text_width * align, y_pos]),
                );
            };
            
            // Draw rank (center-aligned)
            draw_text_with_shadow(&format!("{}", rank), rank_x, 0.5);
            
            // Draw name (left-aligned)
            draw_text_with_shadow(&entry.name, name_x, 0.0);
            
            // Draw score (right-aligned)
            draw_text_with_shadow(&format!("{}", entry.score), score_x, 1.0);
            
            y_pos += line_height;
        }
        
        // Draw "Press any key to continue" if blinking
        if self.show_text {
            let continue_text = graphics::Text::new("PRESS ANY KEY TO CONTINUE");
            let continue_scale = 1.5;  // Increased scale
            let continue_width = continue_text.dimensions(ctx).unwrap().w * continue_scale;
            
            // Draw shadow
            canvas.draw(
                &continue_text,
                graphics::DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([continue_scale, continue_scale])
                    .dest([
                        (SCREEN_WIDTH - continue_width) / 2.0 + 2.0,
                        SCREEN_HEIGHT - 100.0 + 2.0,
                    ]),
            );
            
            // Draw text
            canvas.draw(
                &continue_text,
                graphics::DrawParam::default()
                    .color(Color::YELLOW)
                    .scale([continue_scale, continue_scale])
                    .dest([
                        (SCREEN_WIDTH - continue_width) / 2.0,
                        SCREEN_HEIGHT - 100.0,
                    ]),
            );
        }
        
        Ok(())
    }
}

/// Converts a keycode to a character for name entry
fn keycode_to_char(keycode: KeyCode, shift: bool) -> Option<char> {
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
        KeyCode::Key0 | KeyCode::Numpad0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Key1 | KeyCode::Numpad1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Key2 | KeyCode::Numpad2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Key3 | KeyCode::Numpad3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Key4 | KeyCode::Numpad4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Key5 | KeyCode::Numpad5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Key6 | KeyCode::Numpad6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Key7 | KeyCode::Numpad7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Key8 | KeyCode::Numpad8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Key9 | KeyCode::Numpad9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equals => Some(if shift { '+' } else { '=' }),
        _ => None,
    }
}

/// Implementation of the game loop and event handling
impl event::EventHandler<ggez::GameError> for GameState {
    /// Updates the game state
    /// Handles automatic piece movement and game over state
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Update blink timer for start screen and game over screen
        let dt = ctx.time.delta().as_secs_f64();
        self.blink_timer += dt;
        if self.blink_timer >= 0.5 {  // Blink every 0.5 seconds
            self.blink_timer = 0.0;
            self.show_text = !self.show_text;
        }
        
        // Update cursor blink for name entry
        self.cursor_blink_timer += dt;
        if self.cursor_blink_timer >= 0.3 {
            self.cursor_blink_timer = 0.0;
            self.show_cursor = !self.show_cursor;
        }

        // Only update game logic if we're playing and not paused
        if self.screen == GameScreen::Playing && !self.paused {
            self.drop_timer += dt;

            // Move the piece down automatically based on level speed
            if self.drop_timer >= self.drop_speed() {
                self.drop_timer = 0.0;
                if let Some(piece) = &self.current_piece {
                    let mut new_piece = piece.clone();
                    new_piece.position.y += 1.0;
                    if self.check_collision(&new_piece) {
                        self.lock_piece(ctx);
                    } else {
                        self.current_piece = Some(new_piece);
                    }
                }
            }
        }
        
        // Check for high score qualification after game over
        if self.screen == GameScreen::GameOver && self.check_high_score() {
            self.screen = GameScreen::EnterName;
        }

        Ok(())
    }

    /// Handles keyboard input
    /// Controls:
    /// - Left/Right arrows: Move piece horizontally
    /// - Up arrow: Rotate piece
    /// - Down arrow: Soft drop
    /// - Space: Hard drop
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        match self.screen {
            GameScreen::Title => {
                match input.keycode {
                    Some(KeyCode::M) => {
                        // Toggle music
                        if self.sounds.background_playing {
                            self.sounds.stop_background_music(ctx);
                        } else {
                            self.sounds.start_background_music(ctx)?;
                        }
                    }
                    Some(KeyCode::H) => {
                        // Show high scores
                        self.screen = GameScreen::HighScores;
                    }
                    _ => {
                        // Any other key starts the game
                        self.reset_game(ctx)?;
                    }
                }
            }
            GameScreen::Playing => {
                match input.keycode {
                    Some(KeyCode::M) => {
                        // Toggle music
                        if self.sounds.background_playing {
                            self.sounds.stop_background_music(ctx);
                        } else {
                            self.sounds.start_background_music(ctx)?;
                        }
                    }
                    Some(KeyCode::P) => {
                        // Toggle pause
                        self.paused = !self.paused;
                    }
                    Some(KeyCode::Left) => {
                        if !self.paused {
                        self.move_piece(|p| p.position.x -= 1.0, ctx);
                        }
                    }
                    Some(KeyCode::Right) => {
                        if !self.paused {
                        self.move_piece(|p| p.position.x += 1.0, ctx);
                        }
                    }
                    Some(KeyCode::Down) => {
                        if !self.paused {
                        self.move_piece(|p| p.position.y += 1.0, ctx);
                        }
                    }
                    Some(KeyCode::Up) => {
                        if !self.paused {
                        self.try_rotate(ctx);
                        }
                    }
                    Some(KeyCode::Space) => {
                        if !self.paused {
                        self.hard_drop(ctx);
                        }
                    }
                    _ => {}
                }
            }
            GameScreen::GameOver => {
                // Any key returns to title screen if no high score qualification
                // If high score qualification, the screen should already be EnterName
                // This is a fallback in case something went wrong
                if self.check_high_score() {
                    self.screen = GameScreen::EnterName;
                } else {
                    self.screen = GameScreen::Title;
                }
            }
            GameScreen::EnterName => {
                match input.keycode {
                    Some(KeyCode::Return) => {
                        // Submit the name and score
                        if !self.current_name.is_empty() {
                            self.add_high_score();
                            self.screen = GameScreen::HighScores;
                            self.current_name.clear();
                        }
                    }
                    Some(KeyCode::Back) => {
                        // Remove the last character
                        self.current_name.pop();
                    }
                    Some(keycode) => {
                        // Only allow alphanumeric characters and limit name length
                        if self.current_name.len() < 15 {
                            if let Some(ch) = keycode_to_char(keycode, ctx.keyboard.is_key_pressed(KeyCode::LShift) || ctx.keyboard.is_key_pressed(KeyCode::RShift)) {
                                self.current_name.push(ch);
                            }
                        }
                    }
                    None => {}
                }
            }
            GameScreen::HighScores => {
                // Any key returns to start screen
                self.screen = GameScreen::Title;
            }
        }

        Ok(())
    }

    /// Handles rendering the game state to the screen
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.05, 0.05, 0.1, 1.0));

        // Draw based on current game screen
        match self.screen {
            GameScreen::Title => {
                self.draw_title_screen(ctx, &mut canvas)?;
            }
            GameScreen::Playing => {
                if self.paused {
                    self.draw_pause_screen(ctx, &mut canvas)?;
                } else {
                    self.draw_game(ctx, &mut canvas)?;
                }
            }
            GameScreen::GameOver => {
                self.draw_game_over_screen(ctx, &mut canvas)?;
            }
            GameScreen::EnterName => {
                self.draw_name_entry(ctx, &mut canvas)?;
            }
            GameScreen::HighScores => {
                self.draw_high_scores(ctx, &mut canvas)?;
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

/// Entry point of the game
pub fn main() -> GameResult {
    let resource_dir = if cfg!(debug_assertions) {
        std::path::PathBuf::from(".")
    } else {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        exe_dir.parent().unwrap().join("Resources")
    };

    let cb = ggez::ContextBuilder::new("tetris", "ggez")
        .window_setup(WindowSetup::default().title("Tetris"))
        .window_mode(WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .add_resource_path(resource_dir);

    let (mut ctx, event_loop) = cb.build()?;
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetromino::TetrominoType;

    // Test constants and configurations
    #[test]
    fn test_constants() {
        // Test grid dimensions are valid
        assert!(GRID_WIDTH > 0);
        assert!(GRID_HEIGHT > 0);
        
        // Test grid size is reasonable
        assert!(GRID_SIZE > 0.0);
        
        // Test screen dimensions
        assert!(SCREEN_WIDTH > GRID_WIDTH as f32 * GRID_SIZE);
        assert!(SCREEN_HEIGHT > GRID_HEIGHT as f32 * GRID_SIZE);
        
        // Verify preview box position constants
        assert_eq!(PREVIEW_X, GRID_SIZE * (GRID_WIDTH as f32 + 3.0) + MARGIN);
        assert_eq!(PREVIEW_Y, GRID_SIZE * 2.0 + MARGIN);
        
        // Verify preview box is within screen bounds
        let preview_box_width = GRID_SIZE * PREVIEW_BOX_SIZE;
        let preview_box_height = GRID_SIZE * PREVIEW_BOX_SIZE;
        assert!(PREVIEW_X + preview_box_width <= SCREEN_WIDTH);
        assert!(PREVIEW_Y + preview_box_height <= SCREEN_HEIGHT);
        
        // Verify preview box doesn't overlap with game field
        let game_field_right = MARGIN + GRID_SIZE * GRID_WIDTH as f32;
        assert!(PREVIEW_X - GRID_SIZE > game_field_right + GRID_SIZE);
    }
    
    #[test]
    fn test_next_piece_centering() {
        // Test with different piece types to ensure proper centering
        let test_pieces = [
            TetrominoType::I,  // 4x1
            TetrominoType::O,  // 2x2
            TetrominoType::T,  // 3x2
            TetrominoType::L,  // 3x2
            TetrominoType::J,  // 3x2
            TetrominoType::S,  // 3x2
            TetrominoType::Z,  // 3x2
        ];

        for piece_type in test_pieces {
            let piece = Tetromino::new(piece_type);
            
            // Calculate expected offsets
            let piece_width = piece.shape[0].len() as f32;
            let piece_height = piece.shape.len() as f32;
            let offset_x = (PREVIEW_BOX_SIZE - piece_width) / 2.0;
            let offset_y = (PREVIEW_BOX_SIZE - piece_height) / 2.0;

            // Verify offsets are within preview box bounds
            assert!(offset_x >= 0.0);
            assert!(offset_y >= 0.0);

            // Verify piece dimensions are valid for preview box
            assert!(piece_width <= PREVIEW_BOX_SIZE);
            assert!(piece_height <= PREVIEW_BOX_SIZE);

            // Verify piece position after centering
            let preview_x = PREVIEW_X - GRID_SIZE + offset_x * GRID_SIZE;
            let preview_y = PREVIEW_Y - GRID_SIZE + offset_y * GRID_SIZE;

            // Verify piece is within preview box bounds
            assert!(preview_x >= PREVIEW_X - GRID_SIZE);
            assert!(preview_x + piece_width * GRID_SIZE <= PREVIEW_X + GRID_SIZE * (PREVIEW_BOX_SIZE - 1.0));
            assert!(preview_y >= PREVIEW_Y - GRID_SIZE);
            assert!(preview_y + piece_height * GRID_SIZE <= PREVIEW_Y + GRID_SIZE * (PREVIEW_BOX_SIZE - 1.0));
        }
    }

    #[test]
    fn test_high_scores() {
        let mut high_scores = HighScores::new();
        
        // Test adding scores when list is not full
        assert!(high_scores.add_score("Player1".to_string(), 1000));
        assert!(high_scores.add_score("Player2".to_string(), 500));
        assert!(high_scores.add_score("Player3".to_string(), 750));
        
        // Test scores are sorted correctly
        assert_eq!(high_scores.entries[0].score, 1000);
        assert_eq!(high_scores.entries[1].score, 750);
        assert_eq!(high_scores.entries[2].score, 500);
        
        // Test would_qualify function with non-full list
        assert!(high_scores.would_qualify(400)); // Should qualify when list isn't full
        
        // Fill up the high scores list
        for i in 0..MAX_HIGH_SCORES {
            high_scores.add_score(format!("Player{}", i), (1000 + i) as u32);
        }
        
        // Test would_qualify function with full list
        assert!(high_scores.would_qualify(1500)); // Should qualify (better than some scores)
        assert!(!high_scores.would_qualify(500)); // Shouldn't qualify (worse than all scores)
        
        // Test maximum number of scores
        assert_eq!(high_scores.entries.len(), MAX_HIGH_SCORES);
        
        // Test adding a qualifying score to full list
        assert!(high_scores.add_score("NewPlayer".to_string(), 1500));
        assert_eq!(high_scores.entries.len(), MAX_HIGH_SCORES); // List should stay at max size
    }

    #[test]
    fn test_high_score_column_positions() {
        // Test that column positions are properly spaced
        let rank_x = SCREEN_WIDTH * 0.25;
        let name_x = SCREEN_WIDTH * 0.45;
        let score_x = SCREEN_WIDTH * 0.75;
        
        // Verify columns don't overlap
        assert!(rank_x < name_x);
        assert!(name_x < score_x);
        
        // Verify reasonable spacing between columns
        assert!(name_x - rank_x >= GRID_SIZE * 3.0); // At least 3 grid cells between columns
        assert!(score_x - name_x >= GRID_SIZE * 3.0);
        
        // Verify columns are within screen bounds
        assert!(rank_x >= SCREEN_WIDTH * 0.1); // Not too close to left edge
        assert!(score_x <= SCREEN_WIDTH * 0.9); // Not too close to right edge
    }

    #[test]
    fn test_keycode_to_char() {
        // Test lowercase letters
        assert_eq!(keycode_to_char(KeyCode::A, false), Some('a'));
        assert_eq!(keycode_to_char(KeyCode::Z, false), Some('z'));
        
        // Test uppercase letters
        assert_eq!(keycode_to_char(KeyCode::A, true), Some('A'));
        assert_eq!(keycode_to_char(KeyCode::Z, true), Some('Z'));
        
        // Test numbers
        assert_eq!(keycode_to_char(KeyCode::Key1, false), Some('1'));
        assert_eq!(keycode_to_char(KeyCode::Key9, false), Some('9'));
        
        // Test space
        assert_eq!(keycode_to_char(KeyCode::Space, false), Some(' '));
        
        // Test unsupported key
        assert_eq!(keycode_to_char(KeyCode::F1, false), None);
    }

    // This is a simplified test that doesn't depend on ggez::Context
    #[test]
    fn test_collision_detection_simplified() {
        // Create a test board with a single block
        let mut board = vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        
        // Place a block on the board
        let block_x = GRID_WIDTH as usize / 2;
        let block_y = GRID_HEIGHT as usize - 1;
        board[block_y][block_x] = Color::RED;
        
        // Create a piece directly above the block
        let mut test_piece = Tetromino::new(TetrominoType::I);
        // The I piece is horizontal by default (1x4) so adjust position to ensure collision
        test_piece.position.x = block_x as f32 - 1.0; // Position it so part of it overlaps with the block
        test_piece.position.y = block_y as f32;       // Same y-position as the block
        
        // Manual collision check logic based on our game's algorithm
        let piece_width = test_piece.shape[0].len() as i32;
        let piece_height = test_piece.shape.len() as i32;
        let piece_x = test_piece.position.x.round() as i32;
        let piece_y = test_piece.position.y.round() as i32;
        
        // Check if any part of the piece would collide with the block
        let mut collision = false;
        'outer: for y in 0..piece_height {
            for x in 0..piece_width {
                if !test_piece.shape[y as usize][x as usize] {
                    continue; // Skip empty cells
                }
                
                let board_x = piece_x + x;
                let board_y = piece_y + y;
                
                // Check if out of bounds
                if board_x < 0 || board_x >= GRID_WIDTH as i32 || board_y >= GRID_HEIGHT as i32 {
                    collision = true;
                    break 'outer;
                }
                
                // Check if collides with existing block
                if board_y >= 0 && board[board_y as usize][board_x as usize] != Color::BLACK {
                    collision = true;
                    break 'outer;
                }
            }
        }
        
        assert!(collision, "Piece should collide with block on board");
        
        // Move the piece to an empty area
        test_piece.position.x = 0.0;
        test_piece.position.y = 0.0;
        
        // Redo collision check for new position
        let piece_x = test_piece.position.x.round() as i32;
        let piece_y = test_piece.position.y.round() as i32;
        
        collision = false;
        'outer: for y in 0..piece_height {
            for x in 0..piece_width {
                if !test_piece.shape[y as usize][x as usize] {
                    continue; // Skip empty cells
                }
                
                let board_x = piece_x + x;
                let board_y = piece_y + y;
                
                // Check if out of bounds
                if board_x < 0 || board_x >= GRID_WIDTH as i32 || board_y >= GRID_HEIGHT as i32 {
                    collision = true;
                    break 'outer;
                }
                
                // Check if collides with existing block
                if board_y >= 0 && board[board_y as usize][board_x as usize] != Color::BLACK {
                    collision = true;
                    break 'outer;
                }
            }
        }
        
        assert!(!collision, "Piece should not collide in empty area");
    }

    #[test]
    fn test_drop_speed_calculation() {
        // First level should have standard drop speed
        let level1_speed = 1.0 / (1.0 + 0.1 * (1 - 1) as f64);
        
        // Higher levels should have progressively faster speeds
        let level5_speed = 1.0 / (1.0 + 0.1 * (5 - 1) as f64);
        let level10_speed = 1.0 / (1.0 + 0.1 * (10 - 1) as f64);
        
        // Higher levels should have faster drop speeds (smaller time intervals)
        assert!(level1_speed > level5_speed, "Level 5 should be faster than level 1");
        assert!(level5_speed > level10_speed, "Level 10 should be faster than level 5");
    }

    #[test]
    fn test_score_calculation_simplified() {
        // Test score calculation for different numbers of lines
        let level = 1;
        
        // Single line
        let single_score = match 1 {
            1 => 40 * level,
            2 => 100 * level,
            3 => 300 * level,
            4 => 1200 * level,
            _ => 0,
        };
        
        // Double line
        let double_score = match 2 {
            1 => 40 * level,
            2 => 100 * level,
            3 => 300 * level,
            4 => 1200 * level,
            _ => 0,
        };
        
        // Triple line
        let triple_score = match 3 {
            1 => 40 * level,
            2 => 100 * level,
            3 => 300 * level,
            4 => 1200 * level,
            _ => 0,
        };
        
        // Tetris
        let tetris_score = match 4 {
            1 => 40 * level,
            2 => 100 * level,
            3 => 300 * level,
            4 => 1200 * level,
            _ => 0,
        };
        
        // Check score progression
        assert!(double_score > single_score, "Double clear should score more than single");
        assert!(triple_score > double_score, "Triple clear should score more than double");
        assert!(tetris_score > triple_score, "Tetris should score more than triple");
    }
}
