# Domain Pitfalls — v1.1 Polish & Feel

**Domain:** Browser game polish (CSS animations, Web Audio, localStorage, dark mode)
**Codebase:** Rust/WASM + Vite 8 + vanilla JS/CSS — existing game, adding features
**Researched:** 2026-04-13
**Confidence:** HIGH — MDN official docs + deep codebase analysis

---

## Context: Why This Document Exists

The v1.0 PITFALLS.md covers Rust/WASM boundary issues (async init, panic hooks, crate-type).
This document covers the six **browser-side** features being added in v1.1:

1. Smooth CSS animations for piece placement and board transitions
2. Animated win line through winning cells
3. Computer "thinking" delay (300–800ms)
4. Persistent scores via localStorage
5. Sound effects with mute toggle (Web Audio API)
6. Dark mode via `prefers-color-scheme`

Each pitfall is mapped to the specific feature and includes the fix for **this** codebase.

---

## Critical Pitfalls

Mistakes that require rewriting significant code or cause obvious broken behavior.

---

### Pitfall 1: Animation Classes Applied to innerHTML-Wiped Elements

**Feature:** CSS animations for piece placement

**What goes wrong:**
The current `renderBoard()` calls `boardEl.innerHTML = ''` then recreates all 9 cells from scratch. When a new piece is placed, the intent is to animate only the *new* cell. But if you apply an animation class before calling `renderBoard()`, the element is destroyed. If you apply it after, you're animating a brand-new DOM node that hasn't been painted yet — which can cause the animation to skip the first frame on some browsers.

More subtly: if `renderBoard()` is called after human move AND after computer move, every cell (including previously placed ones) gets a freshly created DOM node. An `animation: pop-in 0.2s` on `.cell--taken` would fire on ALL placed pieces every time any move is made — so move 5 animates cells 1, 2, 3, 4, and 5 simultaneously.

**Why it happens:**
The innerHTML-clear approach was the right call for v1.0 (simplicity, stateless rendering). But CSS animations are stateful — they fire when a class is added to an element that didn't have it. Recreating all elements from scratch defeats this.

**Consequences:**
- All pieces animate on every render — jarring instead of polished
- The last-placed piece is indistinguishable from re-rendered old pieces
- Potential flicker during animation on Safari (paint timing issue)

**Prevention:**
Two valid strategies:

*Strategy A — Incremental DOM update (recommended):*
Track which cells were already rendered. On `renderBoard()`, update existing cells instead of wiping and recreating. Only add `.cell--animate-in` on the newly placed cell:
```js
// Instead of innerHTML = '', update existing cells in-place
const cells = boardEl.querySelectorAll('.cell');
// Update only changed cells; add animation class only to newly filled ones
```

*Strategy B — Animation class with immediate removal:*
After `renderBoard()` clears and recreates, compare the WASM board state before/after to find the new position, then add the animation class only to that one cell. Force-restart the animation by voiding the offset:
```js
cell.classList.remove('cell--animate-in');
void cell.offsetWidth; // forces reflow, resets animation
cell.classList.add('cell--animate-in');
```

Strategy A is cleaner for win-line animation too. Strategy B works but requires careful bookkeeping of "what was the board before this move."

**Detection:** Play the game and open DevTools Animation panel — look for multiple animations firing simultaneously when they should be one.

**Phase:** Animations phase — must decide which strategy before writing any animation CSS.

---

### Pitfall 2: Win Line Animation Requires Absolute Positioning Over the Board

**Feature:** Animated win line through three winning cells

**What goes wrong:**
Developers try to animate a win line using `border` on individual cells, `background` gradients, or SVG `line` inside each cell. None of these produce a clean straight line through the center of three cells that feels like a real strikethrough.

The natural implementation is an overlay `<div>` or `<svg>` element positioned absolutely over the board, with a `clip-path` or `stroke-dasharray` animation drawing the line from one end to the other. But the board uses `overflow: hidden` (to clip the cell corners), which clips absolutely positioned children too — the overlay is hidden.

**Why it happens:**
`overflow: hidden` on `.board` clips all descendant elements, including absolutely positioned ones. This is well-documented but easy to forget.

