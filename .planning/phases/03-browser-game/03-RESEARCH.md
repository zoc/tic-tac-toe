# Phase 3: Browser Game — Frontend Research

**Captured:** 2026-04-13
**Phase:** 3 — Browser Game
**Confidence:** HIGH — based on existing stack research, Phase 2 artifacts, and CONTEXT.md decisions

## Summary

Phase 3 builds a vanilla JS/CSS/HTML frontend on top of the WASM bridge delivered in Phase 2. No new external libraries are needed beyond what is already established in the stack.

## Vite Setup for WASM

**Pattern:** `vite-plugin-wasm` + `vite-plugin-top-level-await` enable seamless `import init, { WasmGame } from './pkg/tic_tac_toe.js'` in an ES module.

```js
// vite.config.js
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  server: { fs: { allow: ['.'] } }  // needed for pkg/ outside src/
});
```

The `server.fs.allow` is required because Vite's dev server restricts file serving to the project root by default — `pkg/` is a sibling directory and must be explicitly allowed.

## Project Layout

```
tic-tac-toe/
  index.html          # Entry point (references src/main.js)
  src/
    main.js           # WASM init, event wiring, game loop
    style.css         # All styles (dark theme, grid, responsive)
  pkg/                # WASM package (from Phase 2, already built)
  vite.config.js
  package.json
```

Keep `src/main.js` as one cohesive module — ~150 LOC for a tic-tac-toe game is perfectly manageable without splitting.

## Responsive Grid Approach

**CSS Grid with `min()` + viewport units — no media queries needed for the board itself:**

```css
.board {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  width: min(90vw, 90vh, 420px);  /* fills viewport, caps at 420px on desktop */
  aspect-ratio: 1;
}
.cell {
  aspect-ratio: 1;        /* keeps cells square */
  font-size: min(15vw, 15vh, 64px);  /* X/O scales with board size */
}
```

No JavaScript resize handling needed. Grid adapts automatically.

## State Machine Pattern (JS-side)

The JS layer maintains a tiny state machine reflecting WASM game state:

```
IDLE → PLAYER_TURN → COMPUTER_TURN → GAME_OVER → IDLE (reset)
```

- **IDLE**: After WASM init, before first click
- **PLAYER_TURN**: Awaiting human click on empty cell
- **COMPUTER_TURN**: WASM computer_move() called synchronously (no delay — per CONTEXT.md C)
- **GAME_OVER**: Win/draw shown, restart button active

The `isComputerTurn` flag prevents human clicks during the instant computer move calculation.

## Score Tracking (In-Memory)

Simple counter object in module scope — persists across games for the session (per CONTEXT.md):

```js
const score = { wins: 0, losses: 0, draws: 0 };
```

No localStorage. Resets on page refresh.

## Win Highlighting

`get_winning_positions()` returns `Uint32Array[3]` with cell indices. Add CSS class to those cells:

```css
.cell.winning {
  background: #e94560;   /* red accent from CONTEXT.md */
  color: #fff;
}
```

For draws: no highlighting, only status message.

## Pitfalls to Avoid

| Risk | Mitigation |
|------|------------|
| WASM init race | Always `await init()` before attaching click handlers |
| Double-click during computer move | `isProcessing` flag — disable board clicks synchronously before calling `computer_move()` |
| Stale board render | Re-render from `game.get_board()` after EVERY state change — never predict state client-side |
| Mobile tap: `touchstart` vs `click` | Use `click` event — it works on modern iOS/Android without issues |
| Cell reuse across games | `game.reset()` clears WASM state; JS `renderBoard()` re-renders from scratch |

## Security Notes

- Game state lives entirely in WASM/JS. No server, no network requests, no user data.
- XSS surface is minimal: no user input is rendered as HTML. Cell content is text nodes only.
- Content-Type for WASM served by Vite automatically (handled by vite-plugin-wasm).

## Sources

- Phase 2 SUMMARY: WASM API surface fully documented, import pattern confirmed
- research/STACK.md: Vite 8 + vite-plugin-wasm 3.6.0 + vite-plugin-top-level-await 1.6.0
- research/PITFALLS.md: Async init pattern, click handling, win highlighting timing
- CONTEXT.md: All design decisions locked (colors, grid, instant computer move, score in-memory)
