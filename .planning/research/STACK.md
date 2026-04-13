# Technology Stack

**Project:** Tic-Tac-Toe WASM — v1.1 Polish & Feel
**Researched:** 2026-04-13
**Confidence:** HIGH (v1.0 base stack validated; new additions all native browser APIs — no npm deps required)

---

## What This Milestone Adds vs. v1.0

| v1.0 (validated, do not re-research) | v1.1 (this document) |
|---------------------------------------|----------------------|
| Rust/WASM game engine via wasm-pack | CSS animations for piece placement |
| Vite 8 + vite-plugin-wasm | Animated SVG win line |
| Vanilla JS + CSS Grid board | `setTimeout`-based thinking delay |
| In-memory score tracking | Persistent scores via `localStorage` |
| Dark navy/red theme | Sound effects via Web Audio API |
| | Dark mode via `prefers-color-scheme` |

**Net new npm dependencies: 0**
**Net new Rust/WASM changes: 0**
All six features are implementable with native browser platform APIs.

---

## New Capability Analysis

### Feature 1: Smooth CSS Animations — Piece Placement & Board Transitions

**Implementation:** Pure CSS — `@keyframes` + `animation` property
**New dependencies:** None

CSS `@keyframes` animations are "Baseline Widely available" across all target browsers (Chrome, Firefox, Safari, Edge) since well before 2020. The existing `style.css` already uses `transition` for hover states — `@keyframes` is the natural extension for entrance animations.

**Approach:**
- Add a `pop-in` keyframe (scale 0→1 with cubic-bezier overshoot) applied via `.cell--x` and `.cell--o` when cells are rendered
- Trigger by adding the CSS class when the cell DOM element is created in `renderBoard()`
- Use `transform: scale()` (GPU-composited, no layout reflow) — safe for `will-change: transform`
- Add `@media (prefers-reduced-motion: reduce)` override to skip animations for accessibility

**Integration point:** `src/style.css` (keyframe definition) + `src/main.js` `renderBoard()` (class assignment when value !== 0)

```css
@keyframes pop-in {
  0%   { transform: scale(0.4); opacity: 0; }
  70%  { transform: scale(1.1); opacity: 1; }
  100% { transform: scale(1);   opacity: 1; }
}

.cell--x, .cell--o {
  animation: pop-in 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) both;
}

@media (prefers-reduced-motion: reduce) {
  .cell--x, .cell--o { animation: none; }
}
```

---

### Feature 2: Animated Win Line

**Implementation:** Inline SVG `<line>` element with CSS stroke-dashoffset animation
**New dependencies:** None

The standard technique for "drawing" a line in CSS is `stroke-dasharray` + `stroke-dashoffset` animation. This is pure CSS on an inline SVG overlaid on the board. No canvas, no library.

**Approach:**
- Add an absolutely positioned `<svg>` overlay inside `.board` (or appended by JS)
- JS injects a `<line>` from start-cell center to end-cell center based on `winningPositions`
- CSS animates `stroke-dashoffset` from `line-length` to `0` over ~400ms
- Coordinates calculated from cell size: `boardSize / 3` per cell, center = `cellSize * col + cellSize/2`

**Integration point:**
- `index.html`: add `<svg class="win-line-overlay" ...>` inside `.board`
- `src/style.css`: `.win-line-overlay` positioned `absolute`, pointer-events none; `@keyframes draw-line`
- `src/main.js`: `handleGameOver()` computes line endpoints from `winningPositions[0]` and `winningPositions[2]`, sets SVG `<line>` attributes and triggers animation class

```css
@keyframes draw-line {
  from { stroke-dashoffset: var(--line-length); }
  to   { stroke-dashoffset: 0; }
}

.win-line {
  stroke: #fff;
  stroke-width: 6px;
  stroke-linecap: round;
  animation: draw-line 0.4s ease-out forwards;
}
```

**Key detail:** Set `stroke-dasharray` and `stroke-dashoffset` both to the line's pixel length (computed via JS `line.getTotalLength()` or manual `Math.hypot(dx, dy)`). The `board` needs `position: relative` (already set for overflow hidden) and the SVG needs `position: absolute; inset: 0; width: 100%; height: 100%`.

---

### Feature 3: Computer "Thinking" Delay (300–800ms)

**Implementation:** `setTimeout` in existing `handleCellClick` — pure JS refactor
**New dependencies:** None

The current code calls `game.computer_move()` synchronously inline. Converting to `setTimeout` wraps the existing logic in a closure.

