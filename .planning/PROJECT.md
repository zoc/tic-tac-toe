# Tic-Tac-Toe WASM

## What This Is

A browser-based tic-tac-toe game where a human player (X) plays against a computer opponent. The game logic is written in Rust and compiled to WebAssembly, with a polished web frontend featuring smooth CSS animations, synthesized sound effects, system dark mode support, score persistence, and an animated win line. The computer is beatable — it plays well but makes occasional mistakes. The game ships as a minimal Docker image (25.9MB) served by nginx with correct WASM MIME types, cache headers, gzip, and a health endpoint.

## Core Value

The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

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
- [x] CSS-only dark/light theming via `prefers-color-scheme` — Validated in Phase 4: CSS Foundation & Persistence — v1.1
- [x] No flash of unstyled content — Validated in Phase 4: CSS Foundation & Persistence — v1.1
- [x] Score persistence across page refresh via localStorage — Validated in Phase 4: CSS Foundation & Persistence — v1.1
- [x] Score persistence degrades gracefully in private/incognito mode — Validated in Phase 4: CSS Foundation & Persistence — v1.1
- [x] Pop-in animation on piece placement (only newly placed piece) — Validated in Phase 5: CSS Piece Animations — v1.1
- [x] `prefers-reduced-motion` suppresses animations — Validated in Phase 5: CSS Piece Animations — v1.1
- [x] Computer thinking delay 300–800ms randomized — Validated in Phase 6: Thinking Delay — v1.1
- [x] Thinking delay cancelled immediately on New Game (no ghost moves) — Validated in Phase 6: Thinking Delay — v1.1
- [x] Synthesized sound effects for all game events (human move, computer move, win, loss, draw) — Validated in Phase 7: Sound Effects & Mute — v1.1
- [x] Mute toggle button with localStorage persistence — Validated in Phase 7: Sound Effects & Mute — v1.1
- [x] Animated win line through all 8 winning orientations — Validated in Phase 8: Animated Win Line — v1.1
- [x] Multi-stage Dockerfile (Rust/wasm-pack build stage → nginx:alpine serve stage, 25.9MB image) — Validated in Phase 9: Docker Image & nginx — v1.2
- [x] nginx serves WASM with correct `Content-Type: application/wasm`, assets with `Cache-Control: immutable`, index.html with `no-cache` — Validated in Phase 9: Docker Image & nginx — v1.2
- [x] gzip compression for HTML, CSS, JS (WASM excluded — pre-optimized by wasm-opt) — Validated in Phase 9: Docker Image & nginx — v1.2
- [x] HEALTHCHECK on `/healthz` endpoint, port 80 exposed — Validated in Phase 9: Docker Image & nginx — v1.2
- [x] `.dockerignore` excludes `target/`, `node_modules/`, `pkg/`, `dist/`, `.git/` — Validated in Phase 9: Docker Image & nginx — v1.2
- [x] README Quick Start with copy-pasteable `docker build` + `docker run` one-liner — Validated in Phase 10: Documentation — v1.2
- [x] README nginx reverse proxy deployment section for VPS — Validated in Phase 10: Documentation — v1.2

## Current Milestone: v1.3 CI/CD & Distribution

**Goal:** Automate multi-arch Docker image builds and publish to Docker Hub on release tags

**Target features:**
- GitHub Actions workflow triggered on git tag push
- Multi-platform builds (linux/amd64 + linux/arm64)
- Docker Hub publishing with semver tag variants
- OCI image labels for metadata

## Active Requirements

### CI/CD Automation
- [ ] **CICD-01**: GitHub Actions workflow triggers on git tag push (tags matching `v*`)
- [ ] **CICD-02**: Workflow builds Docker image for linux/amd64 platform
- [ ] **CICD-03**: Workflow builds Docker image for linux/arm64 platform
- [ ] **CICD-04**: Multi-arch manifest created combining both platform images

### Distribution
- [ ] **DIST-01**: Images published to Docker Hub under user's repository
- [ ] **DIST-02**: Semver tags generated automatically (v1.3.0 → tags: 1.3.0, 1.3, 1, latest)
- [ ] **DIST-03**: OCI image labels attached (org.opencontainers.image.* annotations)
- [ ] **DIST-04**: Docker Hub credentials securely managed via GitHub Secrets

### Out of Scope

- Multiplayer / two-human mode — single-player focus
- Online / networked play — local browser only
- Mobile native app — web only
- Difficulty selection — single beatable difficulty level
- Player choosing X or O — human is always X
- Manual dark/light toggle button — pure CSS `@media (prefers-color-scheme)` handles the use case
- Volume slider — binary mute provides 80% value at 10% complexity
- Confetti / particle effects — obscures board, overkill for 30-second games
- Audio files (.mp3/.ogg) — synthesized Web Audio oscillators are sufficient
- localStorage board state — 30-second games; mid-game restoration not a felt pain point

## Context

- Rust + WebAssembly via wasm-pack and wasm-bindgen
- Game logic (board state, AI, win detection) lives entirely in Rust/WASM (~927 LOC)
- Frontend (rendering, event handling, score display, animations, audio) in HTML/CSS/JS (~762 LOC: 400 JS + 449 CSS — 13 LOC index.html excluded)
- Vanilla JS + CSS — no framework needed for a 9-cell game
- AI uses minimax with ~25% mistake injection rate — tunable constant in `src/ai.rs`
- Vite 8 dev server + production build; `vite-plugin-wasm` for WASM ESM import
- `build.target: 'esnext'` replaces `vite-plugin-top-level-await` (incompatible with Vite 8)
- CSS Grid with explicit `grid-template-rows` required for stable cell sizing
- Web Audio OscillatorNode synthesizer — 5 game sounds, ~82 LOC, zero audio files
- Lazy AudioContext singleton satisfies browser autoplay policy
- Incremental DOM update pattern in `renderBoard()` — only newly placed pieces trigger pop-in animation
- Docker: multi-stage image (Rust/Node build → nginx:alpine serve), 25.9MB final image
- wasm-pack pinned via `cargo install wasm-pack@0.14.0 --locked`; Node 20 via NodeSource
- nginx:alpine serves WASM with correct MIME type natively; gzip excludes WASM (pre-optimized)

