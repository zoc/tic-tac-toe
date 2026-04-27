# Requirements: v1.4 Difficulty Levels

**Defined:** 2026-04-27
**Core Value:** The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

## Milestone Goal

Let the player choose how hard the computer plays, with the setting persisted across sessions.

## v1.4 Requirements

### Difficulty Levels (AI)

- [ ] **AI-01**: User can play against Easy AI that makes frequent mistakes (~65% mistake rate) and is regularly beatable
- [ ] **AI-02**: User can play against Medium AI that plays at the existing default skill level (~25% mistake rate)
- [ ] **AI-03**: User can play against Hard AI that rarely makes mistakes (~8% mistake rate) and is challenging to beat
- [ ] **AI-04**: User can play against Unbeatable AI with perfect minimax play (0% mistake rate) that never loses

### Difficulty Selector (UI)

- [ ] **UI-01**: User can select difficulty via a dropdown in the game UI before and after games
- [ ] **UI-02**: Selected difficulty persists across page refreshes via localStorage (`ttt-difficulty` key)
- [ ] **UI-03**: First visit defaults to Medium difficulty
- [ ] **UI-04**: Changing difficulty resets the current game immediately
- [ ] **UI-05**: Difficulty dropdown is disabled while the computer is thinking

## Future Requirements

Features deferred beyond v1.4.

### Difficulty

- Per-difficulty score tracking (separate win/loss/draw tallies per level)
- "You can only draw" indicator on Unbeatable difficulty
- Animated difficulty change transition

## Out of Scope

Explicitly excluded to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Per-difficulty score tracking | Adds complexity to localStorage schema; single shared score keeps existing behavior |
| Unbeatable warning callout | User chose to let players discover it themselves |
| Volume slider | Out of scope since v1.1 — binary mute is sufficient |
| Multiplayer | Out of scope — single-player focus |

## Traceability

Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AI-01 | — | Pending |
| AI-02 | — | Pending |
| AI-03 | — | Pending |
| AI-04 | — | Pending |
| UI-01 | — | Pending |
| UI-02 | — | Pending |
| UI-03 | — | Pending |
| UI-04 | — | Pending |
| UI-05 | — | Pending |

**Coverage:**
- v1.4 requirements: 9 total
- Mapped to phases: 0 (pending roadmap)
- Unmapped: 9 ⚠️

---
*Requirements defined: 2026-04-27*
*Last updated: 2026-04-27 — initial v1.4 requirements*
