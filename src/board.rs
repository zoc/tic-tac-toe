#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won { winner: Player, positions: [usize; 3] },
    Draw,
}

pub struct Game {
    cells: [Option<Player>; 9],
    current_player: Player,
    status: GameStatus,
}

impl Game {
    pub fn new() -> Game {
        todo!()
    }

    pub fn make_move(&mut self, _position: usize) -> Result<(), String> {
        todo!()
    }

    pub fn cells(&self) -> &[Option<Player>; 9] {
        &self.cells
    }

    pub fn current_player(&self) -> Player {
        self.current_player
    }

    pub fn status(&self) -> &GameStatus {
        &self.status
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
        // A well-known draw sequence:
        // X O X
        // X X O
        // O X O
        game.make_move(0).unwrap(); // X at 0
        game.make_move(1).unwrap(); // O at 1
        game.make_move(2).unwrap(); // X at 2
        game.make_move(4).unwrap(); // O at 4  -- wait, let me construct a proper draw
        // Let me use: X(0), O(4), X(3), O(6), X(4) -- no, 4 taken
        // Classic draw: X(0) O(4) X(8) O(2) X(6) O(3) X(5) O(7) X(1)
        // Actually, let me restart with a fresh game
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
