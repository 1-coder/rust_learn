use macroquad::prelude::*;
use macroquad::color::Color;

// --- Color Constants ---
// We define colors manually to avoid issues with library imports.
const COLOR_BG: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }; // Black
const COLOR_GRID: Color = Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }; // Dark Gray
const COLOR_UI_TEXT: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }; // White
const COLOR_GAMEOVER: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }; // Red

// --- Constants ---
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const CELL_SIZE: f32 = 36.0;

// Screen dimensions derived from board constants
const SCREEN_WIDTH: f32 = (BOARD_WIDTH as f32) * CELL_SIZE + 200.0; // Add space for UI
const SCREEN_HEIGHT: f32 = (BOARD_HEIGHT as f32) * CELL_SIZE;

// Center the board on the screen
const BOARD_X_OFFSET: f32 = (SCREEN_WIDTH - (BOARD_WIDTH as f32 * CELL_SIZE)) / 2.0;
const BOARD_Y_OFFSET: f32 = 0.0;

// Game timing (in seconds)
const FALL_DELAY: f64 = 0.5;

// --- SRS and Piece Data ---
// Base shapes for each tetromino at rotation 0
const BASE_SHAPES: [[[u8; 4]; 4]; 7] = [
    [[0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0]], // I
    [[0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // O
    [[0, 1, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // T
    [[0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // S
    [[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // Z
    [[1, 0, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // J
    [[0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // L
];

// SRS Kick Data for J, L, S, T, Z pieces.
// [start_rotation][kick_test_index] -> (x_offset, y_offset)
const KICK_DATA_JLSTZ: [[(isize, isize); 5]; 4] = [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)], // 0 -> 1
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],  // 1 -> 2
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],   // 2 -> 3
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // 3 -> 0
];

// SRS Kick Data for the I piece.
const KICK_DATA_I: [[(isize, isize); 5]; 4] = [
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],  // 0 -> 1
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],  // 1 -> 2
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],  // 2 -> 3
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],  // 3 -> 0
];

// Colors for each tetromino type (index + 1)
const TETROMINO_COLORS: [Color; 8] = [
    Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },   // 0: Gray
    Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },   // 1: Cyan
    Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },   // 2: Yellow
    Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0 },   // 3: Purple
    Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },   // 4: Green
    Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },   // 5: Red
    Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },   // 6: Blue
    Color { r: 1.0, g: 0.65, b: 0.0, a: 1.0 },  // 7: Orange
];

// --- Structs ---

#[derive(Clone, Copy)]
struct Piece {
    shape_type: usize,
    rotation: usize,
    x: isize,
    y: isize,
    shape: [[u8; 4]; 4],
}

impl Piece {
    fn new(shape_type: usize) -> Self {
        Piece {
            shape_type,
            rotation: 0,
            x: (BOARD_WIDTH / 2 - 2) as isize,
            y: 0,
            shape: BASE_SHAPES[shape_type],
        }
    }
}

