# Pitfalls Research — v1.4 Difficulty Levels

**Domain:** Adding difficulty levels to a browser-based Rust/WASM game
**Codebase:** Rust/wasm-pack + Vite 8 + vanilla JS/CSS — existing `WasmGame` struct, `MISTAKE_RATE` const in `src/ai.rs`, `computer_move()` in `wasm_api.rs`
**Researched:** 2026-04-27
**Confidence:** HIGH — wasm-bindgen official docs (Context7) + direct codebase inspection + wasm-bindgen number coercion spec

---

## Context: Why This Document Exists

v1.2 PITFALLS covers Docker multi-arch deployment.
v1.1 PITFALLS covers browser-side polish (animations, audio, localStorage, dark mode).

This document covers pitfalls specific to **adding parameterized difficulty levels** to the existing
Rust/WASM game engine — specifically:

- Changing `const MISTAKE_RATE` from a compile-time constant to a runtime parameter
- Passing the difficulty value across the WASM boundary via wasm-bindgen
- Preventing mid-game difficulty changes from corrupting game state
- Integrating a difficulty dropdown into the existing JS game flow
- Persisting difficulty preference to localStorage alongside the existing score key

Every pitfall here is specific to this codebase and this change.

---

## Critical Pitfalls

---

### Pitfall 1: JavaScript Number to Rust u8 Wraps on Out-of-Range Values

**What goes wrong:**
`computer_move()` currently takes no arguments. When adding a `u8 difficulty` parameter (or
making `MISTAKE_RATE` a method parameter), JavaScript passes a `Number`. wasm-bindgen's
conversion rule for `Number → u8` is:

- `NaN`, `Infinity`, `-Infinity` → **0**
- Numbers outside 0–255 → **wrap** (e.g., `256` → `0`, `300` → `44`)

If the JS side passes a dropdown string value that wasn't parsed — `game.computer_move("medium")`
— JavaScript coerces `"medium"` to `NaN`, which becomes `0` in Rust. If `0` maps to the hardest
or softest difficulty unintentionally, the game silently plays at the wrong level with no error.

**Why it happens:**
Developers expect Rust's type system to protect against invalid inputs at the boundary. It does
not. wasm-bindgen performs a silent coercion — no panic, no exception, just a wrong value.
The generated glue code calls `Math.trunc` and applies masking, so all JS numbers are valid at
the boundary level; Rust just receives an unexpected value.

**How to avoid:**
1. Validate on the JavaScript side before calling into WASM: parse the dropdown value to an integer
   and range-check it (`if (isNaN(level) || level < 0 || level > 3) throw ...`).
2. In Rust, add a guard in the method that receives the difficulty parameter: clamp the value to
   the valid range and use a match-with-default that converts unknown values to a safe fallback
   (e.g., Medium):

```rust
pub fn computer_move(&mut self, difficulty: u8) -> u8 {
    let rate = match difficulty {
        0 => 0.75, // Easy
        1 => 0.40, // Medium
        2 => 0.15, // Hard
        _ => 0.0,  // Unbeatable — also handles any wrapped/unexpected value
    };
    // ...
}
```

3. Never pass the raw dropdown `.value` string directly to a Rust function expecting a number.

**Warning signs:**
- AI plays at unexpectedly easy or hard level when dropdown shows a specific setting
- Difficulty level appears correct in the UI but the AI behavior does not match
- No console errors despite wrong behavior — the silent coercion hides the bug

**Phase to address:** Rust AI parameterization phase — guard the parameter in Rust at the point
where the difficulty is consumed. Validate on the JS side at the call site in the same phase.

---

### Pitfall 2: f64 Mistake Rate Passed Directly Across WASM Boundary Instead of Discrete Levels

**What goes wrong:**
The tempting approach is to expose `mistake_rate: f64` directly across the WASM boundary so the
JS side can pass `0.75` (Easy) or `0.0` (Unbeatable). This creates several problems:

1. **No validation at the boundary**: JS can pass `-0.5`, `2.0`, or `NaN`. `NaN` in Rust's
   `rand::rng().random_bool(NaN)` behavior is undefined (the `random_bool` function in the `rand`
   crate requires a value in `[0.0, 1.0]`; passing NaN will panic via debug assertions or produce
   incorrect behavior in release builds).
