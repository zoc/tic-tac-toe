# Phase 13: Rust AI Parameterization & WASM API - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Parameterize the Rust AI's mistake rate by difficulty level (u8 0–3) and expose `set_difficulty(u8)` on `WasmGame` through the WASM boundary. All game logic and WASM API changes are in Rust; no JavaScript changes in this phase.

</domain>

<decisions>
## Implementation Decisions

### set_difficulty Behavior
- **D-01:** `set_difficulty(level: u8)` is a **pure setter** — stores the level on `WasmGame`, does NOT reset the board. Phase 14 JS orchestrates the reset separately when the user changes the dropdown.
- **D-02:** Difficulty **persists across `game.reset()`** — `reset()` only reinitializes `self.inner = Game::new()`. The `difficulty` field on `WasmGame` is untouched. Phase 14 does not need to re-apply difficulty after every reset.
- **D-03:** `WasmGame::new()` defaults to **Medium (difficulty=1)** — matches existing behavior before Phase 13 and satisfies UI-03 (first-visit default is Medium).

### WASM API Design
- **D-04:** `mistake_rate_for_level(u8) -> f64` is a **named function** in `src/ai.rs` with a match arm per level and an inline comment on each arm. Prevents accidental inversion of the "higher = easier" direction. Rates: 0→0.65, 1→0.25, 2→0.08, 3→0.0.
- **D-05:** `get_computer_move` signature becomes `get_computer_move(game: &Game, difficulty: u8) -> Option<usize>` — AI function owns the difficulty-to-rate mapping via `mistake_rate_for_level`.
- **D-06:** For Unbeatable (level=3): `rng.random_bool(0.0)` always returns false in rand 0.10 — no special-case needed; the mistake branch is simply never taken.

### CLI Binary Scope
- **D-07:** `src/main.rs` hardcodes `difficulty=1` (Medium) at its `get_computer_move` call site. CLI behavior is unchanged from today. No `--difficulty` flag or arg parsing in Phase 13.

### Test Strategy
- **D-08:** All 5 existing tests updated to pass `difficulty=1` (Medium) explicitly. The `MISTAKE_RATE: f64 = 0.25` constant is removed from `src/ai.rs`.
- **D-09:** Add `test_mistake_rate_for_level()` — direct unit test of the mapping function: asserts 0→0.65, 1→0.25, 2→0.08, 3→0.0. Fully deterministic.
- **D-10:** Add `test_ai_unbeatable_never_loses()` — plays 50 games at `difficulty=3` with a random human player, asserts zero human wins. Unbeatable at 0% mistake rate is fully deterministic (no probabilistic flakiness).
- **D-11:** No statistical tests for Easy/Hard mistake rates — probabilistic assertions on 65%/8% rates risk CI flakiness. The `mistake_rate_for_level` unit test (D-09) is sufficient to verify rate assignment.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — v1.4 requirements AI-01 through AI-04 (difficulty level behaviors with exact mistake rates)
- `.planning/ROADMAP.md` — Phase 13 success criteria (5 numbered assertions defining correct behavior per level)

### Existing Rust Source (extend, not replace)
- `src/ai.rs` — Current `get_computer_move()` and `MISTAKE_RATE` constant; this is the function to parameterize
- `src/wasm_api.rs` — `WasmGame` struct and all `#[wasm_bindgen]` exports; `set_difficulty` added here

### Project Context
- `.planning/PROJECT.md` — Key Decisions table (WASM boundary type rationale, AI mistake rate decisions from prior phases)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `get_computer_move(game: &Game)` in `src/ai.rs`: Already has minimax + mistake injection via `rng.random_bool(MISTAKE_RATE)`. Parameterization is a one-line change to replace the constant with `mistake_rate_for_level(difficulty)`.
- `WasmGame { inner: Game }` in `src/wasm_api.rs`: Add `difficulty: u8` field. `set_difficulty` stores it; `computer_move()` passes it through.

### Established Patterns
- **Scalar-only WASM boundary**: All existing exports use primitives (u8, bool, String, Vec<u8>). `set_difficulty(u8)` fits this pattern exactly — no serialization overhead.
- **`WasmGame` as opaque handle**: JS never sees `difficulty` directly; it only calls `set_difficulty`. Consistent with how `inner: Game` is hidden.
- **`#[wasm_bindgen]` method annotation**: All public WasmGame methods follow the same pattern — add `pub fn set_difficulty(&mut self, level: u8)` with the same annotation style.

### Integration Points
- `WasmGame::computer_move()` → calls `get_computer_move(&self.inner)` today; becomes `get_computer_move(&self.inner, self.difficulty)` after Phase 13.
- `src/main.rs` CLI entry: calls `get_computer_move(&game)` in the game loop; updated to `get_computer_move(&game, 1)`.

</code_context>

<specifics>
## Specific Ideas

No specific references or "I want it like X" moments — decisions above fully specify the approach.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 13-Rust AI Parameterization & WASM API*
*Context gathered: 2026-04-28*