**Approach:**
- Replace the synchronous computer move block with `setTimeout(() => { ... }, delay)`
- Delay = random value in [300, 800]ms: `300 + Math.random() * 500`
- The `isProcessing` flag and `board--disabled` class are set *before* the timeout (already done) — no change needed there
- Cancel pending timeout on `resetGame()` with `clearTimeout(thinkingTimer)`

**Integration point:** `src/main.js` — `handleCellClick()` and `resetGame()`

```js
const THINKING_MIN = 300;
const THINKING_MAX = 800;
let thinkingTimer = null;

// In handleCellClick, replace synchronous computer_move() block:
thinkingTimer = setTimeout(() => {
  const compPos = game.computer_move();
  // ... rest of existing logic unchanged
}, THINKING_MIN + Math.random() * (THINKING_MAX - THINKING_MIN));

// In resetGame():
clearTimeout(thinkingTimer);
thinkingTimer = null;
```

---

### Feature 4: Persistent Scores via localStorage

**Implementation:** `window.localStorage` — native browser API
**New dependencies:** None

`localStorage` is "Baseline Widely available" since July 2015 (MDN). It persists key/value strings across sessions for the same origin. The current `score` object is in-memory; persistence requires wrapping read/write with `localStorage`.

**Approach:**
- On startup: read `localStorage.getItem('ttt-score')`, parse JSON, hydrate the `score` object
- On score update: `localStorage.setItem('ttt-score', JSON.stringify(score))`
- Defensive: wrap in `try/catch` — `localStorage` throws `SecurityError` in `file://` URLs and when storage is blocked by user preferences
- Store schema: `{ wins: N, losses: N, draws: N }` — simple, forward-compatible

**Integration point:** `src/main.js`

```js
const SCORE_KEY = 'ttt-score';

function loadScore() {
  try {
    const saved = localStorage.getItem(SCORE_KEY);
    if (saved) Object.assign(score, JSON.parse(saved));
  } catch { /* blocked or file:// — silent fallback to in-memory */ }
}

function persistScore() {
  try {
    localStorage.setItem(SCORE_KEY, JSON.stringify(score));
  } catch { /* quota exceeded or blocked — silent */ }
}
```

**Call sites:** `loadScore()` before `updateScoreDisplay()` in `main()`, `persistScore()` after each score increment in `handleGameOver()`.

---

### Feature 5: Sound Effects with Mute Toggle

**Implementation:** Web Audio API (`AudioContext` + `OscillatorNode` + `GainNode`) — native browser API
**New dependencies:** None
**⚠️ Critical: Autoplay Policy**

Web Audio is "Baseline Widely available" since April 2021 (MDN). However, all browsers block `AudioContext` from producing sound until a user gesture has occurred on the page. This is critical for our game:

- **Safe:** First sound fires on a cell click (user gesture) — this satisfies the browser autoplay policy
- **Unsafe:** Any sound fired on page load (before interaction) — will be blocked silently

The game already waits for click interaction before any game logic runs, so the autoplay constraint is naturally satisfied.

**Why Web Audio API over `<audio>` elements?** For short, synthesized game sounds (100–300ms), Web Audio `OscillatorNode` produces sounds programmatically with zero file assets, no loading delay, and precise timing. No `.mp3`/`.ogg` files to ship. The sounds (place move, computer move, win, loss, draw) are simple tones — perfect for oscillators.

**Approach:**
- Single shared `AudioContext` created lazily on first click
- Factory function `playSound(type)` creates `OscillatorNode` → `GainNode` → `destination`, plays, and auto-disconnects after duration
- Mute toggle: `GainNode` master gain set to 0 when muted, 1 when unmuted
- Mute state persisted in `localStorage` key `'ttt-muted'`
- Add mute button `<button id="mute-btn">` to `index.html`

```js
let audioCtx = null;
let masterGain = null;
let isMuted = false;

function ensureAudio() {
  if (audioCtx) return;
  audioCtx = new AudioContext();
  masterGain = audioCtx.createGain();
  masterGain.connect(audioCtx.destination);
  masterGain.gain.value = isMuted ? 0 : 1;
}

function playSound(type) {
  ensureAudio();
  const sounds = {
    place:    { freq: 440, duration: 0.08, type: 'sine' },
    computer: { freq: 300, duration: 0.08, type: 'sine' },
    win:      { freq: 660, duration: 0.35, type: 'triangle' },
    loss:     { freq: 220, duration: 0.4,  type: 'sawtooth' },
    draw:     { freq: 380, duration: 0.25, type: 'sine' },
  };
  const s = sounds[type];
  const osc = audioCtx.createOscillator();
  const gain = audioCtx.createGain();
  osc.type = s.type;
  osc.frequency.value = s.freq;
  gain.gain.setValueAtTime(0.3, audioCtx.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + s.duration);
  osc.connect(gain);
  gain.connect(masterGain);
  osc.start();
  osc.stop(audioCtx.currentTime + s.duration + 0.05);
}
```

