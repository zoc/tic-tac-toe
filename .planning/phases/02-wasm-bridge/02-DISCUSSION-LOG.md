# Phase 2: WASM Bridge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-12
**Phase:** 02-wasm-bridge
**Areas discussed:** JS API shape, Board state transfer, Verification approach

---

## JS API shape

| Option | Description | Selected |
|--------|-------------|----------|
| Opaque struct (Recommended) | Export Game struct directly with #[wasm_bindgen]. JS calls game.make_move(pos), game.current_player(), etc. Feels natural, minimal glue code. | ✓ |
| Scalar wrapper struct | Wrap Game in a new WasmGame struct that exposes only scalar-returning methods (u8, String, bool). No complex types cross the boundary. More boilerplate but zero serialization. | |
| JsValue via serde | Use serde-wasm-bindgen to return full game state as a JS object. JS gets {cells: [...], status: '...', currentPlayer: '...'} in one call. More data per call but fewer round-trips. | |

**User's choice:** Opaque struct (Recommended)
**Notes:** User chose the simplest path — export Game directly, let wasm-bindgen handle pointer management.

### Follow-up: GameStatus enum handling

| Option | Description | Selected |
|--------|-------------|----------|
| Getter methods (Recommended) | Add methods like is_won(), is_draw(), winner(), winning_positions() on the Game struct. JS never sees the Rust enum directly. | |
| String status + getters | Return status as a string ('in_progress', 'won', 'draw') plus separate getters for winner/positions. | |
| You decide | Let the agent decide the best approach for crossing the enum boundary. | ✓ |

**User's choice:** You decide
**Notes:** Agent has discretion on GameStatus boundary handling.

---

## Board state transfer

| Option | Description | Selected |
|--------|-------------|----------|
| Numeric array (Recommended) | Return a Vec<u8> or Uint8Array where 0=empty, 1=X, 2=O. JS gets a typed array in one call, fast to iterate for rendering. | ✓ |
| Cell-by-cell getter | Expose a get_cell(index) -> u8 method. JS calls it 9 times to build the board. Simple but more WASM boundary crossings. | |
| String encoding | Return the full board as a JS string like 'XO__X_OX_' (9 chars). Easy to parse, single call, human-readable for debugging. | |
| You decide | Let the agent decide the most practical encoding for a 9-cell board. | |

**User's choice:** Numeric array (Recommended)
**Notes:** 0=empty, 1=X, 2=O as a flat 9-element array. Maps directly to Phase 3 rendering needs.

---

## Verification approach

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal HTML test page (Recommended) | A bare HTML file that loads the WASM, calls each exported function, and logs results to the page. Manual open-in-browser check. | ✓ |
| wasm-bindgen-test (headless) | Use wasm-bindgen-test with headless Chrome/Firefox. Automated but requires browser tooling setup. | |
| Both approaches | wasm-bindgen-test for automated CI, plus a manual HTML page for visual smoke testing. | |
| You decide | Let the agent decide the verification strategy. | |

**User's choice:** Minimal HTML test page (Recommended)
**Notes:** Matches success criteria exactly. Manual browser check proves the bridge works end-to-end.

---

## Agent's Discretion

- GameStatus enum boundary handling (getter methods, string codes, or hybrid)
- Whether to annotate existing Game struct or create a thin wrapper
- Dependency versions and feature flags
- HTML test page structure

## Deferred Ideas

None — discussion stayed within phase scope.
