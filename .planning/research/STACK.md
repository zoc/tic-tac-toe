# Stack Research

**Domain:** Rust/WASM game — difficulty level parameter passing across the WASM boundary
**Researched:** 2026-04-27
**Confidence:** HIGH

---

## Context

This is a subsequent-milestone research file for v1.4 (Difficulty Levels). The full stack
(Rust, wasm-pack 0.14.0, wasm-bindgen 0.2, Vite 8, Vanilla JS) is validated and unchanged.
This file covers only what changes at the WASM boundary to pass a difficulty level from
JavaScript to the Rust AI.

**Current WASM boundary** (`src/wasm_api.rs`):
- `WasmGame` struct exported via `#[wasm_bindgen]`
- `computer_move(&mut self) -> u8` calls `get_computer_move(&self.inner)` with the global
  constant `MISTAKE_RATE: f64 = 0.25` hardcoded in `src/ai.rs`
- No difficulty parameter exists anywhere in the call chain

**Goal:** JS passes a difficulty level (Easy/Medium/Hard/Unbeatable) to `WasmGame` on startup
and on dropdown change. The Rust AI uses the matching mistake rate for each subsequent
`computer_move()` call.

---

## No New Dependencies Required

Zero new crates or npm packages are needed for this feature. Every capability required is
already present in the validated stack:

| Need | Already provided by |
|------|---------------------|
| Method on exported struct accepting u8 | wasm-bindgen 0.2 — u8 crosses boundary as JS Number natively |
| f64 parameter in AI function | Pure Rust — no boundary crossing needed |
| Randomized move selection | rand 0.10 — `random_bool(f64)` already takes a plain f64 |
| Dropdown value in JS | Native HTML select element — value is already an integer string |
| Persistence of difficulty selection | localStorage — already in use for score and mute state |

---

## Recommended Stack

### Core Technologies (unchanged)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| wasm-bindgen | 0.2 (Cargo.toml) | Rust/JS interop | Already in use; u8 and f64 both cross the WASM boundary as JS Number — no serialization, no BigInt, zero overhead (verified via Context7 wasm-bindgen docs, types/numbers.md) |
| wasm-pack | 0.14.0 (pinned) | WASM compilation | Already pinned in Dockerfile; no version change needed |
| rand | 0.10 (Cargo.toml) | Random move injection | `rng.random_bool(rate)` accepts a plain f64; signature change in `get_computer_move` is non-breaking to the crate |

### Supporting Libraries (unchanged)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| console_error_panic_hook | 0.1 | Panic messages in browser devtools | No change needed |
| getrandom | 0.4 (wasm_js feature) | WASM-compatible entropy source | No change needed |

---

## WASM Boundary Design

### Chosen approach: `set_difficulty(level: u8)` method on `WasmGame`

Add a `mistake_rate: f64` field to `WasmGame` that stores the active mistake rate. JS calls
`set_difficulty(level)` once when the player changes the dropdown (or on page load to restore
a persisted value). `computer_move()` reads the stored rate instead of the removed constant.

**Why this approach:**

| Approach | Decision | Rationale |
|----------|----------|-----------|
| `set_difficulty(u8)` stored on WasmGame | **Recommended** | Single method; u8 is the simplest boundary-safe type; state lives where it belongs (on the game struct); JS calls it only on dropdown change or page load |
| Pass `f64` mistake rate directly to `computer_move` | Rejected | Leaks implementation detail to JS — JS would need to know numeric rates. Difficulty semantics (rate = 0.65 for Easy) belong in Rust, not the caller |
| Pass `String` ("easy", "medium", ...) | Rejected | Strings cross the WASM boundary with heap allocation; a u8 discriminant is sufficient and has zero overhead |
| Re-create `WasmGame` on difficulty change | Rejected | Loses active game state; player mid-game would see board reset |
| Hardcode 4 separate JS-callable functions (`computer_move_easy`, etc.) | Rejected | Explodes the API surface; adding a 5th difficulty level requires a JS change |
| Export a Rust enum via `#[wasm_bindgen]` | Rejected | wasm-bindgen cannot export Rust enums as typed JS enum values without extra scaffolding; a documented u8 mapping is simpler and equally safe |

### Why `u8` for the level discriminant

wasm-bindgen documents that `u8`, `u16`, `u32`, `i32`, `f32`, and `f64` are all represented as
JavaScript `Number` — zero overhead, no BigInt, no string parsing. `u8` is the smallest
sufficient integer for 4 levels (0–3). A dropdown `<option value="0">` gives a JS string that
`parseInt(value, 10)` converts to a Number — standard practice, no special handling needed.

Source: Context7 `/wasm-bindgen/wasm-bindgen` `types/numbers.md` — HIGH confidence.

### Concrete Rust changes required

**`src/wasm_api.rs` — add field + method, update `computer_move` and `reset`:**

