#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    /// Returns the other player.
    fn opponent(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won {
        winner: Player,
        positions: [usize; 3],
    },
    Draw,
}

/// All eight winning lines on a 3x3 board (row-major indices 0-8).
const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2], // top row
    [3, 4, 5], // middle row
    [6, 7, 8], // bottom row
    [0, 3, 6], // left column
    [1, 4, 7], // middle column
    [2, 5, 8], // right column
    [0, 4, 8], // diagonal
    [2, 4, 6], // anti-diagonal
];

pub struct Game {
    cells: [Option<Player>; 9],
    current_player: Player,
    status: GameStatus,
}

impl Game {
    /// Creates a new game with an empty board and X going first.
    pub fn new() -> Game {
        Game {
            cells: [None; 9],
            current_player: Player::X,
            status: GameStatus::InProgress,
        }
    }

    /// Creates a game from a given state — used for testing.
    #[cfg(test)]
    pub fn from_state(cells: [Option<Player>; 9], current_player: Player) -> Game {
        let mut game = Game {
            cells,
            current_player,
            status: GameStatus::InProgress,
        };
        game.update_status();
        game
    }

    /// Attempts to place the current player's piece at `position` (0-8).
    ///
    /// Returns `Err` if:
    /// - position is out of bounds (>= 9)
    /// - cell is already occupied
    /// - game is already over
    pub fn make_move(&mut self, position: usize) -> Result<(), String> {
        if !matches!(self.status, GameStatus::InProgress) {
            return Err("Game is already over".to_string());
        }
        if position >= 9 {
            return Err(format!(
                "Position {} is out of bounds (must be 0-8)",
                position
            ));
        }
        if self.cells[position].is_some() {
            return Err(format!("Cell {} is already occupied", position));
        }

        self.cells[position] = Some(self.current_player);
        let last_player = self.current_player;
        self.current_player = self.current_player.opponent();
        self.update_status_for(last_player);
        Ok(())
    }

    /// Read access to the board cells.
    pub fn cells(&self) -> &[Option<Player>; 9] {
        &self.cells
    }

    /// Returns whose turn it is.
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    /// Returns the current game status.
    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    /// Checks all win lines for the player who just moved, then checks for draw.
    fn update_status_for(&mut self, last_player: Player) {
        for line in &WIN_LINES {
            if self.cells[line[0]] == Some(last_player)
                && self.cells[line[1]] == Some(last_player)
                && self.cells[line[2]] == Some(last_player)
            {
                self.status = GameStatus::Won {
                    winner: last_player,
                    positions: *line,
                };
                return;
            }
        }

        // Check for draw: all cells filled, no winner
        if self.cells.iter().all(|c| c.is_some()) {
            self.status = GameStatus::Draw;
        }
        // Otherwise remains InProgress
    }

    /// General status update (used by from_state).
    #[cfg(test)]
    fn update_status(&mut self) {
        // Check both players
        for player in &[Player::X, Player::O] {
            for line in &WIN_LINES {
                if self.cells[line[0]] == Some(*player)
                    && self.cells[line[1]] == Some(*player)
                    && self.cells[line[2]] == Some(*player)
                {
                    self.status = GameStatus::Won {
                        winner: *player,
                        positions: *line,
                    };
                    return;
                }
            }
        }
        if self.cells.iter().all(|c| c.is_some()) {
            self.status = GameStatus::Draw;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_empty_board_and_x_starts() {
        let game = Game::new();
        for cell in game.cells().iter() {
            assert_eq!(*cell, None, "New game should have all empty cells");
        }
        assert_eq!(game.current_player(), Player::X, "X should go first");
        assert_eq!(*game.status(), GameStatus::InProgress);
    }

    #[test]
    fn test_make_move_places_piece_and_switches_player() {
        let mut game = Game::new();
        assert!(game.make_move(0).is_ok());
        assert_eq!(game.cells()[0], Some(Player::X));
        assert_eq!(game.current_player(), Player::O);
    }

    #[test]
    fn test_move_on_occupied_cell_returns_error() {
        let mut game = Game::new();
        game.make_move(0).unwrap();
        let result = game.make_move(0);
        assert!(result.is_err(), "Should not allow move on occupied cell");
        // Board should not have changed
        assert_eq!(game.cells()[0], Some(Player::X));
        assert_eq!(game.current_player(), Player::O);
    }

    #[test]
    fn test_move_after_game_over_returns_error() {
        let mut game = Game::new();
        // X wins top row: X(0), O(3), X(1), O(4), X(2)
        game.make_move(0).unwrap(); // X
        game.make_move(3).unwrap(); // O
        game.make_move(1).unwrap(); // X
        game.make_move(4).unwrap(); // O
        game.make_move(2).unwrap(); // X wins
        assert!(matches!(game.status(), GameStatus::Won { .. }));
        let result = game.make_move(5);
        assert!(result.is_err(), "Should not allow move after game over");
    }

    #[test]
    fn test_top_row_win() {
        let mut game = Game::new();
        // X: 0, 1, 2  O: 3, 4
        game.make_move(0).unwrap(); // X
        game.make_move(3).unwrap(); // O
        game.make_move(1).unwrap(); // X
        game.make_move(4).unwrap(); // O
        game.make_move(2).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [0, 1, 2]
            }
        );
    }