**`AudioContext` suspended state:** Chrome suspends `AudioContext` if created before a user gesture. `ensureAudio()` called on first click guarantees the context is created after a gesture. Alternatively, call `audioCtx.resume()` inside any click handler for safety.

**Mute button HTML addition:**
```html
<button class="mute-btn" id="mute-btn" aria-label="Toggle sound" aria-pressed="false">🔊</button>
```

**Integration points:**
- `index.html`: mute button (near scoreboard or title)
- `src/style.css`: `.mute-btn` styling (small, unobtrusive)
- `src/main.js`: `playSound()` calls at cell placement, computer move, and game over

---

### Feature 6: Dark Mode — `prefers-color-scheme`

**Implementation:** CSS `@media (prefers-color-scheme: light)` override — pure CSS
**New dependencies:** None

`prefers-color-scheme` is "Baseline Widely available" since January 2020 (MDN). The current app already uses a dark theme (`#1a1a2e` navy, `#e94560` red). The dark theme becomes the *default* and light mode overrides it.

**Approach:**
- Keep existing CSS variables in `:root` as the dark defaults (no change to existing rules)
- Add `@media (prefers-color-scheme: light)` block overriding CSS variables to light values
- Light palette suggestion: white/near-white background, darker navy text, same red accent

```css
@media (prefers-color-scheme: light) {
  :root {
    --bg:       #f5f5f5;   /* light grey background */
    --surface:  #ffffff;   /* white cells */
    --accent:   #c0392b;   /* slightly darker red for contrast on white */
    --text:     #1a1a2e;   /* navy text — inverts from dark mode */
    --text-dim: #555;
  }
}
```

**No JS required.** The CSS variable override cascades to all components that already use `var(--bg)`, `var(--surface)`, `var(--accent)`, `var(--text)`, `var(--text-dim)` — the entire existing UI adapts automatically.

**Manual override (optional, future):** If a user toggle is later needed, a `data-theme="light"` attribute on `<html>` with matching CSS selector `[data-theme="light"]` can be added. Not required for this milestone.

**Integration point:** `src/style.css` only — appended at bottom

---

## Dependency Summary

### New npm Dependencies: NONE

No additions to `package.json`. All six features are native browser platform.

### New Rust/Cargo Dependencies: NONE

No Rust code changes. All features live in the JS/CSS layer. The thinking delay is a JS `setTimeout`. Score persistence is JS `localStorage`. Sounds are JS Web Audio. Animations are CSS. Dark mode is CSS.

### Existing Stack — Unchanged

| Technology | Version | Status |
|------------|---------|--------|
| Rust (stable) | 1.94.1 | Unchanged |
| wasm-bindgen | 0.2.118 | Unchanged |
| wasm-pack | 0.14.0 | Unchanged |
| Vite | 8.0.8 | Unchanged |
| vite-plugin-wasm | 3.6.0 | Unchanged |
| All Cargo dependencies | (see v1.0 STACK.md) | Unchanged |

---

## Browser API Compatibility

| API / Feature | Chrome | Firefox | Safari | Edge | Baseline |
|---------------|--------|---------|--------|------|----------|
| CSS `@keyframes` animations | 43+ | 16+ | 9+ | 12+ | Widely available |
| SVG `stroke-dashoffset` animation | All | All | All | All | Widely available |
| `prefers-color-scheme` | 76+ | 67+ | 12.1+ | 79+ | Widely available (Jan 2020) |
| `window.localStorage` | 4+ | 3.5+ | 4+ | 12+ | Widely available (Jul 2015) |
| Web Audio API | 35+ | 25+ | 14.1+ | 12+ | Widely available (Apr 2021) |
| `setTimeout` | All | All | All | All | Universal |

