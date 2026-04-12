# Phase 1: Rust Game Engine - Context

**Gathered:** 2026-04-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete game logic in pure Rust — board state management, move validation, win/draw detection, and a beatable AI opponent. All verified by native unit tests (`cargo test`). No WASM compilation, no browser, no UI. This phase produces a Rust library crate that Phase 2 will compile to WebAssembly.

</domain>

<decisions>
## Implementation Decisions

### AI Mistake Behavior
- **D-01:** AI uses imperfect minimax with a flat probability of making mistakes — ~25% of the time, it picks a random valid move instead of the optimal one
- **D-02:** Mistake rate is a constant (~25%), not scaling or adaptive. Tuning may happen after playtesting in later phases but the initial implementation is flat probability

### Board & Game State Design
- **D-03:** Board is a flat array of 9 cells (`[Option<Player>; 9]`), indexed 0-8 in row-major order
- **D-04:** Game state carries: board cells, current player (X or O), and game status (InProgress / Won(player) / Draw)
- **D-05:** No move history tracked — undo/replay is out of scope per REQUIREMENTS.md

### API Surface for Phase 2
- **D-06:** Engine must return the winning cell positions (e.g., `[0, 1, 2]` for top row win) when a win is detected — needed for UI highlighting in Phase 3
- **D-07:** Agent's discretion on whether to use a Game struct with methods or free functions — choose what maps cleanly to wasm_bindgen in Phase 2

### Test Strategy
- **D-08:** Thorough test coverage matching success criteria: all 8 winning lines (3 rows, 3 columns, 2 diagonals), draw detection, occupied-cell rejection, turn alternation
- **D-09:** 100-game automated batch test verifying AI produces valid moves and human wins occur (mistake injection works)
- **D-10:** AI must never make an illegal move — test that AI never targets occupied cells or moves after game over

### Agent's Discretion
- API shape (struct with methods vs free functions) — choose what's cleanest for wasm_bindgen export
- Internal minimax implementation details
- Exact Rust project structure (module layout, file organization)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and in project planning docs:

### Project requirements
- `.planning/REQUIREMENTS.md` — ENG-03, ENG-04, AI-01, AI-02 define the requirements for this phase
- `.planning/PROJECT.md` — Core value, constraints (Rust + wasm-pack only), key decisions (imperfect minimax, human always X)
- `.planning/ROADMAP.md` §Phase 1 — Success criteria (4 items), dependencies, goal statement

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield phase, no existing code in the repository

### Established Patterns
- None yet — this phase establishes the Rust crate structure that Phase 2 will build on

### Integration Points
- Phase 2 will compile this crate to WASM via wasm-pack and expose functions through wasm_bindgen
- The flat [9] board representation and winning-cell return values are designed to serialize cleanly across the WASM boundary

</code_context>

<specifics>
## Specific Ideas

- AI mistake rate (~25%) was flagged in STATE.md as needing playtesting — implement as a constant that's easy to adjust later
- The winning cell positions (D-06) should be ready for Phase 3's win highlighting feature

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-rust-game-engine*
*Context gathered: 2026-04-12*
