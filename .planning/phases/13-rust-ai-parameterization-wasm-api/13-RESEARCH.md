# Phase 13: Rust AI Parameterization & WASM API - Research

**Researched:** 2026-04-28
**Domain:** Rust AI parameterization, wasm-bindgen WASM boundary
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `set_difficulty(level: u8)` is a pure setter — stores the level on `WasmGame`, does NOT reset the board.
- **D-02:** Difficulty persists across `game.reset()` — `reset()` only reinitializes `self.inner = Game::new()`. The `difficulty` field on `WasmGame` is untouched.
- **D-03:** `WasmGame::new()` defaults to Medium (difficulty=1) — matches existing behavior and satisfies UI-03.
- **D-04:** `mistake_rate_for_level(u8) -> f64` is a named function in `src/ai.rs` with a match arm per level and an inline comment on each arm. Rates: 0→0.65, 1→0.25, 2→0.08, 3→0.0.
- **D-05:** `get_computer_move` signature becomes `get_computer_move(game: &Game, difficulty: u8) -> Option<usize>`.
- **D-06:** For Unbeatable (level=3): `rng.random_bool(0.0)` always returns false in rand 0.10 — no special case needed.
- **D-07:** `src/main.rs` hardcodes `difficulty=1` (Medium) at its `get_computer_move` call site. No `--difficulty` flag.
- **D-08:** All 5 existing tests updated to pass `difficulty=1` explicitly. `MISTAKE_RATE: f64 = 0.25` constant removed.
- **D-09:** Add `test_mistake_rate_for_level()` — deterministic unit test of 0→0.65, 1→0.25, 2→0.08, 3→0.0.
- **D-10:** Add `test_ai_unbeatable_never_loses()` — plays 50 games at difficulty=3 with random human, asserts zero human wins.
- **D-11:** No statistical tests for Easy/Hard rates — probabilistic assertions on 65%/8% rates risk CI flakiness.

### Claude's Discretion

None — all decisions are locked.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AI-01 | User can play against Easy AI that makes frequent mistakes (~65% mistake rate) | `mistake_rate_for_level(0) -> 0.65` feeds into `rng.random_bool(0.65)` in `get_computer_move` |
| AI-02 | User can play against Medium AI at existing default skill level (~25% mistake rate) | `mistake_rate_for_level(1) -> 0.25` — identical to current `MISTAKE_RATE` constant value |
| AI-03 | User can play against Hard AI that rarely makes mistakes (~8% mistake rate) | `mistake_rate_for_level(2) -> 0.08` |
| AI-04 | User can play against Unbeatable AI with perfect minimax play (0% mistake rate) | `mistake_rate_for_level(3) -> 0.0`; rand 0.10 `Bernoulli::new(0.0)` is guaranteed false [VERIFIED] |
</phase_requirements>

---

## Summary

Phase 13 is a narrow, well-scoped Rust-only change: parameterize `get_computer_move` to accept a `difficulty: u8` argument, add the `mistake_rate_for_level` mapping function, add a `difficulty` field to `WasmGame`, and expose `set_difficulty(u8)` through the WASM boundary. There is no new library dependency, no new tooling, and no JavaScript change.

The existing code already has all the structural pieces. The current `MISTAKE_RATE: f64 = 0.25` constant drives `rng.random_bool(MISTAKE_RATE)`. The parameterization replaces that constant with a function call. `WasmGame` currently holds only `inner: Game`; adding `difficulty: u8` is a one-field struct extension. All 5 tests in `src/ai.rs` call `get_computer_move(&game)` and require a second argument after the signature change.

The only non-obvious correctness concern is `random_bool(0.0)` for Unbeatable. This is **verified** against the local `rand-0.10.1` source: `Bernoulli::new(0.0)` stores `p_int = 0`; any random u64 sampled by the engine is always `> 0`, so the sample always returns `false`. No special-casing is needed.