2. **Floating-point precision**: `0.1 + 0.2` is not `0.3` in JavaScript. If JS constructs the
   mistake rate arithmetically, the value arriving in Rust may not be what was intended.
3. **Coupling**: The JS side now owns the difficulty-to-mistake-rate mapping. If mistake rates are
   tuned in the future, the JS side must change too.

**Why it happens:**
`MISTAKE_RATE` is already a `f64` in `src/ai.rs`. The reflex is to expose it directly as a
parameter. A `u8` level that maps internally to `f64` seems like an unnecessary layer.

**How to avoid:**
Pass a `u8` level index across the boundary (0=Easy, 1=Medium, 2=Hard, 3=Unbeatable). Map to
`f64` mistake rates inside Rust, in one place. The JS side only knows level integers — it never
touches floating-point rates. This keeps the tuning logic in Rust.

```rust
fn mistake_rate_for_level(level: u8) -> f64 {
    match level {
        0 => 0.75,  // Easy: mostly random
        1 => 0.40,  // Medium: current ~25% → bump to 40% for "Medium"
        2 => 0.15,  // Hard: rarely mistakes
        _ => 0.0,   // Unbeatable: perfect minimax always
    }
}
```

**Warning signs:**
- Method signature in `wasm_api.rs` shows `f64` parameter named `mistake_rate`
- JavaScript constructs the rate as `difficulty * 0.25` or similar arithmetic
- Different browsers produce slightly different AI behaviors (floating-point divergence)

**Phase to address:** Rust AI parameterization phase — decision must be made before writing
any WASM API changes.

---

### Pitfall 3: Mid-Game Difficulty Change Corrupts Game State

**What goes wrong:**
If the difficulty dropdown is enabled while a game is in progress and the player changes it,
the AI will use the new difficulty for its next move. This is subtle corruption: the game started
with Hard AI but the player switches to Easy after they're losing, getting an easier opponent
mid-game. Worse, the `thinkingTimer` scenario: the player changes difficulty after clicking a
cell but during the 300–800ms thinking delay. The computer's move will execute with the new rate
despite the game having been played at the original difficulty.

**Why it happens:**
The difficulty is a piece of UI state. The game flow in `main.js` is async (the thinking delay
uses `await`). A user gesture (dropdown change) can fire between the `await` and `game.computer_move()`.
If `currentDifficulty` is a module-level variable that the dropdown event handler updates
unconditionally, it can change at any point in the async flow.

**How to avoid:**
Two approaches — choose one based on product decision:

**Option A (simpler, recommended for this project):** Disable the dropdown while a game is
in progress. Enable it only on the game-over screen or before the first move. Reset it on
`resetGame()`.

```javascript
function setDifficultyEnabled(enabled) {
  difficultyEl.disabled = !enabled;
}
// Disable on first human move; re-enable in resetGame()
```

**Option B (if mid-game changes are desired):** Latch the difficulty at game start.

```javascript
let activeDifficulty; // set at game start, not from dropdown during play

function startGame() {
  activeDifficulty = parseInt(difficultyEl.value, 10);
  setDifficultyEnabled(false);
  // ...
}
```

The existing codebase already has the `isProcessing` guard and `thinkingTimer` cancel pattern —
difficulty locking should follow the same discipline.

**Warning signs:**
- Difficulty dropdown is enabled during computer thinking delay
- `currentDifficulty` is read directly inside `handleCellClick` from the dropdown rather than
  from a latched value set at game start
- Integration test: change difficulty during the thinking delay and observe if the computer
  plays at the new or old level

**Phase to address:** UI integration phase — difficulty locking must be part of the initial
dropdown implementation, not a later fix.

---

### Pitfall 4: `computer_move()` Signature Change Breaks the Existing JS Call Site

**What goes wrong:**
`computer_move()` currently takes no arguments. Changing its signature to accept `difficulty: u8`
requires updating every call site in `main.js`. There is only one call site today
(`const compPos = game.computer_move();`), but it is easy to add the new signature in Rust, rebuild
the WASM, and then get a runtime error — not a compile error — when the JS glue code changes.

