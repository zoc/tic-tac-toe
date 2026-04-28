# Phase 13: Rust AI Parameterization & WASM API - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 3 modified files
**Analogs found:** 3 / 3 (all files are self-analogs — this phase extends existing files)

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `src/ai.rs` | service / utility | transform (deterministic + probabilistic) | `src/ai.rs` (self) | exact — parameterize existing function |
| `src/wasm_api.rs` | WASM boundary / controller | request-response | `src/wasm_api.rs` (self) | exact — extend existing struct |
| `src/main.rs` | CLI entry point | request-response | `src/main.rs` (self) | exact — update one call site |

No new files are created. All changes are in-place modifications of existing files.

---

## Pattern Assignments

### `src/ai.rs` (service, transform)

**Analog:** `src/ai.rs` (self — lines 1–59 show the function to parameterize)

**Imports pattern** (lines 1–2):
```rust
use crate::board::{Game, GameStatus, Player};
use rand::RngExt;
```
These imports are unchanged. No new imports are required for Phase 13.

**Constant to remove** (line 5):
```rust
const MISTAKE_RATE: f64 = 0.25;
```
This constant is removed and replaced by `mistake_rate_for_level(difficulty)`.

**New function to add — `mistake_rate_for_level`** (insert after imports, before `get_computer_move`):
```rust
/// Maps a difficulty level (0–3) to an AI mistake probability.
/// Higher level = fewer mistakes = harder to beat.
/// Level 0 (Easy):       65% chance of a random move
/// Level 1 (Medium):     25% chance of a random move  <- existing default
/// Level 2 (Hard):        8% chance of a random move
/// Level 3 (Unbeatable):  0% chance — pure minimax
fn mistake_rate_for_level(level: u8) -> f64 {
    match level {
        0 => 0.65, // Easy      — frequently beatable
        1 => 0.25, // Medium    — occasionally beatable (existing behavior)
        2 => 0.08, // Hard      — rarely beatable
        3 => 0.0,  // Unbeatable — perfect minimax; random_bool(0.0) always false
        _ => 0.25, // unknown level defaults to Medium
    }
}
```

**Core pattern — `get_computer_move` signature change** (line 8):
```rust
// BEFORE:
pub fn get_computer_move(game: &Game) -> Option<usize> {

// AFTER (D-05):
pub fn get_computer_move(game: &Game, difficulty: u8) -> Option<usize> {
```

**Mistake injection line change** (line 32):
```rust
// BEFORE:
if rng.random_bool(MISTAKE_RATE) {

// AFTER:
if rng.random_bool(mistake_rate_for_level(difficulty)) {
```

**Surrounding context unchanged** (lines 29–34) — copy this structure:
```rust
    let mut rng = rand::rng();

    // With mistake_rate_for_level(difficulty) probability, pick a random empty cell
    if rng.random_bool(mistake_rate_for_level(difficulty)) {
        return Some(empty[rng.random_range(0..empty.len())]);
    }
```

**Existing test structure to update** (lines 153–339) — all 5 tests follow this pattern for `get_computer_move` calls:
```rust
// BEFORE (found in tests at lines 162, 174, 178, 194, 207, 237, 279, 322):
get_computer_move(&game)

// AFTER (D-08 — pass difficulty=1 explicitly at all call sites):
get_computer_move(&game, 1)
```

**New test to add — `test_mistake_rate_for_level`** (D-09, insert in `mod tests`):
```rust
#[test]
fn test_mistake_rate_for_level() {
    assert_eq!(mistake_rate_for_level(0), 0.65);
    assert_eq!(mistake_rate_for_level(1), 0.25);
    assert_eq!(mistake_rate_for_level(2), 0.08);
    assert_eq!(mistake_rate_for_level(3), 0.0);
}
```

