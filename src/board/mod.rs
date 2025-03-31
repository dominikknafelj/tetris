use ggez::graphics::Color;
use crate::constants::*;
use crate::tetromino::Tetromino;

/// Represents the game board
#[derive(Clone, Debug)]
pub struct GameBoard {
    // Using fixed-size array for better performance
    cells: [[Color; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
    lines_cleared: u32,
}

impl GameBoard {
    /// Creates a new empty game board
    pub fn new() -> Self {
        Self {
            cells: [[Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            lines_cleared: 0,
        }
    }
    
    /// Reset the board to empty state
    pub fn reset(&mut self) {
        self.cells = [[Color::BLACK; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        self.lines_cleared = 0;
    }
    
    /// Get a cell color at the specified position
    pub fn get_cell(&self, x: i32, y: i32) -> Option<Color> {
        if x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT {
            Some(self.cells[y as usize][x as usize])
        } else {
            None
        }
    }
    
    /// Set a cell color at the specified position
    pub fn set_cell(&mut self, x: i32, y: i32, color: Color) -> bool {
        if x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT {
            self.cells[y as usize][x as usize] = color;
            true
        } else {
            false
        }
    }
    
    /// Checks if a piece collides with the board boundaries or existing pieces
    pub fn check_collision(&self, piece: &Tetromino) -> bool {
        let piece_x = piece.position.x as i32;
        let piece_y = piece.position.y as i32;
        
        for (y, row) in piece.shape.iter().enumerate() {
            let board_y = piece_y + y as i32;
            
            // Quick boundary check for vertical
            if board_y >= GRID_HEIGHT {
                return true;
            }
            
            for (x, &cell) in row.iter().enumerate() {
                if !cell {
                    continue; // Skip empty cells for efficiency
                }
                
                let board_x = piece_x + x as i32;
                
                // Check for collisions with boundaries and existing pieces
                if board_x < 0 || board_x >= GRID_WIDTH || 
                   (board_y >= 0 && self.cells[board_y as usize][board_x as usize] != Color::BLACK) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Lock a piece into the board
    pub fn lock_piece(&mut self, piece: &Tetromino) {
        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    let board_x = piece.position.x as i32 + x as i32;
                    let board_y = piece.position.y as i32 + y as i32;
                    
                    if board_x >= 0 && board_x < GRID_WIDTH && board_y >= 0 && board_y < GRID_HEIGHT {
                        self.cells[board_y as usize][board_x as usize] = piece.color;
                    }
                }
            }
        }
    }
    
    /// Clear any complete lines and return the number of lines cleared
    pub fn clear_lines(&mut self) -> u32 {
        let mut lines_cleared = 0;
        let mut y = GRID_HEIGHT - 1;
        
        while y >= 0 {
            if self.is_line_complete(y) {
                // Remove the line
                self.remove_line(y);
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }
        
        self.lines_cleared += lines_cleared;
        lines_cleared
    }
    
    /// Check if a line is complete (all cells filled)
    fn is_line_complete(&self, y: i32) -> bool {
        if y < 0 || y >= GRID_HEIGHT {
            return false;
        }
        
        self.cells[y as usize].iter().all(|&cell| cell != Color::BLACK)
    }
    
    /// Remove a line and shift all lines above down
    fn remove_line(&mut self, y: i32) {
        if y < 0 || y >= GRID_HEIGHT {
            return;
        }
        
        // Shift lines down
        for y2 in (1..=y).rev() {
            self.cells[y2 as usize] = self.cells[(y2 - 1) as usize];
        }
        
        // Add empty line at top
        self.cells[0] = [Color::BLACK; GRID_WIDTH as usize];
    }
    
    /// Calculate the drop position for ghost piece
    pub fn calculate_drop_position(&self, piece: &Tetromino) -> i32 {
        let mut test_piece = piece.clone();
        let original_y = test_piece.position.y;
        
        // Move the piece down until it collides
        while !self.check_collision(&test_piece) {
            test_piece.position.y += 1.0;
        }
        
        // Move back up one step since we found collision
        test_piece.position.y -= 1.0;
        
        // Return the number of cells dropped
        (test_piece.position.y - original_y) as i32
    }
    
    /// Get the total number of lines cleared
    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }
    
    /// Access to the cells for rendering
    pub fn cells(&self) -> &[[Color; GRID_WIDTH as usize]; GRID_HEIGHT as usize] {
        &self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tetromino::{Tetromino, TetrominoType};
    use glam::Vec2;
    
    #[test]
    fn test_new_board() {
        let board = GameBoard::new();
        
        // Check all cells are black
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                assert_eq!(board.get_cell(x, y).unwrap(), Color::BLACK);
            }
        }
    }
    
    #[test]
    fn test_set_cell() {
        let mut board = GameBoard::new();
        let test_color = Color::RED;
        
        // Set a cell in the middle of the board
        assert!(board.set_cell(5, 5, test_color));
        assert_eq!(board.get_cell(5, 5).unwrap(), test_color);
        
        // Try setting an out-of-bounds cell
        assert!(!board.set_cell(GRID_WIDTH, 5, test_color));
        assert!(!board.set_cell(5, GRID_HEIGHT, test_color));
        assert!(!board.set_cell(-1, 5, test_color));
    }
    
    #[test]
    fn test_collision_detection() {
        let mut board = GameBoard::new();
        
        // Create a piece
        let mut piece = Tetromino::new(TetrominoType::I);
        
        // Test no collision at starting position
        assert!(!board.check_collision(&piece));
        
        // Test collision with bottom boundary
        piece.position = Vec2::new(5.0, GRID_HEIGHT as f32 - 1.0);
        assert!(board.check_collision(&piece));
        
        // Test collision with existing piece
        board.set_cell(5, 10, Color::RED);
        piece.position = Vec2::new(5.0, 9.0);
        assert!(board.check_collision(&piece));
    }
    
    #[test]
    fn test_line_clearing() {
        let mut board = GameBoard::new();
        
        // Fill a line
        for x in 0..GRID_WIDTH {
            board.set_cell(x, 10, Color::RED);
        }
        
        // Clear lines
        let lines = board.clear_lines();
        assert_eq!(lines, 1);
        
        // Check that the line is cleared
        for x in 0..GRID_WIDTH {
            assert_eq!(board.get_cell(x, 10).unwrap(), Color::BLACK);
        }
    }
} 