wasm-bindgen generates new JavaScript glue in `pkg/tic_tac_toe.js` that expects the argument.
If the JS call site is not updated, the call becomes `game.computer_move(undefined)`, which
means `difficulty = 0` in Rust (NaN→0 coercion). The game silently defaults to Easy regardless
of the dropdown.

**Why it happens:**
The Rust/WASM build and the JS code are not type-checked together. TypeScript would catch the
mismatch (the generated `.d.ts` would show the updated signature), but this project uses vanilla
JS. The mismatch only surfaces at runtime via wrong behavior — no thrown exception.

**How to avoid:**
1. Update the JS call site in the same commit that changes the Rust signature. Treat this as a
   cross-boundary interface change that must be done atomically.
2. Add a runtime assertion in JS to verify the returned position is valid:
   `if (compPos === NO_MOVE) { /* handle */ }` — already present. No change needed here.
3. After rebuilding WASM, check `pkg/tic_tac_toe.js` to confirm the generated signature matches
   what the JS side is passing. The generated file is the ground truth.

**Warning signs:**
- AI always plays at the easiest level regardless of dropdown selection
- `console.log(game.computer_move.toString())` shows an argument in the function signature that
  JS is not passing
- Building WASM succeeds but the game behavior is wrong after the rebuild

**Phase to address:** WASM bridge update phase — update `wasm_api.rs` and `main.js` together in
the same phase. Do not split across phases.

---

### Pitfall 5: localStorage Key Collision Between Difficulty and Score Data

**What goes wrong:**
The existing `SCORE_KEY = 'ttt-score'` stores `{ wins, losses, draws }` as JSON in localStorage.
If the difficulty setting is stored under the same key (e.g., appended to the score object), there
is a schema migration problem: existing users have `ttt-score` without a `difficulty` field. If
the new code reads `stored.difficulty` without a default, it gets `undefined`, which becomes `NaN`
when passed to `parseInt()`, which then becomes `0` after the NaN→fallback logic (if present) —
or silently operates at the wrong difficulty if the fallback is missing.

Additionally, if difficulty is stored under `'ttt-score'` but score reading happens before
difficulty reading, the score object may get re-parsed incorrectly if the shape changes.

**Why it happens:**
Convenience: there's already a localStorage read/write pattern for score. Reusing the key with
an extended object seems like less code. But it breaks existing stored data for users upgrading
from v1.3 → v1.4.

**How to avoid:**
Use a **separate key** for difficulty:

```javascript
const DIFFICULTY_KEY = 'ttt-difficulty';

function loadDifficulty() {
  try {
    const saved = localStorage.getItem(DIFFICULTY_KEY);
    const parsed = parseInt(saved, 10);
    return isNaN(parsed) || parsed < 0 || parsed > 3 ? 1 : parsed; // default: Medium
  } catch {
    return 1; // SecurityError in private browsing
  }
}

function saveDifficulty(level) {
  try {
    localStorage.setItem(DIFFICULTY_KEY, String(level));
  } catch { /* quota exceeded — ignore */ }
}
```

This matches the existing `MUTE_KEY = 'ttt-muted'` pattern in `audio.js` — separate key per concern.

**Warning signs:**
- `JSON.parse(localStorage.getItem('ttt-score'))` returns an object with unexpected shape
- A user who played v1.3 sees a wrong difficulty level when first loading v1.4
- `parseInt(undefined)` or `parseInt(null)` warnings appear in console after upgrade

**Phase to address:** localStorage persistence phase — use separate keys from the start; do not
mix difficulty into the score object.

---

### Pitfall 6: Difficulty Dropdown Reads Stale Value After `resetGame()`

