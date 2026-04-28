---
phase: 13-rust-ai-parameterization-wasm-api
reviewed: 2026-04-28T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src/ai.rs
  - src/wasm_api.rs
  - src/main.rs
findings:
  critical: 1
  warning: 4
  info: 2
  total: 7
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-04-28
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

This phase introduces AI difficulty parameterization (`mistake_rate_for_level`) and a WASM API surface (`WasmGame`) for browser consumption. The core minimax logic is correct, the difficulty mapping is sound, and the board integration is properly re-used. However, the WASM API has a critical correctness bug where a silently-failed internal `make_move` is not reflected back to the caller, and several contract-enforcement gaps exist at the JS/Rust boundary. The CLI binary also retains a hardcoded difficulty value despite the phase goal being AI parameterization.

---

## Critical Issues

### CR-01: `computer_move` silently eats a failed internal `make_move` and returns a stale position

**File:** `src/wasm_api.rs:89-96`

**Issue:** `computer_move` calls `get_computer_move`, then unconditionally calls `self.inner.make_move(pos)` and discards the `Result` with `let _ = ...`. It then returns `pos as u8` regardless of whether the move succeeded. Although in the current implementation `get_computer_move` only returns positions from the `empty` cell list and can never return an occupied or out-of-bounds cell, this creates a latent correctness contract violation: the method's documented contract ("Ask the AI to make a move. Returns the chosen position") implies the board was updated. If `make_move` fails for any reason (e.g., a race, a future board-state change, or a double-call from JS before checking status), the board is not updated but the caller is told a move was made at position `pos`. The JS caller has no mechanism to detect this.

```rust
pub fn computer_move(&mut self) -> u8 {
    match get_computer_move(&self.inner, self.difficulty) {
        Some(pos) => {
            // BUG: make_move result is discarded; caller is told move succeeded even if it didn't
            let _ = self.inner.make_move(pos);
            pos as u8
        }
        None => 255,
    }
}
```

**Fix:** Propagate the failure by returning the sentinel on `make_move` error:

```rust
pub fn computer_move(&mut self) -> u8 {
    match get_computer_move(&self.inner, self.difficulty) {
        Some(pos) => {
            if self.inner.make_move(pos).is_ok() {
                pos as u8
            } else {
                255 // internal error — move was not applied
            }
        }
        None => 255,
    }
}
```

---

## Warnings

### WR-01: `set_difficulty` accepts and silently normalizes out-of-range values

**File:** `src/wasm_api.rs:29-31`

**Issue:** The public WASM API documents levels 0–3 but accepts any `u8` value (0–255) without validation or error feedback. Values 4–255 pass into `mistake_rate_for_level`, fall through to the wildcard arm `_ => 0.25`, and silently behave as Medium. The caller has no way to know their input was invalid.

```rust
pub fn set_difficulty(&mut self, level: u8) {
    self.difficulty = level; // no range check
}
```

**Fix:** Clamp or reject invalid values explicitly:

```rust
pub fn set_difficulty(&mut self, level: u8) {
    // Clamp to valid range 0-3; callers passing invalid values get the safest fallback
    self.difficulty = level.min(3);
}
```

Or return a boolean indicating success so JS can handle the error:

```rust
pub fn set_difficulty(&mut self, level: u8) -> bool {
    if level > 3 {
        return false;
    }
    self.difficulty = level;
    true
}
```

---

### WR-02: `computer_move` return type uses a magic sentinel (`255`) with no type-level enforcement

**File:** `src/wasm_api.rs:89`

**Issue:** Returning `u8` with the convention that `255` means "no move" is an unenforced contract. Any JS caller that forgets to check `=== 255` before using the value as a board position will attempt to read index 255 from the 9-element board array, silently accessing `undefined`. There is no compile-time or WASM-level enforcement. WebAssembly/wasm-bindgen supports returning `Option<u8>`, which would translate to a nullable value in JS and makes the "no move" case explicit.

