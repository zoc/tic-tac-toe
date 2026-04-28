# Phase 14: Difficulty UI & Persistence - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a difficulty `<select>` dropdown to the game UI: wired to `game.set_difficulty(u8)`, persisting the selection via `ttt-difficulty` localStorage key, defaulting to Medium (level 1) on first visit, resetting the game on any change, and disabled while the computer is thinking.

</domain>

<decisions>
## Implementation Decisions

### Dropdown Placement
- **D-01:** Difficulty selector lives inside the existing `.title-row` flex container, to the right of the mute button — no new wrapper element needed; the existing `display:flex; align-items:center; gap:12px` layout handles it.
- **D-02:** A visible `Difficulty:` text label appears to the left of the `<select>` element — makes the control self-explanatory for first-time visitors, consistent with the scoreboard label convention.

### Dropdown Styling
- **D-03:** Minimal dark theme for the `<select>` — `var(--surface)` background, light text color, thin 1px border that matches the theme; native OS `<select>` rendering is preserved (no custom dropdown arrow, no `appearance: none` override). Low CSS complexity, reliable cross-browser.
- **D-04:** Disabled state uses the CSS `:disabled` pseudo-class only — browser-native opacity dimming and `cursor: not-allowed`; no custom class needed. Matches the `cursor: not-allowed` already applied to `.board--disabled .cell`.

### Game-over & Empty-board Behavior
- **D-05:** `resetGame()` is called unconditionally on any difficulty `change` event — regardless of current game state (empty board, mid-game, or game-over). This satisfies UI-04 ("resets immediately") in all states: restart button hides, board clears, status resets to "Your turn".
- **D-06:** No conditional check for board state before calling `resetGame()` — always reset. On an empty board, `reset()` is cheap and produces no visible change. Simpler code, consistent behavior.

### isProcessing Integration
- **D-07:** The `difficultyEl.disabled` property mirrors `isProcessing`: set to `true` alongside `boardEl.classList.add('board--disabled')` in `handleCellClick()`, and set back to `false` when `isProcessing` is cleared (post-thinking delay and in `resetGame()`). This satisfies UI-05 with no new state variable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Scope
- `.planning/REQUIREMENTS.md` — UI-01 through UI-05 (the 5 requirements this phase implements), plus Out of Scope table
- `.planning/ROADMAP.md` — Phase 14 success criteria (5 numbered assertions defining correct behavior)
- `.planning/PROJECT.md` — Key Decisions table (localStorage pattern rationale, ttt-difficulty key, single shared score tally)

### Prior Phase Decisions (WASM API)
- `.planning/phases/13-rust-ai-parameterization-wasm-api/13-CONTEXT.md` — D-01 through D-07: `set_difficulty(u8)` is a pure setter (no reset); `reset()` does NOT touch difficulty field; `WasmGame::new()` defaults to Medium (level=1); u8 0–3 maps to Easy/Medium/Hard/Unbeatable

### Existing Source Files (extend, not replace)
- `src/main.js` — Current game loop: `isProcessing` flag, `thinkingTimer`, `resetGame()`, `loadScore()`/`saveScore()` localStorage pattern (replicate for difficulty), DOM refs pattern (`const boardEl = document.getElementById(...)`)
- `src/style.css` — CSS custom properties (`--surface`, `--accent`, `--text`, `--text-dim`), `.title-row` layout, `.mute-btn` styling (closest analog to the new control)
- `index.html` — HTML structure to extend: `.title-row` is where the label + select are added

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `loadScore()` / `saveScore()` in `src/main.js` (lines 16–31): exact template for `loadDifficulty()` / `saveDifficulty()` — same try/catch localStorage pattern, same safe-default fallback. Replicate verbatim with `DIFFICULTY_KEY = 'ttt-difficulty'` and default `1`.
- `resetGame()` in `src/main.js` (lines 255–270): already handles full reset — cancel thinking timer, clear `isProcessing`, remove `board--disabled`, hide restart button, clear win line, rebuild board. Call directly from the difficulty `change` handler with no modifications.
- `.title-row` in `index.html` / `src/style.css`: existing `display:flex; align-items:center; gap:12px` — add `<span class="difficulty-label">Difficulty:</span>` + `<select class="difficulty-select">` as last children; no CSS layout changes needed.
- `isProcessing` flag in `src/main.js`: already tracks thinking state — sync `difficultyEl.disabled = isProcessing` at all points where `isProcessing` changes.

### Established Patterns
- **localStorage with try/catch**: All persistence uses try/catch for SecurityError (private/incognito) — difficulty must follow the same pattern.
- **DOM refs queried once at startup**: All element refs (`boardEl`, `muteBtn`, etc.) are `const` at module top-level — add `difficultyEl` in the same block.
- **`game.set_difficulty(u8)` call site**: Must be called before `resetGame()` in the change handler so the WASM engine picks up the new level before the reset takes effect. (Though difficulty survives `reset()` per Phase 13 D-02, explicit ordering avoids confusion.)
- **Scalar WASM boundary**: `set_difficulty` takes u8 0–3; parse the `<select>` value with `parseInt(difficultyEl.value, 10)` — same defensive pattern used elsewhere.

### Integration Points
- `handleCellClick()` (line 211): sets `isProcessing = true` and adds `board--disabled` — add `difficultyEl.disabled = true` immediately after.
- Post-thinking-delay (line 241): `isProcessing = false` — add `difficultyEl.disabled = false` immediately after.
- `resetGame()` (line 262): clears `isProcessing` — add `difficultyEl.disabled = false` at same point.
- `main()` startup (line 273): load persisted difficulty, call `game.set_difficulty(level)`, set `difficultyEl.value = String(level)`.

</code_context>

<specifics>
## Specific Ideas

No "I want it like X" references — decisions above fully specify the approach.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 14-Difficulty UI & Persistence*
*Context gathered: 2026-04-28*