**What goes wrong:**
After `resetGame()` is called (New Game button or auto-reset), the difficulty dropdown may be
disabled (from Pitfall 3's prevention). If `resetGame()` forgets to re-enable the dropdown, the
user can never change difficulty after the first game. The game silently locks to whatever
difficulty was set for game 1.

A related problem: if the dropdown value is reset to a hardcoded default on `resetGame()` instead
of being read from the current selection or localStorage, the user's chosen difficulty is lost
after each New Game.

**Why it happens:**
`resetGame()` currently resets: `game.reset()`, clears the board DOM, clears the win line,
restores turn status, cancels the thinking timer, and hides the restart button. Difficulty
enable/disable is a new concern that needs to be added to this function explicitly — it won't
happen automatically.

**How to avoid:**
Add explicit difficulty management to `resetGame()`:

```javascript
function resetGame() {
  // ... existing reset logic ...
  setDifficultyEnabled(true); // Re-enable dropdown after game ends
  // Do NOT reset dropdown value — preserve the user's selection
}
```

And at the start of a game (on first human move or on computer_move call), latch the value and
disable the dropdown:

```javascript
function latchDifficulty() {
  activeDifficulty = parseInt(difficultyEl.value, 10);
  setDifficultyEnabled(false);
}
```

**Warning signs:**
- After completing one game and clicking New Game, the difficulty dropdown is grayed out
- The difficulty resets to "Easy" after every New Game even if the user selected "Hard"
- `resetGame()` does not contain any reference to `difficultyEl`

**Phase to address:** UI integration phase — difficulty enable/disable must be explicitly wired
to `resetGame()` and game-start flow in the same phase.

---

### Pitfall 7: Unbeatable Mode Does Not Actually Play Perfect Minimax Without Code Change

**What goes wrong:**
The current `get_computer_move()` function in `src/ai.rs` uses `MISTAKE_RATE` to decide whether
to play randomly or run minimax. Setting the rate to `0.0` for "Unbeatable" means
`rng.random_bool(0.0)` always returns `false`, so minimax always runs. This is correct.

However, there is a subtle issue: the `random_bool(0.0)` call **still executes** and consumes
an RNG invocation even when the rate is `0.0`. For `Unbeatable` mode, this is wasted computation.
More importantly: if the refactoring adds the mistake rate as a parameter and the guard is
`if rate > 0.0 && rng.random_bool(rate)`, the behavior is correct. But if the guard is written
as `if rng.random_bool(rate)` with `rate = 0.0`, it still works — `random_bool(0.0)` returns
`false` in the `rand` crate.

The real pitfall is the opposite: if `rate` is somehow `1.0` for Easy (always random), minimax
is never called. But Easy mode should be random **most of the time**, not always. If the tuning
table maps Easy to `1.0`, the AI never blocks or threatens — it is completely random, which may
feel broken rather than easy. A value of `0.75` (75% random) produces better "easy" feel.

**Why it happens:**
The mistake rate is a probability and its UX meaning is easy to invert. "Easy" means "high
mistake rate" (large number) which is counterintuitive — higher rate = worse AI. Developers
accidentally swap Easy and Hard when building the lookup table.

**How to avoid:**
Name the mapping clearly and test it:

```rust
fn mistake_rate_for_level(level: u8) -> f64 {
    // Higher rate = more mistakes = easier for human
    match level {
        0 => 0.75, // Easy: 75% chance of random move
        1 => 0.40, // Medium: 40% chance of random move (harder than current 25%)
        2 => 0.15, // Hard: 15% chance of random move
        _ => 0.0,  // Unbeatable: pure minimax, no random moves
    }
}
```

Add a test that verifies the Easy AI loses more than the Hard AI in N games.

**Warning signs:**
- The existing test `test_ai_beatable_in_100_games` fails after the difficulty refactor with
  `Unbeatable` settings (good — the test protects against this)
- Easy mode plays so well it feels like Hard (rates accidentally swapped)
- Hard mode is beatable nearly every game (rates accidentally too high)

**Phase to address:** Rust AI parameterization phase — define the mapping table with comments;
add a unit test covering all 4 levels before wiring up the WASM boundary.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Pass `f64` mistake rate directly across boundary | One fewer abstraction layer | JS owns tuning; NaN/range issues; coupling | Never — use `u8` level, map in Rust |
| Mix difficulty into `ttt-score` localStorage key | One fewer key to manage | Schema migration breaks existing users; entangles two concerns | Never — use separate `ttt-difficulty` key |
| Leave difficulty dropdown always enabled | No enable/disable logic needed | Player can change difficulty mid-game; inconsistent state | Never — disable on game start, re-enable on reset |
| Hardcode `activeDifficulty = 1` (Medium) in `resetGame()` | Simple reset | User's chosen difficulty lost after every New Game | Never — preserve dropdown value across resets |
| Read difficulty directly from dropdown inside thinking delay | No latch variable needed | Race condition: difficulty can change during `await` | Never — latch at game start |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| WASM boundary (u8 parameter) | Pass dropdown `.value` string directly: `game.computer_move(difficultyEl.value)` → NaN → 0 | Parse first: `game.computer_move(parseInt(difficultyEl.value, 10))` |
| WASM boundary (signature change) | Update Rust but forget to update JS call site; game silently plays at level 0 | Update `wasm_api.rs` and `main.js` in the same phase; verify via `pkg/tic_tac_toe.js` |
| localStorage (difficulty key) | Reuse `ttt-score` key with extended object; breaks existing stored data | Separate `ttt-difficulty` key; default fallback for missing/invalid stored values |
| Thinking delay + difficulty | Read `difficultyEl.value` inside async function after `await` — may have changed | Latch difficulty to `activeDifficulty` before the `await`; read `activeDifficulty` in `computer_move()` call |
| Reset flow | `resetGame()` does not re-enable dropdown → permanently locked after first game | Explicitly call `setDifficultyEnabled(true)` in `resetGame()` |
| Dropdown initial value | Dropdown renders with first `<option>` selected instead of persisted value | In `main()`, after loading WASM, call `difficultyEl.value = String(loadDifficulty())` |

---

## Performance Traps

This feature has no performance traps of note. The AI runs synchronously in WASM (~1ms for
minimax on a 3x3 board) and the difficulty only affects the probability of skipping minimax —
easier levels are actually faster (random move skips minimax). No scale concerns apply.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Running minimax inside `random_bool` guard for Unbeatable | Minimax called even for random-move branch check | Guard: `if rate > 0.0 && rng.random_bool(rate)` vs `if rng.random_bool(rate)` | Never breaks functionally, but `rate = 0.0` short-circuits correctly either way — not a real trap |

---

## Security Mistakes

This feature has a minimal attack surface (local game, no network). The relevant concern is:

| Mistake | Risk | Prevention |
|---------|------|------------|
| Reading localStorage `ttt-difficulty` without `try/catch` | Throws `SecurityError` in Safari private browsing; game fails to load | Wrap in `try/catch`; return default `1` (Medium) on any error — identical pattern to existing `loadScore()` |
| Using `innerHTML` to render difficulty label in status message | XSS if difficulty label string is injected from localStorage without sanitization | Never use localStorage values in `innerHTML`; only use them as numeric indices to look up hardcoded strings |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Difficulty changes mid-game silently affect AI strength | Player feels cheated or confused; Hard game suddenly becomes Easy | Disable dropdown during active game; only changeable between games |
| Difficulty resets to default on every New Game | Player must re-select difficulty every game; annoying for players who always play Hard | Persist selection; preserve across `resetGame()` |
| "Unbeatable" level is still beatable due to a bug | Trust broken; labeling misleads the player | Test Unbeatable in 100 games against perfect counter-play; assert 0 wins for human |
| Dropdown styled inconsistently with existing theme | Visual jarring; looks like an afterthought | Match existing navy/red CSS variables; use `appearance: none` with a custom `select` style |
| Score continues accumulating across all difficulties without indication | Player cannot tell if their win streak is against Easy or Unbeatable | Per PROJECT.md: single shared score tally is intentional; add difficulty label to game status instead |

---

## "Looks Done But Isn't" Checklist

- [ ] **Difficulty persists on refresh:** Open game, select Hard, refresh page — verify Hard is still selected, not Medium/default
- [ ] **Difficulty locked during play:** Start a game, try changing dropdown during computer thinking delay — verify dropdown is disabled
- [ ] **Difficulty re-enables after game:** Finish a game (win/lose/draw), click New Game — verify dropdown is enabled
- [ ] **Invalid localStorage value handled:** Set `localStorage.setItem('ttt-difficulty', 'garbage')` in console, refresh — verify game loads at Medium (default), not broken
- [ ] **Unbeatable is actually unbeatable:** Run 100 games against Unbeatable playing optimal moves — verify 0 human wins
- [ ] **Easy actually loses often:** Run 100 games against Easy playing optimal moves — verify human wins significantly more than against Hard
- [ ] **No score contamination:** Changing difficulty between games does not reset or alter the score tally
- [ ] **NaN defense:** Call `game.computer_move(NaN)` in browser console directly — verify Rust treats it as fallback level, not a crash
- [ ] **WASM signature up to date:** After `wasm-pack build`, check `pkg/tic_tac_toe.js` — verify `computer_move` signature matches JS call site

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong level due to NaN coercion | LOW | Add JS-side `parseInt` + range check; rebuild WASM not required |
| f64 passed instead of u8 level | MEDIUM | Change WASM API signature; rebuild WASM; update JS call site |
| Difficulty stored in wrong localStorage key | LOW | Add migration in `loadDifficulty()`: check both old and new key; migrate on load |
| Mid-game difficulty change producing wrong behavior | LOW | Add `disabled` attribute to dropdown at game start; no WASM change needed |
| `resetGame()` permanently disables dropdown | LOW | Add `setDifficultyEnabled(true)` to `resetGame()` |
| Easy/Hard rates swapped in lookup table | LOW | Fix constants in Rust; rebuild WASM; no JS change |
| Unbeatable is still beatable | MEDIUM | Debug minimax; the issue is likely in `mistake_rate_for_level` returning non-zero for level 3 |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| NaN/out-of-range u8 coercion | Rust AI parameterization | Unit test: call `computer_move` with level=255 and level=99; verify sane fallback |
| f64 across boundary instead of u8 level | Rust AI parameterization (design decision) | Code review: `wasm_api.rs` signature uses `u8`, not `f64` |
| Mid-game difficulty change | UI integration (dropdown) | Manual test: change dropdown during thinking delay; verify disabled |
| Signature change breaks JS call site | WASM bridge update | After wasm-pack build: `grep -n 'computer_move' pkg/tic_tac_toe.js` matches JS usage |
| localStorage key collision | localStorage persistence | `localStorage.getItem('ttt-score')` still valid JSON after v1.4 loads for first time |
| Dropdown not re-enabled after reset | UI integration (reset flow) | Play game to completion; click New Game; verify dropdown is interactive |
| Stale dropdown value read inside async delay | UI integration (latch pattern) | Code review: `activeDifficulty` latched before `await`, not read from `difficultyEl` inside delay |
| Easy/Hard rates accidentally swapped | Rust AI parameterization | Test: Easy AI loses to random play more than Hard AI; Unbeatable draws/wins in 100 games |

---

## Sources

- wasm-bindgen official docs — Numbers: `u8, i8, ..., f32, f64` represented as JavaScript Numbers;
  `NaN/Infinity/-Infinity → 0`; out-of-range integers wrap. (Context7: `/wasm-bindgen/wasm-bindgen`,
  topic: "Number to u8, i8, u16, i16, u32, i32, isize, and usize") — HIGH confidence
- wasm-bindgen design: `random_bool(f64)` behavior for `0.0` — passes silently, returns `false`
  (rand crate 0.10, `RngExt::random_bool`) — HIGH confidence (direct codebase reading `Cargo.toml` rand=0.10)
- Direct codebase inspection: `src/ai.rs`, `src/wasm_api.rs`, `src/main.js`, `src/audio.js`
  — HIGH confidence
- Existing localStorage pattern in `src/main.js` (`SCORE_KEY`, `loadScore`, `saveScore`) and
  `src/audio.js` (`MUTE_KEY`) — HIGH confidence (direct reading)
- Existing thinking delay / async race condition pattern in `main.js` (`thinkingTimer`,
  `isProcessing` guard, FEEL-02 guard after `await`) — HIGH confidence (direct reading)

---

## Prior Version References

- v1.2 PITFALLS.md covers Docker multi-architecture deployment pitfalls (no overlap)
- v1.1 PITFALLS.md covers browser polish (CSS animations, Web Audio, localStorage patterns) — the
  localStorage `try/catch` and separate-key patterns established there apply directly to this milestone

---
*Pitfalls research for: v1.4 Difficulty Levels — Rust/WASM AI parameterization + JS integration*
*Researched: 2026-04-27*
