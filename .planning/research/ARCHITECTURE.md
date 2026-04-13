# Architecture Patterns — v1.1 Polish & Feel

**Project:** Tic-Tac-Toe WASM  
**Milestone:** v1.1 Polish & Feel  
**Researched:** 2026-04-13  
**Scope:** Integration of CSS animations, Web Audio API, localStorage, and dark mode into the existing JS/CSS layer. This document supersedes the v1.0 foundation architecture for this milestone.

---

## Existing Architecture Baseline

Understanding what exists before describing what changes:

```
┌─────────────────────────────────────────────────────┐
│  RUST / WASM  (src/lib.rs, src/game.rs, src/ai.rs)  │
│  board state · move validation · AI · win detection  │
│  ~927 LOC — NO CHANGES NEEDED FOR THIS MILESTONE     │
└─────────────────────┬───────────────────────────────┘
                      │  wasm_bindgen exported functions
                      │  get_board() → Uint8Array[9]
                      │  make_move(idx) → bool
                      │  computer_move() → u8
                      │  get_status() → &str
                      │  get_winning_positions() → Vec<usize>
                      ▼
┌─────────────────────────────────────────────────────┐
│  JAVASCRIPT LAYER  (src/main.js  ~215 LOC)          │
│  renderBoard() · handleCellClick() · resetGame()    │
│  handleGameOver() · updateScoreDisplay()            │
│                                                     │
│  State: game (WasmGame), score {w/l/d}, isProcessing│
└──────┬──────────────────┬──────────────────────────-┘
       │                  │
       ▼                  ▼
┌─────────────┐   ┌───────────────────────────────────┐
│ index.html  │   │ src/style.css                     │
│ .board      │   │ :root CSS custom properties       │
│ .cell[0-8]  │   │ .cell--x, .cell--o, .cell--winning│
│ .scoreboard │   │ .board--disabled                  │
│ #status-msg │   │ transition: background 0.1s ease  │
└─────────────┘   └───────────────────────────────────┘
```

**Critical constraint:** The WASM bridge (`src/wasm_api.rs`) does NOT need any changes for this milestone. All six new features integrate exclusively in the JS/CSS/HTML layer.

---

## Feature Integration Map

Each feature mapped to its exact integration points in the existing codebase.

---

### Feature 1: CSS Piece Placement Animations

**What:** `scale + opacity` pop-in when an X or O appears in a cell.

**Integration point:** `renderBoard()` in `main.js` (lines 28–65).

The current flow re-creates all 9 cells on every `renderBoard()` call via `boardEl.innerHTML = ''`. This is the ideal hook: newly-created `.cell--x` and `.cell--o` elements automatically receive the animation class on DOM insertion — no JS animation logic needed.

**New vs Modified:**
- **MODIFIED:** `src/style.css` — add `@keyframes cell-pop` and apply to `.cell--x`, `.cell--o`
- **MODIFIED:** `src/main.js` — no JS change needed; CSS handles it on class assignment

**Pattern:**
```css
@keyframes cell-pop {
  from { transform: scale(0.3); opacity: 0; }
  to   { transform: scale(1);   opacity: 1; }
}

/* Only animate if user hasn't requested reduced motion */
@media (prefers-reduced-motion: no-preference) {
  .cell--x, .cell--o {
    animation: cell-pop 0.18s cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
}
```

`cubic-bezier(0.34, 1.56, 0.64, 1)` is a spring easing — slight overshoot then snaps back. Feels tactile. `animation-fill-mode: both` holds the from-state before paint, preventing a flash.

**`transform: scale()` is the right property:** Animating `transform` is GPU-composited; it does not trigger layout reflow. Avoid animating `font-size`, `width`, or `height` for this effect.

**Dependency:** None. Fully isolated.

---

### Feature 2: Animated Win Line

**What:** A line or highlight animation drawn through the 3 winning cells after game ends.

**Integration point:** `handleGameOver()` in `main.js` (line 81) and `renderBoard(winningPositions)` (lines 28–65). Currently `cell--winning` flips background to accent red. The win line extends this.

**Approach: CSS-only positioned overlay with JS class selection.**

Win lines in tic-tac-toe are always one of 8 fixed lines (3 rows, 3 columns, 2 diagonals). These can be pre-defined as 8 CSS classes, selected by JS based on the winning positions array returned by `game.get_winning_positions()`.