All APIs are supported in every target browser (Chrome, Firefox, Safari, Edge modern).

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Howler.js / Tone.js | External audio libs for 5 simple tones — massive overkill. Adds ~50–200KB bundle. | Web Audio API `OscillatorNode` — zero deps, zero bytes |
| GSAP / Anime.js | JS animation libs for CSS `@keyframes` that take 5 lines — unjustified complexity. | CSS `@keyframes` + `animation` property |
| Preloaded audio files (.mp3/.ogg) | Requires asset management, codec support matrix, loading logic, fallbacks. | Synthesized oscillator tones — plays instantly, no assets |
| `prefers-color-scheme` JS listener | Real-time theme switching without page reload adds complexity for near-zero benefit. | Pure CSS `@media` — OS change reapplies automatically |
| IndexedDB | Transactional async storage for 3 integers — extreme overkill. | `localStorage` — sync, simple, right-sized |
| CSS custom property animation with `@property` | Houdini — not fully supported in Safari without flags. | Standard `@keyframes` — universally supported |
| Motion library (Framer Motion) | React-only, 35KB+ for a vanilla JS page. | CSS `animation` property |
| Web Animations API (WAAPI) | More JS complexity for what CSS keyframes handle in 10 lines. Useful if animations need JS-controlled playback; we don't. | CSS `@keyframes` |

---

## Integration Map

```
src/main.js changes:
  handleCellClick()     → setTimeout for thinking delay
                        → playSound('place') on human move
                        → playSound('computer') after computer move
  handleGameOver()      → playSound('win'|'loss'|'draw')
                        → persistScore()
                        → renderWinLine() [new helper]
  resetGame()           → clearTimeout(thinkingTimer)
                        → removeWinLine() [new helper]
  main()                → loadScore() before updateScoreDisplay()
                        → loadMuteState()

index.html changes:
  <svg class="win-line-overlay"> inside .board
  <button id="mute-btn"> near scoreboard

src/style.css changes:
  @keyframes pop-in       (piece placement animation)
  @keyframes draw-line    (win line animation)
  .win-line-overlay       (SVG overlay positioning)
  .mute-btn               (button style)
  @media prefers-color-scheme: light  (dark mode override)
  @media prefers-reduced-motion       (animation accessibility)

Rust/WASM (src/*.rs): NO CHANGES
Cargo.toml:            NO CHANGES
package.json:          NO CHANGES
vite.config.js:        NO CHANGES
```

---

## Critical Implementation Notes

### Web Audio: `AudioContext` Lifecycle
Create `AudioContext` lazily inside the first user-gesture handler (cell click). Do NOT create at module top level — Chrome and Safari will suspend it immediately and may not resume. Pattern:

```js
// WRONG: creates suspended context before any gesture
const audioCtx = new AudioContext();

// RIGHT: lazy creation on first gesture
function ensureAudio() {
  if (audioCtx) return;
  audioCtx = new AudioContext();
  // ...
}
boardEl.addEventListener('click', (e) => {
  ensureAudio(); // safe — inside click handler
  handleCellClick(e);
});
```

### localStorage: Always `try/catch`
`localStorage.setItem()` throws `QuotaExceededError` if storage is full, and `SecurityError` in `file://` URLs and private browsing in some configurations. Silent fallback to in-memory is the right behavior.

### CSS Animations: Re-triggering on Re-render
`renderBoard()` rebuilds all 9 cell DOM nodes via `boardEl.innerHTML = ''`. This means newly created cells with a CSS class automatically trigger their `animation` from scratch — no need to force-reflow tricks. The existing innerHTML rebuild pattern is actually ideal for animation triggering.

### SVG Win Line: Board `position: relative`
The `.board` already has `overflow: hidden` which establishes a containing block. The SVG overlay needs `position: absolute; inset: 0; pointer-events: none; width: 100%; height: 100%`. The `width/height: var(--board-size)` on `.board` means the SVG coordinate space matches board pixel dimensions.

### Mute Toggle: Accessibility
The `<button>` must have `aria-pressed` toggled between `"true"` and `"false"` on click. Use `aria-label="Toggle sound"` rather than emoji text for screen reader clarity.

---

## Sources

- MDN Web Docs — Web Audio API (last modified Oct 2025, HIGH confidence — official)
- MDN Web Docs — `prefers-color-scheme` (last modified Dec 2025, HIGH confidence — official)
- MDN Web Docs — `window.localStorage` (last modified Nov 2025, HIGH confidence — official)
- MDN Web Docs — Autoplay guide for media and Web Audio APIs (last modified Sep 2025, HIGH confidence — official)
- MDN Web Docs — Using CSS animations (last modified Dec 2025, HIGH confidence — official)
- Existing codebase: `src/main.js`, `src/style.css`, `index.html` (direct inspection, HIGH confidence)

---

*Stack research for: Tic-Tac-Toe WASM v1.1 Polish & Feel*
*Researched: 2026-04-13*
