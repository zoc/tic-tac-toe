# Architecture Research — Difficulty Levels Integration

**Domain:** Rust/WASM game engine + Vanilla JS frontend — parameterizing AI behavior across the WASM boundary
**Milestone:** v1.4 Difficulty Levels
**Researched:** 2026-04-27
**Confidence:** HIGH — based on direct source inspection of the full codebase

---

## Existing System Map

```
src/board.rs          Game struct — cells, turn, status (no AI knowledge)
src/ai.rs             get_computer_move(&Game) → Option<usize>
                        └── MISTAKE_RATE: f64 = 0.25  ← HARD-CODED CONSTANT
src/wasm_api.rs       WasmGame — wasm-bindgen exported struct
                        └── computer_move(&mut self) → u8
                              └── calls get_computer_move(&self.inner)  ← no rate param
src/main.js           game.computer_move()  ← no difficulty context
                        loadScore() / saveScore()  ← key: 'ttt-score'
```

The entire difficulty surface is one constant in `src/ai.rs`. Everything else is wiring.

---

## Recommended Architecture

### Design Decision: Where Difficulty Lives

Difficulty is a **session-level setting**, not a per-move setting. It is set once (on page load or dropdown change) and applies to all subsequent `computer_move()` calls until changed.

**Two viable options:**

| Option | Where rate is stored | WASM boundary change |
|--------|----------------------|----------------------|
| A. Rate stored on WasmGame | `WasmGame` holds `mistake_rate: f64`; JS sets it once via `set_difficulty(u8)` | 1 new WASM method |
| B. Rate passed per move call | `computer_move_with_rate(rate: f64)` — JS passes rate each call | 1 changed WASM method signature |

**Recommendation: Option A** — store rate on `WasmGame`. Rationale:
- Difficulty is session state, not per-move state. It belongs alongside the game instance.
- JS only needs to call `set_difficulty(level)` once on change and once on init. No risk of forgetting to pass it on each `computer_move()` call.
- Matches the existing pattern: JS holds a `game` object and calls methods on it.
- `reset()` should NOT change difficulty — game resets, difficulty persists. Option A makes this natural (rate is a separate field from board state).

---

### Component Boundaries After Change

| Component | Change | Details |
|-----------|--------|---------|
| `src/ai.rs` | MODIFIED | `get_computer_move` gains `mistake_rate: f64` parameter; `MISTAKE_RATE` constant removed |
| `src/wasm_api.rs` | MODIFIED | `WasmGame` gains `mistake_rate: f64` field; new `set_difficulty(level: u8)` method; `computer_move()` passes rate to AI |
| `src/main.js` | MODIFIED | Dropdown element queried; `loadDifficulty()` / `saveDifficulty()` helpers added; `set_difficulty()` called on init and on dropdown change; `resetGame()` updated to preserve difficulty |
| `index.html` | MODIFIED | `<select id="difficulty">` dropdown added to UI |
| `src/style.css` | MODIFIED | Dropdown styled to match dark/light theme |

No new files required. No changes to `src/board.rs`, `Cargo.toml`, `vite.config.js`, `package.json`, `Dockerfile`, or CI workflow.

---

### Data Flow: Difficulty Setting

```
Page load
  ↓
loadDifficulty()  →  localStorage.getItem('ttt-difficulty') || 'medium'
  ↓
difficultyEl.value = savedLevel
  ↓
game.set_difficulty(levelToU8(savedLevel))
  ↓
WasmGame.set_difficulty(level: u8)
  ↓
  match level {
    0 => self.mistake_rate = 0.65,   // Easy
    1 => self.mistake_rate = 0.25,   // Medium
    2 => self.mistake_rate = 0.08,   // Hard
    3 => self.mistake_rate = 0.0,    // Unbeatable
    _ => self.mistake_rate = 0.25,   // fallback
  }
```

```
User changes dropdown
  ↓
difficultyEl.addEventListener('change', ...)
  ↓
saveDifficulty(newLevel)  →  localStorage.setItem('ttt-difficulty', newLevel)
  ↓
game.set_difficulty(levelToU8(newLevel))
  (board state unchanged — only mistake_rate updates)
```

