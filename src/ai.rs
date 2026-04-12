use crate::board::{Game, GameStatus, Player};
use rand::Rng;

/// Probability that the AI makes a random move instead of the optimal one.
const MISTAKE_RATE: f64 = 0.25;

/// Returns the computer's chosen move (0-8), or None if the game is already over.
pub fn get_computer_move(game: &Game) -> Option<usize> {
    todo!()
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
        let mut rng = rand::thread_rng();
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
                    let pos = empty[rng.gen_range(0..empty.len())];
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
        let mut rng = rand::thread_rng();
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
                    let pos = empty[rng.gen_range(0..empty.len())];
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
        let mut rng = rand::thread_rng();
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
                    let pos = empty[rng.gen_range(0..empty.len())];
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