**New vs Modified:**
- **MODIFIED:** `index.html` — add `<div class="win-line" id="win-line" hidden></div>` as a **sibling** of `.board` (not inside it), using absolute/relative positioning to overlay it
- **MODIFIED:** `src/style.css` — `.win-line` with `position: absolute`, `transform-origin: left center`, and 8 `win-line--*` position classes + `@keyframes win-draw`
- **MODIFIED:** `src/main.js` `handleGameOver()` — call `showWinLine(winPositions)` after `renderBoard(winPositions)`; call `clearWinLine()` in `resetGame()`

> **Why sibling, not child:** The current `renderBoard()` calls `boardEl.innerHTML = ''` to rebuild cells. A win-line `<div>` inside `.board` would be deleted on every render. As a sibling overlaid with `position: absolute`, it survives board re-renders.

**Pattern:**
```js
// src/main.js — new constants and helpers
const WIN_LINE_CONFIGS = {
  '0,1,2': 'win-line--row0',
  '3,4,5': 'win-line--row1',
  '6,7,8': 'win-line--row2',
  '0,3,6': 'win-line--col0',
  '1,4,7': 'win-line--col1',
  '2,5,8': 'win-line--col2',
  '0,4,8': 'win-line--diag-lr',
  '2,4,6': 'win-line--diag-rl',
};

function showWinLine(positions) {
  const key = [...positions].sort((a, b) => a - b).join(',');
  const cls = WIN_LINE_CONFIGS[key];
  if (!cls) return;
  winLineEl.className = `win-line ${cls}`;
  winLineEl.hidden = false;
}

function clearWinLine() {
  winLineEl.hidden = true;
  winLineEl.className = 'win-line';
}
```

```css
/* ─── Win line overlay ─── */
.board-wrapper {
  position: relative;  /* new wrapper around .board + .win-line */
}

.win-line {
  position: absolute;
  height: 6px;
  background: #fff;
  border-radius: 3px;
  transform-origin: left center;
  pointer-events: none;
  opacity: 0.9;
}

@media (prefers-reduced-motion: no-preference) {
  .win-line {
    animation: win-draw 0.3s ease-out both;
    animation-delay: 0.15s;  /* slight delay after last piece pops in */
  }
}

@keyframes win-draw {
  from { transform: scaleX(0); }
  to   { transform: scaleX(1); }
}

/* 8 fixed line positions, sized/positioned to span exactly 3 cells */
.win-line--row0 { top: calc(1/6 * 100%); left: 5%; width: 90%; }
.win-line--row1 { top: calc(3/6 * 100%); left: 5%; width: 90%; }
.win-line--row2 { top: calc(5/6 * 100%); left: 5%; width: 90%; }
/* columns: rotate 90deg around center */
.win-line--col0 { top: 50%; left: calc(1/6 * 100%); width: 90%; transform-origin: left center; rotate: 90deg; }
/* ... etc — exact values tuned during implementation */
```

**Dependency:** Implementation should follow Feature 1 (animations established) to understand timing. The `board-wrapper` div addition to `index.html` is required before coding this.

---

### Feature 3: Computer "Thinking" Delay

**What:** 300–800ms artificial delay before `game.computer_move()` executes.

**Integration point:** `handleCellClick()` in `main.js` (lines 113–166), specifically the synchronous computer move block (lines 138–165).

**Current flow (synchronous):**
```
human click → make_move() → renderBoard() → [instant] → computer_move() → renderBoard()
```

**New flow (async with delay):**
```
human click → make_move() → renderBoard()
  → isProcessing=true, board disabled, setStatus("Computer's turn")
  → await thinkDelay()   ← new step
  → computer_move() → renderBoard()
  → isProcessing=false, board enabled
```

**New vs Modified:**
- **MODIFIED:** `src/main.js` `handleCellClick()` — add `thinkDelay()` helper; convert `function handleCellClick` to `async function handleCellClick`; `await thinkDelay()` before `game.computer_move()`

```js
// New helper
const THINK_MIN = 300;
const THINK_MAX = 800;
function thinkDelay() {
  const ms = THINK_MIN + Math.random() * (THINK_MAX - THINK_MIN);
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Modified inside handleCellClick():
isProcessing = true;
boardEl.classList.add('board--disabled');
setStatus("Computer's turn", 'computer-turn');

await thinkDelay();  // ← insert here

const compPos = game.computer_move();
```

**The `isProcessing` guard already exists** and blocks all clicks during the delay window — no additional concurrency guard needed. Making `handleCellClick` async is safe because it is called as a DOM event handler; unhandled async rejections are the only risk, which is already handled by the outer try/catch pattern.

