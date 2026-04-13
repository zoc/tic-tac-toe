# Project Research Summary

**Project:** Tic-Tac-Toe WASM — v1.1 Polish & Feel
**Domain:** Browser casual game polish (animations, audio, persistence, theming)
**Researched:** 2026-04-13
**Confidence:** HIGH

## Executive Summary

v1.1 is a pure frontend polish milestone — zero changes to the Rust/WASM game engine. All six features (piece animations, animated win line, thinking delay, localStorage scores, sound effects, dark mode) are implemented exclusively in the JS/CSS/HTML layer using native browser platform APIs. No new npm or Cargo dependencies are required. The existing split architecture — Rust owns game logic, JS/CSS owns presentation — is the correct foundation and needs no restructuring for this milestone.

The recommended approach is to implement features in risk-ascending order: start with pure CSS changes (dark mode, piece animations), move to isolated JS data-layer changes (localStorage, thinking delay), then tackle the two medium-complexity features (win line SVG, Web Audio) that have the most integration surface. The thinking delay must be implemented before sound effects because audio timing depends on the async event-handler structure. The win line must be wrapped in a `board-wrapper` element — adding that HTML wrapper is the only structural change to `index.html` that has downstream dependencies.

The dominant risks are: (1) the `boardEl.innerHTML = ''` render pattern that fights CSS animations — must solve before writing any keyframes; (2) Web Audio autoplay policy silently blocking the first sound; and (3) the thinking-delay timer not being cancelled on game reset, causing ghost moves. All three are well-understood and have documented prevention patterns. The total code delta is modest: ~210 LOC across 3 modified files + 1 new `audio.js` module.

---

## Key Findings

### Recommended Stack

**No new dependencies.** All six v1.1 features are implementable with Baseline Widely Available browser platform APIs. The existing stack (Rust 1.94.1 + wasm-pack 0.14.0 + Vite 8.0.8 + vanilla JS/CSS) is unchanged. The only new file is `src/audio.js`, an ES module that the existing Vite build picks up without configuration changes.

**Core technologies — unchanged:**
- **Rust/WASM via wasm-pack** — game logic, AI, win detection — zero v1.1 changes
- **Vite 8 + vite-plugin-wasm** — dev server and bundler — zero config changes needed
- **Vanilla JS (ES modules)** — all 6 features live here; ~210 LOC delta
- **CSS with custom properties** — all color theming via `var(--*)` makes dark mode a single `@media` block

**New platform API surface (all zero-dependency, native browser):**
- `@keyframes` + CSS `animation` property — piece pop-in and win line draw animations
- `stroke-dashoffset` on inline SVG — win line animation (or CSS `scaleX` on a `<div>`)
- `window.localStorage` — score persistence and mute preference
- Web Audio API (`AudioContext`, `OscillatorNode`, `GainNode`) — synthesized game sounds
- `@media (prefers-color-scheme: light)` — light theme override
- `setTimeout` / `clearTimeout` — thinking delay

### Expected Features

**Must have (table stakes) — these make v1.0 feel "finished":**
- **Piece placement animation** — bare text pop-in feels jarring; CSS `@keyframes` scale animation with spring easing fixes this in ~20 CSS LOC
- **Persistent scores via localStorage** — resetting on F5 feels like a bug; users expect persistence universally
- **Computer thinking delay (300–800ms)** — instant AI response feels robotic; already flagged in v1.0 code as explicitly deferred
- **Dark mode (`prefers-color-scheme`)** — app already uses dark theme; not adapting to light OS preference is an oversight

**Should have (differentiators) — rare in competing implementations:**
- **Animated win line** — most TTT games just highlight cells; a line drawing through the winners is theatrically satisfying and visually rare
- **Sound effects + mute toggle** — neither Neave's nor Google's embedded TTT has sounds; Web Audio synthesized tones add zero bundle bytes

**Defer (anti-features for this milestone):**
- Confetti / particle effects — obscures board, adds 20–40KB library weight, overkill for 30-second games
- Volume slider — binary mute provides 80% value at 10% complexity
- Manual theme toggle — pure CSS `@media` handles use case without UI chrome or JS
- `localStorage` board state — 30-second games; mid-game restoration is not a felt pain point
- Audio files (.mp3/.ogg) — synthesized oscillators are instant, zero-byte, and sufficient for game UI sounds

