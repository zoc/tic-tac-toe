# Requirements: Tic-Tac-Toe WASM

**Milestone:** v1.1 Polish & Feel
**Defined:** 2026-04-13
**Core Value:** The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

## v1.1 Requirements

### Animations

- [ ] **ANIM-01**: User sees a pop-in animation when X or O is placed on the board
- [ ] **ANIM-02**: User sees a line animate through the three winning cells when the game ends in a win
- [ ] **ANIM-03**: Animations are suppressed when `prefers-reduced-motion` is set

### Feel

- [ ] **FEEL-01**: Computer waits 300–800ms (randomized) before placing its move
- [ ] **FEEL-02**: Thinking delay is cancelled immediately when the user starts a new game (no ghost moves)

### Persistence

- [x] **PERS-01**: Score totals (wins/losses/draws) persist across page refresh via localStorage
- [x] **PERS-02**: Score persistence degrades gracefully in private/incognito mode (no crash)

### Audio

- [x] **AUDI-01**: User hears a distinct sound when placing their move
- [x] **AUDI-02**: User hears a distinct sound when the computer places its move
- [x] **AUDI-03**: User hears a win fanfare when they win
- [x] **AUDI-04**: User hears a loss sound when they lose
- [x] **AUDI-05**: User hears a draw sound when the game ends in a draw
- [x] **AUDI-06**: User can toggle sound on/off with a mute button
- [x] **AUDI-07**: Mute preference persists across page refresh

### Theming

- [x] **THEM-01**: The UI adapts to the user's OS light/dark mode preference automatically (`prefers-color-scheme`)
- [x] **THEM-02**: No flash of unstyled content — theme applies via CSS only, no JS

## Future Requirements

(None identified — all v2 candidates have been promoted to v1.1)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Manual dark/light toggle button | Pure CSS `@media (prefers-color-scheme)` handles the use case without UI chrome or JS |
| Volume slider | Binary mute provides 80% value at 10% complexity — slider is overkill |
| Confetti / particle effects | Obscures board, adds library weight, overkill for 30-second games |
| Audio files (.mp3/.ogg) | Synthesized Web Audio oscillators are instant, zero-byte, and sufficient for game UI sounds |
| localStorage board state | 30-second games; mid-game restoration is not a felt pain point |
| Multiplayer / two-human mode | Single-player focus (carried from v1.0) |
| Online / networked play | Local browser only (carried from v1.0) |
| Difficulty selection | Single beatable difficulty level (carried from v1.0) |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| THEM-01 | Phase 4 — CSS Foundation & Persistence | Complete |
| THEM-02 | Phase 4 — CSS Foundation & Persistence | Complete |
| PERS-01 | Phase 4 — CSS Foundation & Persistence | Complete |
| PERS-02 | Phase 4 — CSS Foundation & Persistence | Complete |
| ANIM-01 | Phase 5 — CSS Piece Animations | Pending |
| ANIM-03 | Phase 5 — CSS Piece Animations | Pending |
| FEEL-01 | Phase 6 — Thinking Delay | Pending |
| FEEL-02 | Phase 6 — Thinking Delay | Pending |
| AUDI-01 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-02 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-03 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-04 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-05 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-06 | Phase 7 — Sound Effects & Mute | Complete |
| AUDI-07 | Phase 7 — Sound Effects & Mute | Complete |
| ANIM-02 | Phase 8 — Animated Win Line | Pending |

**Coverage:**
- v1.1 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-13*
*Last updated: 2026-04-13 — traceability finalized after roadmap creation*