**Primary recommendation:** Implement in a single plan covering (1) `src/ai.rs` changes, (2) `src/wasm_api.rs` changes, (3) `src/main.rs` call-site update, (4) test updates + new tests, (5) `wasm-pack build` verification.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Difficulty-to-mistake-rate mapping | Rust/WASM | — | Pure logic; belongs in Rust with `mistake_rate_for_level`, not in JS |
| Difficulty state storage | Rust/WASM (`WasmGame.difficulty`) | — | WASM boundary owns game state; JS calls setter but never reads field directly |
| WASM API (`set_difficulty`) | Rust/WASM boundary | — | `#[wasm_bindgen]` method on `WasmGame`, consistent with all existing exports |
| AI move selection | Rust/WASM (`get_computer_move`) | — | No change in ownership; only signature gains a `difficulty` param |
| CLI entry point | Rust binary (`main.rs`) | — | Hardcodes `difficulty=1`; no UI concern |

---

## Standard Stack

### Core (no changes needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rand | 0.10.1 | RNG for mistake injection | Already in use; `random_bool(p)` is the exact API needed |
| wasm-bindgen | 0.2.x | WASM/JS boundary | Already in use; `#[wasm_bindgen]` annotation pattern is established |
| getrandom | 0.4 (wasm_js feature) | Entropy for WASM target | Already in Cargo.toml; no change needed |

No new dependencies. No `Cargo.toml` changes required.

**Version verification:** `rand 0.10.1` confirmed in local registry at
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/`. [VERIFIED: local cargo registry]

---

## Architecture Patterns

### System Architecture Diagram

```
JS caller
   |
   | game.set_difficulty(level: u8)      [WASM boundary — new]
   v
WasmGame { inner: Game, difficulty: u8 } [src/wasm_api.rs]
   |
   | game.computer_move()
   v
get_computer_move(&self.inner, self.difficulty)   [src/ai.rs — signature extended]
   |
   | mistake_rate_for_level(difficulty)            [src/ai.rs — new fn]
   v
rng.random_bool(rate)  ─── true ──▶  random move (mistake)
                       └── false ─▶  minimax optimal move
```

### Recommended File Structure

No new files. All changes are within:
```
src/
├── ai.rs          # Add mistake_rate_for_level(); extend get_computer_move signature; remove MISTAKE_RATE constant; update 5 tests + add 2 new tests
├── wasm_api.rs    # Add difficulty: u8 field; update new(); add set_difficulty(); update computer_move() call
└── main.rs        # Update get_computer_move call to pass difficulty=1
```

### Pattern 1: Adding a Field to a #[wasm_bindgen] Struct

`#[wasm_bindgen]` structs support plain Rust field additions. Fields not marked `#[wasm_bindgen(getter/setter)]` are invisible to JS — consistent with how `inner: Game` is hidden today.

```rust
// Source: src/wasm_api.rs (existing pattern extended)
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
    difficulty: u8,   // new — JS never sees this directly
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame { inner: Game::new(), difficulty: 1 }  // D-03: default Medium
    }

    pub fn set_difficulty(&mut self, level: u8) {  // D-01: pure setter
        self.difficulty = level;
    }

    pub fn reset(&mut self) {
        self.inner = Game::new();
        // difficulty intentionally NOT reset — D-02
    }

    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner, self.difficulty) {  // D-05
            Some(pos) => { let _ = self.inner.make_move(pos); pos as u8 }
            None => 255,
        }
    }
}
```

[VERIFIED: wasm-bindgen struct field pattern from existing src/wasm_api.rs]

### Pattern 2: mistake_rate_for_level with Match Arms