**Consequences:**
- Win line overlay is completely invisible — silent failure
- Developers waste time debugging animation-related CSS instead of the real culprit

**Prevention:**
Two options:

*Option A — Remove `overflow: hidden` from `.board`, handle corner rounding differently:*
Replace `border-radius + overflow: hidden` with `border-radius` only on individual cells (or on a wrapper). The current design uses `background: var(--accent)` on the board with `overflow: hidden` to show gap color. With careful restructuring this can still work.

*Option B — Position the overlay on a parent wrapper element (recommended):*
Add a `position: relative` wrapper around the board, then absolutely position the win line overlay on that wrapper. The board keeps `overflow: hidden` for its own purposes, but the overlay sits outside it (as a sibling):
```html
<div class="board-wrapper"> <!-- position: relative -->
  <div class="board" id="board">...</div>
  <svg class="win-line" id="win-line">...</svg>
</div>
```

The SVG/div overlay is a sibling of `.board`, not a child — it isn't clipped.

**Detection:** Add a brightly colored `position: absolute` element inside `.board` and verify it gets clipped. That confirms the issue before spending time on the win line implementation.

**Phase:** Win line animation phase — resolve layout strategy before writing animation code.

---

### Pitfall 3: Web Audio Context Blocked by Autoplay Policy

**Feature:** Sound effects for moves and game outcomes

**What goes wrong:**
Creating an `AudioContext` before any user gesture results in the context being created in `suspended` state. All sounds queued before the first interaction are silently dropped. On some browsers, the context never resumes without explicit `audioCtx.resume()` in response to a user gesture.

Developers create `AudioContext` at module load time (alongside `new WasmGame()` in `main()`), play sounds normally in response to clicks, and then wonder why sounds don't play on the first click.

**Why it happens:**
All major browsers implement the Autoplay Policy (formalized ~2018, enforced strictly since 2020). The policy requires user gesture → audio start. Creating `AudioContext` before the first interaction is technically allowed, but the context starts `suspended`. MDN: "if you create the context outside of a user gesture, its state will be set to suspended."

The first cell click IS a user gesture and DOES resume a suspended context — but only if you call `audioCtx.resume()` explicitly. If you don't check `audioCtx.state === 'suspended'`, the sound code runs but produces no output.

**Consequences:**
- First move in every game session is silent (the most important sound UX moment)
- No error thrown, no warning in console — silent failure
- Hard to reproduce in development if you click the page as part of dev workflow

**Prevention:**
```js
// Create AudioContext lazily or check state before playing
async function playSound(audioCtx, buffer) {
  if (audioCtx.state === 'suspended') {
    await audioCtx.resume();
  }
  const source = audioCtx.createBufferSource();
  source.buffer = buffer;
  source.connect(audioCtx.destination);
  source.start(0);
}
```

For this game, the cleanest approach is to initialize `AudioContext` on the first user click (lazy init), not in `main()`. The click handlers already exist (`handleCellClick`, `resetGame`) — hook in there.

**Mute toggle interaction:** If the user mutes then unmutes, don't destroy and recreate `AudioContext`. Use a `GainNode` with `gain.value = 0` for mute — the context stays running, the mute is instant. Destroying/recreating `AudioContext` is expensive and can cause audio glitches.

**Detection:** Load the game in a fresh incognito window. Click a cell immediately. If no sound plays, autoplay policy is blocking.

**Phase:** Sound effects phase — must handle lazy init and suspended-state resume from day one.

---

### Pitfall 4: AudioContext Accumulates Nodes (Memory Leak Pattern)

**Feature:** Sound effects for moves and game outcomes

**What goes wrong:**
`AudioBufferSourceNode` is a single-use object — it cannot be restarted after `source.start()`. Each sound play creates a new node. If the node isn't explicitly disconnected, old nodes accumulate in the audio graph, consuming memory. In a long game session, this can cause audible artifacts or excessive memory use.

**Why it happens:**
MDN: "An `AudioBufferSourceNode` can only be played once; after each call to `start()`, you have to create a new node if you want to play the same sound again." Developers cache the source node and call `start()` again — throws a DOM exception. Or they create new nodes per play but forget to disconnect finished nodes.

