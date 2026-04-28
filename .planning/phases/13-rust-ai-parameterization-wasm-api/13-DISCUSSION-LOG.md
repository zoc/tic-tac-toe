# Phase 13: Rust AI Parameterization & WASM API - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 13-Rust AI Parameterization & WASM API
**Areas discussed:** set_difficulty behavior, CLI binary scope, Test strategy

---

## set_difficulty Behavior

### Does set_difficulty also reset the board?

| Option | Description | Selected |
|--------|-------------|----------|
| Pure setter | set_difficulty only stores the new level. Phase 14 JS calls game.reset() separately. Clean separation of concerns. | ✓ |
| Auto-reset the board | set_difficulty updates the level AND resets the game board internally. Phase 14 JS only needs one call. But it bakes a UI policy into the Rust layer. | |

**User's choice:** Pure setter
**Notes:** Chosen to keep reset and difficulty as independent operations. UI policy (reset on change) belongs in Phase 14 JS.

---

### Does reset() preserve difficulty?

| Option | Description | Selected |
|--------|-------------|----------|
| Persist across resets | difficulty stored on WasmGame, not on Game. reset() only reinitializes inner Game board. Phase 14 doesn't need to re-apply after every game. | ✓ |
| Reset to Medium | reset() also clears difficulty back to 1. Phase 14 JS must call set_difficulty again after every reset. | |

**User's choice:** Persist across resets

---

### Default difficulty on WasmGame::new()?

| Option | Description | Selected |
|--------|-------------|----------|
| Medium (1) | Matches existing behavior before Phase 13. Satisfies UI-03 (first-visit default is Medium). | ✓ |
| Easy (0) | Starts friendlier but doesn't match UI-03. | |

**User's choice:** Medium (1)

---

## CLI Binary Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcode Medium | Update the one call site to pass difficulty=1. CLI behavior stays identical to today. | ✓ |
| Add --difficulty flag | Parse a difficulty arg from args in the CLI. Keeps CLI parity with WASM. ~20-30 lines of arg parsing — likely scope creep. | |
| You decide | Let Claude choose the cleanest approach. | |

**User's choice:** Hardcode Medium
**Notes:** CLI is a dev tool; difficulty selection is a UI milestone feature.

---

## Test Strategy

### Existing test signature update

| Option | Description | Selected |
|--------|-------------|----------|
| Update all to pass difficulty=1 | Simple, transparent. Every test explicitly declares Medium. MISTAKE_RATE constant removed. | ✓ |
| Add medium_move() convenience wrapper | Keeps all existing tests unchanged. Adds a shim. Hides difficulty concept from existing tests. | |

**User's choice:** Update all to pass difficulty=1

---

### New test coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Unbeatable + mistake_rate_for_level unit test | Direct mapping test (deterministic). Unbeatable never-loses test (50 games, deterministic). Skip statistical tests for Easy/Hard. | ✓ |
| Statistical tests for all 4 levels | Play 200+ games per level, assert win-rate ranges. Probabilistic — can flake on CI. | |
| You decide | Let Claude choose the right coverage depth. | |

**User's choice:** Unbeatable + mistake_rate_for_level unit test
**Notes:** Statistical tests for 65%/8% rates would be probabilistic and risk CI flakiness. Deterministic tests cover what matters.

---

## Claude's Discretion

- Internal function architecture: `get_computer_move(game, difficulty)` signature — Claude decided AI function owns the mapping via `mistake_rate_for_level`
- `mistake_rate_for_level` placement: in `src/ai.rs` alongside `get_computer_move`

## Deferred Ideas

None — discussion stayed within phase scope.
