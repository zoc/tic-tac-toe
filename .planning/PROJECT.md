# Tic-Tac-Toe WASM

## What This Is

A browser-based tic-tac-toe game where a human player (X) plays against a computer opponent. The game logic is written in Rust and compiled to WebAssembly, with a polished web frontend featuring animations, color, and win highlighting. The computer is beatable — it plays well but makes occasional mistakes.

## Core Value

The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Rust game engine compiled to WebAssembly
- [ ] 3x3 grid rendered in the browser
- [ ] Human plays as X, always goes first
- [ ] Computer opponent (O) with beatable AI
- [ ] Win/loss/draw detection with visual highlighting
- [ ] Score tracking across games (wins, losses, draws)
- [ ] Polished UI — animations, colors, visual flair
- [ ] New game / restart functionality

### Out of Scope

- Multiplayer / two-human mode — single-player focus
- Online / networked play — local browser only
- Mobile native app — web only
- Difficulty selection — single beatable difficulty level
- Player choosing X or O — human is always X

## Context

- Rust + WebAssembly via wasm-pack and wasm-bindgen
- Game logic (board state, AI, win detection) lives in Rust/WASM
- Frontend (rendering, animations, event handling) in HTML/CSS/JS
- No heavy framework needed — vanilla JS or lightweight approach
- The AI should be based on minimax with random mistakes to keep it beatable

## Constraints

- **Tech stack**: Rust compiled to WASM via wasm-pack — no other compiled language
- **Runtime**: Modern browsers with WebAssembly support (Chrome, Firefox, Safari, Edge)
- **Scope**: Single-player tic-tac-toe only — no feature creep beyond the core game

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + wasm-pack for WASM compilation | Mature toolchain, wasm-bindgen for JS interop | — Pending |
| Beatable AI via imperfect minimax | Perfect play is frustrating; occasional mistakes make it fun | — Pending |
| Human always X, goes first | Simplifies UX — no pregame choice needed | — Pending |
| Score tracking across games | Adds replayability without complexity | — Pending |

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
*Last updated: 2026-04-12 after initialization*
