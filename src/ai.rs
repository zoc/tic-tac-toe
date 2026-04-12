use crate::board::{Game, GameStatus, Player};
use rand::RngExt;

/// Probability that the AI makes a random move instead of the optimal one.
const MISTAKE_RATE: f64 = 0.25;

/// Returns the computer's chosen move (0-8), or None if the game is already over.
pub fn get_computer_move(game: &Game) -> Option<usize> {
    // Return None if game is over
    if !matches!(game.status(), GameStatus::InProgress) {
        return None;
    }

    let cells = game.cells();
    let current = game.current_player();

    // Collect empty cell indices
    let empty: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_none())
        .map(|(i, _)| i)
        .collect();

    if empty.is_empty() {
        return None;
    }

    let mut rng = rand::rng();

    // With MISTAKE_RATE probability, pick a random empty cell
    if rng.random_bool(MISTAKE_RATE) {
        return Some(empty[rng.random_range(0..empty.len())]);
    }

    // Otherwise, run minimax to find the optimal move
    let mut best_score = i32::MIN;
    let mut best_move = empty[0];

    for &pos in &empty {
        let mut trial_cells = *cells;
        trial_cells[pos] = Some(current);

        let score = minimax(
            &trial_cells,
            opponent(current),
            false, // next turn is opponent's (minimizing)
            current,
            0,
        );

        if score > best_score {
            best_score = score;
            best_move = pos;
        }
    }

    Some(best_move)
}

/// Returns the opponent of the given player.
fn opponent(player: Player) -> Player {
    match player {
        Player::X => Player::O,
        Player::O => Player::X,
    }
}

