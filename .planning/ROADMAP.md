# Roadmap: Tic-Tac-Toe WASM

## Overview

Build a browser-based tic-tac-toe game bottom-up: first implement complete game logic and AI in pure Rust (testable without a browser), then compile to WebAssembly and define the JS interop surface, then build the full browser frontend that delivers a playable game with scoring and responsive layout. All v2 polish (animations, sound, dark mode) is deferred — v1 ships a complete, functional game.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Rust Game Engine** - Board logic, win/draw detection, move validation, and beatable AI — all in pure Rust with native tests
- [ ] **Phase 2: WASM Bridge** - Compile game engine to WebAssembly via wasm-pack with a clean scalar-based JS interop surface
- [ ] **Phase 3: Browser Game** - Fully playable tic-tac-toe in the browser with grid rendering, turn flow, outcome display, scoring, and responsive layout

## Phase Details

### Phase 1: Rust Game Engine
**Goal**: Complete game logic exists in pure Rust — board state, move validation, win/draw detection, and a beatable AI opponent — fully verified by native unit tests before any WASM or browser work begins
**Depends on**: Nothing (first phase)
**Requirements**: ENG-03, ENG-04, AI-01, AI-02
**Success Criteria** (what must be TRUE):
  1. `cargo test` passes with tests covering move placement, occupied-cell rejection, win detection (rows, columns, diagonals), and draw detection
  2. AI opponent generates a valid move for any non-terminal board state
  3. AI is beatable — running 100 automated games produces at least some human wins (mistake injection works)
  4. AI never makes an illegal move (never places on an occupied cell, never moves when game is over)
**Plans:** 1 plan
Plans:
- [x] 01-01-PLAN.md — Board state, move validation, win/draw detection, beatable AI, and comprehensive tests

### Phase 2: WASM Bridge
**Goal**: The Rust game engine compiles to a WebAssembly module via wasm-pack, producing a `pkg/` directory with `.wasm` binary and JS/TS glue that can be imported by a browser application
**Depends on**: Phase 1
**Requirements**: ENG-01
**Success Criteria** (what must be TRUE):
  1. `wasm-pack build --target web` succeeds and produces a `pkg/` directory with `.wasm` + JS glue + TypeScript definitions
  2. WASM module can be loaded in a browser (`await init()`) without errors — verified by a minimal HTML test page
  3. All game operations (make move, get board state, get game status, computer move, reset) are callable from JavaScript through the exported API
**Plans:** 1 plan
Plans:
- [ ] 02-01-PLAN.md — WASM dependencies, wasm_bindgen API surface, and browser test page

### Phase 3: Browser Game
**Goal**: A human can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with clear visual feedback, score tracking, and responsive layout
**Depends on**: Phase 2
**Requirements**: ENG-02, UI-01, UI-02, UI-03, UI-04, UI-05, SCORE-01
**Success Criteria** (what must be TRUE):
  1. User can click any empty cell on a 3x3 grid and see their X piece appear immediately
  2. Computer responds with an O move after each human turn, and a turn indicator shows whose turn it is throughout
  3. When a game ends, the user sees a clear outcome message ("You win!", "Computer wins!", "It's a draw!") and can start a new game via a restart button without refreshing the page
  4. A running score tally (wins, losses, draws) updates after each game and persists across multiple games in the same session
  5. The game is playable on both phone and desktop screens — grid and controls adapt to viewport size
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Rust Game Engine | 0/0 | Not started | - |
| 2. WASM Bridge | 0/0 | Not started | - |
| 3. Browser Game | 0/0 | Not started | - |