```rust
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
    mistake_rate: f64,  // derived from difficulty; default 0.25 = Medium (existing behavior)
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame {
            inner: Game::new(),
            mistake_rate: 0.25,  // Medium is the current shipped default
        }
    }

    /// Set AI difficulty. Level values:
    ///   0 = Easy       (~65% mistake rate)
    ///   1 = Medium     (~25% — current default)
    ///   2 = Hard       (~8%)
    ///   3 = Unbeatable (0%)
    /// Unknown values are silently ignored (no state change).
    pub fn set_difficulty(&mut self, level: u8) {
        self.mistake_rate = match level {
            0 => 0.65,
            1 => 0.25,
            2 => 0.08,
            3 => 0.0,
            _ => self.mistake_rate,  // unknown level: leave unchanged
        };
    }

    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner, self.mistake_rate) {
            Some(pos) => {
                let _ = self.inner.make_move(pos);
                pos as u8
            }
            None => 255,
        }
    }

    pub fn reset(&mut self) {
        // Intentionally keep mistake_rate — difficulty persists across games in a session.
        // The JS side handles cross-session persistence via localStorage.
        self.inner = Game::new();
    }
    // ... all other existing methods unchanged
}
```

**`src/ai.rs` — remove constant, add parameter:**

```rust
// REMOVE: const MISTAKE_RATE: f64 = 0.25;

// Change signature from:
pub fn get_computer_move(game: &Game) -> Option<usize>
// To:
pub fn get_computer_move(game: &Game, mistake_rate: f64) -> Option<usize>

// Replace:
if rng.random_bool(MISTAKE_RATE) {
// With:
if rng.random_bool(mistake_rate) {
// All other logic is identical.
```

The only change is adding `mistake_rate: f64` as a second parameter and deleting the module
constant. All minimax logic, draw handling, and return types are unchanged.

**`src/ai.rs` — Unbeatable mode (mistake_rate = 0.0) requires no special case:**

`rng.random_bool(0.0)` in the rand crate returns `false` because `random_bool(p)` returns
`true` with probability `p`. At p = 0.0 it is always false — the mistake branch is never
taken, and minimax always runs. No special-casing needed.

**Existing tests in `src/ai.rs`:**

All tests call `get_computer_move(&game)` (one argument). After the signature change they must
call `get_computer_move(&game, 0.25)` to preserve current behavior. The tests themselves need
no other changes — their assertions remain valid.

### JS side changes (minimal)

```javascript
// On dropdown change (new event listener):
difficultySelect.addEventListener('change', () => {
    const level = parseInt(difficultySelect.value, 10);
    game.set_difficulty(level);
    saveDifficulty(level);  // localStorage, same pattern as mute
});

// On page load (restore from localStorage):
const savedLevel = loadDifficulty();  // returns 0-3, default 1
game.set_difficulty(savedLevel);
difficultySelect.value = String(savedLevel);

// computer_move() call in JS is UNCHANGED — no arguments added.
```

No new JS imports, no new npm packages, no changes to the Vite config.

---

## What NOT to Add

| Avoid | Why |
|-------|-----|
| `serde` / `serde-wasm-bindgen` | No complex types cross the boundary; u8 is sufficient |
| `js_sys` or `wasm_bindgen::JsValue` | Not needed; all types crossing the boundary are primitives |
| New npm packages | JS needs only `parseInt` and a localStorage read — both already in use |
| Passing `f64` mistake rate from JS | Leaks difficulty-to-rate mapping to JS; that logic belongs in Rust |
| Difficulty enum exported via `#[wasm_bindgen]` | Requires non-trivial scaffolding; a u8 with documented values is equivalent and simpler |
| A separate `DifficultyWasmGame` type or subclass | Over-engineering; difficulty is a single field on the existing struct |

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `set_difficulty(u8)` method | Pass difficulty to constructor: `WasmGame.new(difficulty)` | Only if game objects were always recreated on difficulty change — they are not; reset() must preserve difficulty |
| Store `mistake_rate: f64` on WasmGame | Store `difficulty: u8` on WasmGame, derive rate in `computer_move` | Either works; storing f64 avoids a match arm on every move call (negligible difference) |
| u8 discriminant mapped to f64 rate in Rust | u8 discriminant mapped to f64 rate in JS | Use JS mapping only if JS needed to display the numeric rate; it does not — it only needs the label string |

---

## Version Compatibility

All `Cargo.toml` version constraints are unchanged. No `cargo update` is required.

| Package | Current pin | Change needed |
|---------|-------------|---------------|
| wasm-bindgen | 0.2 | None — `u8`/`f64` method parameters verified supported |
| wasm-pack | 0.14.0 | None |
| rand | 0.10 | None — `random_bool(f64)` accepts a plain f64 parameter |
| getrandom | 0.4 (wasm_js) | None |

---

## Sources

- Context7 `/wasm-bindgen/wasm-bindgen` (`types/numbers.md`) — u8 represented as JS Number, no BigInt; HIGH confidence
- Context7 `/wasm-bindgen/wasm-bindgen` (`getter-and-setter.md`) — `&mut self` method pattern on exported struct; HIGH confidence
- `src/wasm_api.rs` (read directly) — existing WasmGame boundary, current `computer_move` signature
- `src/ai.rs` (read directly) — MISTAKE_RATE constant, `get_computer_move` signature, rand usage pattern
- `Cargo.toml` (read directly) — wasm-bindgen 0.2, rand 0.10, getrandom 0.4 confirmed

---

*Stack research for: Rust/WASM difficulty level boundary — v1.4 Difficulty Levels milestone*
*Researched: 2026-04-27*