struct GameState {
    board: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Piece,
    last_fall_time: f64,
    score: u32,
    game_over: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            board: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Piece::new(rand::gen_range(0, 7)),
            last_fall_time: get_time(),
            score: 0,
            game_over: false,
        }
    }

    fn reset(&mut self) {
        *self = GameState::new();
    }

    // Check if the piece's current position is valid
    fn is_valid_position(&self, piece: &Piece) -> bool {
        for y in 0..4 {
            for x in 0..4 {
                if piece.shape[y][x] != 0 {
                    let board_x = piece.x + x as isize;
                    let board_y = piece.y + y as isize;

                    // Check bounds
                    if board_x < 0 || board_x >= BOARD_WIDTH as isize || board_y >= BOARD_HEIGHT as isize {
                        return false;
                    }
                    
                    // Check collision with existing blocks (only if on board)
                    if board_y >= 0 && self.board[board_y as usize][board_x as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }
    
    // Lock the current piece onto the board
    fn lock_piece(&mut self) {
        for y in 0..4 {
            for x in 0..4 {
                if self.current_piece.shape[y][x] != 0 {
                    let board_x = self.current_piece.x + x as isize;
                    let board_y = self.current_piece.y + y as isize;
                    if board_y >= 0 {
                        self.board[board_y as usize][board_x as usize] = self.current_piece.shape_type as u8 + 1;
                    }
                }
            }
        }
        self.clear_lines();
        self.spawn_new_piece();
    }

    // Spawn a new piece, checking for game over
    fn spawn_new_piece(&mut self) {
        self.current_piece = Piece::new(rand::gen_range(0, 7));
        if !self.is_valid_position(&self.current_piece) {
            self.game_over = true;
        }
    }
    
    // Clear completed lines and update score
    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut y = BOARD_HEIGHT - 1;

        while y > 0 {
            let is_full = self.board[y].iter().all(|&cell| cell != 0);
            if is_full {
                lines_cleared += 1;
                // Move all lines above this one down
                for row in (1..=y).rev() {
                    self.board[row] = self.board[row - 1];
                }
                // Clear the top line
                self.board[0] = [0; BOARD_WIDTH];
            } else {
                y -= 1;
            }
        }
        
        // Update score
        self.score += match lines_cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
    }
}

// --- Rotation Logic ---
fn handle_rotation(game: &mut GameState, direction: isize) {
    let mut test_piece = game.current_piece;
    let new_rotation = (test_piece.rotation as isize + direction + 4) as usize % 4;

    // Mathematically rotate the shape
    let mut new_shape = [[0u8; 4]; 4];
    for y in 0..4 {
        for x in 0..4 {
            if test_piece.shape[y][x] != 0 {
                let (new_x, new_y) = if direction > 0 {
                    // Clockwise
                    (3 - y, x)
                } else {
                    // Counter-clockwise
                    (y, 3 - x)
                };
                new_shape[new_y][new_x] = 1;
            }
        }
    }
    test_piece.shape = new_shape;

    // Get the correct kick data table
    let kick_table = if test_piece.shape_type == 0 { // 'I' piece
        &KICK_DATA_I
    } else {
        &KICK_DATA_JLSTZ
    };

    // Determine which set of kicks to use based on rotation
    let kick_index = if direction > 0 {
        test_piece.rotation
    } else {
        new_rotation
    };
    
    // Perform the 5 kick tests
    for i in 0..5 {
        let kick = kick_table[kick_index][i];
        let mut final_piece = test_piece;
        
        let offset_x = if direction > 0 { kick.0 } else { -kick.0 };
        let offset_y = if direction > 0 { kick.1 } else { -kick.1 };

        final_piece.x += offset_x;
        final_piece.y -= offset_y; // SRS y-axis is inverted

        if game.is_valid_position(&final_piece) {
            game.current_piece = final_piece;
            game.current_piece.rotation = new_rotation;
            return; // Success!
        }
    }
}


// --- Main Game Loop ---

#[macroquad::main("Tetris")]
async fn main() {
    let mut game = GameState::new();

    loop {
        // --- Handle Input ---
        if !game.game_over {
            if is_key_pressed(KeyCode::Left) {
                let mut test_piece = game.current_piece;
                test_piece.x -= 1;
                if game.is_valid_position(&test_piece) {
                    game.current_piece.x -= 1;
                }
            }
            if is_key_pressed(KeyCode::Right) {
                let mut test_piece = game.current_piece;
                test_piece.x += 1;
                if game.is_valid_position(&test_piece) {
                    game.current_piece.x += 1;
                }
            }
            if is_key_pressed(KeyCode::Down) {
                 let mut test_piece = game.current_piece;
                test_piece.y += 1;
                if game.is_valid_position(&test_piece) {
                    game.current_piece.y += 1;
                }
            }
            if is_key_pressed(KeyCode::Up) {
                handle_rotation(&mut game, 1); // Clockwise
            }
            if is_key_pressed(KeyCode::Z) {
                handle_rotation(&mut game, -1); // Counter-clockwise
            }
            if is_key_pressed(KeyCode::Space) {
                while game.is_valid_position(&game.current_piece) {
                    game.current_piece.y += 1;
                }
                game.current_piece.y -= 1; // Go back to last valid position
                game.lock_piece();
            }
        } else {
            if is_key_pressed(KeyCode::Enter) {
                game.reset();
            }
        }

        // --- Update Game State (Gravity) ---
        if !game.game_over && get_time() - game.last_fall_time > FALL_DELAY {
            let mut test_piece = game.current_piece;
            test_piece.y += 1;
            if game.is_valid_position(&test_piece) {
                game.current_piece.y += 1;
            } else {
                game.lock_piece();
            }
            game.last_fall_time = get_time();
        }

        // --- Draw Everything ---
        clear_background(COLOR_BG);

        // Draw locked pieces on the board
        for (y, row) in game.board.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let color = if cell == 0 { COLOR_GRID } else { TETROMINO_COLORS[cell as usize] };
                draw_rectangle(
                    BOARD_X_OFFSET + x as f32 * CELL_SIZE,
                    BOARD_Y_OFFSET + y as f32 * CELL_SIZE,
                    CELL_SIZE - 1.0,
                    CELL_SIZE - 1.0,
                    color,
                );
            }
        }

        // Draw the current falling piece
        let piece_color = TETROMINO_COLORS[game.current_piece.shape_type + 1];
        for y in 0..4 {
            for x in 0..4 {
                if game.current_piece.shape[y][x] != 0 {
                    draw_rectangle(
                        BOARD_X_OFFSET + (game.current_piece.x + x as isize) as f32 * CELL_SIZE,
                        BOARD_Y_OFFSET + (game.current_piece.y + y as isize) as f32 * CELL_SIZE,
                        CELL_SIZE - 1.0,
                        CELL_SIZE - 1.0,
                        piece_color,
                    );
                }
            }
        }
        
        // Draw UI
        draw_text(&format!("Score: {}", game.score), 20.0, 40.0, 40.0, COLOR_UI_TEXT);

        if game.game_over {
            let text = "GAME OVER";
            let text_dims = measure_text(text, None, 50, 1.0);
            draw_text(text, SCREEN_WIDTH / 2.0 - text_dims.width / 2.0, SCREEN_HEIGHT / 2.0, 50.0, COLOR_GAMEOVER);

            let restart_text = "Press Enter to Restart";
            let restart_text_dims = measure_text(restart_text, None, 30, 1.0);
            draw_text(restart_text, SCREEN_WIDTH / 2.0 - restart_text_dims.width / 2.0, SCREEN_HEIGHT / 2.0 + 50.0, 30.0, COLOR_UI_TEXT);
        }

        next_frame().await
    }
}
