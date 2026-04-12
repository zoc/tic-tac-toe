# Project Research Summary

**Project:** Tic-Tac-Toe (Rust/WASM)
**Domain:** Rust-to-WebAssembly browser game
**Researched:** 2026-04-12
**Confidence:** HIGH

## Executive Summary

This is a single-player tic-tac-toe browser game where game logic is written in Rust, compiled to WebAssembly, and rendered with vanilla HTML/CSS/JS. The established way to build this — validated by the official Rust WASM Book and the broader Rust/WASM ecosystem — is a **split architecture**: Rust owns all game state and AI logic as a pure state machine, JavaScript owns all rendering and DOM interaction, and wasm-bindgen bridges the two via a thin command/query API returning scalar values. The toolchain is mature: Rust stable + wasm-pack + Vite handles the full build pipeline with minimal configuration.

The recommended approach is to build bottom-up following the dependency chain: board logic first (pure Rust, no WASM needed), then AI (depends on board), then the WASM export surface (composes board + AI), then the browser frontend (depends on compiled WASM), and finally visual polish. This ordering maximizes testability — the first two phases can be fully unit-tested with `cargo test` before ever opening a browser. The feature set is well-scoped: 10 table-stakes features for v1, 6 polish features for v1.x, and a firm boundary against scope creep (no multiplayer, no difficulty selector, no move history).

The key risks are all Phase 1 concerns and all easily preventable: forgetting async WASM initialization (causes cryptic `TypeError`), missing the `console_error_panic_hook` setup (makes Rust panics undebuggable), incorrect `crate-type` config (breaks native testing), and the `getrandom` crate needing the `wasm_js` feature for browser entropy. Every critical pitfall has a one-line fix if addressed during scaffolding. The project has no server component, no external APIs, and no complex integrations — the hardest engineering problem is tuning the minimax AI's mistake rate to feel fun.

## Key Findings

### Recommended Stack

The stack is a standard Rust/WASM browser app: Rust stable (1.94.1) compiled via wasm-pack (0.14.0) using wasm-bindgen (0.2.118) for interop, served by Vite (8.0.8) with the vite-plugin-wasm adapter. The frontend is vanilla HTML/CSS/JS — no framework needed for 9 squares and a score counter.

**Core technologies:**
- **Rust + wasm-pack**: Game logic language + build tool — compiles to compact WASM with one command (`wasm-pack build --target web`)
- **wasm-bindgen**: Rust↔JS bridge — generates type-safe JS glue and TypeScript definitions; the canonical interop solution
- **Vite**: Dev server and bundler — fast HMR, native ESM, zero-config for static sites; replaces Webpack
- **Vanilla JS/CSS**: Frontend rendering — trivial DOM manipulation (~100 LOC), no framework overhead justified
- **serde-wasm-bindgen**: Struct↔JsValue bridge — only if complex data transfer needed; prefer scalar returns for game queries
- **rand + getrandom**: AI randomness — powers minimax mistake injection; requires `wasm_js` feature for browser entropy

**Critical version notes:** rand 0.10 requires getrandom 0.4+ with `wasm_js` feature (not older `js` feature). wasm-bindgen/web-sys/js-sys share lockstep versioning. Rust edition 2024 requires Rust 1.85+.

### Expected Features

**Must have (table stakes — v1):**
- Playable 3×3 grid with click/tap input
- Computer opponent with beatable AI (imperfect minimax)
- Win/loss/draw detection with winning line highlighting
- Game outcome message and turn indicator
- New game/restart button
- Score tracking (W/L/D)
- Responsive layout (phone + desktop)
- Occupied cell rejection

**Should have (differentiators — v1.x):**
- CSS animations (piece placement, board transitions)
- Win celebration effect (confetti/glow)
- Computer "thinking" delay (300-800ms)
- Sound effects with mute toggle
- Persistent scores via localStorage
- Dark mode / prefers-color-scheme

**Defer (v2+):**
- Keyboard accessibility (tab navigation)
- Animated SVG pieces
- Tutorial/hint system

**Hard no (anti-features):**
- Online multiplayer, two-player local, difficulty selection, move history/undo, leaderboards, board size options, achievements. These all add complexity without proportional value for a single-player casual game.

### Architecture Approach

