# Feature Research

**Domain:** Difficulty levels for a browser single-player tic-tac-toe game (Rust/WASM + vanilla JS)
**Milestone scope:** v1.4 Difficulty Levels
**Researched:** 2026-04-27
**Confidence:** HIGH — tic-tac-toe AI difficulty is a fully-solved, well-understood domain; patterns are stable and canonical

---

## Context: What Already Exists

This is a subsequent milestone on an existing, polished game. The following are already built and validated:

| Already Built | Implementation |
|---------------|----------------|
| Minimax AI with 25% mistake rate | `src/ai.rs` — `MISTAKE_RATE: f64 = 0.25` constant; `rng.random_bool(MISTAKE_RATE)` short-circuits minimax |
| WASM API | `computer_move()` on `WasmGame` — calls `get_computer_move(&self.inner)` which reads `MISTAKE_RATE` at compile time |
| localStorage persistence | `SCORE_KEY = 'ttt-score'` — `loadScore()` / `saveScore()` pattern with try/catch for incognito |
| Score display | `score-wins`, `score-losses`, `score-draws` DOM elements, updated via `updateScoreDisplay()` |
| Thinking delay | `THINK_MIN = 300`, `THINK_MAX = 800` ms — cancelable via `clearTimeout(thinkingTimer)` |
| Theme | Dark navy/red (`--bg: #1a1a2e`, `--accent: #e94560`) — full dark/light via `prefers-color-scheme` |
| UI controls | Mute button, New Game button — styled with `restart-btn` / `mute-btn` classes |

**The AI mistake rate is a compile-time constant.** Parameterizing it at runtime is the core technical change for this milestone — it requires passing a difficulty value through JS → WASM → Rust.

---

## Table Stakes (Users Expect These)

Features users assume exist when a difficulty selector is present. Missing any of these makes the feature feel broken or amateur.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Difficulty selector visible before game starts | Users need to set difficulty before the first move, not mid-game | LOW | A `<select>` or equivalent placed above the board, visible at page load |
| Selected difficulty shows as current value | The control must reflect the active level — a dropdown that resets to default on page load (when a non-default was saved) feels broken | LOW | Read persisted value during init; set `select.value` before first render |
| Difficulty persists across page refresh | The same localStorage pattern used for score and mute — users don't expect to re-choose on every visit | LOW | New localStorage key e.g. `ttt-difficulty`; load in `main()` alongside score load |
| AI behavior actually changes per level | If Easy feels the same as Hard, the feature is meaningless. The levels must be perceptibly different to a casual player | MEDIUM | Mistake rates must be calibrated: Easy is noticeably beatable, Hard is noticeably hard, Unbeatable never loses |
| Difficulty change takes effect for next game, not mid-game | Changing difficulty mid-game and having the AI immediately switch strategy is jarring and confusing | LOW | Read the difficulty setting when `computer_move()` is called (or at `resetGame()`), not reactively during a live game |
| Four named levels with clear labels | "Easy / Medium / Hard / Unbeatable" is the canonical naming in this domain — casual players understand these labels without explanation | LOW | Single `<select>` with four `<option>` elements — no custom widget needed |

---

## Differentiators (Competitive Advantage)

Features that go beyond the bare minimum and make the difficulty system feel polished.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Difficulty label visible in status area during gameplay | "Playing on Hard" or a small badge near the board reminds the player which level they are on, reducing confusion after level switches | LOW | Small text below or beside the scoreboard — does not need to be prominent |
| Single shared score across all difficulties | Removes the need for per-difficulty score tracking; the tally reflects overall play history regardless of level. Simpler UX than per-level scores, which fragment the scoreboards and create "what counts?" questions | LOW | Already the target requirement — the existing `SCORE_KEY` object stays as-is |
| Dropdown styled to match the existing theme | A browser-default `<select>` on the dark navy background looks out of place in most browsers. Matching the `--accent` / `--surface` / `--text` CSS variables makes the selector feel native to the game | LOW | CSS `select` styling with `background: var(--surface)`, `color: var(--text)`, `border: 1px solid var(--accent)`, `border-radius: 6px` — no JS widget needed |
| Difficulty change resets current game | Changing from Easy to Hard mid-game should start fresh (the board position that was playable on Easy might be unwinnable on Hard). Immediate reset on change avoids a confusing half-played game under wrong conditions | LOW | `select.addEventListener('change', resetGame)` — reuses existing `resetGame()` entirely |
| Accessible dropdown (keyboard + screen reader) | A native `<select>` element is keyboard-navigable and announced by screen readers without extra ARIA markup | LOW | Native `<select>` provides this for free — no custom dropdown widget required |

