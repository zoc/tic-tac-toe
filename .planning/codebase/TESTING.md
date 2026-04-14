# Testing Patterns

**Analysis Date:** 2026-04-14

## Test Framework

**Runner:**
- `cargo test` — standard Rust test runner
- No external test framework (no `nextest`, no `rstest`)
- Config: `Cargo.toml` (dev-dependencies only)

**Dev Dependencies:**
```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**Assertion Library:**
- Standard Rust macros: `assert!`, `assert_eq!`, `assert_ne!`
- No third-party assertion library

**Run Commands:**
```bash
cargo test                # Run all unit tests (native, no WASM)
cargo test --lib          # Run lib tests only (excludes main.rs binary)
wasm-pack test --headless --firefox   # Run #[wasm_bindgen_test] in browser (not yet used)
```

**JS Test Commands:**
- No JS test runner configured — `package.json` has no `test` script
- Manual browser tests via `test.html` (opened directly against built WASM)

## Test File Organization

**Location:**
- Inline `#[cfg(test)]` modules within the source file they test — co-located, not a separate directory
- No `tests/` top-level integration test directory

**Files containing tests:**
- `src/board.rs` — `mod tests` at bottom of file (lines 155–381)
- `src/ai.rs` — `mod tests` at bottom of file (lines 153–339)
- No tests in `src/wasm_api.rs`
- No tests in `src/lib.rs`
- No tests in `src/main.rs`

**Naming convention for test module:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

**Test function naming:**
- `test_` prefix on every test function
- Describes what is being tested and expected outcome: `test_top_row_win`, `test_move_on_occupied_cell_returns_error`, `test_ai_returns_none_when_game_over`
- Uses `snake_case` throughout

## Test Structure

**Suite Organization:**
```
src/board.rs
└── mod tests
    ├── test_new_game_empty_board_and_x_starts        (initial state)
    ├── test_make_move_places_piece_and_switches_player
    ├── test_move_on_occupied_cell_returns_error       (error path)
    ├── test_move_after_game_over_returns_error        (error path)
    ├── test_top_row_win                               (all 8 win lines)
    ├── test_middle_row_win
    ├── test_bottom_row_win
    ├── test_left_column_win
    ├── test_middle_column_win
    ├── test_right_column_win
    ├── test_diagonal_win
    ├── test_anti_diagonal_win
    ├── test_draw_detection
    ├── test_in_progress_with_empty_cells_and_no_winner
    └── test_move_out_of_bounds

src/ai.rs
└── mod tests
    ├── test_ai_returns_valid_move          (valid position, non-nil)
    ├── test_ai_returns_none_when_game_over
    ├── test_ai_100_games_all_valid         (property test: 100 random games)
    ├── test_ai_beatable_in_100_games       (property test: human wins >= 1)
    └── test_ai_never_illegal_move          (property test: 100 random games)
```

**Pattern — arrange/act/assert (no helpers needed; games built inline):**
```rust
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
```

**Pattern — error path testing:**
```rust
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
```

## Mocking

**Framework:** None — no mocking library used (`mockall`, `double`, etc.)

**Approach:** Pure functions and value-based game state allow tests to construct scenarios directly without mocks. The AI's randomness is tested statistically rather than eliminated via seeding.

**What is tested directly (no mocks needed):**
- `Game` struct is constructed via `Game::new()` or `Game::from_state()` in tests
- AI calls are made on real `Game` instances
- No external I/O in testable code (no DB, network, etc.)

## Fixtures and Factories

**Test-only constructor gated with `#[cfg(test)]`:**
```rust
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
```

**Usage example (in `src/ai.rs` tests):**
```rust
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
assert_eq!(mv.unwrap(), 7, "Only empty cell is position 7");
```

**No fixture files** — all test data constructed inline.

## Coverage

**Requirements:** None enforced — no `cargo-tarpaulin`, no coverage CI step

**What is covered:**
- All 8 win line combinations individually tested in `src/board.rs`
- Draw detection tested
- All error conditions on `make_move` tested (occupied, out-of-bounds, game over)
- AI validity tested via 3 separate 100-game property tests
- AI beatability verified statistically (human wins >= 1 in 100 random games)

**What is NOT covered:**
- `src/wasm_api.rs` — zero Rust unit tests for the WASM bridge layer
- `src/main.rs` — CLI binary has no unit tests
- JavaScript code — no automated tests; only manual browser test via `test.html`
- `src/audio.js` — no tests for sound synthesis logic

## Test Types

**Unit Tests (Rust):**
- Scope: individual `Game` methods and `get_computer_move` function
- Location: inline `#[cfg(test)]` in `src/board.rs` and `src/ai.rs`
- Run with: `cargo test`

**Property / Simulation Tests (Rust):**
- Scope: AI behavior over 100 randomly-played full games
- Tests three properties: all moves valid, no illegal moves chosen, human can win
- Location: `src/ai.rs` — `test_ai_100_games_all_valid`, `test_ai_beatable_in_100_games`, `test_ai_never_illegal_move`
- Uses `rand::rng()` (not seeded — non-deterministic; relies on statistical guarantees)

**Manual Browser Tests (JavaScript + WASM):**
- Scope: WASM bridge API (`WasmGame` JS methods)
- Location: `test.html` — opened manually in browser after running `wasm-pack build`
- Framework: custom inline `assert()` function printing PASS/FAIL to DOM
- 13 assertions covering constructor, board state, move making, computer move, reset, win detection, sentinel values
- Not integrated into any CI pipeline

**Integration Tests:**
- None — no `tests/` directory, no `wasm-pack test` invocations in scripts

**E2E Tests:**
- Not used — no Playwright, Cypress, or Puppeteer

## Common Patterns

**Unwrap in tests (intentional panic on unexpected failure):**
```rust
game.make_move(0).unwrap(); // X
game.make_move(3).unwrap(); // O
```

**Move sequence comments (player + position labeling):**
```rust
game.make_move(0).unwrap(); // X
game.make_move(3).unwrap(); // O
game.make_move(1).unwrap(); // X
game.make_move(4).unwrap(); // O
game.make_move(2).unwrap(); // X wins
```

**Assertion with failure messages:**
```rust
assert!(mv.is_some(), "AI should return a move");
assert!(pos < 9, "Move should be in range 0-8");
assert!(game.cells()[pos].is_none(), "AI should pick an empty cell");
```

**Property test loop pattern (100 games):**
```rust
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
                // human plays random valid move
            } else {
                // AI plays; assert validity of chosen position
            }
        }
        assert!(matches!(game.status(), GameStatus::Won { .. } | GameStatus::Draw));
    }
}
```

**Manual JS test assert pattern (in `test.html`):**
```js
function assert(condition, name) {
    if (condition) {
        log(`✓ PASS: ${name}`, 'pass');
        passed++;
    } else {
        log(`✗ FAIL: ${name}`, 'fail');
        failed++;
    }
}
```

---

*Testing analysis: 2026-04-14*
