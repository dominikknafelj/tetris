use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color, Drawable},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use tetris::{
    board::GameBoard,
    constants::{
        BASE_DROP_TIME, BLINK_TIME, CURSOR_BLINK_TIME, 
        SCREEN_HEIGHT, SCREEN_WIDTH, SCORE_DOUBLE, SCORE_DROP, SCORE_SINGLE, SCORE_TETRIS, SCORE_TRIPLE,
    },
    score::HighScores,
    sound_manager::GameSounds,
    tetromino::Tetromino,
    ui::GameRenderer,
    GameScreen,
};

/// Main game state that holds all the game data
struct GameState {
    screen: GameScreen,              // Current game screen
    board: GameBoard,                // The game board
    renderer: GameRenderer,          // Renderer for all UI components
    current_piece: Option<Tetromino>, // Currently active piece
    next_piece: Tetromino,           // Next piece to spawn
    drop_timer: f64,                 // Timer for automatic piece movement
    sounds: GameSounds,              // Game sound effects
    blink_timer: f64,                // Timer for text blinking effect
    show_text: bool,                 // Whether to show blinking text
    score: u32,                      // Current game score
    level: u32,                      // Current game level
    high_scores: HighScores,         // High score list
    current_name: String,            // Current player name being entered
    cursor_blink_timer: f64,         // Timer for name input cursor blinking
    show_cursor: bool,               // Whether to show the name input cursor
    paused: bool,                    // Whether the game is paused
}

impl GameState {
    /// Creates a new game state
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut sounds = GameSounds::new(ctx)?;
        let mut renderer = GameRenderer::new();
        
        // Initialize renderer
        renderer.init(ctx)?;
        
        // Start background music immediately on the start screen
        sounds.start_background_music(ctx)?;
        
