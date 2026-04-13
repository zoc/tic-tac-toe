# Feature Landscape

**Domain:** Browser-based casual game (tic-tac-toe vs computer)
**Milestone scope:** v1.1 Polish & Feel — animation, persistence, audio, dark mode additions
**Researched:** 2026-04-13
**Confidence:** HIGH (MDN official docs + source code inspection)

---

## Context: What Already Exists

v1.0 is complete. The following are already built and validated:

| Already Built | Implementation |
|---------------|----------------|
| 3x3 CSS Grid board, X/O as text | CSS Grid, `renderBoard()` in main.js |
| Win highlighting (3 cells lit) | `cell--winning` class, `cell--winning { background: var(--accent) !important }` |
| Draw detection, game-over lockout | `game.get_status()`, `board--disabled` class |
| Score tracking in memory | `score` object, `updateScoreDisplay()` |
| Restart button | `resetGame()` + `hidden` toggle |
| Keyboard navigation (tab + enter/space) | `tabIndex`, keydown listener |
| Beatable AI (~25% mistake rate) | Rust minimax with random injection |
| Thinking delay: **NOT BUILT** | Comment in code says "synchronous — no artificial delay" |

**The 6 target features are all additive** — they enhance the existing foundation without changing game logic.

---

## Table Stakes for This Milestone

Features expected in any polished casual game. Missing = feels unfinished.

| Feature | Why Expected | Complexity | Implementation Path |
|---------|--------------|------------|---------------------|
| Smooth piece placement animation | Every polished game animates pieces appearing. Bare text pop-in feels jarring by modern standards. Users notice within 2 clicks. | LOW | CSS `@keyframes` + `.cell--x`, `.cell--o` class triggers. Scale from 0 to 1, or fade in. Add animation to cell when X/O class is added. |
| Persistent scores via localStorage | Players instinctively expect scores to survive a page refresh — they exist on every other game site. Score resetting on F5 feels like a bug. | LOW | `localStorage.setItem('ttt-score', JSON.stringify(score))` on every update. `getItem` + `JSON.parse` on init. Wrap in try/catch for SecurityError. |
| Dark mode respecting prefers-color-scheme | The game already uses a dark navy theme! But users with light-mode OS should see adapted colors. `prefers-color-scheme` is a Baseline "Widely available" CSS feature (since Jan 2020). Not respecting it feels like an oversight for a modern web app. | LOW | `@media (prefers-color-scheme: light) { :root { --bg: ...; --surface: ...; } }` overrides. CSS custom properties already in `:root` make this trivial — only need a light-mode override block. |
| Computer "thinking" delay (300–800ms) | Instant AI response feels robotic. Every board game with computer opponent (chess.com, lichess, casual games) adds an artificial pause. Without it, the game feels like a demo. v1.0 code already has a placeholder comment: *"Computer move is synchronous — no artificial delay (per CONTEXT.md Decision C)"* — this decision was explicitly deferred. | LOW | `setTimeout` wrapper in `handleCellClick()`. Convert synchronous `game.computer_move()` call to run inside 300–800ms timeout. The existing `isProcessing` guard and `board--disabled` class already provide the blocking needed. |

**Why these are "table stakes" for v1.1:** All four were explicitly listed in the prior FEATURES.md as P2 features ("add after validation"). v1.0 has been validated. These are now the baseline for a "finished product" feel rather than a demo.

---

## Differentiators for This Milestone

Features that set this implementation apart from the thousands of bare-bones tic-tac-toe implementations.

