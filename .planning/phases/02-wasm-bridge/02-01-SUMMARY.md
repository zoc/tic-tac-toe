---
phase: 02-wasm-bridge
plan: 01
subsystem: wasm-bridge
tags: [wasm, wasm-bindgen, wasm-pack, rust, getrandom, console_error_panic_hook]

# Dependency graph
requires:
  - phase: 01-rust-game-engine
    provides: "Game struct, AI (get_computer_move), Player/GameStatus enums, 20 native tests"
provides:
  - "WasmGame opaque handle with all game operations exported via wasm_bindgen"
  - "pkg/ directory with .wasm binary, JS glue, and TypeScript definitions"
  - "Minimal HTML test page (test.html) exercising all WASM exports"
affects: [03-browser-frontend]

# Tech tracking
tech-stack:
  added: [wasm-bindgen 0.2, console_error_panic_hook 0.1, getrandom 0.4 (wasm_js), wasm-pack 0.14, wasm-bindgen-test 0.3]
  patterns: [opaque-handle-export, scalar-type-boundary, numeric-board-array]

key-files:
  created: [src/wasm_api.rs, test.html, .gitignore]
  modified: [Cargo.toml, Cargo.lock, src/ai.rs, src/lib.rs]

key-decisions:
  - "GameStatus exposed via getter methods (get_status, get_winner, get_winning_positions) — avoids serde dependency"
  - "rand upgraded 0.8→0.10 with RngExt trait for WASM-compatible getrandom 0.4"
  - "computer_move() both computes and applies the AI move in one call"

patterns-established:
  - "Opaque handle pattern: WasmGame wraps Game, JS interacts through methods only"
  - "Numeric encoding: Player→u8 (1=X, 2=O), Board→Vec<u8> (0=empty), no serde at boundary"
  - "Return value 255 signals 'no move' (game over) for computer_move()"

requirements-completed: [ENG-01]

# Metrics
duration: 5min
completed: 2026-04-12
---

# Phase 02 Plan 01: WASM Bridge Summary

**Rust game engine compiled to WASM via wasm-pack with WasmGame opaque handle exporting all game operations through scalar-type wasm_bindgen boundary**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-12T18:39:24Z
- **Completed:** 2026-04-12T18:45:12Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Compiled Rust game engine to WebAssembly via wasm-pack (ENG-01 satisfied)
- Created WasmGame wrapper with all game operations: make_move, get_board, current_player, get_status, get_winner, get_winning_positions, computer_move, reset
- Upgraded rand 0.8→0.10 with getrandom wasm_js feature for browser entropy — all 20 native tests pass
- Produced pkg/ with .wasm (33KB), JS glue, TypeScript definitions
- Created test.html with 20 assertions exercising every WasmGame method

## Task Commits

Each task was committed atomically:

1. **Task 1: Add WASM dependencies and create wasm_bindgen API surface** - `0aff002` (feat)
2. **Task 2: Create minimal HTML test page and verify WASM module in browser** - `05f99ac` (feat)

## Files Created/Modified
- `src/wasm_api.rs` - WasmGame wrapper struct with #[wasm_bindgen] methods (D-01 opaque handle)
- `test.html` - Browser test page with 20 assertions covering all WASM exports (D-04)
- `Cargo.toml` - Added wasm-bindgen, console_error_panic_hook, getrandom (wasm_js), upgraded rand 0.8→0.10
- `Cargo.lock` - Updated dependency tree for new/upgraded dependencies
- `src/ai.rs` - Migrated rand API: thread_rng→rng, gen_bool→random_bool, gen_range→random_range, Rng→RngExt
- `src/lib.rs` - Registered wasm_api module
- `.gitignore` - Added target/ and pkg/ build output exclusions

## Decisions Made
- **GameStatus boundary (D-02):** Exposed via separate getter methods (get_status returns string, get_winner returns u8, get_winning_positions returns Vec<usize>) — clean scalar-type boundary, no serde needed
- **Board encoding (D-03):** Vec<u8> with 0=empty, 1=X, 2=O — maps to JS Uint8Array
- **computer_move convenience:** Computes AND applies AI move in a single call — reduces JS-side API surface
- **255 sentinel value:** computer_move returns 255 when game is over — unambiguous since valid positions are 0-8

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed rand 0.10 trait import: Rng → RngExt**
- **Found during:** Task 1 (rand upgrade)
- **Issue:** Plan said `use rand::Rng;` would work with rand 0.10, but `random_bool`/`random_range` methods are on `RngExt` trait in rand 0.10
- **Fix:** Changed import from `use rand::Rng;` to `use rand::RngExt;`
- **Files modified:** src/ai.rs
- **Verification:** All 20 native tests pass, WASM build succeeds
- **Committed in:** 0aff002 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Minor API naming difference in rand 0.10. No scope creep.

## Issues Encountered
- `cargo` not on PATH — used direct path via `$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo` and added `$HOME/.cargo/bin` after wasm-pack install
- wasm-pack first run downloaded and compiled `wasm-bindgen-cli` binary (~18s) — expected one-time cost

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- pkg/ directory contains everything Phase 3 needs: .wasm binary, JS glue (`init()` + `WasmGame` class), TypeScript definitions
- Import pattern for Phase 3: `import init, { WasmGame } from './pkg/tic_tac_toe.js'`
- All WasmGame methods documented in TypeScript definitions (pkg/tic_tac_toe.d.ts)
- test.html serves as a reference for how to initialize and call the WASM module

---
*Phase: 02-wasm-bridge*
*Completed: 2026-04-12*

## Self-Check: PASSED

All files verified present, all commits found, all build outputs exist.