```rust
// Source: src/ai.rs (new function — D-04)
/// Maps a difficulty level (0–3) to an AI mistake probability.
/// Higher level = fewer mistakes = harder to beat.
/// Level 0 (Easy):       65% chance of a random move
/// Level 1 (Medium):     25% chance of a random move  ← existing default
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

[ASSUMED: wildcard arm fallback to Medium — not specified in CONTEXT.md decisions but is defensive practice; planner should confirm]

### Pattern 3: Updated get_computer_move Signature

```rust
// Source: src/ai.rs (D-05)
pub fn get_computer_move(game: &Game, difficulty: u8) -> Option<usize> {
    // ... same logic as today, replacing MISTAKE_RATE:
    if rng.random_bool(mistake_rate_for_level(difficulty)) {
        return Some(empty[rng.random_range(0..empty.len())]);
    }
    // ... minimax path unchanged
}
```

[VERIFIED: existing ai.rs structure read directly]

### Pattern 4: test_ai_unbeatable_never_loses

```rust
// Source: derived from existing test_ai_beatable_in_100_games pattern in src/ai.rs
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
                let mv = get_computer_move(&game, 3).unwrap(); // Unbeatable
                game.make_move(mv).unwrap();
            }
        }
        // Unbeatable AI never loses
        assert!(
            !matches!(game.status(), GameStatus::Won { winner: Player::X, .. }),
            "Unbeatable AI lost a game"
        );
    }
}
```

[VERIFIED: pattern mirrors existing test_ai_beatable_in_100_games in src/ai.rs]

### Anti-Patterns to Avoid

- **Resetting difficulty in `reset()`:** D-02 explicitly prohibits this. Phase 14 JS handles re-applying the user's chosen level after a UI-triggered reset — the Rust side must not touch `difficulty` in `reset()`.
- **Removing the wildcard arm from `mistake_rate_for_level`:** Any u8 value 4–255 could arrive from JS. A wildcard arm (fallback to Medium) prevents a compile error; exhaustiveness without it requires a `_ => unreachable!()` macro which panics at runtime on bad input.
- **Using `f64::NAN` or checking `p.is_nan()` in the caller:** `Bernoulli::new` panics on out-of-range input. All rates (0.0, 0.08, 0.25, 0.65) are valid [0.0, 1.0) values. No defensive NaN check needed.
- **Adding `#[wasm_bindgen(getter)]` to `difficulty`:** The field is Rust-internal state. JS uses `set_difficulty()` to write and never needs to read it — exposing it as a getter creates unnecessary API surface.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random mistake injection | Custom probability check | `rng.random_bool(p)` from rand 0.10 | Already used in code; Bernoulli distribution is mathematically correct |
| Boundary behavior for p=0.0 | Special-case `if difficulty == 3` block | `random_bool(0.0)` returns false deterministically | Verified in rand source: `p_int = 0`, always false |
| u8-to-f64 level mapping | Inline magic numbers at call site | `mistake_rate_for_level(u8) -> f64` named fn | D-04 decision; prevents inversion bugs |

**Key insight:** Every hard problem in this phase is already solved by the existing code or the rand crate. This phase is purely wiring existing pieces together.

---

## Common Pitfalls

### Pitfall 1: Breaking Existing Tests Before Updating Them

**What goes wrong:** Changing `get_computer_move` signature from `(game: &Game)` to `(game: &Game, difficulty: u8)` causes 5 existing call sites in `src/ai.rs` tests and 1 call site in `src/main.rs` to fail compilation.

**Why it happens:** Rust enforces arity at compile time — partial updates leave the crate uncompilable.

**How to avoid:** Update ALL call sites in a single atomic step: the 5 tests in `src/ai.rs`, `computer_move()` in `src/wasm_api.rs`, and the `get_computer_move(&game)` call in `src/main.rs`. Do not attempt to compile until all sites are updated.

**Warning signs:** `cargo build` errors citing "expected 2 arguments, found 1" at multiple locations.

### Pitfall 2: Forgetting `use crate::ai::get_computer_move` Import in wasm_api.rs

**What goes wrong:** After the signature change, `wasm_api.rs` already imports `get_computer_move` (line 3 of current file). The import itself doesn't need changing — only the call site argument does. Modifying the import by mistake introduces a compile error.