### Architecture Approach

All changes are additive to the JS/CSS/HTML layer. The Rust/WASM binary (`src/wasm_api.rs`, `Cargo.toml`) is untouched. The one structural HTML change is wrapping `.board` in a `<div class="board-wrapper">` (required for the win line overlay to escape `overflow: hidden`). A new `src/audio.js` ES module encapsulates the `AudioContext` singleton, sound definitions, and mute state — keeping audio complexity out of the already 215-LOC `main.js`.

**Major components and their v1.1 changes:**
1. **`src/style.css`** *(modified)* — `@keyframes cell-pop`; `.win-line` + 8 position classes + `@keyframes win-draw`; `@media (prefers-color-scheme: light)` override block; `--hover-bg` variable; `@media (prefers-reduced-motion)` guards
2. **`src/main.js`** *(modified)* — `thinkDelay()` + `async handleCellClick`; `loadScore()`/`saveScore()`; `showWinLine()`/`clearWinLine()` helpers; `import { sounds }` + 5 call sites; mute button wiring
3. **`index.html`** *(modified)* — `<div class="board-wrapper">` wrapper, `<div id="win-line" hidden>` sibling of board, `<button id="mute-btn">` near title
4. **`src/audio.js`** *(new)* — `AudioContext` singleton (lazy init), `playTone()` primitive with envelope shaping, `sounds.*` named exports, `toggleMute()`/`isMuted()`, mute localStorage persistence

### Critical Pitfalls

1. **`innerHTML = ''` kills animation state** — `renderBoard()` wipes all 9 cells on every call. Naive approach animates all pieces on every move. **Fix:** Either switch to incremental DOM updates (update existing cells in-place, only animate the newly-added cell) or track the new-cell index before wiping and re-apply the animation class only to that cell using `void el.offsetWidth` force-reflow to restart animation.

2. **`overflow: hidden` on `.board` clips the win-line overlay** — Any overlay positioned absolutely *inside* `.board` is silently clipped. **Fix:** Add `<div class="board-wrapper" style="position:relative">` and position the win-line element as a *sibling* of `.board` inside the wrapper — not a child.

3. **Web Audio autoplay policy blocks the first sound** — `AudioContext` created before a user gesture starts in `suspended` state. First cell click produces no sound, no error. **Fix:** Create `AudioContext` lazily inside the first click handler (`ensureAudio()` pattern); always call `audioCtx.resume()` if `state === 'suspended'` before playing.

4. **Thinking timer not cancelled on game reset causes ghost moves** — If the user clicks "New Game" during the 300–800ms delay, the pending `setTimeout` fires `game.computer_move()` on the freshly reset board. **Fix:** Store `thinkingTimer` ID; call `clearTimeout(thinkingTimer)` in `resetGame()` before resetting WASM state; guard inside callback with `if (game.get_status() !== 'playing') return`.

5. **Hardcoded hex values survive theme change** — `.cell:hover` uses `#1e2a4a` (hardcoded dark hex) and the JS error handler uses inline `cssText` with hardcoded dark colors. `@media (prefers-color-scheme: light)` only overrides `var(--*)` usages. **Fix:** Extract `#1e2a4a` to `var(--hover-bg)` before implementing dark mode; replace inline error styles with a CSS class.

---

## Implications for Roadmap

Based on combined research, the natural phase structure follows risk-ascending order while respecting implementation dependencies. All features are additive — none require revisiting v1.0 game logic.

### Phase 1: CSS Foundation & Persistence
**Rationale:** Lowest-risk changes first. Dark mode and localStorage are fully isolated with no interdependencies. Dark mode validates that the CSS custom property system covers all color usages (audit for hardcoded hex values before writing the `@media` block). localStorage establishes the `try/catch` persistence pattern that sound's mute preference will reuse.
**Delivers:** Scores survive page refresh; light-mode OS users see adapted colors; `--hover-bg` CSS variable extracted; all hardcoded colors replaced with `var(--*)` references
**Addresses:** Dark mode (table stakes), persistent scores (table stakes)
**Avoids:** Pitfall 7 (FOUC — pure CSS only, no JS theme switching), Pitfall 16 (audit hardcoded hex before writing `@media` block), Pitfall 6 (all localStorage calls in try/catch), Pitfall 10 (namespaced key: `ttt-v1-scores`)

