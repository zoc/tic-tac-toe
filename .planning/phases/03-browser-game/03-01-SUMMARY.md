---
phase: 03-browser-game
plan: "01"
subsystem: frontend
tags: [vite, wasm, game-loop, css-grid, responsive, dark-theme]
requirements-completed: [ENG-02, UI-01, UI-02, UI-03, UI-04, UI-05, SCORE-01]

dependency-graph:
  requires:
    - "02-01: WASM bridge (pkg/tic_tac_toe.js, pkg/tic_tac_toe_bg.wasm)"
  provides:
    - "Runnable browser game with Vite dev server and production build"
    - "Complete frontend: HTML structure, dark theme CSS, WASM-wired JS game loop"
  affects:
    - "index.html, src/main.js, src/style.css"

tech-stack:
  added:
    - "Vite 8.0.8 — dev server and production bundler"
    - "vite-plugin-wasm 3.6.0 — WASM ESM import support"
  patterns:
    - "Event delegation — single click listener on board div for all 9 cells"
    - "WASM as single source of truth — renderBoard() always calls game.get_board()"
    - "CSS custom properties — dark navy/red theme via --bg, --accent, --surface"
    - "min() viewport sizing — --board-size: min(90vw, 90vh, 440px)"

key-files:
  created:
    - path: package.json
      role: "Vite dev dependencies and npm scripts (dev/build/preview)"
    - path: vite.config.js
      role: "Vite + vite-plugin-wasm config, server.fs.allow for pkg/ access, build.target=esnext"
    - path: index.html
      role: "HTML entry point: board container, scoreboard, status message, restart button"
    - path: src/main.js
      role: "WASM init, game loop, event wiring, score tracking (~160 LOC)"
    - path: src/style.css
      role: "Dark navy/red theme, responsive CSS Grid board, win highlight, disabled states"
  modified:
    - path: .gitignore
      role: "Added node_modules/ and dist/ to ignore list"

decisions:
  - id: D-A-impl
    description: "Decision A (dark navy/red) implemented via CSS custom properties --bg=#1a1a2e, --accent=#e94560"
  - id: D-B-impl
    description: "Decision B (responsive fill) via --board-size: min(90vw, 90vh, 440px); overflow:hidden on html/body"
  - id: D-C-impl
    description: "Decision C (no delay) — computer_move() called synchronously, isProcessing guard prevents re-entrant clicks"
  - id: D-D-impl
    description: "Decision D (win highlight) via .cell--winning class: background=accent, color=#fff !important"
  - id: D-no-tla
    description: "Removed vite-plugin-top-level-await (incompatible with Vite 8); used build.target=esnext instead"

metrics:
  duration: "~20 minutes"
  completed: "2026-04-13"
  tasks-completed: 5
  tasks-total: 5
  files-created: 5
  files-modified: 3
---

# Phase 03 Plan 01: Browser Game Frontend Summary

**One-liner:** Vite 8 + vite-plugin-wasm frontend wiring Phase 2 WASM to dark navy/red responsive CSS Grid game UI.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Scaffold Vite project | 932bec4 | package.json, vite.config.js, package-lock.json |
| 2 | Write index.html | 3d711bb | index.html |
| 3 | Write src/style.css | 84ddfb7 | src/style.css |
| 4 | Write src/main.js | a99166b | src/main.js |
| 5 | Human verify checkpoint | ✓ Approved | — |
| fix | vite-plugin-top-level-await → build.target=esnext | bae18e5 | vite.config.js, package.json |
| fix | Prevent cell size shift when X/O appears | a0554cf | src/style.css |
| fix | Force equal row heights with grid-template-rows | c34a0d3 | src/style.css |

## What Was Built

A complete browser-based tic-tac-toe game frontend:

- **`package.json` + `vite.config.js`** — Vite 8 project with `vite-plugin-wasm` for WASM ESM imports. `server.fs.allow: ['.']` permits serving `pkg/` from the project root. `build.target: 'esnext'` enables top-level await without a plugin.

- **`index.html`** — Semantic HTML skeleton with `id="board"` (grid container), `id="status-message"` (aria-live turn indicator), `id="restart-btn"` (hidden until game ends), and score elements (`score-wins`, `score-draws`, `score-losses`).

- **`src/style.css`** — Dark navy (`#1a1a2e`) + red (`#e94560`) theme. Responsive board via `--board-size: min(90vw, 90vh, 440px)`. Win highlight via `.cell--winning`. No-scroll enforced by `overflow: hidden` on `html, body`.

- **`src/main.js`** — Full game loop: imports `WasmGame` from `../pkg/tic_tac_toe.js`, awaits `init()`, renders board from WASM state, handles human clicks, triggers synchronous computer move, detects game end, updates score, shows restart button.