**Why it happens:** The import line looks like a place to "update" alongside the call, but it only names the function, not its arity.

**How to avoid:** Leave the `use crate::ai::get_computer_move;` import line unchanged. Only update `get_computer_move(&self.inner)` → `get_computer_move(&self.inner, self.difficulty)` on the call site in `computer_move()`.

### Pitfall 3: test_ai_unbeatable_never_loses Checking Wrong Player

**What goes wrong:** The test asserts the computer never loses, but the computer is `Player::O`. Asserting `winner != Player::X` is correct; asserting `winner == Player::O` misses draws and gets an unexpected pass.

**Why it happens:** The existing `test_ai_beatable_in_100_games` counts `Player::X` wins — same pattern applies here, but semantically inverted (zero X wins is the assertion).

**How to avoid:** Assert `!matches!(game.status(), GameStatus::Won { winner: Player::X, .. })` — any X win (human) is a failure; draws and O wins are both acceptable.

### Pitfall 4: Struct Initialization Missing the difficulty Field

**What goes wrong:** After adding `difficulty: u8` to `WasmGame`, the `new()` constructor must initialize it. Rust's struct literal initialization is exhaustive — omitting `difficulty` is a compile error, not a silent zero.

**Why it happens:** Only one struct literal instantiation exists (`WasmGame { inner: Game::new() }`), so it's easy to miss.

**How to avoid:** Update `WasmGame::new()` to `WasmGame { inner: Game::new(), difficulty: 1 }` at the same time the field is added to the struct definition.

---

## Code Examples

### Verified: rand 0.10 random_bool(0.0) is always false

From `rand-0.10.1/src/distr/bernoulli.rs` (local registry, line 101 doc comment):
> "For p = 0.0, the resulting distribution will always generate false."

And from `rand-0.10.1/src/rng.rs` test:
```rust
assert_eq!(r.random_bool(0.0), false);
assert_eq!(r.random_bool(1.0), true);
```

Implementation: `0.0` is in `0.0..1.0` (exclusive end), so `p_int = (0.0 * SCALE) as u64 = 0`. Sample always returns `false` because any random u64 is `>= 0` but the comparison requires `< p_int = 0`. [VERIFIED: rand-0.10.1 local source]

### Verified: existing test patterns in src/ai.rs

All 5 existing tests call `get_computer_move(&game)` (no difficulty arg). After the signature change, each must become `get_computer_move(&game, 1)` per D-08.

Tests requiring update:
1. `test_ai_returns_valid_move` — 3 `get_computer_move` calls
2. `test_ai_returns_none_when_game_over` — 1 call
3. `test_ai_100_games_all_valid` — 1 call
4. `test_ai_beatable_in_100_games` — 1 call
5. `test_ai_never_illegal_move` — 1 call

[VERIFIED: counted from src/ai.rs source read]

---

## Runtime State Inventory

Phase 13 is a source code change only — no rename, no migration, no external service state.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — no database, no persisted state touched | None |
| Live service config | None — no external service configuration | None |
| OS-registered state | None | None |
| Secrets/env vars | None — no new credentials | None |
| Build artifacts | `pkg/` (wasm-pack output) — stale after rebuild | `wasm-pack build` regenerates; no manual cleanup needed |

**Nothing found in categories 1–4:** Verified by reading all source files. This phase adds a parameter to a Rust function and a field to a struct. No runtime state is affected.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | All Rust compilation | ✓ | verified via Cargo.toml presence | — |
| wasm-pack | WASM build verification | Must verify at plan time | — | `cargo build` for unit tests only |
| rand 0.10.1 | `random_bool` in ai.rs | ✓ | 0.10.1 in local registry | — |

