use ggez::{
    graphics::{self, Color, Drawable, DrawParam, Mesh, Rect, Text, Canvas},
    Context, GameResult,
};
use crate::constants::*;
use crate::tetromino::Tetromino;
use crate::board::GameBoard;
use crate::score::{HighScores, format_score};

/// Renderer for the game UI
pub struct GameRenderer {
    // Cached block meshes for better performance
    block_meshes: Vec<Option<Mesh>>,
}

impl GameRenderer {
    /// Create a new game renderer
    pub fn new() -> Self {
        Self {
            block_meshes: vec![None; 8],
        }
    }

    /// Initialize any needed rendering resources
    pub fn init(&mut self, ctx: &mut Context) -> GameResult {
        // Pre-create block meshes for common colors
        // This is a performance optimization to avoid creating meshes every frame
        let colors = [
            Color::from_rgb(0, 240, 240),    // Cyan (I piece)
            Color::from_rgb(240, 240, 0),    // Yellow (O piece)
            Color::from_rgb(160, 0, 240),    // Purple (T piece)
            Color::from_rgb(0, 240, 0),      // Green (S piece)
            Color::from_rgb(240, 0, 0),      // Red (Z piece)
            Color::from_rgb(0, 0, 240),      // Blue (J piece)
            Color::from_rgb(240, 160, 0),    // Orange (L piece)
            Color::WHITE,                    // Generic white
        ];
        
        for (i, &color) in colors.iter().enumerate() {
            self.block_meshes[i] = Some(self.create_block_mesh(ctx, color)?);
        }
        
        Ok(())
    }
    
    /// Create a mesh for a block with the given color
    fn create_block_mesh(&self, ctx: &Context, color: Color) -> GameResult<Mesh> {
        // Main block
        let block_rect = Rect::new(
            GRID_LINE_WIDTH, 
            GRID_LINE_WIDTH,
            GRID_SIZE - 2.0 * GRID_LINE_WIDTH, 
            GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
        );
        
        // Highlight and shadow colors
        let highlight_color = Color::new(
            f32::min(color.r + 0.2, 1.0),
            f32::min(color.g + 0.2, 1.0),
            f32::min(color.b + 0.2, 1.0),
            color.a,
        );
        
        let shadow_color = Color::new(
            f32::max(color.r - 0.3, 0.0),
            f32::max(color.g - 0.3, 0.0),
            f32::max(color.b - 0.3, 0.0),
            color.a,
        );
        
        // Create a mesh builder to combine all parts
        let mut mesh_builder = graphics::MeshBuilder::new();
        
        // Add the main block
        mesh_builder.rectangle(graphics::DrawMode::fill(), block_rect, color)?;
        
        // Add top highlight
        mesh_builder.rectangle(
            graphics::DrawMode::fill(),
            Rect::new(
                GRID_LINE_WIDTH,
                GRID_LINE_WIDTH,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                BLOCK_PADDING,
            ),
            highlight_color,
        )?;
        
        // Add left highlight
        mesh_builder.rectangle(
            graphics::DrawMode::fill(),
            Rect::new(
                GRID_LINE_WIDTH,
                GRID_LINE_WIDTH,
                BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
            ),
            highlight_color,
        )?;
        
        // Add bottom shadow
        mesh_builder.rectangle(
            graphics::DrawMode::fill(),
            Rect::new(
                GRID_LINE_WIDTH,
                GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
                BLOCK_PADDING,
            ),
            shadow_color,
        )?;
        
        // Add right shadow
        mesh_builder.rectangle(
            graphics::DrawMode::fill(),
            Rect::new(
                GRID_SIZE - GRID_LINE_WIDTH - BLOCK_PADDING,
                GRID_LINE_WIDTH,
                BLOCK_PADDING,
                GRID_SIZE - 2.0 * GRID_LINE_WIDTH,
            ),
            shadow_color,
        )?;
        
        // Build and return the mesh
        let mesh_data = mesh_builder.build();
        Ok(Mesh::from_data(ctx, mesh_data))
    }