/// Minimax algorithm for tic-tac-toe.
///
/// - `cells`: current board state
/// - `current`: whose turn it is at this node
/// - `is_maximizing`: true if this node is maximizing for `ai_player`
/// - `ai_player`: the player the AI is optimizing for
/// - `depth`: recursion depth (used to prefer faster wins)
fn minimax(
    cells: &[Option<Player>; 9],
    current: Player,
    is_maximizing: bool,
    ai_player: Player,
    depth: i32,
) -> i32 {
    // Check for terminal state
    if let Some(winner) = check_winner(cells) {
        return if winner == ai_player {
            10 - depth
        } else {
            depth - 10
        };
    }

    // Check for draw
    if cells.iter().all(|c| c.is_some()) {
        return 0;
    }

    let empty: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_none())
        .map(|(i, _)| i)
        .collect();

    if is_maximizing {
        let mut best = i32::MIN;
        for &pos in &empty {
            let mut trial = *cells;
            trial[pos] = Some(current);
            let score = minimax(&trial, opponent(current), false, ai_player, depth + 1);
            if score > best {
                best = score;
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        for &pos in &empty {
            let mut trial = *cells;
            trial[pos] = Some(current);
            let score = minimax(&trial, opponent(current), true, ai_player, depth + 1);
            if score < best {
                best = score;
            }
        }
        best
    }
}

/// Win lines for a 3x3 board.
const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// Checks if there is a winner on the board. Returns Some(Player) or None.
fn check_winner(cells: &[Option<Player>; 9]) -> Option<Player> {
    for line in &WIN_LINES {
        if let Some(player) = cells[line[0]] {
            if cells[line[1]] == Some(player) && cells[line[2]] == Some(player) {
                return Some(player);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_returns_valid_move() {
        // Early game: only one move made
        let mut game = Game::new();
        game.make_move(0).unwrap(); // X plays, now O's turn
        let mv = get_computer_move(&game);
        assert!(mv.is_some(), "AI should return a move");
        let pos = mv.unwrap();
        assert!(pos < 9, "Move should be in range 0-8");
        assert!(game.cells()[pos].is_none(), "AI should pick an empty cell");

        // Mid game: several moves made
        let mut game = Game::new();
        game.make_move(0).unwrap(); // X
        game.make_move(4).unwrap(); // O
        game.make_move(1).unwrap(); // X - now O's turn
        let mv = get_computer_move(&game);
        assert!(mv.is_some());
        let pos = mv.unwrap();
        assert!(pos < 9);
        assert!(game.cells()[pos].is_none());

        // Near-end game: one empty cell
        let cells = [
            Some(Player::X),
            Some(Player::O),
            Some(Player::X),
            Some(Player::X),
            Some(Player::X),
            Some(Player::O),
            Some(Player::O),
            None,
            Some(Player::O),
        ];
        let game = Game::from_state(cells, Player::X);
        let mv = get_computer_move(&game);
        assert!(mv.is_some());
        assert_eq!(mv.unwrap(), 7, "Only empty cell is position 7");
    }

    #[test]
    fn test_ai_returns_none_when_game_over() {
        // Create a won game
        let mut game = Game::new();
        game.make_move(0).unwrap(); // X
        game.make_move(3).unwrap(); // O
        game.make_move(1).unwrap(); // X
        game.make_move(4).unwrap(); // O
        game.make_move(2).unwrap(); // X wins
        assert!(matches!(game.status(), GameStatus::Won { .. }));
        assert_eq!(get_computer_move(&game), None);
    }

    #[test]
    fn test_ai_100_games_all_valid() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let mut game = Game::new();
            loop {
                match game.status() {
                    GameStatus::Won { .. } | GameStatus::Draw => break,
                    GameStatus::InProgress => {}
                }

                if game.current_player() == Player::X {
                    // Human plays randomly
                    let empty: Vec<usize> = game
                        .cells()
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_none())
                        .map(|(i, _)| i)
                        .collect();
                    if empty.is_empty() {
                        break;
                    }
                    let pos = empty[rng.random_range(0..empty.len())];
                    game.make_move(pos).unwrap();
                } else {
                    // AI plays
                    let mv = get_computer_move(&game);
                    assert!(mv.is_some(), "AI should return a move for in-progress game");
                    let pos = mv.unwrap();
                    assert!(pos < 9, "AI move out of range");
                    assert!(game.cells()[pos].is_none(), "AI chose occupied cell");
                    game.make_move(pos).unwrap();
                }
            }
            // Game must have ended properly
            assert!(
                matches!(game.status(), GameStatus::Won { .. } | GameStatus::Draw),
                "Game should end in win or draw"
            );
        }
    }

    #[test]
    fn test_ai_beatable_in_100_games() {
        let mut rng = rand::rng();
        let mut human_wins = 0;
        for _ in 0..100 {
            let mut game = Game::new();
            loop {
                match game.status() {
                    GameStatus::Won { .. } | GameStatus::Draw => break,
                    GameStatus::InProgress => {}
                }

                if game.current_player() == Player::X {
                    let empty: Vec<usize> = game
                        .cells()
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_none())
                        .map(|(i, _)| i)
                        .collect();
                    if empty.is_empty() {
                        break;
                    }
                    let pos = empty[rng.random_range(0..empty.len())];
                    game.make_move(pos).unwrap();
                } else {
                    let mv = get_computer_move(&game).unwrap();
                    game.make_move(mv).unwrap();
                }
            }
            if let GameStatus::Won {
                winner: Player::X, ..
            } = game.status()
            {
                human_wins += 1;
            }
        }
        assert!(
            human_wins >= 1,
            "Human should win at least 1 out of 100 games (got {})",
            human_wins
        );
    }

    #[test]
    fn test_ai_never_illegal_move() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let mut game = Game::new();
            loop {
                match game.status() {
                    GameStatus::Won { .. } | GameStatus::Draw => break,
                    GameStatus::InProgress => {}
                }

                if game.current_player() == Player::X {
                    let empty: Vec<usize> = game
                        .cells()
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_none())
                        .map(|(i, _)| i)
                        .collect();
                    if empty.is_empty() {
                        break;
                    }
                    let pos = empty[rng.random_range(0..empty.len())];
                    game.make_move(pos).unwrap();
                } else {
                    let mv = get_computer_move(&game);
                    if let Some(pos) = mv {
                        assert!(pos < 9, "AI move out of range: {}", pos);
                        assert!(
                            game.cells()[pos].is_none(),
                            "AI chose occupied cell {} on move",
                            pos
                        );
                        game.make_move(pos).unwrap();
                    } else {
                        // AI returned None for non-terminal state — that's a bug
                        panic!("AI returned None for in-progress game");
                    }
                }
            }
        }
    }
}