**Fix:** Change the return type to `Option<u8>`:

```rust
pub fn computer_move(&mut self) -> Option<u8> {
    match get_computer_move(&self.inner, self.difficulty) {
        Some(pos) => {
            if self.inner.make_move(pos).is_ok() {
                Some(pos as u8)
            } else {
                None
            }
        }
        None => None,
    }
}
```

This makes the JS side explicit: `const pos = game.computer_move(); if (pos !== undefined) { ... }`.

---

### WR-03: CLI `main.rs` hardcodes difficulty and ignores the phase's parameterization goal

**File:** `src/main.rs:101`

**Issue:** The comment on line 101 explicitly notes "D-07: hardcode Medium; no --difficulty flag." Despite the phase being named "Rust AI parameterization & WASM API", the CLI binary receives no difficulty parameterization. A user playing the CLI gets no control over AI strength. Additionally, this leaves the `mistake_rate_for_level` code path for levels 0, 2, and 3 completely untested via the CLI code path and exercisable only through WASM.

```rust
if let Some(mv) = get_computer_move(&game, 1) { // D-07: hardcode Medium
```

**Fix:** Parse a difficulty argument from `std::env::args()` or default to Medium with a startup prompt:

```rust
let difficulty: u8 = std::env::args()
    .nth(1)
    .and_then(|s| s.parse().ok())
    .unwrap_or(1)
    .min(3);
// then pass `difficulty` instead of literal `1`
```

---

### WR-04: `get_winning_positions` return type is `Vec<usize>` — `usize` is 64-bit on most targets but WASM is 32-bit

**File:** `src/wasm_api.rs:80-85`

**Issue:** `get_winning_positions` is annotated `#[wasm_bindgen]` (via the impl block) and returns `Vec<usize>`. In Rust, `usize` is pointer-width (32-bit in WASM32). wasm-bindgen will serialize this as a `Uint32Array` on the JS side, which is correct for WASM32. However, if the crate is ever compiled for WASM64 or native targets, `usize` becomes 64-bit and wasm-bindgen may refuse to compile (wasm-bindgen does not support `Vec<u64>` directly). Using `Vec<u32>` is more explicit and portable for a board-position type that will never exceed 8.

```rust
pub fn get_winning_positions(&self) -> Vec<usize> {
```

**Fix:**

```rust
pub fn get_winning_positions(&self) -> Vec<u32> {
    match self.inner.status() {
        GameStatus::Won { positions, .. } => positions.iter().map(|&p| p as u32).collect(),
        _ => vec![],
    }
}
```

---

## Info

### IN-01: `WIN_LINES` is duplicated between `ai.rs` and `board.rs`

**File:** `src/ai.rs:143-152` and `src/board.rs:28-37`

**Issue:** The identical `WIN_LINES` constant is defined in both `ai.rs` (line 143) and `board.rs` (line 28). If a future board variant (e.g., larger board) is introduced, both definitions must be updated in sync. This is a maintenance hazard.

**Fix:** Move `WIN_LINES` to `board.rs` (already there) and `pub use` or re-export it in `ai.rs`, or have `check_winner` in `ai.rs` delegate to a public function exposed by `board.rs`.

---

### IN-02: `test_ai_beatable_in_100_games` is a probabilistic test that can produce false failures

**File:** `src/ai.rs:267-308`

**Issue:** The test asserts `human_wins >= 1` over 100 games with a random human player against Level 1 AI (25% mistake rate). While statistically very likely to pass (probability of 0 human wins in 100 games is astronomically low), probabilistic assertions in unit tests are fragile: CI runs with fixed seeds are not guaranteed unless the RNG is seeded deterministically. A flaky test erodes confidence in the test suite.

**Fix:** Either seed the RNG deterministically for reproducibility, or restructure the test to assert that `mistake_rate_for_level(1) > 0.0` (which is the actual property being tested) rather than running a stochastic simulation.

---

_Reviewed: 2026-04-28_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