    /// Get the cached mesh for a color if available, otherwise create it
    fn get_block_mesh(&mut self, ctx: &mut Context, color: Color) -> GameResult<Mesh> {
        // Check common colors first
        let color_index = if color == Color::from_rgb(0, 240, 240) {
            Some(0)
        } else if color == Color::from_rgb(240, 240, 0) {
            Some(1)
        } else if color == Color::from_rgb(160, 0, 240) {
            Some(2)
        } else if color == Color::from_rgb(0, 240, 0) {
            Some(3)
        } else if color == Color::from_rgb(240, 0, 0) {
            Some(4)
        } else if color == Color::from_rgb(0, 0, 240) {
            Some(5)
        } else if color == Color::from_rgb(240, 160, 0) {
            Some(6)
        } else if color == Color::WHITE {
            Some(7)
        } else {
            None
        };
        
        if let Some(idx) = color_index {
            if let Some(mesh) = &self.block_meshes[idx] {
                return Ok(mesh.clone());
            }
        }
        
        // If not cached, create a new mesh
        self.create_block_mesh(ctx, color)
    }
    
    /// Draw a single block at the specified position
    pub fn draw_block(&mut self, ctx: &mut Context, canvas: &mut Canvas, x: f32, y: f32, color: Color) -> GameResult {
        let mesh = self.get_block_mesh(ctx, color)?;
        canvas.draw(&mesh, DrawParam::default().dest([x, y]));
        Ok(())
    }
    
    /// Draw the game board
    pub fn draw_board(&mut self, ctx: &mut Context, canvas: &mut Canvas, board: &GameBoard) -> GameResult {
        // Draw the board background
        let board_bg = Rect::new(
            MARGIN - BORDER_WIDTH, 
            MARGIN - BORDER_WIDTH,
            GRID_SIZE * GRID_WIDTH as f32 + 2.0 * BORDER_WIDTH, 
            GRID_SIZE * GRID_HEIGHT as f32 + 2.0 * BORDER_WIDTH,
        );
        
        // Draw border
        let border_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            board_bg,
            Color::new(0.3, 0.3, 0.4, 1.0), // Dark border color
        )?;
        canvas.draw(&border_mesh, DrawParam::default());
        
