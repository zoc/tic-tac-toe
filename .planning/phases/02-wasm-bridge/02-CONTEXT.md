# Phase 2: WASM Bridge - Context

**Gathered:** 2026-04-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Compile the existing Rust game engine to a WebAssembly module via wasm-pack, producing a `pkg/` directory with `.wasm` binary, JS glue, and TypeScript definitions. Define a clean JS interop surface using `#[wasm_bindgen]` so the game engine can be imported and called from a browser application. Verify the module loads and all game operations work through a minimal HTML test page.

This phase does NOT build any game UI — it bridges the Rust engine (Phase 1) to JavaScript so Phase 3 can render a playable game.

</domain>

<decisions>
## Implementation Decisions

### JS API Shape
- **D-01:** Export the `Game` struct directly with `#[wasm_bindgen]` as an opaque handle. JS interacts through methods on the struct (e.g., `game.make_move(pos)`, `game.current_player()`). No wrapper structs or serde serialization layer.
- **D-02:** Agent's discretion on how to handle the `GameStatus` enum boundary (it has a `Won { winner, positions }` variant that can't cross WASM directly). Options include getter methods, string status codes, or a hybrid — choose what maps cleanest to wasm-bindgen.

### Board State Transfer
- **D-03:** Board state is returned to JS as a numeric array (9 elements): `0 = empty`, `1 = X`, `2 = O`. Single call, minimal overhead, natural for rendering a 9-cell grid.

### Verification Approach
- **D-04:** Verification uses a minimal HTML test page that loads the WASM module (`await init()`), calls each exported function, and displays results. Manual browser check — no automated headless browser testing in this phase.

### Agent's Discretion
- How to handle `GameStatus` enum data crossing the WASM boundary (D-02)
- Whether to create a thin `#[wasm_bindgen]`-annotated wrapper or annotate the existing `Game` struct directly
- Crate dependency versions and feature flags (wasm-bindgen, getrandom wasm_js, rand upgrade path)
- Internal module organization for WASM-specific code
- HTML test page structure and level of detail

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project requirements
- `.planning/REQUIREMENTS.md` — ENG-01 defines the requirement for this phase (Rust compiled to WASM via wasm-pack)
- `.planning/PROJECT.md` — Constraints (Rust + wasm-pack only), context (game logic in Rust/WASM, frontend in HTML/CSS/JS)
- `.planning/ROADMAP.md` §Phase 2 — Success criteria (3 items), dependencies (Phase 1), goal statement

### Phase 1 decisions (affects this phase)
- `.planning/phases/01-rust-game-engine/01-CONTEXT.md` — D-03 through D-07 define the data structures this phase must export: flat `[Option<Player>; 9]` board, `GameStatus::Won` with positions, `Game` struct with methods

### Technology stack
- `AGENTS.md` §Technology Stack — wasm-bindgen 0.2.118, wasm-pack 0.14.0, getrandom 0.4.2 with `wasm_js` feature, rand 0.10.1, serde-wasm-bindgen 0.6.5 (if needed), console_error_panic_hook 0.1.7

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/board.rs` — `Game` struct with `new()`, `make_move()`, `cells()`, `current_player()`, `status()` methods. This is the primary target for `#[wasm_bindgen]` annotation.
- `src/ai.rs` — `get_computer_move(&Game) -> Option<usize>` function. Needs to be exposed through the WASM API.
- `Cargo.toml` — Already has `crate-type = ["cdylib", "rlib"]` configured. Ready for WASM compilation.

### Established Patterns
- `Game` struct uses methods, not free functions — consistent with the opaque struct export approach
- `Player` enum: `X`, `O` — simple, can map to u8 (1, 2) for the numeric array
- `GameStatus` enum has variants with data (`Won { winner, positions }`) — needs boundary handling
- Uses `rand = "0.8"` with `rand::thread_rng()` — must upgrade to `rand 0.10` + `getrandom 0.4` with `wasm_js` feature for browser WASM compatibility

### Integration Points
- `src/lib.rs` — Currently just `pub mod board; pub mod ai;`. WASM exports will be added here or in the existing modules.
- `src/main.rs` — CLI runner from Phase 1. Not needed for WASM but should remain functional (`rlib` crate type preserves this).
- `pkg/` — wasm-pack output directory. Phase 3 will import from here.

</code_context>

<specifics>
## Specific Ideas

- The `rand` crate upgrade from 0.8 to 0.10 is a known concern (STATE.md blocker note: "Verify `getrandom` crate needs `wasm_js` feature for browser entropy"). This must be handled as part of the WASM compilation setup.
- The numeric board array (D-03) maps directly to Phase 3's rendering needs — JS can iterate `[0,1,2,0,0,0,0,1,2]` and render X/O/empty without parsing.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-wasm-bridge*
*Context gathered: 2026-04-12*
