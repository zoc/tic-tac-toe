# Phase 8: Animated Win Line - Context

**Gathered:** 2026-04-13
**Status:** Ready for planning

<domain>
## Phase Boundary

When the game ends in a win (human or computer), a line animates through the three winning cells — drawing itself from one end to the other. The line correctly overlays all 8 win orientations (3 rows, 3 columns, 2 diagonals), is removed when a new game starts, and is suppressed under `prefers-reduced-motion`.

**Important:** Codebase scouting confirmed that the ANIM-02 requirement is **fully pre-implemented** during Phase 3. Phase 8 is therefore **verification-only** — no new code is expected. The planner should produce a structured manual test checklist confirming each success criterion is satisfied.

</domain>

<decisions>
## Implementation Decisions

### Scope
- **D-01:** Phase 8 is verification-only. ANIM-02 is already satisfied by Phase 3 code. No new implementation needed.

### Pre-implemented evidence
- **D-02:** `index.html:41` — `<div class="win-line" id="win-line" hidden aria-hidden="true"></div>` — overlay element is a sibling of `.board` inside `.board-wrapper`, so it survives `boardEl.innerHTML = ''` during reset.
- **D-03:** `src/main.js:47–64` — `WIN_LINE_CLASSES` lookup table maps all 8 win patterns (`0,1,2` → `win-line--row0`, etc.) to CSS classes. `showWinLine(positions)` sorts positions, looks up the class, sets `winLineEl.className`, and un-hides the element. `clearWinLine()` hides it and resets `className`.
- **D-04:** `src/main.js:154` — `showWinLine(winPositions)` is called inside `handleGameOver()` when `winPositions.length > 0`.
- **D-05:** `src/main.js:266` — `clearWinLine()` is called inside `resetGame()`.
- **D-06:** `src/style.css:261–362` — `.win-line` positioned absolutely over `.board-wrapper`. Eight position classes cover all win orientations. Separate `@keyframes` for rows (`win-draw`, `scaleX`), columns (`win-draw-col`, rotate + `scaleX`), and diagonals (`win-draw-diag-lr`, `win-draw-diag-rl`). All animations scoped under `@media (prefers-reduced-motion: no-preference)`.
- **D-07:** `src/style.css:276–281` — `prefers-reduced-motion` guard is in place. Under `reduce`, the line appears instantly (no animation), satisfying ANIM-03.

### Verification approach
- **D-08:** Manual test checklist is the required output — no automated browser tests. Checklist must cover: human win (row, column, diagonal), computer win, draw (no line), new game clears line, `prefers-reduced-motion` suppression (line appears instantly), and small-viewport alignment check.

### Agent's Discretion
- Exact checklist format and ordering
- Which browser(s) to recommend for manual verification
- Whether to include devtools steps for simulating `prefers-reduced-motion`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and in project planning docs.

### Phase 8 requirements
- `.planning/REQUIREMENTS.md` §Animations — ANIM-02 defines the animated win line requirement; ANIM-03 defines reduced-motion suppression
- `.planning/ROADMAP.md` §Phase 8 — Success criteria (4 items), dependencies, goal statement

### Existing implementation to verify
- `src/style.css` lines 253–362 — `.board-wrapper` positioning context, `.win-line` base styles, `@keyframes win-draw` / `win-draw-col` / `win-draw-diag-lr` / `win-draw-diag-rl`, eight `.win-line--*` position classes, `prefers-reduced-motion` guard
- `src/main.js` lines 44–69 — `winLineEl` DOM reference, `WIN_LINE_CLASSES` lookup table, `showWinLine()`, `clearWinLine()`
- `src/main.js` line 154 — `showWinLine(winPositions)` called in `handleGameOver()`
- `src/main.js` line 266 — `clearWinLine()` called in `resetGame()`
- `index.html` line 41 — `#win-line` element in DOM

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.win-line` overlay — absolutely positioned inside `.board-wrapper`, already wired to JS
- `WIN_LINE_CLASSES` — complete 8-entry lookup table covering all win patterns
- `showWinLine()` / `clearWinLine()` — fully implemented show/hide helpers

### Established Patterns
- Verify-first pattern: Phases 4 and 7 were also verification-only after Phase 3 pre-implemented requirements
- `prefers-reduced-motion` scoping via `@media (prefers-reduced-motion: no-preference)` — same pattern as ANIM-01 (piece pop-in)
- `[hidden]` CSS rule (`display: none !important`) used to hide the win line between games
- Percentage-based positioning (`5%`/`95%`, `127.28%` diagonal width) ensures small-viewport alignment

### Integration Points
- `handleGameOver()` calls `showWinLine()` → draws the line on win
- `resetGame()` calls `clearWinLine()` → removes the line on new game
- Draw/game-over-with-no-winner: `winPositions.length === 0` → `showWinLine()` is never called → no line shown

</code_context>

<specifics>
## Specific Ideas

- The diagonal fix from Phase 4 is already reflected in the code: `diag-rl` uses `left: 95%` + `rotate(135deg)` (not `rotate(-45deg)` with `translateY`). This was a known correction logged in STATE.md decisions.
- Win line color is white (`#fff` at `0.9` opacity) — intentional contrast against both the winning cell background (`--accent` red) and the board surface.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 08-animated-win-line*
*Context gathered: 2026-04-13*