    #[test]
    fn test_middle_row_win() {
        let mut game = Game::new();
        // X: 3, 4, 5  O: 0, 1
        game.make_move(3).unwrap(); // X
        game.make_move(0).unwrap(); // O
        game.make_move(4).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(5).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [3, 4, 5]
            }
        );
    }

    #[test]
    fn test_bottom_row_win() {
        let mut game = Game::new();
        // X: 6, 7, 8  O: 0, 1
        game.make_move(6).unwrap(); // X
        game.make_move(0).unwrap(); // O
        game.make_move(7).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(8).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [6, 7, 8]
            }
        );
    }

    #[test]
    fn test_left_column_win() {
        let mut game = Game::new();
        // X: 0, 3, 6  O: 1, 2
        game.make_move(0).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(3).unwrap(); // X
        game.make_move(2).unwrap(); // O
        game.make_move(6).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [0, 3, 6]
            }
        );
    }

    #[test]
    fn test_middle_column_win() {
        let mut game = Game::new();
        // X: 1, 4, 7  O: 0, 2
        game.make_move(1).unwrap(); // X
        game.make_move(0).unwrap(); // O
        game.make_move(4).unwrap(); // X
        game.make_move(2).unwrap(); // O
        game.make_move(7).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [1, 4, 7]
            }
        );
    }

    #[test]
    fn test_right_column_win() {
        let mut game = Game::new();
        // X: 2, 5, 8  O: 0, 1
        game.make_move(2).unwrap(); // X
        game.make_move(0).unwrap(); // O
        game.make_move(5).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(8).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [2, 5, 8]
            }
        );
    }

    #[test]
    fn test_diagonal_win() {
        let mut game = Game::new();
        // X: 0, 4, 8  O: 1, 2
        game.make_move(0).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(4).unwrap(); // X
        game.make_move(2).unwrap(); // O
        game.make_move(8).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [0, 4, 8]
            }
        );
    }

    #[test]
    fn test_anti_diagonal_win() {
        let mut game = Game::new();
        // X: 2, 4, 6  O: 0, 1
        game.make_move(2).unwrap(); // X
        game.make_move(0).unwrap(); // O
        game.make_move(4).unwrap(); // X
        game.make_move(1).unwrap(); // O
        game.make_move(6).unwrap(); // X wins
        assert_eq!(
            *game.status(),
            GameStatus::Won {
                winner: Player::X,
                positions: [2, 4, 6]
            }
        );
    }

    #[test]
    fn test_draw_detection() {
        let mut game = Game::new();
        // Board layout for a draw:
        // X O X
        // X X O
        // O X O
        // Moves: X(0), O(1), X(2), O(5), X(3), O(6), X(4), O(8), X(7)
        game.make_move(0).unwrap(); // X at 0
        game.make_move(1).unwrap(); // O at 1
        game.make_move(2).unwrap(); // X at 2
        game.make_move(5).unwrap(); // O at 5
        game.make_move(3).unwrap(); // X at 3
        game.make_move(6).unwrap(); // O at 6
        game.make_move(4).unwrap(); // X at 4
        game.make_move(8).unwrap(); // O at 8
        game.make_move(7).unwrap(); // X at 7
                                    // Board: X O X / X X O / O X O  -- all filled, no winner
        assert_eq!(*game.status(), GameStatus::Draw);
    }

    #[test]
    fn test_in_progress_with_empty_cells_and_no_winner() {
        let mut game = Game::new();
        game.make_move(0).unwrap(); // X at 0
        game.make_move(4).unwrap(); // O at 4
        assert_eq!(*game.status(), GameStatus::InProgress);
    }

    #[test]
    fn test_move_out_of_bounds() {
        let mut game = Game::new();
        let result = game.make_move(9);
        assert!(result.is_err(), "Position 9 should be out of bounds");
    }
}