The architecture follows the canonical Rust/WASM pattern: an opaque `Game` struct lives in WASM linear memory, exposed to JS via `#[wasm_bindgen]` methods that accept commands (`make_move`, `computer_move`, `reset`) and answer queries (`get_cell`, `get_status`, `get_winner_line`). All return values are scalars (`u8`, `u32`, `bool`) to minimize FFI overhead. JS controls the render loop and timing; Rust never touches the DOM.

**Major components:**
1. **Board Logic** (`board.rs`) — cell state, move validation, win/draw detection; zero dependencies
2. **AI Engine** (`ai.rs`) — minimax with random mistake injection; depends on board representation
3. **Game State Machine** (`game.rs`) — turn management, scores, game phase; composes board + AI
4. **WASM Export Surface** (`lib.rs`) — thin `#[wasm_bindgen]` layer re-exporting Game methods
5. **Frontend** (`www/`) — HTML grid, CSS animations, JS event handlers + renderer (~3 files)

**Key patterns:**
- Opaque Handle Pattern: JS holds a pointer to Game, never reads struct internals
- Scalar Returns: `#[repr(u8)]` enums cross FFI with zero allocation
- JS-Driven Render Loop: WASM is a passive state machine, JS orchestrates timing and display

### Critical Pitfalls

1. **Async WASM initialization** — Must `await init()` before calling any export. Fix: structure entry point as `async function run()`. Phase 1.
2. **Missing panic hook** — Rust panics become silent `unreachable` traps. Fix: call `console_error_panic_hook::set_once()` at init. Phase 1.
3. **Wrong crate-type** — `cdylib` only breaks `cargo test`. Fix: always use `["cdylib", "rlib"]`. Phase 1.
4. **Ownership confusion at FFI boundary** — Pass-by-value consumes JS handle. Fix: use `&self`/`&mut self` methods exclusively. Phase 1.
5. **WASM binary size bloat** — `format!`, `unwrap()`, and serde can inflate the binary. Fix: release profile with `lto = true`, `opt-level = "z"`. Phase 2.

## Implications for Roadmap

Based on research, the project naturally decomposes into 5 phases following the dependency chain identified in ARCHITECTURE.md. Each phase produces a testable, demonstrable artifact.