| Feature | Value Proposition | Complexity | Implementation Path |
|---------|-------------------|------------|---------------------|
| Animated win line through winning cells | Most implementations just change the background color of winning cells (that's what v1.0 does). An animated line *drawing itself* through the three cells is rare and visually satisfying. Gives the moment of winning a theatrical quality. | MEDIUM | SVG overlay positioned absolutely over the board. Animate `stroke-dashoffset` from full line length to 0 — the classic "draw a line" animation. JS calculates start/end points for each of the 8 possible win lines. Requires knowing the win line direction from `game.get_winning_positions()` (already returns 3 cell indices). |
| Sound effects with mute toggle | Sound feedback is the rarest feature in browser tic-tac-toe (competitor analysis: neither Neave's playtictactoe.org nor Google's embedded game has it). Audio feedback makes clicks feel tangible and wins feel rewarding. With a mute toggle, it's unobtrusive. | MEDIUM | Web Audio API with OscillatorNode — no file downloads needed. Synthesized beeps are tiny (zero bytes). Create a single shared `AudioContext` on first user interaction (satisfies autoplay policy). Three sounds: move click (short sine pulse ~200ms), win fanfare (rising 3-note sequence), lose buzz (falling tone). Mute state persisted in localStorage. |

---

## Anti-Features: What NOT to Build

| Anti-Feature | Why It Seems Attractive | Why to Avoid | What to Do Instead |
|--------------|------------------------|--------------|-------------------|
| Confetti / particle effects | "Winning should be celebrated!" | A confetti library (canvas-confetti etc.) adds 20–40KB to the bundle. Tic-tac-toe games are ~30 seconds each — confetti covers the board and blocks the restart flow. Overkill for a casual game. | The animated win line + win sound together provide sufficient celebration without obscuring the board. |
| CSS board flip/explosion on new game | "Board reset should be cinematic!" | Exit animations delay the new game start, frustrate repeat players, and are very hard to get right on the board-rebuild cycle (renderBoard clears innerHTML). | Simple board fade-in on reset is sufficient. The thinking delay and entry animations handle the pacing. |
| Animated SVG X and O glyphs | Hand-drawn animated marks look impressive | High implementation cost (custom SVG path animation per cell, coordination with renderBoard's innerHTML-clear pattern), significant visual risk. The current clean text glyphs are faster to render and more readable. | Stick with text. Add CSS entry animation (scale, fade) on the text characters instead. |
| Volume slider | "Let users control volume" | A range input adds UI chrome to a minimal interface. The mute toggle provides 80% of the value at 10% of the complexity. | Binary mute toggle (🔊/🔇 icon button). |
| Theme switcher (manual light/dark toggle) | "Let users override their OS preference" | Adds a UI control, a persisted preference key, JavaScript logic to manage `data-theme` attribute. Significant complexity for minimal gain when `prefers-color-scheme` already handles the use case automatically. | Pure CSS `@media (prefers-color-scheme: light)` — zero JS, zero UI, respects user preference automatically. |
| `localStorage` for game board state | "Restore game in progress after refresh" | Tic-tac-toe is a 30-second game. Mid-game persistence is not a felt pain point. Adds complexity to `renderBoard` and `init()` flow. | Only persist scores, not board state. |
| Audio files (.mp3/.ogg) | "Better sound quality" | Requires HTTP requests, encoding decisions, format compatibility (`<audio>` fallbacks), and bundle size. | Web Audio API OscillatorNode produces clean synthesized tones with zero bytes. Perfect for game UI sounds. |

---

## Feature-by-Feature: Expected User-Visible Behavior

### Feature 1: Smooth CSS Animations (Piece Placement)

**Trigger:** Human clicks a cell or computer places O.
**Visible behavior:** The X or O glyph appears with a quick entry animation — grows from 0 to full size with a slight overshoot (`cubic-bezier(0.34, 1.56, 0.64, 1)` "spring" easing), taking ~150–200ms. Feels snappy, not sluggish.
**Board reset:** When "New Game" is clicked, the board clears (cells empty) with no exit animation (instant clear). New pieces animate in on placement as normal.
**Win state:** Winning cells already get `cell--winning` background change. The win line SVG animation handles the celebration moment (see Feature 2).
**Prefers-reduced-motion:** Must respect `@media (prefers-reduced-motion: reduce)` — animations should be disabled or instant for users with vestibular disorders. This is an accessibility requirement.

**Implementation note:** The current `renderBoard()` clears `boardEl.innerHTML` and rebuilds all 9 cells. CSS animations must trigger on newly-added cells. Since cells are added to DOM fresh each time, `@keyframes` assigned to `.cell--x` and `.cell--o` will auto-trigger on insertion. No JS needed for the animation — just CSS.

---

### Feature 2: Animated Win Line

**Trigger:** Game ends in a win (for either player).
**Visible behavior:** After the win cells highlight (immediate), a line draws itself over the three winning cells — animating from one end to the other in ~400ms. Line is white or accent-colored, 4–6px thick. The draw effect uses SVG `stroke-dashoffset` animation (the "pen drawing a line" effect).
**Position:** SVG overlay absolutely positioned over `#board` at `position: absolute; inset: 0; pointer-events: none`. Board needs `position: relative`.
**8 possible lines:** horizontal (rows 0, 1, 2), vertical (cols 0, 1, 2), diagonal (TL→BR, TR→BL). JS computes SVG start/end `(x1, y1, x2, y2)` coordinates from the 3 winning cell indices returned by `game.get_winning_positions()`.
**Cleanup:** SVG element removed or hidden when `resetGame()` is called.

**Implementation note:** The SVG coordinate calculation is the most complex part. Each cell is 1/3 of board size. Center of cell at index `i`: `x = (col + 0.5) * (boardSize/3)`, `y = (row + 0.5) * (boardSize/3)`. Line length is computed at runtime from actual board pixel dimensions using `getBoundingClientRect()`. Must recalculate if viewport resizes.

---

### Feature 3: Computer "Thinking" Delay (300–800ms)

**Trigger:** Human places a valid move and game is still in progress.
**Visible behavior:** After human's move renders, the status shows "Computer is thinking…" and the board stays locked (already implemented via `board--disabled`). After 300–800ms delay, the computer's move appears (with piece animation from Feature 1). The delay makes the computer feel deliberate and prevents the jarring instant-response feel.
**Random range:** `Math.random() * 500 + 300` — produces 300–800ms. The randomness prevents a rhythmic "beat" that would feel mechanical.
**No WASM change:** The delay is pure JS (`setTimeout`). WASM `game.computer_move()` still executes synchronously inside the callback.

**Implementation note:** Current code already has the `isProcessing` guard and `board--disabled` applied before the computer move. The only change is wrapping `game.computer_move()` in a `setTimeout`. The `async`/`await` pattern with a `delay()` helper is cleaner than nested callbacks.

---

### Feature 4: Persistent Scores via localStorage

**Trigger:** Page load (read) and score update (write).
**Visible behavior:** User wins 3 games, closes the tab, reopens the URL — score shows 3 wins. Score persists indefinitely (no expiration). In private/incognito, localStorage is available but cleared when tab closes (expected behavior, no special handling needed).
**Key:** `'ttt-scores'` → JSON string `{"wins":3,"losses":1,"draws":2}`.
**Read on init:** In `main()`, after WASM init, read localStorage and populate `score` object before `updateScoreDisplay()`.
**Write on change:** In `handleGameOver()`, after incrementing `score.wins/losses/draws`, call `saveScore()`.
**Error handling:** Wrap in try/catch — `localStorage` throws `SecurityError` when cookies are blocked or origin is `file://`. Fall back silently to in-memory score.

**Implementation note:** This is the simplest of the 6 features. ~10 lines of JS. No UI change needed — the existing scoreboard display is reused.

---

### Feature 5: Sound Effects with Mute Toggle

**Trigger:** Move placement, win, loss, draw.
**Visible behavior:**
- **Move sound:** Short, satisfying click/plop (~150ms). Sine wave, 220Hz, fast decay. Distinct for X vs O (optional).
- **Win sound:** Brief ascending 3-note sequence (~400ms). Feels celebratory without being obnoxious.
- **Lose sound:** Falling tone (~300ms). Sad but brief.
- **Draw sound:** Neutral 2-tone (~250ms). Neither win nor lose.
- **Mute button:** Icon button (🔊/🔇) in header area. Clicking toggles mute state. State persisted in localStorage (`'ttt-muted'`).

**Web Audio API approach (recommended over `<audio>` elements):**
- Single `AudioContext` instance, created lazily on first user interaction (satisfies autoplay policy)
- `AudioContext.state` may be `'suspended'` if created outside click event — call `audioCtx.resume()` inside click handler
- Each sound: `createOscillator()` → configure frequency/type → `createGain()` → schedule `gain.exponentialRampToValueAtTime(0.001, ...)` for decay → `connect(audioCtx.destination)` → `oscillator.start()` → `oscillator.stop(t + duration)`
- Oscillators are created fresh per sound (they're one-shot — `OscillatorNode` cannot be restarted after `stop()`)
- Mute: check `muted` flag before calling `play*` functions, OR set `masterGain.gain.value = 0`

**Why Web Audio API over `<audio>` elements:**
- Zero file size (synthesized tones)
- Precise timing control
- No HTTP requests
- Works offline
- No audio format compatibility issues
- Perfect for short game UI sounds

**Accessibility note (MDN confirmed):** Sound must be user-controlled. The mute toggle satisfies this requirement. `aria-pressed` on the mute button communicates state to screen readers.

---

### Feature 6: Dark Mode (prefers-color-scheme)

**Trigger:** OS/browser dark mode setting changes.
**Visible behavior:**
- **Dark OS (current):** Unchanged — dark navy/red theme already looks correct.
- **Light OS:** Background becomes light gray/white, text becomes dark, cells become white/light gray, accent color (red) remains. The board still has clear contrast. Score display adapts.
- **Dynamic:** If user switches OS theme mid-session, CSS media query responds immediately without page refresh.

**Implementation approach — pure CSS (no JS required):**
```css
@media (prefers-color-scheme: light) {
  :root {
    --bg:      #f0f0f5;
    --surface: #ffffff;
    --text:    #1a1a2e;
    --text-dim: #666;
  }
  /* accent (--accent: #e94560) stays — red works on both themes */
}
```

**Why pure CSS not JS:** `window.matchMedia('(prefers-color-scheme: dark)')` + JS `addEventListener` approach is unnecessary complexity when the same result is achieved with a single `@media` block. CSS custom properties on `:root` mean all color usages inherit automatically.

**Manual override question:** The scope explicitly says "respecting prefers-color-scheme" — this means automatic, no manual toggle. A toggle button is an anti-feature for this milestone (see Anti-Features section).

**Board gap color:** The board background is `var(--accent)` (red), which forms the grid lines. This intentionally stays constant across themes — red grid lines on both dark and light themes is the brand identity.

---

## Feature Dependencies on Existing Code

```
[Feature 1: CSS Animations]
    └──modifies──> style.css (add @keyframes, animation to .cell--x, .cell--o)
    └──requires──> renderBoard() builds cells with .cell--x/.cell--o classes (already does)
    └──must-add──> @media (prefers-reduced-motion) override

[Feature 2: Win Line Animation]
    └──requires──> game.get_winning_positions() returns 3 indices (already works)
    └──requires──> #board has position: relative (add to style.css)
    └──modifies──> handleGameOver() — create SVG overlay after win
    └──modifies──> resetGame() — remove SVG overlay
    └──depends-on──> Feature 1 (piece animations) visually precede the line

[Feature 3: Thinking Delay]
    └──modifies──> handleCellClick() — wrap computer_move() in setTimeout
    └──requires──> isProcessing guard (already exists)
    └──requires──> board--disabled class (already applied before computer move)
    └──no WASM changes needed

[Feature 4: localStorage]
    └──modifies──> main() — read score from localStorage on init
    └──modifies──> handleGameOver() — write score after increment
    └──adds──> saveScore() helper function
    └──adds──> loadScore() helper function
    └──independent of all other features

[Feature 5: Sound Effects]
    └──adds──> audio.js module (or inline in main.js)
    └──adds──> mute button to index.html
    └──modifies──> handleCellClick() — play move sound after valid move
    └──modifies──> handleGameOver() — play win/lose/draw sound
    └──modifies──> style.css — mute button styles
    └──modifies──> localStorage — persist mute preference
    └──requires──> AudioContext created on first user interaction (click event)

[Feature 6: Dark Mode]
    └──modifies──> style.css only (add @media block)
    └──independent of all other features
    └──no JS, no HTML changes
```

---

## Complexity Summary

| Feature | Complexity | Estimated LOC | Risk |
|---------|------------|---------------|------|
| CSS animations (piece placement) | LOW | 20–30 CSS | LOW — pure CSS, no JS logic |
| Animated win line | MEDIUM | 30–50 JS + 10 CSS | MEDIUM — SVG coordinate math, viewport-relative sizing |
| Thinking delay | LOW | 10–15 JS | LOW — setTimeout wrapper, flag already exists |
| localStorage persistence | LOW | 20–30 JS | LOW — well-understood API, just needs try/catch |
| Sound effects + mute | MEDIUM | 60–80 JS | MEDIUM — AudioContext autoplay policy, state management, mute persistence |
| Dark mode (CSS only) | LOW | 10–15 CSS | LOW — pure CSS media query overrides |
| **Total estimate** | | **~150–200 LOC** | |

---

## Implementation Order Recommendation

**Suggested order (each is independent but some build nicely on each other):**

1. **Dark mode** — Pure CSS, zero risk, zero interdependency. Gets it out of the way immediately. (~15 min)
2. **localStorage** — Isolated JS change, easy to test independently. (~20 min)
3. **Thinking delay** — Small isolated change to `handleCellClick()`. Test carefully with the `isProcessing` guard. (~15 min)
4. **CSS animations (piece placement)** — Add after thinking delay so animations can be tested with the natural pace the delay provides. (~30 min)
5. **Sound effects + mute** — Requires AudioContext setup and autoplay policy awareness. Test across all 3 outcomes. (~60 min)
6. **Animated win line** — Most complex feature. Build last when all other polish is visible. SVG math is the only real challenge. (~60 min)

**Why this order:**
- Low-complexity features first builds confidence
- Thinking delay before animations: the delay makes animations more visible/testable
- Sound last (before win line) because it needs careful autoplay testing
- Win line last because it depends on win state being visually complete (pieces rendered, win cells highlighted)

---

## API Browser Compatibility (Confirmed via MDN Baseline)

| API / Feature | Baseline Status | Notes |
|---------------|-----------------|-------|
| `localStorage` | Widely available (since July 2015) | Universal. No compatibility concerns. |
| `prefers-color-scheme` | Widely available (since Jan 2020) | Universal. No polyfill needed. |
| Web Audio API (`AudioContext`, `OscillatorNode`, `GainNode`) | Widely available (since July 2015) | Universal in Chrome, Firefox, Safari, Edge. `AudioContext` may be `webkitAudioContext` on very old Safari but modern Safari (2020+) uses standard name. |
| CSS `@keyframes` + `animation` | Universal | No concerns for target browsers. |
| `prefers-reduced-motion` | Widely available | Must implement for accessibility. |
| SVG `stroke-dashoffset` animation | Universal | CSS animatable property, widely supported. |

All 6 features work in Chrome, Firefox, Safari (modern), and Edge — the stated target browsers.

---

## Pitfall Preview (Documented in PITFALLS.md)

| Feature | Key Pitfall |
|---------|-------------|
| Sound effects | `AudioContext` created outside user gesture starts in `'suspended'` state — must call `audioCtx.resume()` inside click handler |
| Sound effects | `OscillatorNode` is a one-shot source — **cannot** be restarted after `stop()`. Create a fresh oscillator for each sound. |
| Win line SVG | `renderBoard()` uses `boardEl.innerHTML = ''` — any SVG appended to `boardEl` gets wiped. SVG must be a **sibling** of `#board`, not a child. |
| CSS animations | `renderBoard()` rebuilds the entire DOM — `animation-fill-mode: forwards` may cause brief flash on re-render. Use `both` mode. |
| Thinking delay | If human clicks rapidly (double-click), the `isProcessing` flag must be set **synchronously** before the setTimeout, not inside the callback. (It already is in v1.0 code.) |
| localStorage | `JSON.parse(null)` returns `null` (first visit, no stored score) — must handle null case in `loadScore()`. |

---

## Sources

- MDN Web Docs: Web Audio API — Using the Web Audio API (updated Sep 18, 2025) — **HIGH confidence**
- MDN Web Docs: Web Audio API Best Practices (updated Sep 18, 2025) — **HIGH confidence**
- MDN Web Docs: OscillatorNode (updated Aug 27, 2025) — **HIGH confidence**
- MDN Web Docs: Window.localStorage (updated Nov 30, 2025) — **HIGH confidence**
- MDN Web Docs: prefers-color-scheme (updated Dec 5, 2025) — **HIGH confidence**
- v1.0 source code inspection: src/main.js (215 lines), src/style.css (186 lines) — **HIGH confidence**
- Competitor analysis (v1.0 research, 2026-04-12): playtictactoe.org (Neave), Google TTT, PaperGames.io — **MEDIUM confidence**

---

*Feature research for: browser-based tic-tac-toe v1.1 Polish & Feel (Rust/WASM)*
*Researched: 2026-04-13*
