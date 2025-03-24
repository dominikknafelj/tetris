mod tetromino;
mod sound_tests;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color},
    input::keyboard::{KeyCode, KeyInput},
    audio::{self, SoundSource},
    Context, GameResult,
};
use tetromino::Tetromino;

// Game constants
const GRID_SIZE: f32 = 30.0;      // Size of each grid cell in pixels
const GRID_WIDTH: i32 = 10;       // Width of the game board in cells
const GRID_HEIGHT: i32 = 20;      // Height of the game board in cells
const MARGIN: f32 = 20.0;         // Margin between game field and window borders
const BORDER_WIDTH: f32 = 2.0;    // Width of the game field border
const PREVIEW_BOX_SIZE: f32 = 6.0;  // Size of the preview box in grid cells
const SCREEN_WIDTH: f32 = GRID_SIZE * (GRID_WIDTH as f32 + PREVIEW_BOX_SIZE + 3.0) + 2.0 * MARGIN;   // Total screen width including preview and margins
const SCREEN_HEIGHT: f32 = GRID_SIZE * GRID_HEIGHT as f32 + 2.0 * MARGIN; // Total screen height including margins
const DROP_TIME: f64 = 1.0;       // Time in seconds between automatic piece movements
const PREVIEW_X: f32 = GRID_SIZE * (GRID_WIDTH as f32 + 3.0) + MARGIN; // X position of preview box, with extra spacing
const PREVIEW_Y: f32 = GRID_SIZE * 2.0 + MARGIN;  // Y position of preview box

/// Sound effects for the game
struct GameSounds {
    move_sound: audio::Source,
    rotate_sound: audio::Source,
    drop_sound: audio::Source,
    clear_sound: audio::Source,
    tetris_sound: audio::Source,
    game_over_sound: audio::Source,
}

impl GameSounds {
    /// Loads all sound effects
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            move_sound: audio::Source::new(ctx, "/sounds/move.wav")?,
            rotate_sound: audio::Source::new(ctx, "/sounds/rotate.wav")?,
            drop_sound: audio::Source::new(ctx, "/sounds/drop.wav")?,
            clear_sound: audio::Source::new(ctx, "/sounds/clear.wav")?,
            tetris_sound: audio::Source::new(ctx, "/sounds/tetris.wav")?,
            game_over_sound: audio::Source::new(ctx, "/sounds/game_over.wav")?,
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
}

/// Main game state that holds all the game data
struct GameState {
    board: Vec<Vec<Color>>,       // 2D grid representing the game board
    current_piece: Option<Tetromino>,  // Currently active piece
    next_piece: Tetromino,        // Next piece to spawn
    game_over: bool,              // Whether the game has ended
    drop_timer: f64,              // Timer for automatic piece movement
    sounds: GameSounds,           // Game sound effects
}

impl GameState {
    /// Creates a new game state with an empty board and a random starting piece
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            board: vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            current_piece: Some(Tetromino::random()),
            next_piece: Tetromino::random(),
            game_over: false,
            drop_timer: 0.0,
            sounds: GameSounds::new(ctx)?,
        })
    }

    /// Spawns a new piece at the top of the board
    /// If the new piece collides with existing pieces, the game is over
    fn spawn_new_piece(&mut self, ctx: &mut Context) {
        let new_piece = self.next_piece.clone();
        if self.check_collision(&new_piece) {
            self.game_over = true;
            self.sounds.play_game_over(ctx).unwrap();
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

    /// Instantly drops the current piece to the lowest possible position
    fn hard_drop(&mut self, ctx: &mut Context) {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return,
        };

        let mut new_piece = current;
        while !self.check_collision(&new_piece) {
            new_piece.move_down();
        }
        // Move back up one step since we found collision
        new_piece.position.y -= 1.0;
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

    /// Clears completed lines and returns the number of lines cleared
    fn clear_lines(&mut self, ctx: &mut Context) -> i32 {
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

        // Play appropriate sound based on number of lines cleared
        if lines_cleared > 0 {
            if lines_cleared == 4 {
                self.sounds.play_tetris(ctx).unwrap();
            } else {
                self.sounds.play_clear(ctx).unwrap();
            }
        }

        lines_cleared
    }

    /// Draws the next piece preview
    fn draw_preview(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw preview box background with rounded corners
        let preview_bg = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE,
            PREVIEW_Y - GRID_SIZE,
            GRID_SIZE * 6.0,
            GRID_SIZE * 6.0,
        );
        
        // Draw the outer frame (darker)
        let frame_mesh = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            preview_bg,
            10.0,  // corner radius
            Color::new(0.2, 0.2, 0.2, 1.0),
        )?;
        canvas.draw(&frame_mesh, graphics::DrawParam::default());

        // Draw the inner frame (lighter)
        let inner_rect = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + 2.0,
            PREVIEW_Y - GRID_SIZE + 2.0,
            GRID_SIZE * 6.0 - 4.0,
            GRID_SIZE * 6.0 - 4.0,
        );
        let inner_mesh = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_rect,
            8.0,  // slightly smaller corner radius
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&inner_mesh, graphics::DrawParam::default());

        // Draw the main background (darkest)
        let main_bg = graphics::Rect::new(
            PREVIEW_X - GRID_SIZE + 4.0,
            PREVIEW_Y - GRID_SIZE + 4.0,
            GRID_SIZE * 6.0 - 8.0,
            GRID_SIZE * 6.0 - 8.0,
        );
        let main_mesh = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            main_bg,
            6.0,  // even smaller corner radius
            Color::new(0.1, 0.1, 0.1, 1.0),
        )?;
        canvas.draw(&main_mesh, graphics::DrawParam::default());

        // Draw "NEXT" text with a shadow
        let text = graphics::Text::new("NEXT");
        // Draw shadow
        canvas.draw(
            &text,
            graphics::DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.5))
                .dest([PREVIEW_X + 1.0, PREVIEW_Y - GRID_SIZE * 2.0 + 1.0]),
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
                    let rect = graphics::Rect::new(
                        PREVIEW_X - GRID_SIZE + (x as f32 + offset_x) * GRID_SIZE,
                        PREVIEW_Y - GRID_SIZE + (y as f32 + offset_y) * GRID_SIZE,
                        GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                        GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                    );
                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        self.next_piece.color,
                    )?;
                    canvas.draw(&mesh, graphics::DrawParam::default());
                }
            }
        }
        Ok(())
    }
}