**Dependency:** None. Fully isolated. Must be done before Feature 6 (sound), since sound timing depends on the async structure being correct.

---

### Feature 4: Persistent Scores via localStorage

**What:** `score` object persisted across page refreshes.

**Integration point:**
- `const score = { wins: 0, losses: 0, draws: 0 }` — line 13 in `main.js`
- `handleGameOver()` where scores increment — lines 90–102
- `main()` where `updateScoreDisplay()` is called on init — line 188

**New vs Modified:**
- **MODIFIED:** `src/main.js` — replace static `score` initialization with `loadScore()`; add `saveScore()` after each increment in `handleGameOver()`

```js
// Replace line 13 and add these functions
const SCORE_KEY = 'ttt-score';

function loadScore() {
  try {
    const saved = localStorage.getItem(SCORE_KEY);
    return saved ? JSON.parse(saved) : { wins: 0, losses: 0, draws: 0 };
  } catch {
    return { wins: 0, losses: 0, draws: 0 };  // SecurityError in private browsing
  }
}

function saveScore() {
  try {
    localStorage.setItem(SCORE_KEY, JSON.stringify(score));
  } catch {
    // Storage quota exceeded or unavailable — silently ignore
  }
}

// line 13 becomes:
const score = loadScore();

// In handleGameOver(), after each increment, add:
saveScore();
```

**localStorage availability:** `localStorage` throws a `SecurityError` in private browsing (Safari especially). The `try/catch` defensive pattern makes it fail-safe — scores fall back to in-memory only when storage is unavailable. This is the MDN-recommended approach.

**Dependency:** None. Fully isolated. Good first feature to implement.

---

### Feature 5: Sound Effects with Mute Toggle

**What:** Synthesized sounds (no audio files) for: human move, computer move, win, loss, draw. Mute toggle button persisted to localStorage.

**Integration point:** Multiple call sites in `main.js` — after `make_move()` succeeds and in `handleGameOver()`.

**Technology: Web Audio API with synthesized oscillator tones. No audio files.**

Reasons: No asset pipeline, no fetch/decode overhead, no file hosting, instant availability. Synthesized bleeps fit the aesthetic. Web Audio API is baseline widely available across all modern browsers since 2013 (HIGH confidence — MDN official docs).

**Autoplay policy:** Web Audio requires a user gesture before sound can play. Every sound call in this app is triggered by a user click (cell click), so `AudioContext` will always be created in response to user interaction. The `audioCtx.state === 'suspended'` check in `getCtx()` handles any edge cases where the context was auto-suspended.

**New vs Modified:**
- **NEW:** `src/audio.js` — `AudioContext` singleton + `sounds` named exports + `toggleMute()`
- **MODIFIED:** `index.html` — add `<button id="mute-btn" aria-label="Toggle sound">🔊</button>` near the title
- **MODIFIED:** `src/main.js` — `import { sounds, toggleMute, isMuted } from './audio.js'`; add sound calls at 5 event sites; wire mute button

**`src/audio.js` structure:**
```js
// src/audio.js — new module
const MUTE_KEY = 'ttt-muted';

let audioCtx = null;
let muted = (() => {
  try { return localStorage.getItem(MUTE_KEY) === 'true'; }
  catch { return false; }
})();

function getCtx() {
  if (!audioCtx) audioCtx = new AudioContext();
  if (audioCtx.state === 'suspended') audioCtx.resume();
  return audioCtx;
}

function playTone({ frequency, type = 'sine', duration = 0.1, gain = 0.25 }) {
  if (muted) return;
  const ctx = getCtx();
  const osc = ctx.createOscillator();
  const gainNode = ctx.createGain();
  osc.connect(gainNode);
  gainNode.connect(ctx.destination);
  osc.type = type;
  osc.frequency.value = frequency;
  gainNode.gain.setValueAtTime(gain, ctx.currentTime);
  gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + duration);
}

export const sounds = {
  move:         () => playTone({ frequency: 440, duration: 0.08 }),
  computerMove: () => playTone({ frequency: 330, duration: 0.08 }),
  win:          () => {
    playTone({ frequency: 523, duration: 0.12 });
    setTimeout(() => playTone({ frequency: 659, duration: 0.15 }), 80);
    setTimeout(() => playTone({ frequency: 784, duration: 0.2  }), 180);
  },
  loss:         () => playTone({ frequency: 200, type: 'sawtooth', duration: 0.35 }),
  draw:         () => playTone({ frequency: 360, duration: 0.2 }),
};

export function toggleMute() {
  muted = !muted;
  try { localStorage.setItem(MUTE_KEY, String(muted)); } catch { /* ignore */ }
  return muted;
}

export function isMuted() { return muted; }
```

