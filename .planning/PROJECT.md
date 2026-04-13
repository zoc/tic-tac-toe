# Tic-Tac-Toe WASM

## What This Is

A browser-based tic-tac-toe game where a human player (X) plays against a computer opponent. The game logic is written in Rust and compiled to WebAssembly, with a polished web frontend featuring color theming, win highlighting, and responsive layout. The computer is beatable — it plays well but makes occasional mistakes.

## Core Value

The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

## Current Milestone: v1.1 Polish & Feel

**Goal:** Elevate the v1.0 game with smooth animations, persistence, sound, and system dark mode — making it feel like a finished product.

**Target features:**
- Smooth CSS animations for piece placement and board transitions
- Animated win line through the three winning cells
- Computer "thinking" delay (300–800ms) before making a move
- Persistent scores via localStorage across page refresh
- Sound effects for moves and game outcomes with mute toggle
- Dark mode support respecting prefers-color-scheme

## Requirements

### Validated

- [x] Win/loss/draw detection — Validated in Phase 1: Rust Game Engine
- [x] Computer opponent (O) with beatable AI — Validated in Phase 1: Rust Game Engine
- [x] Human plays as X, always goes first — Validated in Phase 1: Rust Game Engine
- [x] Rust game engine compiled to WebAssembly — Validated in Phase 2: WASM Bridge
- [x] 3x3 grid rendered in the browser — Validated in Phase 3: Browser Game
- [x] Win/loss/draw detection with visual highlighting — Validated in Phase 3: Browser Game
- [x] Score tracking across games (wins, losses, draws) — Validated in Phase 3: Browser Game
- [x] Polished UI — dark navy/red theme, responsive grid, win highlight — Validated in Phase 3: Browser Game
- [x] New game / restart functionality — Validated in Phase 3: Browser Game
- [x] Keyboard navigation (tab + enter/space to place moves) — Validated in Phase 3: Browser Game (code review fix)
- [x] XSS-safe error handling — Validated in Phase 3: Browser Game (code review fix)
- [x] Hover suppressed during disabled board state — Validated in Phase 3: Browser Game (code review fix)

### Active

<!-- Current milestone v1.1 scope -->

  - [x] Smooth CSS animations for piece placement and board transitions — Validated in Phase 04: css-piece-animations
  - [x] Animated win line through the three winning cells — Validated in Phase 04: css-piece-animations
  - [x] Computer "thinking" delay (300–800ms) before making a move — Validated in Phase 06: thinking-delay
  - [ ] Persistent scores via localStorage across page refresh
  - [ ] Sound effects for moves and game outcomes with mute toggle
  - [ ] Dark mode support respecting prefers-color-scheme

### Out of Scope

- Multiplayer / two-human mode — single-player focus
- Online / networked play — local browser only
- Mobile native app — web only
- Difficulty selection — single beatable difficulty level
- Player choosing X or O — human is always X

## Context

- Rust + WebAssembly via wasm-pack and wasm-bindgen
- Game logic (board state, AI, win detection) lives entirely in Rust/WASM (~927 LOC)
- Frontend (rendering, event handling, score display) in HTML/CSS/JS (~446 LOC)
- Vanilla JS + CSS — no framework needed for a 9-cell game
- AI uses minimax with ~25% mistake injection rate — tunable constant in `src/ai.rs`
- Vite 8 dev server + production build; `vite-plugin-wasm` for WASM ESM import
- `build.target: 'esnext'` replaces `vite-plugin-top-level-await` (incompatible with Vite 8)
- CSS Grid with explicit `grid-template-rows` required for stable cell sizing

## Constraints

- **Tech stack**: Rust compiled to WASM via wasm-pack — no other compiled language
- **Runtime**: Modern browsers with WebAssembly support (Chrome, Firefox, Safari, Edge)
- **Scope**: Single-player tic-tac-toe only — no feature creep beyond the core game

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + wasm-pack for WASM compilation | Mature toolchain, wasm-bindgen for JS interop | Validated Phase 2 — 33KB .wasm binary, scalar-type boundary, all ops exported |
| Beatable AI via imperfect minimax | Perfect play is frustrating; occasional mistakes make it fun | Validated Phase 1 — 25% mistake rate, minimax with depth scoring |
| Human always X, goes first | Simplifies UX — no pregame choice needed | Validated Phase 1 — X starts, turn alternation enforced |
| Score tracking across games | Adds replayability without complexity | Validated Phase 3 — in-memory wins/losses/draws tally, resets on page refresh |
| `build.target: 'esnext'` over vite-plugin-top-level-await | Plugin incompatible with Vite 8 | Validated Phase 3 — top-level await works natively in esnext targets |
| `grid-template-rows: repeat(3, 1fr)` explicit sizing | Without it, rows collapse/expand as X/O text renders | Validated Phase 3 — stable cell dimensions throughout game |
| DOM text node construction over innerHTML for error display | Prevents XSS injection from error message strings | Validated Phase 3 — code review fix MD-01 |

## Current State

**Phase 06 complete** (2026-04-13). Cancelable thinking delay delivered — `thinkingTimer` clearTimeout pattern, post-delay `get_status()` guard, FEEL-01 and FEEL-02 human-verified.

**Milestone v1.0 complete** (2026-04-13). Full playable tic-tac-toe in the browser:
- Vite 8 dev server and production build
- Dark navy/red UI, responsive CSS Grid board
- WASM-powered AI (imperfect minimax, ~25% mistake rate)
- Win highlighting, draw detection, game-over lockout
- Score tracking (wins/losses/draws) across sessions
- Restart button (no page refresh needed)
- Keyboard navigation (tab + enter/space)
- XSS-safe, hover-suppressed-when-disabled
- 44 commits, 2 days (2026-04-12 → 2026-04-13)
- ~1,373 total LOC (927 Rust + 446 HTML/CSS/JS)

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-13 — Phase 06 thinking-delay complete*
