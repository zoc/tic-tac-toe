---
phase: 05-css-piece-animations
plan: "01"
subsystem: ui

tags: [css, animation, prefers-reduced-motion, DOM, incremental-update]

requires:
  - phase: 04-css-foundation-persistence
    provides: "CSS theming, score persistence, win-line, audio"

provides:
  - "Incremental renderBoard() — only newly placed cells receive cell--x/cell--o class; existing pieces are never recreated"
  - "Pop-in animation (cell-pop) fires only for the piece just placed, not the entire board"
  - "ANIM-03 satisfied by construction — animation is inside @media (prefers-reduced-motion: no-preference)"
  - "resetGame() explicitly clears boardEl.innerHTML before renderBoard() to force full rebuild on reset"

affects: [06-thinking-delay, 07-sound-effects-mute, 08-animated-win-line]

tech-stack:
  added: []
  patterns:
    - "Incremental DOM update: check boardEl.children.length !== 9 to decide between full-build and patch path"
    - "CSS animation fires only on newly added classes — DOM nodes that are not recreated do not re-trigger animation"
    - "prefers-reduced-motion guard in CSS is sufficient; no JS feature detection needed"

key-files:
  created: []
  modified:
    - src/main.js

key-decisions:
  - "The CSS animation (cell-pop) and its reduced-motion guard were already correct from Phase 3 — no style.css changes needed"
  - "Root cause was structural: innerHTML='' rebuild on every render recreated all 9 cells, triggering animation on all of them"
  - "Reset path intentionally uses innerHTML='' wipe (boardEl.children.length !== 9 triggers full rebuild)"

patterns-established:
  - "Incremental DOM update pattern: build once, patch later — avoids animation re-trigger on untouched nodes"
  - "Reset detection via children.length guard rather than a separate boolean flag"

requirements-completed: [ANIM-01, ANIM-03]

duration: 15min
completed: 2026-04-13
---

# Phase 5 Plan 01: CSS Piece Animations Summary

**Incremental `renderBoard()` fix — only the newly placed piece pops in. Existing pieces are left untouched so their CSS animation never re-fires. ANIM-03 was already correct by construction (animation gated on `prefers-reduced-motion: no-preference`).**

## Performance

- **Duration:** ~15 min
- **Completed:** 2026-04-13
- **Tasks:** 2/2 (Task 1: automated fix; Task 2: human-verified)
- **Files modified:** 1 (`src/main.js`)

## Accomplishments

- Replaced full-wipe `renderBoard()` with an incremental version:
  - Builds all 9 cells from scratch only when `boardEl.children.length !== 9` (first render or after reset)
  - On subsequent calls, iterates existing cells and only adds `cell--taken`/`cell--x`/`cell--o` to cells whose state changed from empty → occupied
  - Existing X/O cells are never touched — CSS animation does not re-fire on unchanged DOM nodes
- Added `boardEl.innerHTML = ''` to `resetGame()` before `renderBoard()` call — ensures the children.length guard triggers a full rebuild after every reset
- Confirmed `@keyframes cell-pop` and its `@media (prefers-reduced-motion: no-preference)` guard were already correct — no `src/style.css` changes required

## Verification Results

### Automated (4/4 PASS)

| Check | Result |
|-------|--------|
| `cell-pop` keyframe exists in `src/style.css` | ✅ PASS |
| Animation is inside `@media (prefers-reduced-motion: no-preference)` block | ✅ PASS |
| `renderBoard()` uses `children.length` incremental guard | ✅ PASS |
| `boardEl.innerHTML = ''` appears inside the guard branch, not at top-level | ✅ PASS |

### Manual Browser (4/4 PASS, human-approved)

| Criterion | Result |
|-----------|--------|
| New X and O pieces scale in with spring/pop animation | ✅ Pass |
| Existing pieces stay static when a new piece is placed | ✅ Pass |
| `prefers-reduced-motion: reduce` emulation — pieces appear instantly, no animation | ✅ Pass |
| Win highlight, win-line, New Game, and score persistence — no regressions | ✅ Pass |

## Deviations from Plan

None — plan was executed as written. The recommended implementation from `05-01-PLAN.md` was applied verbatim.

## Known Stubs

None.

## Threat Flags

| Threat | Disposition | Verification |
|--------|-------------|--------------|
| T-05-01: XSS via incremental update | accept | Updates use `textContent` ('X'/'O') and `classList` — no `innerHTML` with user data. Surface unchanged from Phase 3. |
| T-05-02: Animation re-trigger | mitigate ✅ | Incremental DOM update confirmed — existing cells are never recreated. |