**Sound call sites in `main.js`:**
| Event | Location in `main.js` | Sound |
|-------|----------------------|-------|
| Human move placed | `handleCellClick()` after `game.make_move()` succeeds (line ~128) | `sounds.move()` |
| Computer move placed | `handleCellClick()` after `game.computer_move()` returns (line ~154) | `sounds.computerMove()` |
| Human wins | `handleGameOver()` in `winner === PLAYER_X` branch (line ~93) | `sounds.win()` |
| Computer wins | `handleGameOver()` in `winner === PLAYER_O` branch (line ~96) | `sounds.loss()` |
| Draw | `handleGameOver()` in draw branch (line ~99) | `sounds.draw()` |

**Mute button wiring in `main.js`:**
```js
const muteBtn = document.getElementById('mute-btn');
muteBtn.addEventListener('click', () => {
  const nowMuted = toggleMute();
  muteBtn.textContent = nowMuted ? '🔇' : '🔊';
  muteBtn.setAttribute('aria-pressed', String(nowMuted));
});
// Set initial state on load:
muteBtn.textContent = isMuted() ? '🔇' : '🔊';
muteBtn.setAttribute('aria-pressed', String(isMuted()));
```

**Dependency:** Depends on Feature 3 (async thinking delay) being implemented first, so that the timing of `sounds.computerMove()` is correct (it should fire *after* the delay, when the computer's piece is actually placed).

---

### Feature 6: Dark Mode (prefers-color-scheme)

**What:** Light theme for users who prefer light mode; dark theme remains default for dark-mode and no-preference users.

**Integration point:** `src/style.css` — the `:root` custom properties block (lines 8–16). Currently hard-coded dark-navy values.

**Strategy — dark-first (matches existing theme):** Keep existing `:root` values unchanged (dark theme). Add a `@media (prefers-color-scheme: light)` block overriding the custom properties with light equivalents. Zero JS needed.

```css
/* ─── Dark mode (default — existing :root values unchanged) ─── */
:root {
  --bg:      #1a1a2e;
  --surface: #16213e;
  --accent:  #e94560;
  --text:    #e0e0e0;
  --text-dim:#888;
  --hover-bg: #1e2a4a;  /* extract hardcoded hover color to variable */
}

/* ─── Light mode (system preference override) ─── */
@media (prefers-color-scheme: light) {
  :root {
    --bg:      #f0f2f7;
    --surface: #ffffff;
    --accent:  #e94560;   /* red accent stays consistent across themes */
    --text:    #1a1a2e;   /* inverted: dark text on light bg */
    --text-dim:#666;
    --hover-bg: #e8eaf0;  /* light hover tint */
  }
}
```

**Because all CSS uses `var(--*)` custom properties consistently**, light mode requires only this one `@media` block. All 14+ custom property usages in `style.css` automatically respond.

**Two hardcoded exceptions to fix:**

1. `.cell:hover { background: #1e2a4a }` (line 126) — hardcoded dark hex. Fix: extract to `var(--hover-bg)` in `:root` (added above).

2. The error overlay in `main.js` (lines 207–214) has hardcoded inline styles (`background: #1a1a2e`, `color: #e94560`). Fix: replace with a CSS class `error-overlay` defined in `style.css`.

**New vs Modified:**
- **MODIFIED:** `src/style.css` — add `@media (prefers-color-scheme: light)` block; add `--hover-bg` CSS variable; replace hardcoded `.cell:hover` color with `var(--hover-bg)`; add `.error-overlay` class
- **MODIFIED:** `src/main.js` — in the `.catch` error handler, replace hardcoded inline styles with `wrapper.className = 'error-overlay'`

**Optional — manual theme toggle:** Not required for this milestone (spec says "respecting prefers-color-scheme"). Future: set `data-theme` attribute on `<html>` and use attribute selectors to override the media query.

**Dependency:** None. Pure CSS change. Can be implemented first.

---

## Component Map: New vs Modified

| File | Status | What Changes |
|------|--------|-------------|
| `src/style.css` | **MODIFIED** | `@keyframes cell-pop`; `.cell--x/.cell--o` animation; `.win-line` + 8 position classes + `@keyframes win-draw`; `@media prefers-color-scheme: light` with light `:root`; `--hover-bg` variable; `.error-overlay` class |
| `src/main.js` | **MODIFIED** | `thinkDelay()` + `async handleCellClick`; `loadScore()`/`saveScore()` replacing static `score` init; `showWinLine()`/`clearWinLine()` helpers; `import { sounds }` + 5 call sites; mute button wiring; replace inline error styles with class |
| `index.html` | **MODIFIED** | Add `<div class="board-wrapper">` around `.board`; add `<div id="win-line" hidden class="win-line">` as sibling of `.board` inside wrapper; add `<button id="mute-btn">` near title |
| `src/audio.js` | **NEW** | `AudioContext` singleton; `playTone()` primitive; `sounds.*` named exports; `toggleMute()`/`isMuted()`; mute localStorage persistence |
| `src/wasm_api.rs` | **UNCHANGED** | No Rust changes needed |
| `Cargo.toml` | **UNCHANGED** | No new Rust dependencies |
| `vite.config.js` | **UNCHANGED** | Vite handles ES modules natively; `audio.js` module is imported like any other |

---

## Data Flow Changes

### Score Data Flow (before → after)

**Before:**
```
in-memory { wins, losses, draws }  →  updateScoreDisplay()  →  DOM
(resets on page refresh)
```

**After:**
```
localStorage ──→ loadScore() at startup ──→ in-memory score object
                                                │
                                saveScore() ◄──┤  (on each game end)
                                                │
                               updateScoreDisplay()  →  DOM
```

### Computer Move Flow (before → after)

**Before (synchronous, ~instant):**
```
handleCellClick → make_move → renderBoard → computer_move → renderBoard
```

**After (async with delay + sound):**
```
handleCellClick (async)
  → make_move() → sounds.move() → renderBoard()
  → isProcessing=true, board disabled
  → await thinkDelay()
  → computer_move() → sounds.computerMove() → renderBoard()
  → isProcessing=false, board enabled
```

### Sound Data Flow (new)

```
user event (click)
  → handleCellClick / handleGameOver
    → sounds.move() / sounds.win() / etc.
      → audio.js getCtx() (lazy init AudioContext)
        → OscillatorNode → GainNode → AudioContext.destination
          (skipped if muted === true)
```

---

## Build Order (Dependency-Aware)

| Order | Feature | Rationale |
|-------|---------|-----------|
| **1** | Dark mode | Pure CSS addition. Zero JS risk. Validates the custom property system and the new `--hover-bg` variable before other CSS changes layer on. |
| **2** | Persistent scores | Pure JS data layer. No visual changes. Easy to validate by refreshing page. No dependencies. |
| **3** | CSS piece placement animations | Style-only. Adding `@keyframes` to existing classes. Zero JS changes. Low risk. |
| **4** | Computer thinking delay | Converts `handleCellClick` to `async`. Medium risk — restructures core flow. Must be done before sounds (Feature 5 depends on timing correctness). |
| **5** | Win line animation | New DOM element + CSS + JS helpers. Requires `index.html` change (board-wrapper). Build after animations (Feature 3) to calibrate timing. |
| **6** | Sound effects + mute toggle | New module + HTML + multiple call sites. Most surface area. Requires Feature 4 (async flow) to be correct. Last because sounds are supplemental. |

---

## Patterns to Follow

### Pattern 1: CSS Class-Based Animation Triggering
**What:** All animations start when a CSS class is added to an element. JS never calls `element.animate()`.  
**Example:** `cell.classList.add('cell--x')` triggers `cell-pop` automatically.  
**Why:** No JS/CSS coupling. Easy to debug in DevTools.

### Pattern 2: `prefers-reduced-motion` Guard on All Animations
**What:** Wrap `@keyframes` usage in `@media (prefers-reduced-motion: no-preference)`.  
**Why:** Users with vestibular disorders or motion sensitivity can disable animations at OS level. Respecting this is required for accessibility. The game remains fully functional without animations.

### Pattern 3: Defensive localStorage Access
**What:** Always `try/catch` localStorage calls. Never throw on storage failure.  
**Why:** `SecurityError` in private browsing (Safari). Quota exceptions on full storage. The game must degrade gracefully.

### Pattern 4: AudioContext Lazy Init + Suspended-State Resume
**What:** Create `AudioContext` on first sound call. Always check `ctx.state === 'suspended'` and call `resume()` before use.  
**Why:** Browsers suspend or block `AudioContext` creation before user gesture. Lazy init guarantees a gesture has occurred.

### Pattern 5: CSS Custom Property Theming (Dark/Light)
**What:** All color values reference `var(--color-name)`. No hardcoded hex values outside `:root`.  
**Why:** Theme switching works by changing `:root` variables only. One `@media` block handles the entire color system.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Win Line as a Child of `.board`
**What:** Placing the win-line `<div>` inside `.board`, which is cleared by `boardEl.innerHTML = ''`.  
**Consequence:** Win line disappears on any re-render.  
**Instead:** Position win-line as a sibling within a shared wrapper `<div class="board-wrapper">`.

### Anti-Pattern 2: Animating Layout Properties for Piece Pop-In
**What:** Using `font-size`, `width`, or `height` keyframes to scale piece content.  
**Consequence:** Layout reflow on every animation frame — janky on mobile.  
**Instead:** `transform: scale()` — GPU-composited, zero reflow.

### Anti-Pattern 3: `new AudioContext()` on Every Sound Call
**What:** Creating a fresh `AudioContext` each time a tone is played.  
**Consequence:** Browsers limit active audio contexts (~6). Rapid game play triggers garbage collection pauses and eventual browser blocking.  
**Instead:** Singleton `audioCtx` in `audio.js`, created once on first sound call.

### Anti-Pattern 4: `setTimeout` to Wait for CSS Animation Completion
**What:** Hardcoded JS delay to "wait" for a CSS animation to finish.  
**Consequence:** Brittle if animation duration changes. Unnecessary delay if `prefers-reduced-motion` disables the animation.  
**Instead:** For timing-critical sequences, use `animationend` event. (The thinking delay is intentional feature behavior, not a wait-for-animation scenario — `setTimeout` is correct there.)

### Anti-Pattern 5: `prefers-color-scheme` Detection in JS
**What:** `window.matchMedia('(prefers-color-scheme: dark)')` → toggle CSS class from JS.  
**Consequence:** Flash of wrong theme before JS runs. Requires event listener for live system theme changes.  
**Instead:** `@media (prefers-color-scheme: light)` in CSS. Browser applies the correct theme before any paint.

---

## Accessibility Checklist

| Feature | Concern | Mitigation |
|---------|---------|------------|
| Piece animations | Motion sensitivity | All `@keyframes` wrapped in `@media (prefers-reduced-motion: no-preference)` |
| Win line animation | Motion sensitivity | Same `prefers-reduced-motion` guard; game remains usable without the line |
| Thinking delay | Screen reader timing | `#status-message` has `aria-live="polite"` — "Computer's turn" message is announced during the delay |
| Sound effects | Deaf/hard-of-hearing users | Sounds are supplemental; all state is communicated visually. Mute button must be keyboard-accessible with `focus-visible` ring. |
| Mute button | Screen reader | `aria-label="Toggle sound"` + `aria-pressed` state update on toggle |
| Dark mode | Photosensitivity | System preference respected; user has already chosen their comfortable theme |

---

## Size Impact Estimate

All features are additive to the JS/CSS layer only. The Rust/WASM binary is untouched.

| Feature | Added LOC |
|---------|----------|
| Dark mode (CSS) | ~15 |
| Persistent scores (JS) | ~20 |
| Piece placement animation (CSS) | ~15 |
| Thinking delay (JS) | ~10 |
| Win line (CSS + JS) | ~80 |
| Sound effects (new module + call sites) | ~70 |
| **Total** | **~210 LOC** |

**New total frontend:** ~446 existing + ~210 new = ~656 LOC across 4 files (3 modified + 1 new module). Well within "vanilla JS + CSS" territory, no framework or build tool changes needed.

---

## Sources

- MDN: Web Audio API (`AudioContext`, `OscillatorNode`, `GainNode`, autoplay policy) — HIGH confidence, official docs, last modified 2025
- MDN: `Window.localStorage` — `SecurityError` handling, string storage — HIGH confidence, official docs, last modified 2025
- MDN: `@media prefers-color-scheme` — `light`/`dark` values, baseline widely available since January 2020 — HIGH confidence, official docs, last modified 2025
- MDN: Using CSS Animations — `@keyframes`, `animation-fill-mode: both`, `cubic-bezier()` — HIGH confidence, official docs, last modified 2025
- Direct code inspection: `src/main.js` (215 LOC), `src/style.css` (186 LOC), `index.html` (45 LOC), `src/wasm_api.rs` (95 LOC) — HIGH confidence, primary source
