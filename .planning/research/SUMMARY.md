# Project Research Summary

**Project:** Tic-Tac-Toe WASM — v1.4 Difficulty Levels
**Domain:** Browser game AI parameterization, WASM boundary design
**Researched:** 2026-04-27
**Confidence:** HIGH

## Executive Summary

Adding difficulty levels to this game is a well-scoped, low-risk milestone. The entire feature reduces to a single architectural change: replacing one compile-time constant (`MISTAKE_RATE: f64 = 0.25` in `src/ai.rs`) with a runtime parameter that flows from a UI dropdown through the WASM boundary into the AI function. No new dependencies are required — the existing stack (Rust, wasm-bindgen 0.2, wasm-pack 0.14.0, rand 0.10, vanilla JS, localStorage) supports every capability needed.

The recommended WASM API design is a `set_difficulty(level: u8)` method on the existing `WasmGame` struct, storing a derived `mistake_rate: f64` field internally. JS sends only a level index (0–3); Rust owns the rate mapping. The UI is a native `<select>` element styled with existing CSS variables. Difficulty persists via a new `ttt-difficulty` localStorage key and triggers a game reset on change.

The primary risks are all implementation-level: silent NaN coercion at the WASM boundary when unparsed strings are passed, a mid-game async race condition if difficulty is changed during the 300–800ms thinking delay, and inadvertently touching the existing `ttt-score` localStorage key. All three have clear prevention patterns and require no architectural rethinking.

## Key Findings

### Recommended Stack

No new dependencies. wasm-bindgen 0.2 already supports `u8` as a native boundary type (JS `Number`). The `u8` design is preferred over `f64` across the boundary because it eliminates NaN/Infinity/out-of-range coercion risk — all silent in Rust, all returning `0`. Rate-to-level mapping belongs in Rust in a single `match` arm.

**Boundary type: `set_difficulty(level: u8)`**
- `0` = Easy (~65% mistake rate)
- `1` = Medium (~25% mistake rate)
- `2` = Hard (~8% mistake rate)
- `3` = Unbeatable (0% mistake rate)

`rng.random_bool(0.0)` in rand 0.10 always returns `false` — Unbeatable works with no special case.

### Expected Features

**Must have (table stakes):**
- 4 named difficulty levels with perceptibly distinct AI behavior
- Persistent setting that survives page refresh (localStorage)
- Game resets when difficulty changes (prevents mid-game inconsistency)
- Difficulty selector disabled during active AI thinking

**Should have (differentiators):**
- Default difficulty of Medium on first visit (matches existing behavior)
- Dropdown styled to match existing dark navy/red theme

**Defer (v2+):**
- Per-difficulty score tracking (explicitly out of scope for this milestone)
- "You can't win" callout for Unbeatable (user chose not to show it)
- Animated difficulty change transitions

### Architecture Approach

5 files change, 0 new files created. The change is mechanical: `src/ai.rs` removes the constant and adds a parameter; `src/wasm_api.rs` adds a field and method; `src/main.js`, `index.html`, and `src/style.css` add the selector and persistence wiring.

**Modified files:**
1. `src/ai.rs` — remove `MISTAKE_RATE` const, add `mistake_rate: f64` param to `get_computer_move`
2. `src/wasm_api.rs` — add `mistake_rate` field to `WasmGame`, add `set_difficulty(u8)` method, preserve through `reset()`
3. `index.html` — add `<select id="difficulty">` element
4. `src/style.css` — style dropdown with existing CSS variables
5. `src/main.js` — wire `loadDifficulty/saveDifficulty`, call `set_difficulty` on init + change, disable during play

**Build order dependency:** Rust must compile (`wasm-pack build`) before JS integration can be tested.

### Critical Pitfalls

1. **Silent WASM coercion** — `parseInt(select.value)` is mandatory before calling `set_difficulty()`. Passing a raw string silently coerces to `0` (Easy) in Rust. Prevention: always `parseInt`, add a Rust `match` with a fallback arm.
2. **Async race condition** — The 300–800ms thinking delay creates a window for mid-game difficulty changes. Prevention: disable the dropdown when `isProcessing` is true, mirroring the existing `isProcessing` guard pattern.
3. **Wrong localStorage key** — Using `ttt-score` instead of a new `ttt-difficulty` key corrupts existing user score data. Prevention: use `'ttt-difficulty'` (matching the `'ttt-score'` / `'ttt-muted'` naming convention).
4. **Rate inversion bug** — `MISTAKE_RATE` is counterintuitive: higher value = more mistakes = easier. Easy is `0.65`, Hard is `0.08`. Prevention: define a named `mistake_rate_for_level(u8) -> f64` function with a match arm and a comment.
5. **resetGame() dropdown leak** — The function does not currently manage the dropdown's enabled state. If the dropdown is disabled at game start, it must be explicitly re-enabled in `resetGame()`. Prevention: add `setDifficultyEnabled(true)` call inside `resetGame()`.

## Implications for Roadmap

Suggested phase structure: **2 phases**

### Phase 13: Rust AI Parameterization & WASM API

**Rationale:** Hard dependency — Rust must compile before JS can call `set_difficulty`. All pitfall-prevention decisions (u8 vs f64, rate mapping direction, test update pattern) must be locked here before any JS is written.

**Delivers:** Parameterized `get_computer_move`, `set_difficulty(u8)` on `WasmGame`, updated test suite, rebuilt WASM pkg.

**Addresses:** Easy/Medium/Hard/Unbeatable AI behavior
**Avoids:** Silent coercion (boundary type decision), rate inversion (named mapping function), test breakage

### Phase 14: UI Integration & Persistence

**Rationale:** Depends on Phase 13 WASM output. Pure frontend work: HTML, CSS, JS wiring.

**Delivers:** `<select>` dropdown in UI, localStorage persistence (`ttt-difficulty`), game reset on change, dropdown disabled during AI thinking.

**Uses:** Existing localStorage pattern from score/mute persistence
**Implements:** Dropdown disable/enable guard mirroring `isProcessing` pattern

### Phase Ordering Rationale

- Rust → JS is a hard build dependency, not a preference
- Locking the `u8` boundary type in Phase 13 prevents JS call-site rework in Phase 14
- `resetGame()` dropdown wiring belongs with dropdown creation in Phase 14 — not split across phases
- Test suite updates belong in Phase 13 so CI passes before Phase 14 begins

### Research Flags

No phases require deeper research:
- **Phase 13:** wasm-bindgen u8/f64 boundary types verified via official docs; rand `random_bool(0.0)` behavior confirmed
- **Phase 14:** localStorage pattern identical to existing `ttt-muted` / `ttt-score` implementations

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | wasm-bindgen boundary types verified via Context7; no new dependencies confirmed |
| Features | HIGH | Solved domain; all features derived from direct codebase inspection |
| Architecture | HIGH | All 5 modified files inspected; change is mechanical |
| Pitfalls | HIGH | WASM coercion from official docs; race condition and localStorage pitfalls from direct code reading |

**Overall confidence:** HIGH

### Gaps to Address

- **Mistake rate calibration** (65%/25%/8%) — tunable in one Rust match arm; validate with manual play session after implementation
- **Default difficulty** — Medium recommended (matches existing 25% rate); Easy defensible for first-time UX

---
*Research completed: 2026-04-27*
*Ready for roadmap: yes*