/// Implementation of the game loop and event handling
impl event::EventHandler<ggez::GameError> for GameState {
    /// Updates the game state
    /// Handles automatic piece movement and game over state
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.game_over {
            return Ok(());
        }

        let dt = ctx.time.delta().as_secs_f64();
        self.drop_timer += dt;

        // Move the piece down automatically after DROP_TIME seconds
        if self.drop_timer >= DROP_TIME {
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

        Ok(())
    }

    /// Renders the game state
    /// Draws both the game board and the current piece
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

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

        // Draw the game board
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let color = self.board[y as usize][x as usize];
                if color != Color::BLACK {
                    let rect = graphics::Rect::new(
                        MARGIN + x as f32 * GRID_SIZE,
                        MARGIN + y as f32 * GRID_SIZE,
                        GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                        GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                    );
                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        color,
                    )?;
                    canvas.draw(&mesh, graphics::DrawParam::default());
                }
            }
        }

        // Draw the current piece
        if let Some(piece) = &self.current_piece {
            for (y, row) in piece.shape.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    if cell {
                        let rect = graphics::Rect::new(
                            MARGIN + (piece.position.x as i32 + x as i32) as f32 * GRID_SIZE,
                            MARGIN + (piece.position.y as i32 + y as i32) as f32 * GRID_SIZE,
                            GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                            GRID_SIZE - 1.0,  // Leave 1 pixel gap for grid lines
                        );
                        let mesh = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            rect,
                            piece.color,
                        )?;
                        canvas.draw(&mesh, graphics::DrawParam::default());
                    }
                }
            }
        }

        // Draw the next piece preview
        self.draw_preview(ctx, &mut canvas)?;

        // Draw game over text if the game is over
        if self.game_over {
            let game_over_text = graphics::Text::new("GAME OVER");
            canvas.draw(
                &game_over_text,
                graphics::DrawParam::default()
                    .dest([
                        MARGIN + (GRID_WIDTH as f32 * GRID_SIZE) / 2.0,
                        MARGIN + (GRID_HEIGHT as f32 * GRID_SIZE) / 2.0,
                    ])
                    .offset([0.5, 0.5])  // Center the text at its position
                    .color(Color::RED),
            );
        }

        canvas.finish(ctx)?;
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
        if self.game_over {
            return Ok(());
        }

        match input.keycode {
            Some(KeyCode::Left) => {
                self.move_piece(|p| p.position.x -= 1.0, ctx);
            }
            Some(KeyCode::Right) => {
                self.move_piece(|p| p.position.x += 1.0, ctx);
            }
            Some(KeyCode::Down) => {
                self.move_piece(|p| p.position.y += 1.0, ctx);
            }
            Some(KeyCode::Up) => {
                self.try_rotate(ctx);
            }
            Some(KeyCode::Space) => {
                self.hard_drop(ctx);
            }
            _ => {}
        }

        Ok(())
    }
}

/// Entry point of the game
pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("tetris", "ggez")
        .window_setup(WindowSetup::default().title("Tetris"))
        .window_mode(WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .add_resource_path("assets");

    let (mut ctx, event_loop) = cb.build()?;
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetromino::TetrominoType;

    #[test]
    fn test_next_piece_preview() {
        let next_piece = Tetromino::random();
        let piece_width = next_piece.shape[0].len() as f32;
        let piece_height = next_piece.shape.len() as f32;

        // Test piece dimensions
        assert!(matches!(piece_width, 2.0..=4.0));
        assert!(matches!(piece_height, 2.0..=4.0));

        // Test centering calculations
        let offset_x = (PREVIEW_BOX_SIZE - piece_width) / 2.0;
        let offset_y = (PREVIEW_BOX_SIZE - piece_height) / 2.0;

        // Verify offsets are within preview box bounds
        assert!(offset_x >= 0.0 && offset_x <= 2.0);
        assert!(offset_y >= 0.0 && offset_y <= 2.0);
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
    fn test_preview_box_position() {
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
}