**Note on wasm-pack:** The CI/CD (Phase 11) uses `wasm-pack 0.14.0 --locked`. For local development the planner should include a `wasm-pack build --target web` step as the final verification, consistent with prior phases. Unit tests (`cargo test`) can run without wasm-pack.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `gen_bool` (rand 0.8) | `random_bool` (rand 0.10) | rand 0.10 release | Already adopted in existing ai.rs code |
| `MISTAKE_RATE` constant | `mistake_rate_for_level(u8) -> f64` fn | Phase 13 | Runtime difficulty selection |

**Deprecated/outdated:**
- `MISTAKE_RATE: f64 = 0.25` constant: Removed in this phase per D-08. Replaced by `mistake_rate_for_level(1)` which returns the same value at the existing call sites.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Wildcard arm `_ => 0.25` in `mistake_rate_for_level` fallback to Medium for unknown levels | Code Examples / Pattern 2 | If level >3 should panic or be forbidden, the fallback silently accepts bad input. Low actual risk since JS controls the u8 value and only sends 0-3. |

---

## Open Questions

1. **Wildcard arm in `mistake_rate_for_level`**
   - What we know: `u8` can hold values 0–255; JS will only send 0–3 in Phase 14
   - What's unclear: Should levels 4–255 silently fall back to Medium, or should they `unreachable!()` / be documented as UB?
   - Recommendation: Use `_ => 0.25` (Medium fallback) for robustness at the WASM boundary. Document the valid range (0–3) in the function's doc comment.

---

## Validation Architecture

> `workflow.nyquist_validation` is `false` in `.planning/config.json` — this section is SKIPPED per config.

---

## Security Domain

> No new authentication, session, access control, or cryptographic concerns introduced. Phase 13 adds a `u8` parameter to an AI function and a field to a WASM game handle. The WASM boundary already uses scalar-only types (u8, bool, String, Vec<u8>). No ASVS categories are newly applicable. Input range for `difficulty: u8` is implicitly bounded by the `match` arms — out-of-range values fall to the wildcard arm, no panic.

---

## Sources

### Primary (HIGH confidence)
- `~/.cargo/registry/src/.../rand-0.10.1/src/distr/bernoulli.rs` — `Bernoulli::new(0.0)` always-false behavior, doc comment line 101, `p_int = 0` computation [VERIFIED: local source]
- `~/.cargo/registry/src/.../rand-0.10.1/src/rng.rs` — `random_bool` implementation and test assertions [VERIFIED: local source]
- `src/ai.rs` — existing test structure, `MISTAKE_RATE` constant, `get_computer_move` signature [VERIFIED: read directly]
- `src/wasm_api.rs` — `WasmGame` struct, `#[wasm_bindgen]` pattern, `computer_move` call site [VERIFIED: read directly]
- `src/main.rs` — `get_computer_move` call site in CLI [VERIFIED: read directly]
- `src/lib.rs` — module structure [VERIFIED: read directly]
- `Cargo.toml` — dependency versions (rand = "0.10", wasm-bindgen = "0.2") [VERIFIED: read directly]
- `.planning/phases/13-rust-ai-parameterization-wasm-api/13-CONTEXT.md` — all locked decisions D-01 through D-11 [VERIFIED: read directly]

### Secondary (MEDIUM confidence)
- `.planning/REQUIREMENTS.md` — AI-01 through AI-04 mistake rate targets [VERIFIED: read directly]
- `.planning/ROADMAP.md` — Phase 13 success criteria [VERIFIED: read directly]
- `.planning/PROJECT.md` — key decisions table, WASM boundary rationale [VERIFIED: read directly]

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing dependencies verified in local registry and Cargo.toml
- Architecture: HIGH — existing code read directly; changes are minimal and well-scoped
- Pitfalls: HIGH — derived from direct reading of all 5 call sites and struct definition
- `random_bool(0.0)` behavior: HIGH — verified in rand-0.10.1 local source with doc comment and test assertions

**Research date:** 2026-04-28
**Valid until:** Stable — no external API lookups; all findings are from locked source files in this repo or the local cargo registry.