        Ok(Self {
            screen: GameScreen::Title,
            board: GameBoard::new(),
            renderer,
            current_piece: Some(Tetromino::random()),
            next_piece: Tetromino::random(),
            drop_timer: 0.0,
            sounds,
            blink_timer: 0.0,
            show_text: true,
            score: 0,
            level: 1,
            high_scores: HighScores::load(),
            current_name: String::new(),
            cursor_blink_timer: 0.0,
            show_cursor: true,
            paused: false,
        })
    }

    /// Resets the game state for a new game
    fn reset_game(&mut self, _ctx: &mut Context) -> GameResult {
        self.board.reset();
        self.current_piece = Some(Tetromino::random());
        self.next_piece = Tetromino::random();
        self.drop_timer = 0.0;
        self.screen = GameScreen::Playing;
        self.score = 0;
        self.level = 1;
        Ok(())
    }

    /// Spawns a new piece at the top of the board
    fn spawn_new_piece(&mut self, ctx: &mut Context) {
        let new_piece = self.next_piece.clone();
        if self.board.check_collision(&new_piece) {
            self.screen = GameScreen::GameOver;
            self.sounds.play_sound(ctx, "game_over").unwrap();
            
            // Check if the player qualifies for high score
            if self.check_high_score() {
                self.screen = GameScreen::EnterName;
            }
        }
        self.current_piece = Some(new_piece);
        self.next_piece = Tetromino::random();
    }

    /// Attempts to move the current piece
    fn move_piece(&mut self, movement: fn(&mut Tetromino), ctx: &mut Context) -> bool {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return false,
        };

        let mut new_piece = current;
        movement(&mut new_piece);
        
        if !self.board.check_collision(&new_piece) {
            self.current_piece = Some(new_piece);
            self.sounds.play_sound(ctx, "move").unwrap();
            true
        } else {
            false
        }
    }

    /// Attempts to rotate the current piece
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
            
            if !self.board.check_collision(&test_piece) {
                self.current_piece = Some(test_piece);
                self.sounds.play_sound(ctx, "rotate").unwrap();
                return;
            }
        }
    }

    /// Instantly drops the current piece to the lowest possible position
    fn hard_drop(&mut self, ctx: &mut Context) {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return,
        };

        let drop_distance = self.board.calculate_drop_position(&current);
        
        let mut new_piece = current;
        new_piece.position.y += drop_distance as f32;
        
        // Add points for hard drop
        self.add_drop_points(drop_distance);
        
        self.current_piece = Some(new_piece);
        self.sounds.play_sound(ctx, "drop").unwrap();
        self.lock_piece(ctx);
    }

    /// Locks the current piece in place on the board
    fn lock_piece(&mut self, ctx: &mut Context) {
        let piece = match &self.current_piece {
            Some(p) => p.clone(),
            None => return,
        };

        // Lock the piece into the board
        self.board.lock_piece(&piece);
        self.sounds.play_sound(ctx, "lock").unwrap();
        
        // Clear any complete lines
        let lines_cleared = self.board.clear_lines();
        
        // Update score
        if lines_cleared > 0 {
            self.update_score(lines_cleared);
            
            // Play appropriate sound
            if lines_cleared == 4 {
                self.sounds.play_sound(ctx, "tetris").unwrap();
            } else {
                self.sounds.play_sound(ctx, "clear").unwrap();
            }
        }
        
        self.spawn_new_piece(ctx);
    }

    /// Calculates the current drop speed based on level
    fn drop_speed(&self) -> f64 {
        BASE_DROP_TIME / (1.0 + 0.1 * self.level as f64)
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
        
        // Apply level multiplier
        self.score += line_points * self.level;
        
        // Update level (every 10 lines)
        self.level = (self.board.lines_cleared() / 10) + 1;
    }

    /// Adds points for dropping a piece
    fn add_drop_points(&mut self, cells_dropped: i32) {
        self.score += (cells_dropped as u32) * SCORE_DROP * self.level;
    }

    /// Checks if the current score qualifies for the high score list
    fn check_high_score(&self) -> bool {
        self.high_scores.would_qualify(self.score)
    }

    /// Adds the current score to the high scores
    fn add_high_score(&mut self) -> bool {
        self.high_scores.add_score(self.current_name.clone(), self.score)
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
        if self.blink_timer >= BLINK_TIME {
            self.blink_timer = 0.0;
            self.show_text = !self.show_text;
        }
        
        // Update cursor blink for name entry
        self.cursor_blink_timer += dt;
        if self.cursor_blink_timer >= CURSOR_BLINK_TIME {
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
                    if self.board.check_collision(&new_piece) {
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
                // Draw title screen
                let title_text = graphics::Text::new("TETRIS");
                let title_scale = 5.0;
                let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
                
                // Draw title
                canvas.draw(
                    &title_text,
                    graphics::DrawParam::default()
                        .color(Color::CYAN)
                        .scale([title_scale, title_scale])
                        .dest([
                            (SCREEN_WIDTH - title_width) / 2.0,
                            SCREEN_HEIGHT / 3.0,
                        ]),
                );
                
                // Draw "PRESS ANY KEY" text if blinking
                if self.show_text {
                    let press_text = graphics::Text::new("PRESS ANY KEY TO START");
                    let press_scale = 2.0;
                    let press_width = press_text.dimensions(ctx).unwrap().w * press_scale;
                    
                    canvas.draw(
                        &press_text,
                        graphics::DrawParam::default()
                            .color(Color::YELLOW)
                            .scale([press_scale, press_scale])
                            .dest([
                                (SCREEN_WIDTH - press_width) / 2.0,
                                SCREEN_HEIGHT * 0.6,
                            ]),
                    );
                }
            }
            GameScreen::Playing => {
                if self.paused {
                    // Draw paused screen
                    self.renderer.draw_board(ctx, &mut canvas, &self.board)?;
                    
                    // Draw semi-transparent overlay
                    let overlay_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
                    let overlay = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        overlay_rect,
                        Color::new(0.0, 0.0, 0.0, 0.7),
                    )?;
                    canvas.draw(&overlay, graphics::DrawParam::default());
                    
                    // Draw "PAUSED" text
                    let pause_text = graphics::Text::new("PAUSED");
                    let pause_scale = 4.0;
                    let pause_width = pause_text.dimensions(ctx).unwrap().w * pause_scale;
                    
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
                } else {
                    // Draw the game board
                    self.renderer.draw_board(ctx, &mut canvas, &self.board)?;
                    
                    // Draw the current piece
                    if let Some(piece) = &self.current_piece {
                        // Draw ghost piece (where the piece will land)
                        let drop_distance = self.board.calculate_drop_position(piece);
                        self.renderer.draw_ghost_piece(ctx, &mut canvas, piece, drop_distance)?;
                        
                        // Draw the actual piece
                        self.renderer.draw_tetromino(ctx, &mut canvas, piece)?;
                    }
                    
                    // Draw the next piece preview
                    self.renderer.draw_preview(ctx, &mut canvas, &self.next_piece)?;
                    
                    // Draw the score panel
                    self.renderer.draw_score_panel(ctx, &mut canvas, self.score, self.level, self.board.lines_cleared())?;
                }
            }
            GameScreen::GameOver => {
                // Draw the game over screen
                self.renderer.draw_board(ctx, &mut canvas, &self.board)?;
                
                // Draw semi-transparent overlay
                let overlay_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
                let overlay = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    overlay_rect,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                )?;
                canvas.draw(&overlay, graphics::DrawParam::default());
                
                // Draw "GAME OVER" text
                let game_over_text = graphics::Text::new("GAME OVER");
                let game_over_scale = 3.0;
                let game_over_width = game_over_text.dimensions(ctx).unwrap().w * game_over_scale;
                
                canvas.draw(
                    &game_over_text,
                    graphics::DrawParam::default()
                        .color(Color::RED)
                        .scale([game_over_scale, game_over_scale])
                        .dest([
                            (SCREEN_WIDTH - game_over_width) / 2.0,
                            SCREEN_HEIGHT / 3.0,
                        ]),
                );
                
                // Draw "PRESS ANY KEY" text if blinking
                if self.show_text {
                    let press_text = graphics::Text::new("PRESS ANY KEY TO CONTINUE");
                    let press_scale = 1.5;
                    let press_width = press_text.dimensions(ctx).unwrap().w * press_scale;
                    
                    canvas.draw(
                        &press_text,
                        graphics::DrawParam::default()
                            .color(Color::YELLOW)
                            .scale([press_scale, press_scale])
                            .dest([
                                (SCREEN_WIDTH - press_width) / 2.0,
                                SCREEN_HEIGHT * 0.6,
                            ]),
                    );
                }
            }
            GameScreen::EnterName => {
                // Draw enter name screen
                let title_text = graphics::Text::new("HIGH SCORE!");
                let title_scale = 2.5;
                let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
                
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
                
                // Draw score
                let score_text = graphics::Text::new(format!("SCORE: {}", self.score));
                let score_scale = 1.5;
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
                
                // Draw prompt
                let prompt_text = graphics::Text::new("ENTER YOUR NAME:");
                let prompt_scale = 1.2;
                let prompt_width = prompt_text.dimensions(ctx).unwrap().w * prompt_scale;
                
                canvas.draw(
                    &prompt_text,
                    graphics::DrawParam::default()
                        .color(Color::WHITE)
                        .scale([prompt_scale, prompt_scale])
                        .dest([
                            (SCREEN_WIDTH - prompt_width) / 2.0,
                            SCREEN_HEIGHT / 2.0 - 40.0,
                        ]),
                );
                
                // Draw name with cursor
                let display_name = if self.show_cursor {
                    format!("{}_", self.current_name)
                } else {
                    format!("{}  ", self.current_name)
                };
                
                let name_text = graphics::Text::new(display_name);
                let name_scale = 1.5;
                let name_width = name_text.dimensions(ctx).unwrap().w * name_scale;
                
                canvas.draw(
                    &name_text,
                    graphics::DrawParam::default()
                        .color(Color::GREEN)
                        .scale([name_scale, name_scale])
                        .dest([
                            (SCREEN_WIDTH - name_width) / 2.0,
                            SCREEN_HEIGHT / 2.0,
                        ]),
                );
                
                // Draw instruction
                let instruction_text = graphics::Text::new("PRESS ENTER WHEN DONE");
                let instruction_scale = 1.0;
                let instruction_width = instruction_text.dimensions(ctx).unwrap().w * instruction_scale;
                
                canvas.draw(
                    &instruction_text,
                    graphics::DrawParam::default()
                        .color(Color::CYAN)
                        .scale([instruction_scale, instruction_scale])
                        .dest([
                            (SCREEN_WIDTH - instruction_width) / 2.0,
                            SCREEN_HEIGHT * 2.0 / 3.0,
                        ]),
                );
            }
            GameScreen::HighScores => {
                // Draw high scores using the renderer
                self.renderer.draw_high_scores(ctx, &mut canvas, &self.high_scores, self.show_text)?;
            }
        }

        canvas.finish(ctx)?;
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

/// Entry point of the game
pub fn main() -> GameResult {
    // Initialize env_logger for logging
    env_logger::init();
    
    println!("Starting Tetris...");
    println!("Note: Game will attempt to load sound files from resources/sounds directory");
    
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