**Prevention:**
```js
function playBuffer(audioCtx, gainNode, buffer) {
  const source = audioCtx.createBufferSource();
  source.buffer = buffer;
  source.connect(gainNode);  // always connect through the gain node (for mute)
  source.onended = () => source.disconnect();  // clean up when done
  source.start(0);
}
```

The `onended` callback disconnects the node after playback, preventing accumulation. This is the canonical pattern.

**Detection:** Play the game many times rapidly. Check memory usage in DevTools performance profiler — audio node count should remain stable, not grow with each game.

**Phase:** Sound effects phase — include `onended` cleanup from the first implementation.

---

### Pitfall 5: Computer Thinking Delay Conflicts with isProcessing Guard

**Feature:** Computer "thinking" delay (300–800ms)

**What goes wrong:**
The current code has a synchronous computer move:
```js
isProcessing = true;
const compPos = game.computer_move();
renderBoard();
isProcessing = false;
```

When introducing a delay via `setTimeout`, the `isProcessing` flag is set `true` before the timeout fires and set `false` only inside the timeout callback. This means during the delay, clicks are correctly blocked. BUT: if the "New Game" button is shown during a delay (it's only shown after game over, so this is rare), resetting the game could clear `isProcessing` while the timeout callback is still pending — when the timeout fires, `game.computer_move()` runs on a reset game state.

More critically: if the delay is implemented carelessly with `await new Promise(resolve => setTimeout(resolve, delay))` inside `handleCellClick`, the async function may not properly guard against re-entry. The existing `if (isProcessing) return` guard at the top is still evaluated synchronously, but multiple async call chains could be in flight.

**Why it happens:**
Converting synchronous game logic to async (with delays) without updating the concurrency guard pattern. The guard was designed for synchronous execution — it works perfectly now, but `async/await` with delays creates new re-entrancy windows.

**Consequences:**
- Computer calls `computer_move()` on a freshly reset board
- Potential double-move, invalid state, or JS exception
- Hard to reproduce (timing-dependent race condition)

**Prevention:**

1. Store the pending timeout ID so it can be cancelled:
```js
let thinkingTimer = null;

// In resetGame():
if (thinkingTimer) {
  clearTimeout(thinkingTimer);
  thinkingTimer = null;
  isProcessing = false;
}
```

2. In the timeout callback, check game state is still valid before calling WASM:
```js
thinkingTimer = setTimeout(() => {
  thinkingTimer = null;
  // Guard: was the game reset during our delay?
  if (game.get_status() !== 'playing') {
    isProcessing = false;
    boardEl.classList.remove('board--disabled');
    return;
  }
  const compPos = game.computer_move();
  // ...
}, delay);
```

**Detection:** Click a cell, then immediately click "New Game" during the thinking delay. Verify the computer doesn't move on the new game's board.

**Phase:** Computer delay phase — update `resetGame()` to cancel pending timers.

---

### Pitfall 6: localStorage Throws in Private/Incognito Mode and file:// Protocol

**Feature:** Persistent scores via localStorage

**What goes wrong:**
Calling `localStorage.setItem()` or `localStorage.getItem()` throws a `SecurityError` exception when:
- User is in private/incognito browsing mode AND their browser is configured to block storage
- Page is served from `file://` (not relevant for this Vite project, but can affect test.html)
- User has explicitly disabled cookies/storage in browser settings

This is a silent failure in development (developer's browser settings allow storage) but crashes for some users in production. The exception propagates up and can break score persistence without any visible error.

MDN: "Thrown in one of the following cases: The origin is not a valid scheme/host/port tuple... The request violates a policy decision."

**Why it happens:**
Developers always test in their own browser with default settings. Incognito-with-storage-blocked is common for privacy-focused users. The exception is thrown at the `localStorage` access itself — not at read time later.

**Consequences:**
- Scores silently fail to persist (if caught); or game crashes (if uncaught)
- Inconsistent behavior: works for developer, broken for some users
- Stack traces pointing at localStorage code confuse developers who never saw the error

**Prevention:**
Wrap all `localStorage` operations in try/catch:
```js
function saveScores(score) {
  try {
    localStorage.setItem('ttt-scores', JSON.stringify(score));
  } catch {
    // Storage blocked (private mode, file://, security policy) — fall back to in-memory
  }
}

function loadScores() {
  try {
    const saved = localStorage.getItem('ttt-scores');
    return saved ? JSON.parse(saved) : null;
  } catch {
    return null;
  }
}
```

Also validate parsed data — `JSON.parse()` can return unexpected shapes if localStorage was manually modified. Always check that the loaded object has the expected shape before using it.

**Detection:** Open in incognito mode, verify scores still display (as 0) even if they can't be saved.

**Phase:** localStorage phase — wrap on first write, never raw-access localStorage.

---

### Pitfall 7: Dark Mode Causes FOUC (Flash of Unstyled Content / Flash of Wrong Color)

**Feature:** Dark mode support respecting `prefers-color-scheme`

**What goes wrong:**
The existing design is already dark navy (`--bg: #1a1a2e`). When adding light mode support via `@media (prefers-color-scheme: light)`, the CSS variables are overridden. If those overrides are defined in an external stylesheet loaded asynchronously, or if JavaScript is reading/writing `data-theme` attributes to the `<html>` element, users with light mode may briefly see the dark theme before the correct colors are applied. This flash is jarring.

Additionally, if you add a manual theme toggle (user can override their OS preference) and store the preference in localStorage, there's a window between page load and JS execution where the OS preference is applied before the manual preference loads.

**Why it happens:**
CSS `@media (prefers-color-scheme: light)` is evaluated by the browser as the stylesheet parses. No flash if done purely in CSS. Flash happens when:
- Theme is applied via a `data-theme` attribute set by JavaScript
- JavaScript reads localStorage to determine the initial theme
- The `<link>` stylesheet loads after initial paint

**Prevention:**
For this project, the cleanest implementation is **CSS-only for the initial theme** — no JS needed for `prefers-color-scheme`. Add overrides directly in `style.css`:
```css
@media (prefers-color-scheme: light) {
  :root {
    --bg: #f0f0f5;
    --surface: #ffffff;
    --text: #1a1a2e;
    /* etc. */
  }
}
```

Since the CSS is inlined via `<link rel="stylesheet" href="/src/style.css" />` which loads synchronously in the `<head>`, there is no flash — the correct colors are set before first paint.

If adding a manual toggle, use the pattern:
```html
<!-- In <head>, before stylesheet: -->
<script>
  // Runs before CSS — sets theme class synchronously to prevent flash
  const saved = localStorage.getItem('ttt-theme');
  if (saved) document.documentElement.dataset.theme = saved;
</script>
```

This inline script runs before CSS is applied and eliminates the flash.

**Detection:** In macOS: System Preferences → Appearance → Light. Open the game. A flash from dark→light means FOUC. It should appear in the correct colors immediately.

**Phase:** Dark mode phase — implement as CSS-only first; add JS toggle only if required, with the inline script pattern.

---

## Moderate Pitfalls

Issues that cause bugs or poor UX but don't require major restructuring to fix.

---

### Pitfall 8: CSS `transition` on `.cell` Conflicts with Animation on New Piece

**Feature:** CSS animations for piece placement

**What goes wrong:**
The existing CSS has `transition: background 0.1s ease` on `.cell`. When a cell gets the `.cell--taken` class added (with a new background color and text), the background color smoothly transitions. If you also add a `@keyframes` animation (e.g., `pop-in`), the two effects can conflict or compound awkwardly:
- `transition` fades the background while `animation` scales the element
- On Safari, transitions and animations on the same property can produce unexpected results
- The `transition` was designed for hover feedback, not for piece placement

**Prevention:**
Scope the transition narrowly. Either:
1. Remove `transition: background` when `.cell--taken` is added (the animation handles the entrance)
2. Change the transition to only apply to hover-state cells: `.cell:not(.cell--taken) { transition: background 0.1s ease; }`

The second option is clean and already logically correct — taken cells shouldn't respond to hover anyway.

**Phase:** Animations phase — audit existing transitions before adding animations.

---

### Pitfall 9: Win Line SVG Coordinate System Doesn't Match CSS Grid

**Feature:** Animated win line through winning cells

**What goes wrong:**
The board is sized with `var(--board-size): min(90vw, 90vh, 440px)` and laid out with CSS Grid. An SVG overlay sized at `100% x 100%` of the board wrapper should have its coordinate system match the board. But if the board wrapper and SVG don't share the same bounding box (due to padding, border, or `overflow: visible` spilling), the calculated cell center coordinates for the SVG line don't align with the visual cell centers.

Additionally: the board uses `gap: var(--cell-gap)` (4px). Win line calculations that assume uniform cell width (`boardSize / 3`) are off by `gap / 3` per cell — small but visible.

**Prevention:**
Calculate cell centers from actual DOM measurements, not CSS math:
```js
function getCellCenter(cellEl, boardEl) {
  const cellRect = cellEl.getBoundingClientRect();
  const boardRect = boardEl.getBoundingClientRect();
  return {
    x: cellRect.left + cellRect.width / 2 - boardRect.left,
    y: cellRect.top + cellRect.height / 2 - boardRect.top,
  };
}
```

This is immune to gap size, padding, or board-size changes. Always measure from the DOM rather than computing from CSS values.

**Phase:** Win line animation phase — use `getBoundingClientRect()` for coordinate calculation.

---

### Pitfall 10: localStorage Key Collision

**Feature:** Persistent scores via localStorage

**What goes wrong:**
Using a generic key like `"scores"` or `"wins"` risks collision with other applications on the same origin (e.g., another Vite app running on `localhost:5173`). During development, localhost is a shared origin for all local projects.

**Prevention:**
Use a namespaced key: `'ttt-v1-scores'` (app prefix + version). The version suffix allows clean migration if the stored data shape changes in future:
```js
const STORAGE_KEY = 'ttt-v1-scores';
```

If the score schema changes (e.g., adding a new field), bump to `ttt-v2-scores` and let v1 data expire naturally.

**Phase:** localStorage phase — namespace keys from first implementation.

---

### Pitfall 11: Mute State Not Persisted Across Sessions

**Feature:** Sound effects with mute toggle

**What goes wrong:**
Users toggle mute, refresh the page, and find themselves back with sound on. For a game with sound effects (which some users find annoying), this is worse than no mute toggle at all — the user must mute again every session.

**Prevention:**
Persist the mute preference in localStorage alongside scores:
```js
const MUTE_KEY = 'ttt-v1-muted';
let isMuted = localStorage.getItem(MUTE_KEY) === 'true';

function toggleMute() {
  isMuted = !isMuted;
  gainNode.gain.value = isMuted ? 0 : 1;
  try { localStorage.setItem(MUTE_KEY, String(isMuted)); } catch {}
  updateMuteButton();
}
```

The mute button UI should also reflect the persisted state on page load.

**Phase:** Sound effects phase AND localStorage phase should be designed together so both are persisted.

---

### Pitfall 12: `prefers-color-scheme` Doesn't React to OS Changes Without `matchMedia` Listener

**Feature:** Dark mode support

**What goes wrong:**
CSS `@media (prefers-color-scheme)` automatically reacts to OS-level theme changes — the browser re-evaluates media queries and applies new styles without any JavaScript. However, if a manual theme toggle is added via a `data-theme` attribute on `<html>`, that attribute overrides the media query. When the user changes their OS theme after the page loads, the CSS media query fires but the JS-applied `data-theme` attribute takes precedence in the cascade, and nothing visually changes.

**Prevention:**
If adding a manual toggle, also add a `matchMedia` listener so the UI (toggle button state) stays in sync:
```js
const mq = window.matchMedia('(prefers-color-scheme: dark)');
mq.addEventListener('change', () => {
  if (!localStorage.getItem('ttt-theme')) {
    // Only sync if user hasn't set a manual preference
    updateThemeDisplay();
  }
});
```

For this project, if the scope is just `prefers-color-scheme` without a manual toggle, no JavaScript is needed — CSS handles it automatically and always stays in sync.

**Phase:** Dark mode phase — keep JS out of pure-CSS dark mode; only add JS if manual toggle is required.

---

## Minor Pitfalls

Small issues that cause subtle bugs or polish problems.

---

### Pitfall 13: Thinking Delay Status Text Race with Animation Timing

**Feature:** Computer thinking delay + CSS animations

**What goes wrong:**
With a 300–800ms thinking delay, the "Computer's turn" status message appears, then the status changes to "Your turn" after the computer moves. If piece placement has a 200ms animation, the visual timeline becomes:
- T+0ms: Human places piece, animation starts
- T+200ms: Animation ends
- T+300–800ms: Computer places piece, animation starts
- T+500–1000ms: Computer animation ends, "Your turn" restored

This is fine. But if the animation and delay use different `setTimeout` chains that are not coordinated, the "Your turn" message can appear while the computer's animation is still running — a minor but noticeable inconsistency.

**Prevention:**
Structure the flow as a sequential async chain:
```
humanMove → renderWithAnimation → await animationDone → 
await thinkingDelay → computerMove → renderWithAnimation → 
await animationDone → updateStatus('Your turn')
```

`animationDone` can be awaited with a `transitionend` or `animationend` listener:
```js
function waitForAnimation(el) {
  return new Promise(resolve => {
    el.addEventListener('animationend', resolve, { once: true });
  });
}
```

**Phase:** Animations + computer delay phase — coordinate animation timing with delay timing.

---

### Pitfall 14: Win-State Animation Fires Then Board Immediately Clears

**Feature:** CSS animations + win line

**What goes wrong:**
`handleGameOver()` currently calls `renderBoard(winPositions)` which highlights winning cells, then shows the restart button. The user clicks "New Game" immediately, and `resetGame()` calls `renderBoard()` without winning positions — all animations/highlights disappear instantly.

This is already partially addressed by `restartBtn.hidden = false` — the button appearing gives users time to see the result. But if the win line animation has a duration of 500ms and the user is a fast clicker, they can dismiss it before the animation completes.

**Prevention:**
Delay enabling the restart button by the animation duration:
```js
// In handleGameOver():
restartBtn.hidden = false;
restartBtn.disabled = true; // prevent immediate click
setTimeout(() => { restartBtn.disabled = false; }, WIN_ANIMATION_DURATION_MS);
```

Or use `animationend` on the win line element to enable the button.

**Phase:** Animations phase — consider animation duration when determining when to re-enable restart.

---

### Pitfall 15: Synthesized Web Audio Sounds Feel Cheap Without Envelope Shaping

**Feature:** Sound effects for moves and game outcomes

**What goes wrong:**
Using `OscillatorNode` directly (easiest code-path for "no audio files") produces sounds that click/pop at start and end if the gain isn't shaped. An oscillator started at full gain and stopped abruptly produces an audible click (a DC discontinuity in the waveform). This is the most common Web Audio beginner mistake.

**Prevention:**
Always use a gain envelope for oscillator-based sounds:
```js
function playTone(audioCtx, masterGain, frequency, duration) {
  const osc = audioCtx.createOscillator();
  const env = audioCtx.createGain();
  
  osc.frequency.value = frequency;
  osc.type = 'sine';
  env.gain.setValueAtTime(0, audioCtx.currentTime);                  // start silent
  env.gain.linearRampToValueAtTime(0.3, audioCtx.currentTime + 0.01); // attack
  env.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + duration); // decay
  
  osc.connect(env);
  env.connect(masterGain); // masterGain controls global mute
  osc.start(audioCtx.currentTime);
  osc.stop(audioCtx.currentTime + duration);
  osc.onended = () => { osc.disconnect(); env.disconnect(); };
}
```

The attack ramp prevents the click at start; the exponential decay prevents the click at stop. Note: `exponentialRampToValueAtTime` cannot ramp to 0 — ramp to a very small value like 0.001 instead.

**Phase:** Sound effects phase — use envelope shaping from the first sound implementation.

---

### Pitfall 16: CSS Variable Overrides in Dark Mode Miss Hardcoded Color Values

**Feature:** Dark mode support

**What goes wrong:**
The existing CSS uses `var(--bg)`, `var(--surface)`, `var(--accent)`, `var(--text)`, `var(--text-dim)` consistently in most places. But there are hardcoded hex values:
- `.cell:hover:not(.cell--taken)` uses `#1e2a4a` (a hardcoded hover color)
- Error display in `main.js` uses `cssText` with hardcoded `color:#e94560; background:#1a1a2e`

A `@media (prefers-color-scheme: light)` block that overrides CSS variables will correctly update all `var()` usages — but the hardcoded values stay dark regardless of the theme.

**Prevention:**
Audit for hardcoded colors before adding dark mode:
- Replace `#1e2a4a` hover background with a new variable `--surface-hover`
- The JS error display (`main().catch(err => {...})`) uses inline `cssText` — add a theme-appropriate fallback or ensure the error state also reads CSS variables

```css
:root {
  --surface-hover: #1e2a4a;  /* new variable */
}
@media (prefers-color-scheme: light) {
  :root {
    --surface-hover: #e8e8f0;  /* light mode hover */
  }
}
```

**Detection:** Switch to light mode and visually inspect every element for remaining dark colors.

**Phase:** Dark mode phase — audit for hardcoded hex values first.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| CSS piece animations | innerHTML wipe kills animation state (Pitfall 1) | Switch to incremental DOM updates or track last-placed cell index |
| Win line animation | `overflow: hidden` clips overlay (Pitfall 2) | Add a wrapper element; place SVG overlay as sibling of board |
| Computer delay | Timer not cancelled on reset causes ghost moves (Pitfall 5) | Store timer ID; cancel in `resetGame()` |
| localStorage | Throws in incognito/private mode (Pitfall 6) | Always wrap in try/catch; degrade gracefully to in-memory |
| Web Audio | Context suspended before user gesture (Pitfall 3) | Lazy init or explicit `resume()` on first interaction |
| Web Audio | AudioBufferSourceNode leaks (Pitfall 4) | `onended = () => source.disconnect()` on every node |
| Dark mode | FOUC if theme applied via JS before CSS loads (Pitfall 7) | Use CSS-only `@media` for initial theme; inline script only if manual toggle added |
| Dark mode | Hardcoded hex values survive theme change (Pitfall 16) | Audit and replace all hardcoded colors with CSS variables |
| Animations + delay | Status text out of sync with animation timing (Pitfall 13) | Chain animation completion → delay → computer move as sequential async flow |
| Win line + restart | Fast-click dismisses win before animation completes (Pitfall 14) | Disable restart button for `WIN_ANIMATION_DURATION_MS` after game over |

---

## Existing Codebase Considerations

Specific things about *this* codebase that affect v1.1 implementation:

| Code Pattern | Impact on v1.1 | Recommendation |
|-------------|----------------|----------------|
| `boardEl.innerHTML = ''` in every `renderBoard()` | Animation classes added before render are destroyed | Must choose incremental update OR track "new cell" index before wipe |
| `isProcessing` flag is synchronous guard | Works for delay, but `resetGame()` must cancel pending timer | Store `thinkingTimer` ID; cancel in `resetGame()` |
| Score is in-memory `const score = {}` | Clean upgrade path to localStorage — just add load/save wrappers | Don't restructure score object; wrap access functions around it |
| All CSS in a single `style.css` via `var()` | Dark mode via `@media (prefers-color-scheme: light)` works perfectly | Just override `:root` vars in the media query |
| `overflow: hidden` on `.board` | Win line overlay must be a sibling, not a child | Add `.board-wrapper` with `position: relative` |
| No existing `<audio>` elements or Audio API usage | Clean state — no legacy patterns to work around | Create AudioContext lazily on first user gesture |

---

## Sources

- [MDN: Web Audio API — Autoplay Policy](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Best_practices#autoplay_policy) — HIGH confidence (official, updated 2025-09-18)
- [MDN: Web Audio API — Using the Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_Web_Audio_API) — HIGH confidence (official, updated 2025-09-18)
- [MDN: Window.localStorage — Exceptions (SecurityError)](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage#exceptions) — HIGH confidence (official, updated 2025-11-30)
- [MDN: prefers-color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme) — HIGH confidence (official, updated 2025-12-05)
- Codebase analysis: `src/main.js`, `src/style.css`, `index.html` — HIGH confidence (direct inspection)

---
*Pitfalls research for: v1.1 Polish & Feel milestone — CSS animations, Web Audio, localStorage, dark mode*
*Researched: 2026-04-13*