---

## Anti-Features (Commonly Done, Actively Harmful)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Per-difficulty score tracking** | "Wins on Unbeatable should count separately from Easy wins" | Fragments the scoreboard into four counters each, requires UI to show which row belongs to which level, and creates "which score resets?" questions when difficulty changes. Tic-tac-toe is a 30-second game — the score is a feel-good counter, not analytics | Single shared score tally — already the design decision in the milestone spec |
| **Custom dropdown widget (JS-driven)** | "Browser `<select>` looks inconsistent across OS/browser" | Custom dropdowns require ARIA roles (`listbox`, `option`), keyboard handling (arrow keys, Enter, Escape, Home/End), click-outside dismiss, and mobile touch handling — easily 80+ LOC and a common source of accessibility regressions. The game is vanilla JS with no framework. | Native `<select>` styled with CSS. It inherits all browser keyboard and accessibility behavior for free. Light CSS (`background`, `color`, `border`, `border-radius`, `padding`) is sufficient to match the theme |
| **Difficulty affects thinking delay timing** | "Unbeatable should 'think' longer; Easy should respond instantly" | The thinking delay (300–800 ms) already feels natural. Tying delay to difficulty adds complexity and can create frustrating wait times on hard levels with no perceived benefit — the AI is instant regardless of the delay | Keep thinking delay constant across all levels. The delay exists for UX feel, not to simulate computation time |
| **Showing the AI's "reasoning"** (highlighting candidate moves) | "Educational — show why the AI picked a cell" | Tic-tac-toe minimax is not explainable in a glanceable way. Showing candidate moves would require additional WASM API surface, DOM overlays, and animation work for a feature that most casual players would find confusing rather than helpful | Status messages ("Computer is thinking…") provide sufficient feedback |
| **Mid-game difficulty switch without reset** | "Let me change difficulty without losing my board position" | A board position that is winnable on Easy (AI makes mistakes) is not the same game on Hard (AI plays optimally). The game state is no longer meaningful. Seamless mid-game switching would require complex state reconciliation | Reset on difficulty change — `select.addEventListener('change', resetGame)`. Clean state, no ambiguity |
| **Score reset when difficulty changes** | "The score should reflect only this difficulty level" | Resets score on every difficulty change, wiping progress. Punishes casual exploration of difficulty levels. Contradicts the "single shared score" design goal | Never reset score on difficulty change. Score is a cross-session, cross-difficulty counter |
| **Difficulty stored in URL hash / query param** | "Shareable links with a specific difficulty pre-selected" | Adds URL parsing logic, must handle invalid values, and creates an inconsistency: URL param vs. localStorage — which wins? | localStorage only. The game is single-player and single-session. URL params add complexity with no use case |

---

## Feature Dependencies

```
[Difficulty Selector UI]
    └──requires──> [WASM API: parameterized difficulty]
                       └──requires──> [Rust: difficulty parameter to get_computer_move()]
                                          └──builds-on──> [Existing: minimax + MISTAKE_RATE constant]

[Difficulty Persistence]
    └──builds-on──> [Existing: localStorage loadScore/saveScore pattern]
    └──requires──> [Difficulty Selector UI]

[Difficulty Change → Reset Game]
    └──builds-on──> [Existing: resetGame() function]
    └──requires──> [Difficulty Selector UI]

[Styled Dropdown]
    └──builds-on──> [Existing: CSS variables --surface, --text, --accent, --text-dim]
    └──requires──> [Difficulty Selector UI]

[Single Shared Score]
    └──builds-on──> [Existing: SCORE_KEY, score object, saveScore/loadScore]
    └──no new dependency — deliberate non-change
```

