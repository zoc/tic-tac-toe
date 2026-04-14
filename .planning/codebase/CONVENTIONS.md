# Coding Conventions

**Analysis Date:** 2026-04-14

## Naming Patterns

**Rust Files:**
- `snake_case` for all module files: `board.rs`, `ai.rs`, `wasm_api.rs`, `main.rs`
- Module names match file names exactly

**Rust Types (enums, structs):**
- `PascalCase` for all types: `Player`, `Game`, `GameStatus`, `WasmGame`
- Enum variants in `PascalCase`: `Player::X`, `Player::O`, `GameStatus::InProgress`, `GameStatus::Won`, `GameStatus::Draw`

**Rust Functions and Methods:**
- `snake_case` for all functions and methods: `get_computer_move`, `make_move`, `update_status_for`, `check_winner`, `init_panic_hook`
- Accessor methods named after the field they return: `cells()`, `current_player()`, `status()`
- Private helpers named descriptively: `opponent()`, `check_winner()`, `update_status_for()`

**Rust Constants:**
- `SCREAMING_SNAKE_CASE`: `WIN_LINES`, `MISTAKE_RATE`

**Rust Variables:**
- `snake_case` throughout: `best_score`, `best_move`, `trial_cells`, `ai_player`, `is_maximizing`

**JavaScript Files:**
- `camelCase` for all JS source files: `main.js`, `audio.js`

**JavaScript Functions:**
- `camelCase` for all functions: `renderBoard`, `handleCellClick`, `resetGame`, `handleGameOver`, `setStatus`, `updateScoreDisplay`, `loadScore`, `saveScore`, `showWinLine`, `clearWinLine`, `getCtx`, `playTone`, `toggleMute`, `isMuted`

**JavaScript Constants:**
- `SCREAMING_SNAKE_CASE` for module-level constants: `PLAYER_X`, `PLAYER_O`, `NO_MOVE`, `SCORE_KEY`, `MUTE_KEY`, `THINK_MIN`, `THINK_MAX`, `WIN_LINE_CLASSES`
- `camelCase` for DOM element references: `boardEl`, `statusEl`, `restartBtn`, `muteBtn`, `scoreWinsEl`

**JavaScript Variables:**
- `camelCase` for all mutable state: `game`, `isProcessing`, `thinkingTimer`, `muted`, `audioCtx`

**CSS Classes:**
- BEM-inspired naming with `--` for modifiers: `.cell`, `.cell--taken`, `.cell--x`, `.cell--o`, `.cell--winning`, `.board--disabled`, `.status--win`, `.status--loss`, `.score-item--loss`
- Block-level elements: `.app`, `.board`, `.board-wrapper`, `.scoreboard`, `.title-row`, `.win-line`
- CSS custom properties in `--kebab-case`: `--bg`, `--surface`, `--accent`, `--text-dim`, `--cell-gap`, `--board-size`

## Code Style

**Rust Formatting:**
- Standard `rustfmt` formatting (no explicit config file — uses edition defaults)
- Rust edition 2021 (declared in `Cargo.toml`)
- Trailing commas in multi-line struct/enum definitions
- Single-space indentation: 4 spaces per level

**JavaScript Formatting:**
- No explicit Prettier or ESLint config — consistent manual style throughout
- 2-space indentation
- Single quotes for strings (with exceptions for contractions: `"It's a draw!"`)
- Semicolons present on all statements
- `const` preferred over `let`; `let` used only for mutable state (`isProcessing`, `thinkingTimer`, `game`, `muted`, `audioCtx`, `score`)

**CSS Formatting:**
- Section headers use horizontal dividers with `─── Title ───` comment style
- Properties grouped logically; each rule block gets a blank line between sections
- `clamp()` used extensively for responsive sizing

## Import Organization

**Rust — order:**
1. Standard library (`use std::...`)
2. External crates (`use wasm_bindgen::prelude::*`, `use rand::RngExt`)
3. Internal crate paths (`use crate::board::...`, `use crate::ai::...`)

**Rust — test modules always use `use super::*`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

**JavaScript — order (in `main.js`):**
1. WASM package imports: `import init, { WasmGame } from '../pkg/tic_tac_toe.js';`
2. Local module imports: `import { sounds, toggleMute, isMuted } from './audio.js';`

**JavaScript — path conventions:**
- WASM package uses relative path from `src/`: `'../pkg/tic_tac_toe.js'`
- Local JS modules use `./` prefix: `'./audio.js'`

## Visibility and Encapsulation

**Rust pub/private pattern:**
- `pub` on types, methods, and modules meant for use across crate boundaries or via WASM
- Internal helpers are always private (no `pub`): `opponent()`, `check_winner()`, `update_status_for()`, `minimax()`
- `pub mod` on all three modules from `lib.rs`: `pub mod board`, `pub mod ai`, `pub mod wasm_api`
- Test-only helpers gated with `#[cfg(test)]`: `Game::from_state()`, `Game::update_status()`

**JavaScript visibility:**
- Module-level state (not exported): `game`, `isProcessing`, `thinkingTimer`, `score`, `boardEl`, etc.
- Named exports only for cross-module consumption: `export const sounds`, `export function toggleMute`, `export function isMuted`
- No default exports from `audio.js`; default export only from WASM package

## Error Handling