```
computer_move() call (unchanged JS call site)
  ↓
WasmGame.computer_move()  →  get_computer_move(&self.inner, self.mistake_rate)
  ↓
ai::get_computer_move(game, mistake_rate)
  if rng.random_bool(mistake_rate) → random move
  else → minimax optimal move
```

```
resetGame() in JS
  ↓
  game.reset()          ← clears board (existing behavior)
  (NO set_difficulty call — difficulty persists across games by design)
```

---

### Data Flow: Score Persistence (Unchanged)

Score uses `'ttt-score'` key. Difficulty uses `'ttt-difficulty'` key. They are independent. Score is shared across all difficulties — no per-difficulty split, consistent with milestone spec.

```
localStorage
  'ttt-score'      → { wins: N, losses: N, draws: N }   (existing, unchanged)
  'ttt-difficulty' → 'easy' | 'medium' | 'hard' | 'unbeatable'  (NEW)
```

---

## Detailed Component Changes

### 1. `src/ai.rs` — Parameterize mistake_rate

**Change:** Remove `MISTAKE_RATE` constant. Add `mistake_rate: f64` parameter to `get_computer_move`.

Before:
```rust
const MISTAKE_RATE: f64 = 0.25;

pub fn get_computer_move(game: &Game) -> Option<usize> {
    // ...
    if rng.random_bool(MISTAKE_RATE) {
```

After:
```rust
pub fn get_computer_move(game: &Game, mistake_rate: f64) -> Option<usize> {
    // ...
    if rng.random_bool(mistake_rate) {
```

`minimax()` is unchanged. `check_winner()` is unchanged. Test suite must be updated: all `get_computer_move(&game)` calls become `get_computer_move(&game, 0.25)` to preserve existing behavior.

The Unbeatable level passes `0.0` — `rng.random_bool(0.0)` always returns false in the `rand` crate, so minimax always runs. No special-casing needed.

---

### 2. `src/wasm_api.rs` — WasmGame gains difficulty state

**Change:** Add `mistake_rate: f64` field to `WasmGame`. Add `set_difficulty(u8)` method. Update `computer_move()` to pass rate. `reset()` must NOT reset `mistake_rate`.

Before:
```rust
pub struct WasmGame {
    inner: Game,
}

impl WasmGame {
    pub fn new() -> WasmGame {
        WasmGame { inner: Game::new() }
    }

    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner) {
```

After:
```rust
pub struct WasmGame {
    inner: Game,
    mistake_rate: f64,
}

impl WasmGame {
    pub fn new() -> WasmGame {
        WasmGame {
            inner: Game::new(),
            mistake_rate: 0.25,   // default: Medium
        }
    }

    pub fn set_difficulty(&mut self, level: u8) {
        self.mistake_rate = match level {
            0 => 0.65,   // Easy
            1 => 0.25,   // Medium
            2 => 0.08,   // Hard
            3 => 0.0,    // Unbeatable
            _ => 0.25,   // fallback to Medium
        };
    }

    pub fn computer_move(&mut self) -> u8 {
        match get_computer_move(&self.inner, self.mistake_rate) {
```

`reset()` unchanged — `game.reset()` only resets `self.inner = Game::new()`, `mistake_rate` field is untouched.

