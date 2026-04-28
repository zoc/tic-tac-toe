use wasm_bindgen::prelude::*;
use crate::board::{Game, GameStatus, Player};
use crate::ai::get_computer_move;

/// Initialize panic hook for better error messages in browser console.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Opaque game handle exported to JavaScript (per D-01).
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
    difficulty: u8, // not exposed via getter — D-01; JS calls set_difficulty() to write (D-03)
}

#[wasm_bindgen]
impl WasmGame {
    /// Create a new game. X goes first.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame { inner: Game::new(), difficulty: 1 } // D-03: default Medium
    }

    /// Set the AI difficulty level (0=Easy, 1=Medium, 2=Hard, 3=Unbeatable).
    /// Pure setter — does NOT reset the board (D-01).
    /// Persists across reset() calls (D-02).
    pub fn set_difficulty(&mut self, level: u8) {
        self.difficulty = level;
    }

    /// Attempt to place the current player's piece at position (0-8).
    /// Returns true on success, false on failure.
    pub fn make_move(&mut self, position: usize) -> bool {
        self.inner.make_move(position).is_ok()
    }

    /// Get the board state as a flat array of 9 u8 values (per D-03):
    /// 0 = empty, 1 = X, 2 = O
    pub fn get_board(&self) -> Vec<u8> {
        self.inner.cells().iter().map(|cell| match cell {
            None => 0u8,
            Some(Player::X) => 1u8,
            Some(Player::O) => 2u8,
        }).collect()
    }

    /// Get the current player: 1 = X, 2 = O
    pub fn current_player(&self) -> u8 {
        match self.inner.current_player() {
            Player::X => 1,
            Player::O => 2,
        }
    }

    /// Get game status as a string (per D-02):
    /// "playing", "won", or "draw"
    pub fn get_status(&self) -> String {
        match self.inner.status() {
            GameStatus::InProgress => "playing".to_string(),
            GameStatus::Won { .. } => "won".to_string(),
            GameStatus::Draw => "draw".to_string(),
        }
    }

    /// Get the winner: 0 = no winner, 1 = X, 2 = O
    pub fn get_winner(&self) -> u8 {
        match self.inner.status() {
            GameStatus::Won { winner, .. } => match winner {
                Player::X => 1,
                Player::O => 2,
            },
            _ => 0,
        }
    }

    /// Get winning positions as a 3-element array, or empty vec if no winner.
    /// Returns the 3 cell indices (0-8) that form the winning line.
    pub fn get_winning_positions(&self) -> Vec<usize> {
        match self.inner.status() {
            GameStatus::Won { positions, .. } => positions.to_vec(),
            _ => vec![],
        }
    }

    /// Ask the AI to make a move. Returns the chosen position (0-8),
    /// or 255 if the game is already over.
    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner, self.difficulty) { // D-05
            Some(pos) => {
                let _ = self.inner.make_move(pos);
                pos as u8
            }
            None => 255,
        }
    }

    /// Reset the game to initial state.
    pub fn reset(&mut self) {
        self.inner = Game::new();
        // difficulty intentionally NOT reset — D-02; Phase 14 JS handles re-applying user choice
    }
}