### Dependency Notes

- **Rust change is the critical path.** `get_computer_move()` currently reads `MISTAKE_RATE` as a compile-time constant. It must become a runtime parameter. This requires: changing the Rust function signature, updating `wasm_api.rs` to pass the value through, and updating `main.js` to pass the difficulty-derived rate when calling `computer_move()`.
- **WASM API boundary decision.** Two options: (a) pass `mistake_rate: f64` directly into `computer_move()` as a JS argument, or (b) store difficulty on the `WasmGame` struct and set it via a separate WASM method. Option (a) is simpler for vanilla JS. Option (b) keeps difficulty state in Rust and is cleaner for the Rust model.
- **localStorage pattern is already proven.** The `loadScore` / `saveScore` pattern with `try/catch` for SecurityError handles incognito mode correctly. The difficulty persistence should use the identical pattern with a new key.
- **Difficulty change reuses resetGame() entirely.** No new reset logic is needed. The event listener on `select.change` can call the existing `resetGame()` directly.

---

## MVP Definition for v1.4

### Launch With (v1.4 — all required)

- [ ] **Rust: runtime mistake rate** — `get_computer_move(game, mistake_rate: f64)` replaces compile-time constant; existing minimax unchanged
- [ ] **WASM API: difficulty-aware computer_move** — `computer_move(mistake_rate: f64)` or `set_difficulty(level: u8)` + `computer_move()` on `WasmGame`
- [ ] **Four difficulty levels with calibrated rates** — Easy (~70% random), Medium (~40% random), Hard (~10% random), Unbeatable (0% random, pure minimax)
- [ ] **Dropdown selector in HTML** — native `<select>` with four `<option>` elements, placed above the board
- [ ] **CSS styling for dropdown** — matches existing dark/light theme via CSS variables; no JS widget
- [ ] **Difficulty persisted to localStorage** — new key (e.g. `ttt-difficulty`), loaded on init, saved on change
- [ ] **Difficulty change triggers game reset** — `select.addEventListener('change', resetGame)` — reuses existing function
- [ ] **Score remains shared** — no change to existing `SCORE_KEY` or score object structure

### Add After Validation (v1.x)

- [ ] **Difficulty label in status area** — small indicator of current level while playing; only if users find levels confusing
- [ ] **ARIA label on difficulty selector** — `aria-label="Difficulty"` for screen reader users; trivial to add

### Future Consideration (v2+)

- [ ] **Difficulty affects board size** (4×4 harder mode) — completely out of scope; changes game rules
- [ ] **Custom mistake rate slider** — power-user feature; not needed for 4-level system
- [ ] **Per-difficulty high score** — would require redesigning the scoreboard; defer until there's user demand

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Runtime mistake rate in Rust | HIGH | LOW | P1 |
| WASM API update | HIGH | LOW | P1 |
| Four calibrated difficulty levels | HIGH | LOW | P1 |
| Dropdown selector HTML | HIGH | LOW | P1 |
| CSS-styled dropdown | MEDIUM | LOW | P1 |
| localStorage persistence | MEDIUM | LOW | P1 |
| Difficulty change → reset | HIGH | LOW | P1 |
| Single shared score (non-change) | HIGH | LOW (no-op) | P1 |
| Difficulty label in status | LOW | LOW | P2 |
| ARIA label on select | LOW | LOW | P2 |

**Priority key:**
- P1: Must have for v1.4 launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Difficulty Level Calibration

This is the most important design decision in the milestone. The four levels must be perceptibly different.

| Level | Behavior | Mistake Rate (random move probability) | Expected Human Win Rate vs Random Human Play |
|-------|----------|----------------------------------------|----------------------------------------------|
| Easy | Mostly random moves; occasionally plays optimally | ~70% | Human wins most games |
| Medium | Mix of random and optimal; will block obvious threats sometimes | ~40% | Human wins roughly half |
| Hard | Mostly optimal; makes occasional mistakes | ~10% | Human rarely wins; requires deliberate play |
| Unbeatable | Pure minimax — never makes a suboptimal move | 0% | Human can only draw with perfect play; cannot win |