`set_difficulty` must have `#[wasm_bindgen]` annotation (it's part of `impl WasmGame` which is already `#[wasm_bindgen]`).

---

### 3. `src/main.js` — Difficulty persistence and wiring

**New constants and DOM reference:**
```js
const DIFFICULTY_KEY = 'ttt-difficulty';
const difficultyEl = document.getElementById('difficulty');
```

**New helpers:**
```js
function loadDifficulty() {
  try {
    return localStorage.getItem(DIFFICULTY_KEY) || 'medium';
  } catch {
    return 'medium';   // SecurityError in private browsing
  }
}

function saveDifficulty(level) {
  try {
    localStorage.setItem(DIFFICULTY_KEY, level);
  } catch {
    // Storage unavailable — silently ignore
  }
}

function levelToU8(level) {
  return { easy: 0, medium: 1, hard: 2, unbeatable: 3 }[level] ?? 1;
}
```

**In `main()` after `game = new WasmGame()`:**
```js
const savedDifficulty = loadDifficulty();
difficultyEl.value = savedDifficulty;
game.set_difficulty(levelToU8(savedDifficulty));

difficultyEl.addEventListener('change', () => {
  const level = difficultyEl.value;
  saveDifficulty(level);
  game.set_difficulty(levelToU8(level));
});
```

**`resetGame()` — no change needed.** `game.reset()` only clears the board. `mistake_rate` on the WASM side is preserved. The dropdown value in the DOM is also preserved. No extra code required.

**`loadScore()` / `saveScore()` — completely unchanged.** Different key, different object.

---

### 4. `index.html` — Dropdown markup

Add a `<select>` near the top controls area (alongside mute button). The exact position depends on existing HTML structure, but the element must have `id="difficulty"` and four `<option>` values matching the JS level strings:

```html
<select id="difficulty" aria-label="Difficulty">
  <option value="easy">Easy</option>
  <option value="medium" selected>Medium</option>
  <option value="hard">Hard</option>
  <option value="unbeatable">Unbeatable</option>
</select>
```

The `selected` attribute on `medium` is a fallback for when JS hasn't run yet. JS immediately overwrites `difficultyEl.value` from localStorage on init.

---

### 5. `src/style.css` — Dropdown styling

The dropdown must follow the existing dark/light theming (CSS `prefers-color-scheme` variables already in `style.css`). Key rules needed:
- Remove default `<select>` browser chrome (or harmonize it)
- Match the button/control visual style — same border radius, font, color scheme as other controls
- Ensure it does not flash unstyled before the stylesheet loads (already handled by existing FOUC prevention pattern)

---

## Build Order for Implementation

Dependencies flow in one direction only:

```
1. src/ai.rs           — add mistake_rate param  (no deps on other changes)
2. src/wasm_api.rs     — add field, set_difficulty(), update computer_move()
                          (depends on: ai.rs change)
3. src/ai.rs tests     — update get_computer_move() call sites in test suite
                          (depends on: ai.rs signature change)
4. index.html          — add <select id="difficulty">
                          (independent of Rust changes)
5. src/style.css       — style the dropdown
                          (depends on: index.html having the element)
6. src/main.js         — wire difficulty persistence and set_difficulty()
                          (depends on: wasm_api.rs compiled pkg/ available;
                           index.html having difficulty element)
```

wasm-pack rebuild is required after steps 1–3 before JS changes can be tested. During development, the usual flow is: `wasm-pack build --target web` then `npm run dev`.

---

## Integration Points Summary

### New vs Modified

| File | Status | What Changes |
|------|--------|-------------|
| `src/ai.rs` | MODIFIED | `get_computer_move` signature: adds `mistake_rate: f64` param; constant removed |
| `src/wasm_api.rs` | MODIFIED | `WasmGame` struct: `mistake_rate` field; `new()` default 0.25; `set_difficulty(u8)` method; `computer_move()` passes rate |
| `src/main.js` | MODIFIED | `DIFFICULTY_KEY` constant; `difficultyEl` DOM ref; `loadDifficulty/saveDifficulty/levelToU8` helpers; `set_difficulty` wired on init and on change event |
| `index.html` | MODIFIED | `<select id="difficulty">` with 4 options added |
| `src/style.css` | MODIFIED | Dropdown styled to match theme |
| `src/board.rs` | UNCHANGED | No AI knowledge; Game struct unaffected |
| `src/lib.rs` | UNCHANGED | Module declarations unaffected |
| `Cargo.toml` | UNCHANGED | No new dependencies |
| `vite.config.js` | UNCHANGED | Build configuration unaffected |
| `package.json` | UNCHANGED | No new JS dependencies |
| `Dockerfile` | UNCHANGED | Build pipeline unaffected |
| `.github/workflows/` | UNCHANGED | CI/CD unaffected |

---

## WASM Boundary Constraints

All types crossing the `#[wasm_bindgen]` boundary must be scalar or JS-representable. The new `set_difficulty(level: u8)` uses `u8` — a scalar type that crosses cleanly, consistent with the existing boundary pattern (`make_move(usize)`, `computer_move() → u8`, etc.).

`mistake_rate: f64` is an internal field — it never crosses the boundary directly. JS only sends a level index (0–3), never a raw float. This is intentional: the difficulty-to-rate mapping is an AI implementation detail that belongs in Rust, not in JS.

---

## localStorage Key Naming

| Key | Value type | Default | Notes |
|-----|-----------|---------|-------|
| `'ttt-score'` | JSON `{ wins, losses, draws }` | `{ wins: 0, losses: 0, draws: 0 }` | Existing, unchanged |
| `'ttt-difficulty'` | String | `'medium'` | New; string not integer — human-readable, forward-compatible |

Using a string value (`'easy'`, `'medium'`, `'hard'`, `'unbeatable'`) rather than an integer ensures:
- The `<select>` value and localStorage value are the same string — no conversion needed when restoring DOM state (`difficultyEl.value = savedDifficulty` just works)
- Forward-compatible if level names change (versus opaque integers)
- Human-readable in DevTools

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Resetting difficulty on game reset

**What goes wrong:** Calling `game.set_difficulty(levelToU8(difficultyEl.value))` inside `resetGame()`.

**Why:** Redundant — difficulty is already set on the WasmGame and the dropdown hasn't changed. More importantly, it couples game reset logic to difficulty logic unnecessarily.

**Do this instead:** Leave `resetGame()` untouched. The `mistake_rate` field on `WasmGame` persists through `reset()` because `reset()` only reassigns `self.inner`.

---

### Anti-Pattern 2: Splitting score by difficulty

**What goes wrong:** Using `'ttt-score-easy'`, `'ttt-score-medium'` etc. as localStorage keys.

**Why:** Milestone spec explicitly requires a single shared score tally across all difficulties. Per-difficulty scores fragment the leaderboard and require UI changes (which score to show?). The shared score preserves the existing `saveScore()` / `loadScore()` logic without any modification.

**Do this instead:** Keep `'ttt-score'` as-is. One score object, all difficulties contribute to it.

---

### Anti-Pattern 3: Passing mistake_rate from JS as a float

**What goes wrong:** Exposing a `set_mistake_rate(rate: f64)` method on `WasmGame` and having JS compute the rate.

**Why:** Leaks AI implementation details into JS. JS would need to know that Easy = 0.65, Hard = 0.08, etc. If rates are tuned later, both Rust and JS must be updated. The boundary is harder to type-check (f64 can be any float; a u8 level is bounded 0–3).

**Do this instead:** JS sends a level index (0–3). The Rust `set_difficulty(u8)` owns the rate mapping.

---

### Anti-Pattern 4: Disabling difficulty dropdown during computer turn

**What goes wrong:** Hiding or disabling the `<select>` when `boardEl.classList.contains('board--disabled')`.

**Why complex:** The board is disabled during the computer's thinking delay AND after game over. If difficulty is changed during the delay, it takes effect on the very next `game.set_difficulty()` call — which is fine, since `set_difficulty` only updates a float field, it doesn't affect an already-dispatched timer. The next game will use the new difficulty automatically.

**Do this instead:** Leave the dropdown always enabled. Difficulty changes mid-game are safe and take effect on the next `computer_move()` call.

---

## Sources

- Direct source inspection: `src/ai.rs`, `src/wasm_api.rs`, `src/main.js`, `src/board.rs` (HIGH confidence — primary sources)
- wasm-bindgen scalar type rules: u8, usize, f64 all cross the boundary cleanly; confirmed by existing usage in wasm_api.rs (HIGH confidence)
- rand crate: `rng.random_bool(0.0)` → always false (probability 0.0 never fires); `rng.random_bool(1.0)` → always true — standard behavior for all probability distributions in the rand crate (HIGH confidence)

---
*Architecture research for: v1.4 Difficulty Levels — Rust/WASM + Vanilla JS integration*
*Researched: 2026-04-27*
