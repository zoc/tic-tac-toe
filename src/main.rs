use std::io::{self, Write};
use tic_tac_toe::ai::get_computer_move;
use tic_tac_toe::board::{Game, GameStatus, Player};

fn display_board(game: &Game) {
    let cells = game.cells();
    println!();
    for row in 0..3 {
        print!("  ");
        for col in 0..3 {
            let idx = row * 3 + col;
            let ch = match cells[idx] {
                Some(Player::X) => "\x1b[1;34mX\x1b[0m", // bold blue
                Some(Player::O) => "\x1b[1;31mO\x1b[0m", // bold red
                None => match idx {
                    0 => "1",
                    1 => "2",
                    2 => "3",
                    3 => "4",
                    4 => "5",
                    5 => "6",
                    6 => "7",
                    7 => "8",
                    8 => "9",
                    _ => " ",
                },
            };
            print!(" {} ", ch);
            if col < 2 {
                print!("|");
            }
        }
        println!();
        if row < 2 {
            println!("  -----------");
        }
    }
    println!();
}

fn main() {
    println!();
    println!("  \x1b[1;33m=== TIC-TAC-TOE ===\x1b[0m");
    println!("  You are \x1b[1;34mX\x1b[0m, Computer is \x1b[1;31mO\x1b[0m");
    println!("  Enter 1-9 to place your move:");
    println!();
    println!("   1 | 2 | 3");
    println!("  -----------");
    println!("   4 | 5 | 6");
    println!("  -----------");
    println!("   7 | 8 | 9");
    println!();

    let mut game = Game::new();

    loop {
        display_board(&game);

        match game.status() {
            GameStatus::Won { winner, positions } => {
                if *winner == Player::X {
                    println!("  \x1b[1;32mYou win!\x1b[0m (cells {}, {}, {})", positions[0] + 1, positions[1] + 1, positions[2] + 1);
                } else {
                    println!("  \x1b[1;31mComputer wins!\x1b[0m (cells {}, {}, {})", positions[0] + 1, positions[1] + 1, positions[2] + 1);
                }
                break;
            }
            GameStatus::Draw => {
                println!("  \x1b[1;33mIt's a draw!\x1b[0m");
                break;
            }
            GameStatus::InProgress => {}
        }

        if game.current_player() == Player::X {
            // Human turn
            loop {
                print!("  Your move (1-9): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let trimmed = input.trim();

                if let Ok(n) = trimmed.parse::<usize>() {
                    if n >= 1 && n <= 9 {
                        match game.make_move(n - 1) {
                            Ok(()) => break,
                            Err(e) => println!("  \x1b[31m{}\x1b[0m", e),
                        }
                    } else {
                        println!("  \x1b[31mPlease enter a number 1-9\x1b[0m");
                    }
                } else {
                    println!("  \x1b[31mPlease enter a number 1-9\x1b[0m");
                }
            }
        } else {
            // Computer turn
            println!("  Computer is thinking...");
            if let Some(mv) = get_computer_move(&game, 1) { // D-07: hardcode Medium; no --difficulty flag
                game.make_move(mv).unwrap();
                println!("  Computer plays \x1b[1;31m{}\x1b[0m", mv + 1);
            }
        }
    }

    println!();
}