### Phase 2: CSS Piece Animations
**Rationale:** Pure CSS changes to `style.css`. Must resolve the `boardEl.innerHTML = ''` animation strategy before writing any keyframes (Pitfall 1) — this decision gates all subsequent animation work. Implementing before the thinking delay means animations can be tested immediately; once the delay is added (Phase 3) the natural pace makes them feel even better.
**Delivers:** Piece pop-in animation on X/O placement; `prefers-reduced-motion` accessibility guard established for all subsequent animations; animation-fill-mode pattern validated
**Addresses:** Smooth piece placement animation (table stakes)
**Avoids:** Pitfall 1 (choose incremental DOM update OR void-reflow strategy before writing CSS), Pitfall 8 (scope `transition: background` to un-taken cells via `:not(.cell--taken)` to prevent conflict with keyframe animation)

### Phase 3: Thinking Delay (async refactor)
**Rationale:** Converts `handleCellClick` to `async` — this is the core structural change that sound timing (Phase 4) and win-line timing (Phase 5) depend on. Medium risk because it touches the main event-handler flow. Must be done before sound effects so `sounds.computerMove()` fires at the right moment (after the delay, when the piece is actually placed — not before the delay).
**Delivers:** Computer feels deliberate rather than robotic; async `handleCellClick` structure ready for downstream integration; `thinkingTimer` cancellation in `resetGame()` prevents ghost moves
**Addresses:** Computer thinking delay (table stakes)
**Avoids:** Pitfall 5 (store timer ID; cancel in `resetGame()`; guard callback with `game.get_status()` check)

### Phase 4: Sound Effects & Mute Toggle
**Rationale:** Most integration surface of any single feature (new module + HTML button + 5 call sites + localStorage mute persistence). Depends on the async flow from Phase 3 being correct. A new `src/audio.js` module isolates all audio complexity from `main.js`.
**Delivers:** Synthesized audio feedback for moves/win/loss/draw; mute toggle with persisted preference; zero new bundle bytes (oscillator tones, no audio files)
**Addresses:** Sound effects with mute toggle (differentiator)
**Avoids:** Pitfall 3 (lazy `AudioContext` init via `ensureAudio()` on first click), Pitfall 4 (`onended` cleanup on every oscillator node), Pitfall 15 (envelope shaping — attack/decay ramps prevent click/pop artifacts), Pitfall 11 (persist mute state to `ttt-v1-muted` in localStorage)

### Phase 5: Animated Win Line
**Rationale:** Most complex feature — SVG overlay or CSS `scaleX` div, coordinate math, CSS animation, HTML structural change (`board-wrapper`). Saved for last because it requires all other visual polish to be in place (pieces animate, timing feels right) so win-line timing and animation delays can be calibrated against real game flow. The `board-wrapper` HTML change is a prerequisite for this phase.
**Delivers:** Line draws itself through winning cells; theatrical, visually rare win moment; restart button timing respects animation duration
**Addresses:** Animated win line (differentiator)
**Avoids:** Pitfall 2 (board-wrapper sibling pattern — must be first step of this phase), Pitfall 9 (use `getBoundingClientRect()` for cell coordinates rather than CSS percentage math to account for grid gaps), Pitfall 13 (chain animation completion → delay → computer move as sequential async flow), Pitfall 14 (disable restart button for `WIN_ANIMATION_DURATION_MS` before re-enabling)

### Phase Ordering Rationale