**Calibration note:** The existing "beatable" AI runs at 25% mistake rate. This maps closest to Medium. The existing codebase test `test_ai_beatable_in_100_games` asserts at least 1 human win in 100 games with random human play — at 0% mistake rate (Unbeatable), this test would fail and must be excluded or made level-aware.

**Unbeatable correctness guarantee:** Tic-tac-toe is a solved game. With perfect minimax and 0% mistake rate, the second player (O = computer) can always force at least a draw. The human (X, first mover) cannot win against a perfect opponent. This is mathematically proven and the codebase minimax already implements it correctly — the only change is removing the `random_bool(MISTAKE_RATE)` short-circuit.

---

## WASM API Design Options

Two valid approaches for passing difficulty through the JS/WASM boundary:

**Option A: Pass mistake_rate per call**
```rust
pub fn computer_move(&mut self, mistake_rate: f64) -> u8
```
JS: `game.computer_move(mistakeRateForCurrentLevel)`

- Simpler to implement — one argument added to existing method
- JS holds the difficulty state; Rust is stateless on this concern
- Slightly awkward: caller must always supply the rate; rate is not validated by Rust type system

**Option B: Store difficulty on WasmGame struct**
```rust
pub fn set_difficulty(&mut self, mistake_rate: f64) { self.mistake_rate = mistake_rate; }
pub fn computer_move(&mut self) -> u8 // unchanged signature
```
JS: `game.set_difficulty(0.0)` on change, then `game.computer_move()` unchanged

- `computer_move()` signature stays the same — less JS call-site change
- Difficulty is "owned" by the game object — semantically cleaner
- Requires adding a field to `WasmGame` struct and `set_difficulty()` method

**Recommendation:** Option B (store on struct). The thinking delay guard in `main.js` already calls `game.computer_move()` with no arguments — Option A would require updating that call site and the guard check. Option B changes only `set_difficulty()` call on difficulty change, and `reset()` should preserve or re-apply the current difficulty.

---

## UX Interaction Model

The recommended interaction flow based on standard browser game conventions:

1. **Page load:** Read difficulty from localStorage (default: Medium if not set). Set `select.value`. Initialize game at that difficulty.
2. **User changes difficulty:** `select.change` event fires → save new difficulty to localStorage → call `resetGame()`. The board clears, the new difficulty is active for the next human move.
3. **Game plays:** `computer_move()` uses the stored `mistake_rate` — difficulty is opaque to the human during play (no announcement needed).
4. **Game ends:** Score updates as today — no change. Restart button appears.
5. **User clicks New Game:** `resetGame()` called — difficulty unchanged, new game starts at the same level.
6. **Page refresh:** Difficulty loaded from localStorage, score loaded from localStorage. Both persisted independently.

**Score is never reset by difficulty changes.** The score object is untouched by the difficulty feature. This is explicit in the milestone spec and matches the anti-feature analysis above.

---

## Sources

- Existing codebase — `/Users/franck/Development/tic-tac-toe/src/ai.rs`: minimax implementation, `MISTAKE_RATE` constant (HIGH confidence — direct inspection)
- Existing codebase — `/Users/franck/Development/tic-tac-toe/src/wasm_api.rs`: WASM boundary, `computer_move()` signature (HIGH confidence — direct inspection)
- Existing codebase — `/Users/franck/Development/tic-tac-toe/src/main.js`: localStorage pattern, thinking delay guard, `resetGame()` (HIGH confidence — direct inspection)
- Tic-tac-toe as a solved game — minimax with perfect play forces at least a draw for O (HIGH confidence — solved 1952, well-documented in game theory)
- Browser `<select>` accessibility — native element provides keyboard navigation and ARIA semantics without custom implementation (HIGH confidence — MDN / WHATWG HTML spec)

---

*Feature research for: v1.4 Difficulty Levels — browser tic-tac-toe (Rust/WASM)*
*Researched: 2026-04-27*