## Architecture

```
index.html
  └── src/main.js (type=module)
        ├── import init, WasmGame from ../pkg/tic_tac_toe.js
        ├── renderBoard() ← game.get_board() [WASM truth]
        ├── handleCellClick() → game.make_move() → game.computer_move()
        ├── handleGameOver() → game.get_winning_positions() → score update
        └── resetGame() → game.reset()
  └── src/style.css
        ├── CSS custom properties: --bg, --accent, --surface
        ├── .board { --board-size: min(90vw, 90vh, 440px) }
        └── .cell--winning { background: var(--accent) !important }
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] vite-plugin-top-level-await incompatible with Vite 8**

- **Found during:** Task 1 verification (npm run build)
- **Issue:** `vite-plugin-top-level-await@1.6.0` requires standalone `rollup` module which is not separately installed in Vite 8 (Vite 8 bundles rollup internally). Error: `Cannot find module 'rollup'`.
- **Fix:** Removed `vite-plugin-top-level-await` from both `package.json` and `vite.config.js`. Added `build: { target: 'esnext' }` to `vite.config.js`. This is the documented alternative from STACK.md: *"Not needed if `build.target = 'esnext'`"*. Modern browsers support top-level await natively.
- **Files modified:** `vite.config.js`, `package.json`, `package-lock.json`
- **Commit:** `bae18e5`

**2. [Rule 2 - Missing] .gitignore missing node_modules/ and dist/**

- **Found during:** Post-task untracked file check
- **Issue:** `.gitignore` only covered `/target/` and `/pkg/`, leaving `node_modules/` and `dist/` untracked.
- **Fix:** Added `/node_modules/` and `/dist/` to `.gitignore`.
- **Files modified:** `.gitignore`
- **Commit:** `bfb0f3c`

**3. [Rule 1 - Bug] Cell size shifts when X/O text appears**

- **Found during:** Task 5 human verification
- **Issue:** Cells resized when X or O text was rendered — font-size scaled with cell dimensions causing layout shift on move.
- **Fix:** Added `line-height: 1` and explicit `width`/`height: 100%` to `.cell` to prevent text from affecting cell dimensions.
- **Files modified:** `src/style.css`
- **Commit:** `a0554cf`

**4. [Rule 1 - Bug] Unequal row heights in board grid**

- **Found during:** Task 5 human verification (follow-on to fix 3)
- **Issue:** CSS Grid rows were not enforced to equal height, causing the board to appear non-square in some viewports.
- **Fix:** Added `grid-template-rows: repeat(3, 1fr)` to `.board` to mirror the existing `grid-template-columns`.
- **Files modified:** `src/style.css`
- **Commit:** `c34a0d3`

## Checkpoint Status

**Task 5 (Human Verify):** ✓ APPROVED — all criteria passed.

Human verified on 2026-04-13. All 9 success criteria confirmed:
- Dark navy board with red grid lines renders correctly
- X appears immediately on click; computer responds with O
- Win: winning cells highlighted red, "You win! 🎉" displayed
- "New Game" button resets board; score persists in session
- Computer win: O cells highlighted, "Computer wins!" displayed
- Draw: "It's a draw!" displayed with score increment
- Responsive: board fills viewport on mobile without scrolling
- Zero console errors during gameplay

## Known Stubs

None — all game state is wired to WASM. Score tracking is in-memory (by design, per CONTEXT.md — localStorage deferred to v2).

## Threat Flags

No new threat surface introduced beyond what's documented in the plan's `<threat_model>`:
- T-03-04 (DoS via rapid clicks): mitigated by `isProcessing` flag ✓
- T-03-05 (XSS via cell content): mitigated by `textContent` not `innerHTML` ✓

## Self-Check: PASSED

Files verified:
- ✓ FOUND: package.json
- ✓ FOUND: vite.config.js
- ✓ FOUND: index.html
- ✓ FOUND: src/main.js
- ✓ FOUND: src/style.css

Commits verified:
- ✓ FOUND: 932bec4 (chore: scaffold Vite project)
- ✓ FOUND: 3d711bb (feat: add index.html)
- ✓ FOUND: 84ddfb7 (feat: add src/style.css)
- ✓ FOUND: a99166b (feat: add src/main.js)
- ✓ FOUND: bae18e5 (fix: vite-plugin-top-level-await)
- ✓ FOUND: bfb0f3c (chore: .gitignore)
- ✓ FOUND: a0554cf (fix: cell size shift)
- ✓ FOUND: c34a0d3 (fix: equal row heights)

Build verified:
- ✓ `npm run build` succeeds — dist/index.html + dist/assets/*.js + dist/assets/*.wasm generated

Human verification: ✓ APPROVED 2026-04-13