**Rust — `Result<(), String>` for fallible operations:**
```rust
pub fn make_move(&mut self, position: usize) -> Result<(), String> {
    if !matches!(self.status, GameStatus::InProgress) {
        return Err("Game is already over".to_string());
    }
    if position >= 9 {
        return Err(format!("Position {} is out of bounds (must be 0-8)", position));
    }
    if self.cells[position].is_some() {
        return Err(format!("Cell {} is already occupied", position));
    }
    // ...
    Ok(())
}
```

**Rust — early returns with guard clauses, not nested if-else**

**Rust — `.unwrap()` is acceptable only in tests:**
- Production code never panics on expected errors
- Tests use `.unwrap()` freely since panics produce clear test failures
- `init_panic_hook()` in `src/wasm_api.rs` catches unexpected panics in browser

**WASM API — errors mapped to sentinel values (no JS exceptions):**
- `make_move()` returns `bool` (true = success, false = failure) instead of propagating `Result`
- `computer_move()` returns `255` as sentinel for "game over, no move"
- `get_status()` returns strings `"playing"`, `"won"`, `"draw"` instead of enum

**JavaScript — guard clauses for early return:**
```js
async function handleCellClick(event) {
  if (isProcessing) return;
  if (game.get_status() !== 'playing') return;
  if (game.current_player() !== PLAYER_X) return;
  // ...
}
```

**JavaScript — localStorage access wrapped in try/catch:**
```js
function loadScore() {
  try {
    const saved = localStorage.getItem(SCORE_KEY);
    return saved ? JSON.parse(saved) : { wins: 0, losses: 0, draws: 0 };
  } catch {
    return { wins: 0, losses: 0, draws: 0 };  // SecurityError in private browsing
  }
}
```

**JavaScript — WASM load failure handled imperatively (avoids XSS):**
```js
main().catch(err => {
  const wrapper = document.createElement('div');
  // safe: textContent, never innerHTML
  pre.textContent = err.message;
  document.body.replaceChildren(wrapper);
});
```

## Documentation / Comments

**Rust — doc comments (`///`) on all public items:**
```rust
/// Attempts to place the current player's piece at `position` (0-8).
///
/// Returns `Err` if:
/// - position is out of bounds (>= 9)
/// - cell is already occupied
/// - game is already over
pub fn make_move(&mut self, position: usize) -> Result<(), String> {
```

**Rust — inline comments for non-obvious logic:**
```rust
// Check for draw: all cells filled, no winner
if self.cells.iter().all(|c| c.is_some()) {
    self.status = GameStatus::Draw;
}
// Otherwise remains InProgress
```

**JavaScript — section dividers with ASCII art:**
```js
// ─── Constants ────────────────────────────────────────────────────────────────
// ─── DOM references (queried once at startup) ─────────────────────────────────
// ─── Game state ───────────────────────────────────────────────────────────────
```

**JavaScript — requirement cross-references in comments:**
- Implementation decisions labeled inline: `// per Decision A`, `// SCORE-01`, `// UI-03`, `// FEEL-01`
- These reference `.planning/` documents

**JSDoc on audio.js parameters:**
```js
// @param {number} frequency   - Hz
// @param {string} type        - OscillatorType ('sine'|'square'|'sawtooth'|'triangle')
// @param {number} duration    - seconds
// @param {number} gain        - peak amplitude (0.0–1.0)
```

## DOM Manipulation Patterns

**DOM refs queried once at startup, stored as `const`:**
```js
const boardEl      = document.getElementById('board');
const statusEl     = document.getElementById('status-message');
const restartBtn   = document.getElementById('restart-btn');
```

**Event delegation used for the board (single listener, not one per cell):**
```js
boardEl.addEventListener('click', handleCellClick);
// Inside handler:
const cell = event.target.closest('.cell');
```

**Cell content set with `textContent`, never `innerHTML` (XSS safety)**

**Incremental DOM updates — only patch cells whose state changed:**
```js
const wasEmpty = !cell.classList.contains('cell--taken');
if (value === PLAYER_X && wasEmpty) {
  cell.textContent = 'X';
  cell.classList.add('cell--taken', 'cell--x');
}
```

**State communicated through CSS classes, not inline styles:**
- `.board--disabled`, `.cell--taken`, `.cell--winning`, `.status--win`

## Rust Idioms

**`matches!` macro for pattern guards:**
```rust
if !matches!(self.status, GameStatus::InProgress) { ... }
matches!(game.status(), GameStatus::Won { .. } | GameStatus::Draw)
```

**Destructuring enum variants in match arms:**
```rust
match self.inner.status() {
    GameStatus::Won { winner, .. } => match winner { ... },
    _ => 0,
}
```

**Iterator chains for collecting empty cells:**
```rust
let empty: Vec<usize> = cells
    .iter()
    .enumerate()
    .filter(|(_, c)| c.is_none())
    .map(|(i, _)| i)
    .collect();
```

**`#[derive]` used liberally on data types:**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player { X, O }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus { InProgress, Won { winner: Player, positions: [usize; 3] }, Draw }
```

**Array copy-semantics for board snapshots (no heap allocation in minimax):**
```rust
let mut trial = *cells;  // copies [Option<Player>; 9] by value — no Vec clone
trial[pos] = Some(current);
```

---

*Convention analysis: 2026-04-14*