**New test to add — `test_ai_unbeatable_never_loses`** (D-10 — mirrors `test_ai_beatable_in_100_games` at lines 254–295):
```rust
#[test]
fn test_ai_unbeatable_never_loses() {
    let mut rng = rand::rng();
    for _ in 0..50 {
        let mut game = Game::new();
        loop {
            match game.status() {
                GameStatus::Won { .. } | GameStatus::Draw => break,
                GameStatus::InProgress => {}
            }
            if game.current_player() == Player::X {
                // Human plays randomly
                let empty: Vec<usize> = game.cells().iter().enumerate()
                    .filter(|(_, c)| c.is_none()).map(|(i, _)| i).collect();
                if empty.is_empty() { break; }
                game.make_move(empty[rng.random_range(0..empty.len())]).unwrap();
            } else {
                let mv = get_computer_move(&game, 3).unwrap(); // Unbeatable (D-10)
                game.make_move(mv).unwrap();
            }
        }
        // Unbeatable AI (O) must never lose — assert X never wins (D-10, Pitfall 3)
        assert!(
            !matches!(game.status(), GameStatus::Won { winner: Player::X, .. }),
            "Unbeatable AI lost a game"
        );
    }
}
```

**Existing test loop pattern** (lines 212–251) — reuse verbatim for the game loop structure:
```rust
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
                let mv = get_computer_move(&game, 1);  // use difficulty param
                // ... handle mv
            }
        }
```

---

### `src/wasm_api.rs` (WASM boundary, request-response)

**Analog:** `src/wasm_api.rs` (self — extend existing struct and impl block)

**Imports pattern** (lines 1–3 — unchanged):
```rust
use wasm_bindgen::prelude::*;
use crate::board::{Game, GameStatus, Player};
use crate::ai::get_computer_move;
```
The import on line 3 does NOT change — only the call site argument changes (Pitfall 2 from RESEARCH.md).

**Struct definition change** (lines 12–15):
```rust
// BEFORE:
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
}

// AFTER (D-03 — add difficulty field; JS never sees it directly):
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
    difficulty: u8,   // not exposed via getter — consistent with inner: Game
}
```

**Constructor change** (lines 20–23):
```rust
// BEFORE:
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame { inner: Game::new() }
    }

// AFTER (D-03 — default Medium; Pitfall 4 — must initialize difficulty):
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame { inner: Game::new(), difficulty: 1 }
    }
```

**New method to add — `set_difficulty`** (D-01 — insert after `new()`, before `make_move()`):
```rust
    /// Set the AI difficulty level (0=Easy, 1=Medium, 2=Hard, 3=Unbeatable).
    /// Pure setter — does NOT reset the board (D-01).
    /// Persists across reset() calls (D-02).
    pub fn set_difficulty(&mut self, level: u8) {
        self.difficulty = level;
    }
```
No `#[wasm_bindgen(getter/setter)]` annotation — this is a plain `pub fn` method on the `#[wasm_bindgen] impl` block, consistent with all other methods (`make_move`, `get_board`, `current_player`, etc.) at lines 27–94.

**`reset()` method** (lines 92–94 — intentionally unchanged to satisfy D-02):
```rust
    /// Reset the game to initial state.
    pub fn reset(&mut self) {
        self.inner = Game::new();
        // difficulty intentionally NOT reset — D-02
    }
```

**`computer_move()` call site change** (lines 81–89):
```rust
// BEFORE:
    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner) {
            Some(pos) => {
                let _ = self.inner.make_move(pos);
                pos as u8
            }
            None => 255,
        }
    }

// AFTER (D-05 — pass self.difficulty):
    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner, self.difficulty) {
            Some(pos) => {
                let _ = self.inner.make_move(pos);
                pos as u8
            }
            None => 255,
        }
    }
```

**Existing method annotation pattern** — all public methods in the `#[wasm_bindgen] impl` block use plain `pub fn` with no additional attributes (except `new()` which has `#[wasm_bindgen(constructor)]`). Copy this style for `set_difficulty`.

---

### `src/main.rs` (CLI entry, request-response)