### Phase 1: Rust Game Engine (Board + AI)
**Rationale:** Board logic has zero dependencies and is the foundation for everything else. AI depends only on board. Both are pure Rust — fully testable with `cargo test`, no WASM or browser needed. This phase also includes project scaffolding (Cargo.toml, crate-type, panic hook) to avoid every critical Phase 1 pitfall.
**Delivers:** Complete game logic — board state, move validation, win/draw detection, minimax AI with mistake injection — all verified by native unit tests.
**Addresses:** Win/loss/draw detection, computer opponent (AI), occupied cell rejection.
**Avoids:** Wrong crate-type (#4), no panic hook (#3), ownership confusion (#5 — API shape decided here).

### Phase 2: WASM Integration
**Rationale:** Now that game logic is tested natively, wrap it in `#[wasm_bindgen]` exports and compile to WASM. This is where the FFI boundary is built and validated. Must happen before frontend because the JS layer depends on the compiled `pkg/` output.
**Delivers:** Working `pkg/` directory with `.wasm` binary + JS glue + TypeScript definitions. WASM module loadable in a browser.
**Addresses:** WASM game engine (the core technical differentiator).
**Avoids:** Async WASM init (#1), excessive boundary crossing (#2 — API is already designed around scalars), ownership confusion (#5 — verified with wasm-bindgen-test).

### Phase 3: Browser Frontend (Core Game Loop)
**Rationale:** With the WASM module built and exported, build the HTML/CSS/JS frontend that renders the game and handles user interaction. This is the first time the game is playable in a browser.
**Delivers:** Fully playable tic-tac-toe in the browser — grid rendering, click handling, computer response, game outcome display, new game flow, score tracking, responsive layout.
**Addresses:** Playable 3×3 grid, turn indicator, game outcome message, new game button, score tracking, responsive layout.
**Avoids:** No loading state during WASM init (show spinner), instant computer moves (add delay wrapper).

### Phase 4: Visual Polish & UX
**Rationale:** Core game works. Now layer on the differentiating polish that makes this feel like a product rather than a tutorial exercise. All additions are CSS/JS — no Rust changes needed.
**Delivers:** Animated piece placement, win celebration effect, thinking delay, win line highlighting animation, smooth transitions.
**Addresses:** CSS animations, win celebration, computer thinking delay, win line highlighting enhancement.
**Avoids:** Binary size bloat (#6 — optimize release build here), UX pitfalls (win timing, click feedback).

### Phase 5: Extended Features & Release
**Rationale:** Game is polished. Add remaining v1.x features and prepare for release (optimize binary, test across browsers/devices).
**Delivers:** Sound effects, persistent scores, dark mode, release-optimized WASM binary, cross-browser verification.
**Addresses:** Sound effects, persistent scores (localStorage), dark mode.
**Avoids:** Debug builds in production (verify release build), mobile click issues (cross-device testing).

### Phase Ordering Rationale

- **Bottom-up from dependencies:** Board → AI → WASM exports → Frontend → Polish mirrors the actual dependency graph. You can't build the frontend without the WASM module, and you can't build the WASM module without the game logic.
- **Maximize testability early:** Phases 1-2 are fully testable without a browser. If the game logic has bugs, you catch them with `cargo test` before wiring up the entire UI.
- **Polish is additive, not structural:** Phases 4-5 are purely layered on top of a working game. CSS animations and sound effects don't change the architecture. If time runs short, the game still works without them.
- **Pitfalls front-loaded:** Every critical pitfall (crate-type, panic hook, WASM init, FFI design) is addressed in Phases 1-2 before complexity increases.

### Research Flags

**Phases likely needing deeper research during planning:**
- **Phase 1 (AI tuning):** Minimax is well-documented, but the "mistake injection" strategy (how often, which suboptimal move to pick) needs experimentation during implementation. Research a range of approaches — random move substitution vs. depth-limited search vs. evaluation noise.
- **Phase 4 (Win celebration):** CSS animation or canvas-based confetti — the specific approach depends on desired visual effect. Brief research during planning to pick a lightweight implementation.

**Phases with standard patterns (skip research-phase):**
- **Phase 2 (WASM integration):** Extremely well-documented. The Rust WASM Book Game of Life tutorial covers this exact pattern step-by-step.
- **Phase 3 (Frontend):** Standard DOM manipulation. Vanilla JS event handling + CSS Grid layout. No novel patterns.
- **Phase 5 (Extended features):** localStorage, Web Audio API, CSS custom properties — all have comprehensive MDN documentation.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified against crates.io and npm APIs; compatibility matrix confirmed; official docs + Context7 sources |
| Features | HIGH | Competitor analysis against 3 real products; clear MVP/v1.x/v2+ boundaries align with PROJECT.md scope |
| Architecture | HIGH | Follows canonical pattern from official Rust WASM Book; no novel architectural decisions needed |
| Pitfalls | HIGH | All pitfalls sourced from official Rust WASM documentation; every pitfall has a verified fix |

**Overall confidence:** HIGH

### Gaps to Address

- **AI mistake rate tuning:** Research identifies minimax-with-mistakes as the approach, but the optimal mistake probability (10%? 25%? variable?) needs playtesting. Plan for iteration time in Phase 1.
- **Computer thinking delay UX:** The ideal delay range (300-800ms mentioned) may need tuning based on feel. Not a research gap — just a playtesting variable.
- **WASM binary size target:** Research suggests <50KB is achievable for tic-tac-toe. Validate during Phase 2 build; if larger, profile with `twiggy` and reduce serde usage.
- **vite-plugin-top-level-await necessity:** May not be needed if targeting `esnext` exclusively. Test without it first; add only if browser compat requires it.
- **CSS animation approach:** Whether to use CSS transitions, CSS `@keyframes`, or Web Animations API for piece placement and win effects. Low risk — decide during Phase 4.

## Sources

### Primary (HIGH confidence)
- Rust and WebAssembly Book — Game of Life tutorial, debugging reference, code size reference
- wasm-bindgen official docs — exported types, closures, deployment, weak references
- wasm-pack official docs — build command, project templates
- crates.io API — version verification for all Rust dependencies
- npmjs.org API — version verification for Vite and plugins
- forge.rust-lang.org — Rust stable version (1.94.1)

### Secondary (HIGH confidence)
- Context7: wasm-pack, serde-wasm-bindgen, vite-plugin-wasm, js-sys documentation
- Competitor analysis — playtictactoe.org, Google Search TTT, PaperGames.io
- MDN Web Docs — Web Audio API, localStorage, CSS animations, game development intro

### Tertiary (MEDIUM confidence)
- Wikipedia: Tic-tac-toe — game theory and solved game analysis
- React.dev tutorial — feature comparison (move history anti-pattern validation)

---
*Research completed: 2026-04-12*
*Ready for roadmap: yes*
