# Roadmap: Tic-Tac-Toe WASM

## Milestones

- ✅ **v1.0 MVP** — Phases 1-3 (shipped 2026-04-13)
- ✅ **v1.1 Polish & Feel** — Phases 4-8 (shipped 2026-04-13)
- ✅ **v1.2 Docker Deployment** — Phases 9-10 (shipped 2026-04-14)
- ✅ **v1.3 CI/CD & Distribution** — Phases 11-12 (shipped 2026-04-25)
- 🚧 **v1.4 Difficulty Levels** — Phases 13-14 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-3) — SHIPPED 2026-04-13</summary>

- [x] Phase 1: Rust Game Engine (1/1 plans) — completed 2026-04-12
- [x] Phase 2: WASM Bridge (1/1 plans) — completed 2026-04-12
- [x] Phase 3: Browser Game (1/1 plans) — completed 2026-04-13

See archive: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 Polish & Feel (Phases 4-8) — SHIPPED 2026-04-13</summary>

- [x] Phase 4: CSS Foundation & Persistence (1/1 plans) — completed 2026-04-13
- [x] Phase 5: CSS Piece Animations (1/1 plans) — completed 2026-04-13
- [x] Phase 6: Thinking Delay (1/1 plans) — completed 2026-04-13
- [x] Phase 7: Sound Effects & Mute (1/1 plans) — completed 2026-04-13
- [x] Phase 8: Animated Win Line (1/1 plans) — completed 2026-04-13

See archive: `.planning/milestones/v1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2 Docker Deployment (Phases 9-10) — SHIPPED 2026-04-14</summary>

- [x] Phase 9: Docker Image & nginx (2/2 plans) — completed 2026-04-14
- [x] Phase 10: Documentation (1/1 plans) — completed 2026-04-14

See archive: `.planning/milestones/v1.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.3 CI/CD & Distribution (Phases 11-12) — SHIPPED 2026-04-25</summary>

- [x] Phase 11: GitHub Actions Workflow (1/1 plans) — completed 2026-04-25
- [x] Phase 12: Release Automation (1/1 plans) — completed 2026-04-25

See archive: `.planning/milestones/v1.3-ROADMAP.md`

</details>

### 🚧 v1.4 Difficulty Levels (In Progress)

**Milestone Goal:** Let the player choose how hard the computer plays, with the setting persisted across sessions.

- [ ] **Phase 13: Rust AI Parameterization & WASM API** - Parameterize AI mistake rate by difficulty level and expose `set_difficulty(u8)` through the WASM boundary
- [ ] **Phase 14: Difficulty UI & Persistence** - Add dropdown selector to the game UI, wire localStorage persistence, and integrate with game reset and thinking-delay guard

## Phase Details

### Phase 13: Rust AI Parameterization & WASM API
**Goal**: The Rust AI accepts a runtime difficulty level and the WASM boundary exposes `set_difficulty(u8)` for JS to call
**Depends on**: Nothing (Phase 12 complete)
**Requirements**: AI-01, AI-02, AI-03, AI-04
**Success Criteria** (what must be TRUE):
  1. Calling `game.set_difficulty(0)` produces AI that loses frequently (Easy, ~65% mistake rate) in manual play
  2. Calling `game.set_difficulty(1)` produces AI at the existing default skill level (Medium, ~25% mistake rate)
  3. Calling `game.set_difficulty(2)` produces AI that wins or draws nearly every game (Hard, ~8% mistake rate)
  4. Calling `game.set_difficulty(3)` produces AI that never loses — every game is a win or draw (Unbeatable, 0% mistake rate)
  5. `wasm-pack build` completes with no errors and all existing Rust tests pass
**Plans**: 1 plan

Plans:
- [ ] 13-01-PLAN.md — Parameterize AI, extend WASM API, update all call sites, verify build

### Phase 14: Difficulty UI & Persistence
**Goal**: The player can pick a difficulty from a dropdown in the UI, the choice survives page refresh, and the dropdown stays disabled while the computer is thinking
**Depends on**: Phase 13
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05
**Success Criteria** (what must be TRUE):
  1. A difficulty dropdown (Easy / Medium / Hard / Unbeatable) is visible in the game UI and can be changed before or after a game
  2. Refreshing the page restores the previously selected difficulty (via `ttt-difficulty` localStorage key)
  3. A player visiting for the first time sees Medium pre-selected
  4. Changing the difficulty mid-game immediately resets the board so the new AI level applies from move one
  5. The difficulty dropdown is disabled (non-interactive) while the computer is calculating its move
**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 13. Rust AI Parameterization & WASM API | 0/1 | Not started | - |
| 14. Difficulty UI & Persistence | 0/TBD | Not started | - |