**Analog:** `src/main.rs` (self — update one call site)

**Import line** (line 2 — unchanged):
```rust
use tic_tac_toe::ai::get_computer_move;
```

**Call site change** (lines 101–103):
```rust
// BEFORE:
            if let Some(mv) = get_computer_move(&game) {

// AFTER (D-07 — hardcode difficulty=1, Medium; no flag parsing):
            if let Some(mv) = get_computer_move(&game, 1) {
```

All other `main.rs` code is unchanged. No argument parsing, no `--difficulty` flag.

---

## Shared Patterns

### `#[wasm_bindgen]` Method Style
**Source:** `src/wasm_api.rs` lines 17–95
**Apply to:** `set_difficulty` method on `WasmGame`

All methods in the `impl WasmGame` block follow the same pattern: plain `pub fn` inside a `#[wasm_bindgen] impl WasmGame` block. Scalar primitives (`u8`, `bool`, `usize`, `String`, `Vec<u8>`) are used at the boundary — no struct types cross WASM. `set_difficulty(level: u8)` follows this exactly.

```rust
#[wasm_bindgen]
impl WasmGame {
    // Each method: pub fn name(&self or &mut self, ..scalar params..) -> ..scalar..
    pub fn make_move(&mut self, position: usize) -> bool { ... }
    pub fn get_board(&self) -> Vec<u8> { ... }
    pub fn current_player(&self) -> u8 { ... }
    // set_difficulty follows the same form:
    pub fn set_difficulty(&mut self, level: u8) { ... }
}
```

### `match` arm pattern for Rust enums / mappings
**Source:** `src/wasm_api.rs` lines 42–47 and lines 61–68
**Apply to:** `mistake_rate_for_level` in `src/ai.rs`

Exhaustive `match` on an enum or scalar, returning a typed value per arm, with a wildcard fallback:
```rust
match self.inner.current_player() {
    Player::X => 1,
    Player::O => 2,
}
```
The `mistake_rate_for_level` match follows the same style, with a wildcard arm `_ => 0.25` for robustness at the WASM boundary.

### `rand::rng()` + `random_bool` / `random_range` idiom
**Source:** `src/ai.rs` lines 29–33 (existing)
**Apply to:** `test_ai_unbeatable_never_loses` test and the updated `get_computer_move` body

```rust
let mut rng = rand::rng();
if rng.random_bool(rate) {
    return Some(empty[rng.random_range(0..empty.len())]);
}
```
This idiom is already established. All new test code and the updated `get_computer_move` body reuse it without modification.

---

## No Analog Found

None. All three files are self-analogs — the phase parameterizes and extends existing well-structured Rust code. No new files are introduced and no patterns need to be invented from scratch.

---

## Critical Constraints (from CONTEXT.md decisions)

| Constraint | Source | What it prevents |
|------------|--------|-----------------|
| `reset()` must NOT touch `difficulty` | D-02 | Phase 14 JS orchestrates reset + difficulty separately |
| `WasmGame::new()` defaults to `difficulty=1` | D-03 | Matches current Medium behavior before Phase 13 |
| No `--difficulty` flag in `main.rs` | D-07 | CLI behavior unchanged |
| `MISTAKE_RATE` constant is removed | D-08 | All 5 existing tests must pass `difficulty=1` explicitly |
| No statistical tests for Easy/Hard rates | D-11 | CI flakiness from probabilistic assertions |
| No `#[wasm_bindgen(getter)]` on `difficulty` | RESEARCH anti-pattern | Unnecessary API surface; JS only writes via setter |
| Import on `wasm_api.rs` line 3 unchanged | RESEARCH Pitfall 2 | Only call site argument changes, not the import |

---

## Metadata

**Analog search scope:** `src/` directory (all Rust source files)
**Files scanned:** `src/ai.rs` (340 lines), `src/wasm_api.rs` (96 lines), `src/main.rs` (110 lines)
**Pattern extraction date:** 2026-04-28
