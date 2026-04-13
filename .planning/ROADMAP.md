# Roadmap: Tic-Tac-Toe WASM

## Milestones

- ✅ **v1.0 MVP** — Phases 1-3 (shipped 2026-04-13)
- 🔄 **v1.1 Polish & Feel** — Phases 4-8 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-3) — SHIPPED 2026-04-13</summary>

- [x] Phase 1: Rust Game Engine (1/1 plans) — completed 2026-04-12
- [x] Phase 2: WASM Bridge (1/1 plans) — completed 2026-04-12
- [x] Phase 3: Browser Game (1/1 plans) — completed 2026-04-13

</details>

### v1.1 Polish & Feel

- [x] **Phase 4: CSS Foundation & Persistence** — Dark mode + persistent scores (lowest risk) (completed 2026-04-13)
- [ ] **Phase 5: CSS Piece Animations** — Pop-in animation on X/O placement
- [ ] **Phase 6: Thinking Delay** — Computer pause before moving (async refactor)
- [ ] **Phase 7: Sound Effects & Mute** — Web Audio synthesized tones + mute toggle
- [ ] **Phase 8: Animated Win Line** — Line draws through winning cells

## Phase Details

### Phase 4: CSS Foundation & Persistence
**Goal**: The game adapts to the user's OS color scheme and scores survive page refresh
**Depends on**: Phase 3 (Browser Game — v1.0 complete)
**Requirements**: THEM-01, THEM-02, PERS-01, PERS-02
**Success Criteria** (what must be TRUE):
  1. User on a light-mode OS sees a light-themed board without any JavaScript running — theme applies from CSS alone
  2. User switching OS to dark mode and back sees the board adapt without a page reload
  3. User who refreshes the page after 5 wins still sees 5 wins in the scoreboard
  4. User in private/incognito mode can play normally — no crash or error when localStorage is unavailable
**Plans**: 1 plan
**UI hint**: yes

Plans:
- [x] 04-01-PLAN.md — Automated spot-checks + manual browser verification of pre-implemented theming and persistence

### Phase 5: CSS Piece Animations
**Goal**: Piece placement feels responsive and tactile with pop-in animation
**Depends on**: Phase 4
**Requirements**: ANIM-01, ANIM-03
**Success Criteria** (what must be TRUE):
  1. When any X or O is placed, user sees it scale in from 0 with a brief pop/spring — not just appear
  2. Only the newly placed piece animates — existing pieces on the board do not re-animate
  3. User with `prefers-reduced-motion: reduce` set in their OS sees pieces appear instantly with no animation
**Plans**: TBD
**UI hint**: yes

### Phase 6: Thinking Delay
**Goal**: The computer feels deliberate — it pauses before responding, like it's thinking
**Depends on**: Phase 5
**Requirements**: FEEL-01, FEEL-02
**Success Criteria** (what must be TRUE):
  1. After the human places a move, the computer's piece appears 300–800ms later (visibly delayed, not instant)
  2. The delay varies from game to game — not always the same pause length
  3. When the user clicks "New Game" mid-delay, the computer does not place a move on the new board — the pending move is cancelled cleanly
**Plans**: TBD

### Phase 7: Sound Effects & Mute
**Goal**: The game has synthesized audio feedback for all game events, with a persistent mute option
**Depends on**: Phase 6
**Requirements**: AUDI-01, AUDI-02, AUDI-03, AUDI-04, AUDI-05, AUDI-06, AUDI-07
**Success Criteria** (what must be TRUE):
  1. User placing their piece hears a distinct click/tap sound; the computer's piece plays a different distinct sound
  2. User hears a win fanfare, a loss sound, or a draw sound — each audibly distinct from each other
  3. User clicks the mute button and all subsequent sounds are silent — no audio plays until unmuted
  4. User who mutes, refreshes the page, and plays a move hears no sound — mute preference was saved
  5. User on first page load can click any cell and hear sound (no silent first click from autoplay policy)
**Plans**: TBD

### Phase 8: Animated Win Line
**Goal**: Winning feels theatrical — a line draws itself through the three winning cells
**Depends on**: Phase 7
**Requirements**: ANIM-02
**Success Criteria** (what must be TRUE):
  1. When the game ends in a win (human or computer), user sees a line animate from one end of the winning trio to the other — it draws itself rather than appearing instantly
  2. The win line correctly overlays all three orientations: horizontal rows, vertical columns, and both diagonals
  3. The win line is gone when a new game starts — no residual line from the previous game
  4. On small viewports the win line stays visually aligned with the winning cells — not offset or misaligned
**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rust Game Engine | v1.0 | 1/1 | Complete | 2026-04-12 |
| 2. WASM Bridge | v1.0 | 1/1 | Complete | 2026-04-12 |
| 3. Browser Game | v1.0 | 1/1 | Complete | 2026-04-13 |
| 4. CSS Foundation & Persistence | v1.1 | 1/1 | Complete   | 2026-04-13 |
| 5. CSS Piece Animations | v1.1 | 0/? | Not started | - |
| 6. Thinking Delay | v1.1 | 0/? | Not started | - |
| 7. Sound Effects & Mute | v1.1 | 0/? | Not started | - |
| 8. Animated Win Line | v1.1 | 0/? | Not started | - |