        // Draw inner background
        let inner_bg = Rect::new(
            MARGIN, 
            MARGIN,
            GRID_SIZE * GRID_WIDTH as f32, 
            GRID_SIZE * GRID_HEIGHT as f32,
        );
        let inner_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_bg,
            Color::new(0.0, 0.0, 0.1, 1.0), // Very dark blue background
        )?;
        canvas.draw(&inner_mesh, DrawParam::default());
        
        // Draw grid lines
        for i in 0..=GRID_WIDTH {
            let x = MARGIN + i as f32 * GRID_SIZE;
            let line_mesh = Mesh::new_line(
                ctx,
                &[
                    [x, MARGIN],
                    [x, MARGIN + GRID_SIZE * GRID_HEIGHT as f32],
                ],
                GRID_LINE_WIDTH / 2.0,
                Color::new(0.2, 0.2, 0.3, 1.0), // Dark grid lines
            )?;
            canvas.draw(&line_mesh, DrawParam::default());
        }
        
        for i in 0..=GRID_HEIGHT {
            let y = MARGIN + i as f32 * GRID_SIZE;
            let line_mesh = Mesh::new_line(
                ctx,
                &[
                    [MARGIN, y],
                    [MARGIN + GRID_SIZE * GRID_WIDTH as f32, y],
                ],
                GRID_LINE_WIDTH / 2.0,
                Color::new(0.2, 0.2, 0.3, 1.0), // Dark grid lines
            )?;
            canvas.draw(&line_mesh, DrawParam::default());
        }
        
        // Draw the filled cells
        let cells = board.cells();
        for y in 0..GRID_HEIGHT as usize {
            for x in 0..GRID_WIDTH as usize {
                let color = cells[y][x];
                if color != Color::BLACK {
                    let block_x = MARGIN + x as f32 * GRID_SIZE;
                    let block_y = MARGIN + y as f32 * GRID_SIZE;
                    self.draw_block(ctx, canvas, block_x, block_y, color)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Draw a tetromino on the board
    pub fn draw_tetromino(&mut self, ctx: &mut Context, canvas: &mut Canvas, piece: &Tetromino) -> GameResult {
        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    let block_x = MARGIN + (piece.position.x as i32 + x as i32) as f32 * GRID_SIZE;
                    let block_y = MARGIN + (piece.position.y as i32 + y as i32) as f32 * GRID_SIZE;
                    
                    // Only draw blocks that are on the visible part of the board
                    if block_y >= MARGIN {
                        self.draw_block(ctx, canvas, block_x, block_y, piece.color)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Draw a ghost piece (preview of where the piece will land)
    pub fn draw_ghost_piece(&mut self, ctx: &mut Context, canvas: &mut Canvas, 
                          piece: &Tetromino, drop_distance: i32) -> GameResult {
        let ghost_color = Color::new(
            piece.color.r * 0.3, 
            piece.color.g * 0.3, 
            piece.color.b * 0.3, 
            0.5
        );
        
        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    let block_x = MARGIN + (piece.position.x as i32 + x as i32) as f32 * GRID_SIZE;
                    let block_y = MARGIN + (piece.position.y as i32 + y as i32 + drop_distance) as f32 * GRID_SIZE;
                    
                    // Only draw blocks that are on the visible part of the board
                    if block_y >= MARGIN {
                        // For ghost pieces, just draw an outline
                        let outline_rect = Rect::new(
                            block_x + GRID_LINE_WIDTH * 2.0, 
                            block_y + GRID_LINE_WIDTH * 2.0,
                            GRID_SIZE - 4.0 * GRID_LINE_WIDTH, 
                            GRID_SIZE - 4.0 * GRID_LINE_WIDTH,
                        );
                        
                        let outline_mesh = Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::stroke(GRID_LINE_WIDTH),
                            outline_rect,
                            ghost_color,
                        )?;
                        
                        canvas.draw(&outline_mesh, DrawParam::default());
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Draw the next piece preview
    pub fn draw_preview(&mut self, ctx: &mut Context, canvas: &mut Canvas, next_piece: &Tetromino) -> GameResult {
        // Draw preview box background
        let preview_bg = Rect::new(
            PREVIEW_X - GRID_SIZE,
            PREVIEW_Y - GRID_SIZE,
            GRID_SIZE * 6.0,
            GRID_SIZE * 6.0,
        );
        
        // Draw the outer frame
        let frame_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            preview_bg,
            Color::new(0.2, 0.2, 0.2, 1.0),
        )?;
        canvas.draw(&frame_mesh, DrawParam::default());

        // Draw the inner frame
        let inner_rect = Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 2.0,
            PREVIEW_Y - GRID_SIZE + GRID_LINE_WIDTH * 2.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 4.0,
        );
        let inner_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_rect,
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&inner_mesh, DrawParam::default());

        // Draw the main background
        let main_bg = Rect::new(
            PREVIEW_X - GRID_SIZE + GRID_LINE_WIDTH * 4.0,
            PREVIEW_Y - GRID_SIZE + GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
            GRID_SIZE * 6.0 - GRID_LINE_WIDTH * 8.0,
        );
        let main_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            main_bg,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )?;
        canvas.draw(&main_mesh, DrawParam::default());

        // Draw "NEXT" text
        let text = Text::new("NEXT");
        // Draw shadow
        canvas.draw(
            &text,
            DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.5))
                .dest([PREVIEW_X + 2.0, PREVIEW_Y - GRID_SIZE * 2.0 + 2.0]),
        );
        // Draw main text
        canvas.draw(
            &text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([PREVIEW_X, PREVIEW_Y - GRID_SIZE * 2.0]),
        );

        // Draw next piece
        let piece_width = next_piece.shape[0].len() as f32;
        let piece_height = next_piece.shape.len() as f32;
        let offset_x = (6.0 - piece_width) / 2.0;  // Center horizontally
        let offset_y = (6.0 - piece_height) / 2.0;  // Center vertically

        for (y, row) in next_piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell {
                    // Calculate position for preview block
                    let block_x = PREVIEW_X - GRID_SIZE + (x as f32 + offset_x) * GRID_SIZE;
                    let block_y = PREVIEW_Y - GRID_SIZE + (y as f32 + offset_y) * GRID_SIZE;
                    
                    self.draw_block(ctx, canvas, block_x, block_y, next_piece.color)?;
                }
            }
        }
        Ok(())
    }
    
    /// Draw the score and other game information
    pub fn draw_score_panel(&self, ctx: &mut Context, canvas: &mut Canvas, 
                        score: u32, level: u32, lines: u32) -> GameResult {
        // Panel position and dimensions
        let panel_x = PREVIEW_X - GRID_SIZE;
        let panel_y = PREVIEW_Y + GRID_SIZE * 6.0;
        let panel_width = GRID_SIZE * 6.0;
        
        // Background for score panel
        let panel_bg = Rect::new(
            panel_x,
            panel_y,
            panel_width,
            GRID_SIZE * 8.0,
        );
        
        // Draw border
        let border_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            panel_bg,
            Color::new(0.2, 0.2, 0.2, 1.0),
        )?;
        canvas.draw(&border_mesh, DrawParam::default());
        
        // Draw inner panel
        let inner_panel = Rect::new(
            panel_x + GRID_LINE_WIDTH * 2.0,
            panel_y + GRID_LINE_WIDTH * 2.0,
            panel_width - GRID_LINE_WIDTH * 4.0,
            GRID_SIZE * 8.0 - GRID_LINE_WIDTH * 4.0,
        );
        
        let inner_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            inner_panel,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )?;
        canvas.draw(&inner_mesh, DrawParam::default());
        
        // Helper function to draw text with shadow
        let draw_text_with_shadow = |text: &str, x: f32, y: f32, color: Color, scale: f32, canvas: &mut Canvas| {
            let text_obj = Text::new(text);
            
            // Draw shadow
            canvas.draw(
                &text_obj,
                DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([scale, scale])
                    .dest([x + 2.0, y + 2.0]),
            );
            
            // Draw text
            canvas.draw(
                &text_obj,
                DrawParam::default()
                    .color(color)
                    .scale([scale, scale])
                    .dest([x, y]),
            );
        };
        
        // Draw score
        draw_text_with_shadow("SCORE", panel_x + 20.0, panel_y + 20.0, Color::WHITE, 1.0, canvas);
        draw_text_with_shadow(
            &format_score(score), 
            panel_x + 20.0, 
            panel_y + 50.0, 
            Color::YELLOW, 
            1.2, 
            canvas
        );
        
        // Draw level
        draw_text_with_shadow("LEVEL", panel_x + 20.0, panel_y + 100.0, Color::WHITE, 1.0, canvas);
        draw_text_with_shadow(
            &level.to_string(), 
            panel_x + 20.0, 
            panel_y + 130.0, 
            Color::GREEN, 
            1.2, 
            canvas
        );
        
        // Draw lines
        draw_text_with_shadow("LINES", panel_x + 20.0, panel_y + 180.0, Color::WHITE, 1.0, canvas);
        draw_text_with_shadow(
            &lines.to_string(), 
            panel_x + 20.0, 
            panel_y + 210.0, 
            Color::CYAN, 
            1.2, 
            canvas
        );
        
        // Draw controls reminder
        draw_text_with_shadow("CONTROLS", panel_x + 20.0, panel_y + 260.0, Color::WHITE, 0.8, canvas);
        draw_text_with_shadow("↑ ROTATE", panel_x + 20.0, panel_y + 290.0, Color::CYAN, 0.7, canvas);
        draw_text_with_shadow("← → MOVE", panel_x + 20.0, panel_y + 310.0, Color::CYAN, 0.7, canvas);
        draw_text_with_shadow("↓ SOFT DROP", panel_x + 20.0, panel_y + 330.0, Color::CYAN, 0.7, canvas);
        draw_text_with_shadow("SPACE HARD DROP", panel_x + 20.0, panel_y + 350.0, Color::CYAN, 0.7, canvas);
        draw_text_with_shadow("P PAUSE", panel_x + 20.0, panel_y + 370.0, Color::CYAN, 0.7, canvas);
        
        Ok(())
    }
    
    /// Draw high scores screen
    pub fn draw_high_scores(&self, ctx: &mut Context, canvas: &mut Canvas, 
                         high_scores: &HighScores, show_text: bool) -> GameResult {
        // Draw title
        let title_text = Text::new("HIGH SCORES");
        let title_scale = 2.0;
        let title_width = title_text.dimensions(ctx).unwrap().w * title_scale;
        
        // Draw shadow
        canvas.draw(
            &title_text,
            DrawParam::default()
                .color(Color::new(0.0, 0.0, 0.0, 0.6))
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0 + 2.0,
                    MARGIN + 2.0,
                ]),
        );
        
        // Draw title
        canvas.draw(
            &title_text,
            DrawParam::default()
                .color(Color::YELLOW)
                .scale([title_scale, title_scale])
                .dest([
                    (SCREEN_WIDTH - title_width) / 2.0,
                    MARGIN,
                ]),
        );
        
        // Draw column headers
        let text_scale = 1.2;
        let rank_x = SCREEN_WIDTH * 0.25;
        let name_x = SCREEN_WIDTH * 0.45;
        let score_x = SCREEN_WIDTH * 0.75;
        let y_pos = MARGIN + 80.0;
        let line_height = 40.0;
        
        let header_color = Color::new(0.7, 0.9, 1.0, 1.0); // Light blue for headers
        
        // Helper to draw text with shadow for headers
        let draw_header = |text: &str, x: f32, align: f32, color: Color, canvas: &mut Canvas| {
            let text_obj = Text::new(text);
            let text_width = text_obj.dimensions(ctx).unwrap().w * text_scale;
            
            // Draw shadow
            canvas.draw(
                &text_obj,
                DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([text_scale, text_scale])
                    .dest([x - text_width * align + 2.0, y_pos - line_height + 2.0]),
            );
            
            // Draw text
            canvas.draw(
                &text_obj,
                DrawParam::default()
                    .color(color)
                    .scale([text_scale, text_scale])
                    .dest([x - text_width * align, y_pos - line_height]),
            );
        };
        
        // Draw headers
        draw_header("RANK", rank_x, 0.5, header_color, canvas);
        draw_header("NAME", name_x, 0.0, header_color, canvas);
        draw_header("SCORE", score_x, 1.0, header_color, canvas);
        
        // Draw separator line
        let line_y = y_pos - line_height / 2.0 + 10.0;
        let line_mesh = Mesh::new_line(
            ctx,
            &[
                [SCREEN_WIDTH * 0.1, line_y],
                [SCREEN_WIDTH * 0.9, line_y],
            ],
            2.0,
            header_color,
        )?;
        canvas.draw(&line_mesh, DrawParam::default());
        
        // Draw each high score entry
        for (i, entry) in high_scores.entries().iter().enumerate() {
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
            
            // Helper to draw entry text with shadow
            let draw_entry = |text: &str, x: f32, align: f32, entry_color: Color, canvas: &mut Canvas| {
                let text_obj = Text::new(text);
                let text_width = text_obj.dimensions(ctx).unwrap().w * text_scale;
                
                // Draw shadow
                canvas.draw(
                    &text_obj,
                    DrawParam::default()
                        .color(Color::new(0.0, 0.0, 0.0, 0.6))
                        .scale([text_scale, text_scale])
                        .dest([x - text_width * align + 2.0, y_pos + i as f32 * line_height + 2.0]),
                );
                
                // Draw text
                canvas.draw(
                    &text_obj,
                    DrawParam::default()
                        .color(entry_color)
                        .scale([text_scale, text_scale])
                        .dest([x - text_width * align, y_pos + i as f32 * line_height]),
                );
            };
            
            // Draw rank (center-aligned)
            draw_entry(&format!("{}", rank), rank_x, 0.5, color, canvas);
            
            // Draw name (left-aligned)
            draw_entry(&entry.name, name_x, 0.0, color, canvas);
            
            // Draw score (right-aligned)
            draw_entry(&format_score(entry.score), score_x, 1.0, color, canvas);
        }
        
        // Draw "Press any key to continue" if needed
        if show_text {
            let press_text = Text::new("PRESS ANY KEY TO CONTINUE");
            let press_scale = 1.2;
            let press_width = press_text.dimensions(ctx).unwrap().w * press_scale;
            
            // Draw shadow
            canvas.draw(
                &press_text,
                DrawParam::default()
                    .color(Color::new(0.0, 0.0, 0.0, 0.6))
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0 + 2.0,
                        SCREEN_HEIGHT - MARGIN - 50.0 + 2.0,
                    ]),
            );
            
            // Draw text
            canvas.draw(
                &press_text,
                DrawParam::default()
                    .color(Color::YELLOW)
                    .scale([press_scale, press_scale])
                    .dest([
                        (SCREEN_WIDTH - press_width) / 2.0,
                        SCREEN_HEIGHT - MARGIN - 50.0,
                    ]),
            );
        }
        
        Ok(())
    }
}

impl Default for GameRenderer {
    fn default() -> Self {
        Self::new()
    }
} 