- **CSS-only phases first** — no JS risk, easy to test in isolation, validates the custom property system
- **localStorage before sounds** — both persist state to `localStorage`; establish the `try/catch` pattern once and reuse it for the mute preference key
- **Async refactor (delay) before sounds** — audio timing correctness (when `sounds.computerMove()` fires) depends on the async event handler being in place
- **Win line last** — requires the `board-wrapper` HTML structural change; best added when all other visual polish is stable and timing calibration is possible against real game feel
- **All phases are purely additive** — no phase modifies or reverts a prior phase's work; game remains playable after each phase completes

### Research Flags

**Phases with well-documented patterns (skip `/gsd-research-phase`):**
- **Phase 1 (Dark mode + localStorage):** Pure CSS media query and well-understood localStorage API — exhaustively documented on MDN, zero unknowns
- **Phase 3 (Thinking delay):** `setTimeout` wrapper pattern — ARCHITECTURE.md has exact implementation code ready to use

**Phases that may benefit from brief pre-implementation review:**
- **Phase 2 (CSS animations):** The `innerHTML = ''` animation strategy choice (incremental DOM vs void-reflow) should be decided by inspecting current `renderBoard()` carefully before writing any CSS. PITFALLS.md documents both strategies with code. Pick one before writing keyframes.
- **Phase 4 (Sound effects):** Web Audio autoplay policy behavior varies slightly by browser. Test the lazy-init `ensureAudio()` pattern in Chrome, Firefox, and Safari before assuming uniform behavior. PITFALLS.md patterns are correct but cross-browser testing is advisable on first play-through.
- **Phase 5 (Win line):** No additional research needed — but allocate time for visual calibration across viewport sizes. The SVG coordinate math is the only genuine implementation variable.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All features use Baseline Widely Available APIs verified against MDN; codebase inspection confirmed no new deps needed; all 6 browser APIs work in Chrome, Firefox, Safari, Edge |
| Features | HIGH | v1.0 codebase directly inspected (215 LOC main.js, 186 LOC style.css); feature boundaries clear; competitor analysis confirms differentiator rationale for sound and win-line |
| Architecture | HIGH | Direct inspection of all 4 files; integration points mapped precisely with line-number references; data-flow diagrams traced before/after for each feature |
| Pitfalls | HIGH | Each pitfall derives from MDN-documented API behavior or direct codebase analysis of specific code patterns; all 16 pitfalls have concrete prevention code |

**Overall confidence:** HIGH

### Gaps to Address

- **Animation DOM strategy:** Research documents two valid approaches (incremental DOM update vs void-reflow) without prescribing one. Inspect `renderBoard()` during Phase 2 to determine which fits better — incremental update is cleaner if the function can be refactored without touching win highlighting logic; void-reflow is simpler if full-board rebuild must be preserved.
- **Win line coordinate precision:** ARCHITECTURE.md uses CSS percentage positioning (`.win-line--row0 { top: calc(1/6 * 100%) }`) while PITFALLS.md recommends `getBoundingClientRect()` for gap-accuracy. Test CSS percentages first (simpler); fall back to DOM measurement if gap misalignment is visible.
- **Cross-browser audio testing:** Web Audio autoplay policy is consistent in principle but implementation varies. Validate the lazy-init pattern in all 4 target browsers during Phase 4 before considering it done.

---

## Sources

### Primary (HIGH confidence)
- MDN Web Docs — Web Audio API, autoplay policy, `OscillatorNode`, `GainNode` (updated Sep–Oct 2025)
- MDN Web Docs — `Window.localStorage`, `SecurityError` exceptions (updated Nov 2025)
- MDN Web Docs — `@media prefers-color-scheme`, `prefers-reduced-motion` (updated Dec 2025)
- MDN Web Docs — CSS `@keyframes`, `animation`, `animation-fill-mode` (updated Dec 2025)
- Codebase direct inspection: `src/main.js` (215 LOC), `src/style.css` (186 LOC), `index.html` (45 LOC), `src/wasm_api.rs` (95 LOC)

### Secondary (MEDIUM confidence)
- Competitor analysis (v1.0 research, 2026-04-12) — playtictactoe.org (Neave), Google TTT, PaperGames.io — confirmed neither Neave nor Google TTT has sound effects (differentiator rationale for Phase 4)

---
*Research completed: 2026-04-13*
*Ready for roadmap: yes*
