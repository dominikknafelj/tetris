mod tetromino;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use tetromino::Tetromino;

// Game constants
const GRID_SIZE: f32 = 30.0;      // Size of each grid cell in pixels
const GRID_WIDTH: i32 = 10;       // Width of the game board in cells
const GRID_HEIGHT: i32 = 20;      // Height of the game board in cells
const SCREEN_WIDTH: f32 = GRID_SIZE * GRID_WIDTH as f32;   // Total screen width
const SCREEN_HEIGHT: f32 = GRID_SIZE * GRID_HEIGHT as f32; // Total screen height
const DROP_TIME: f64 = 1.0;       // Time in seconds between automatic piece movements

/// Main game state that holds all the game data
struct GameState {
    board: Vec<Vec<Color>>,       // 2D grid representing the game board
    current_piece: Option<Tetromino>,  // Currently active piece
    game_over: bool,              // Whether the game has ended
    drop_timer: f64,              // Timer for automatic piece movement
}

impl GameState {
    /// Creates a new game state with an empty board and a random starting piece
    fn new() -> Self {
        Self {
            board: vec![vec![Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            current_piece: Some(Tetromino::random()),
            game_over: false,
            drop_timer: 0.0,
        }
    }

    /// Spawns a new piece at the top of the board
    /// If the new piece collides with existing pieces, the game is over
    fn spawn_new_piece(&mut self) {
        let new_piece = Tetromino::random();
        if self.check_collision(&new_piece) {
            self.game_over = true;
        }
        self.current_piece = Some(new_piece);
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

    /// Locks the current piece in place on the board
    /// This happens when a piece can't move down further
    fn lock_piece(&mut self) {
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
        self.clear_lines();
        self.spawn_new_piece();
    }

    /// Clears any completed lines from the board
    /// Completed lines are removed and all lines above are moved down
    fn clear_lines(&mut self) {
        let mut y = GRID_HEIGHT - 1;
        while y >= 0 {
            if self.board[y as usize].iter().all(|&cell| cell != Color::BLACK) {
                // Move all lines above the cleared line down
                for y2 in (1..=y).rev() {
                    self.board[y2 as usize] = self.board[(y2 - 1) as usize].clone();
                }
                // Add a new empty line at the top
                self.board[0] = vec![Color::BLACK; GRID_WIDTH as usize];
            } else {
                y -= 1;
            }
        }
    }

    /// Attempts to move the current piece using the provided movement function
    /// Returns true if the movement was successful, false if it caused a collision
    fn move_piece(&mut self, movement: fn(&mut Tetromino)) -> bool {
        let current = match &self.current_piece {
            Some(piece) => piece.clone(),
            None => return false,
        };

        let mut new_piece = current;
        movement(&mut new_piece);
        
        if !self.check_collision(&new_piece) {
            self.current_piece = Some(new_piece);
            true
        } else {
            false
        }
    }

    /// Attempts to rotate the current piece
    /// If the rotation would cause a collision, tries various offsets to make it fit
    fn try_rotate(&mut self) {
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
                return;
            }
        }
    }

    /// Instantly drops the current piece to the lowest possible position
    fn hard_drop(&mut self) {
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
        self.lock_piece();
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
            if !self.move_piece(Tetromino::move_down) {
                self.lock_piece();
            }
        }

        Ok(())
    }

    /// Renders the game state
    /// Draws both the game board and the current piece
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw the game board
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let color = self.board[y as usize][x as usize];
                let rect = graphics::Rect::new(
                    x as f32 * GRID_SIZE,
                    y as f32 * GRID_SIZE,
                    GRID_SIZE - 1.0,
                    GRID_SIZE - 1.0,
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

        // Draw the current piece
        if let Some(piece) = &self.current_piece {
            for (y, row) in piece.shape.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    if cell {
                        let rect = graphics::Rect::new(
                            (piece.position.x as f32 + x as f32) * GRID_SIZE,
                            (piece.position.y as f32 + y as f32) * GRID_SIZE,
                            GRID_SIZE - 1.0,
                            GRID_SIZE - 1.0,
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
        _ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        if self.game_over {
            return Ok(());
        }

        match input.keycode {
            Some(KeyCode::Left) => {
                self.move_piece(Tetromino::move_left);
            }
            Some(KeyCode::Right) => {
                self.move_piece(Tetromino::move_right);
            }
            Some(KeyCode::Down) => {
                if !self.move_piece(Tetromino::move_down) {
                    self.lock_piece();
                }
            }
            Some(KeyCode::Up) => {
                self.try_rotate();
            }
            Some(KeyCode::Space) => {
                self.hard_drop();
            }
            _ => {}
        }

        Ok(())
    }
}

/// Entry point of the game
fn main() -> GameResult {
    // Set up the game window
    let cb = ggez::ContextBuilder::new("tetris", "tetris")
        .window_setup(WindowSetup::default().title("Tetris"))
        .window_mode(
            WindowMode::default()
                .dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
                .resizable(false),
        );
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state)
}