## Constraints

- **Tech stack**: Rust compiled to WASM via wasm-pack — no other compiled language
- **Runtime**: Modern browsers with WebAssembly support (Chrome, Firefox, Safari, Edge)
- **Scope**: Single-player tic-tac-toe only — no feature creep beyond the core game

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + wasm-pack for WASM compilation | Mature toolchain, wasm-bindgen for JS interop | ✓ Validated Phase 2 — 33KB .wasm binary, scalar-type boundary, all ops exported |
| Beatable AI via imperfect minimax | Perfect play is frustrating; occasional mistakes make it fun | ✓ Validated Phase 1 — 25% mistake rate, minimax with depth scoring |
| Human always X, goes first | Simplifies UX — no pregame choice needed | ✓ Validated Phase 1 — X starts, turn alternation enforced |
| Score tracking across games | Adds replayability without complexity | ✓ Validated Phase 3 — in-memory wins/losses/draws tally, resets on page refresh |
| `build.target: 'esnext'` over vite-plugin-top-level-await | Plugin incompatible with Vite 8 | ✓ Validated Phase 3 — top-level await works natively in esnext targets |
| `grid-template-rows: repeat(3, 1fr)` explicit sizing | Without it, rows collapse/expand as X/O text renders | ✓ Validated Phase 3 — stable cell dimensions throughout game |
| DOM text node construction over innerHTML for error display | Prevents XSS injection from error message strings | ✓ Validated Phase 3 — code review fix MD-01 |
| CSS-only theming via `prefers-color-scheme` | No JS needed, no FOUC, adapts automatically | ✓ Validated Phase 4 — zero-JS theme, instant from first paint |
| localStorage persistence with try/catch | Graceful degradation in private/incognito mode | ✓ Validated Phase 4 — returns safe defaults on SecurityError |
| Incremental DOM update in renderBoard() | Full innerHTML wipe re-triggered pop-in on all cells | ✓ Validated Phase 5 — children.length guard, existing cells untouched |
| clearTimeout pattern for thinking delay | AbortController is overkill for setTimeout cancellation | ✓ Validated Phase 6 — thinkingTimer variable, clearTimeout in resetGame() |
| OscillatorNode synthesizer over audio files | Zero network requests, no asset loading latency, ~82 LOC | ✓ Validated Phase 7 — 5 distinct sounds, all human-approved |
| Lazy AudioContext singleton | Chrome/Safari autoplay policy: context only in user gesture | ✓ Validated Phase 7 — no silent first click in any browser |
| wasm-pack pinned via cargo install --locked | Eliminates curl\|sh supply-chain risk; reproducible builds | ✓ Validated Phase 9 — 0.14.0 pinned, no unversioned script fetching |
| Node 20 via NodeSource setup_20.x | Debian bookworm apt default is Node 18; README promised Node 20+ | ✓ Validated Phase 9 — Node 20.20.2 confirmed in build |
| gzip_types excludes application/wasm | WASM pre-optimized by wasm-opt; double-gzip adds CPU cost, negligible benefit | ✓ Validated Phase 9 — text assets gzipped, WASM served pre-compressed |
| HEALTHCHECK on /healthz (not /) | Dedicated endpoint keeps health probe noise out of nginx access logs | ✓ Validated Phase 9 — HTTP 200 "ok", access_log off |
| Plain docker build in Quick Start docs | No --platform/buildx prerequisites; works for any local machine | ✓ Validated Phase 10 — simplest copy-paste path for new users |

## Current State

**Milestone v1.2 complete** (2026-04-14). Production Docker image ships the game:
- Multi-stage Dockerfile: Rust/Node build stage → nginx:alpine serve stage (25.9MB image)
- wasm-pack 0.14.0 pinned via `cargo install --locked`; Node 20 via NodeSource
- nginx: correct WASM MIME type, `Cache-Control: immutable` for assets, `no-cache` for index.html
- gzip for HTML/CSS/JS (WASM excluded — pre-optimized), HEALTHCHECK on `/healthz`
- README Quick Start (`docker build` + `docker run`) and nginx reverse proxy docs
- 10 phases total, 11 plans complete, ~1,689 LOC game + Docker/nginx config

<details>
<summary>v1.1 state (2026-04-13)</summary>

Full polished tic-tac-toe in the browser:
- CSS-only dark/light theming via `prefers-color-scheme`, no JS, no FOUC
- localStorage score persistence with graceful incognito degradation
- Pop-in piece animation (incremental DOM update, only new piece animates)
- Computer thinking delay 300–800ms randomized, cancelable on New Game
- Web Audio synthesized sounds (5 distinct) + mute toggle with persistence
- Animated win line for all 8 orientations, `prefers-reduced-motion` aware
- 8 phases total, 8 plans complete, 70 commits, 2 days (2026-04-12 → 2026-04-13)
- ~1,689 total LOC (927 Rust + 762 JS/CSS)

</details>

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
*Last updated: 2026-04-25 after v1.3 milestone started — CI/CD & Distribution*
