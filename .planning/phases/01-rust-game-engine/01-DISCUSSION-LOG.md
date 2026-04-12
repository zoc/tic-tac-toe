# Phase 1: Rust Game Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-12
**Phase:** 01-Rust Game Engine
**Areas discussed:** AI mistake behavior, Board & game state design, API surface for Phase 2, Test strategy

---

## AI Mistake Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Flat probability per move | AI picks a random valid move instead of optimal X% of the time. Simple, predictable, easy to tune. | ✓ |
| Escalating mistakes | AI starts strong, then mistake rate increases as game progresses (5% early, 30% late). | |
| Weighted suboptimal picks | AI evaluates top N moves and sometimes picks 2nd or 3rd best. More natural-feeling. | |
| You decide | Let the agent pick the approach | |

**User's choice:** Flat probability per move
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| ~15% mistake rate | Balance — human wins sometimes but has to work for it | |
| ~25% mistake rate | More forgiving — casual players win more often | ✓ |
| ~10% mistake rate | Harder to beat — AI mostly plays optimally | |
| You decide, make it configurable | Start with a rate and let playtesting decide | |

**User's choice:** ~25% mistake rate
**Notes:** STATE.md flagged this needs playtesting

---

## Board & Game State Design

| Option | Description | Selected |
|--------|-------------|----------|
| Flat array [9] | [Option<Player>; 9], index 0-8, row-major. Simplest, cache-friendly, easy WASM serialization. | ✓ |
| 2D array [3][3] | [[Option<Player>; 3]; 3]. More readable for row/column logic, slightly more complex for WASM. | |
| Bitboards | Two u16s (one per player). Fast win checks via bitmask. Overkill but fun. | |
| You decide | Let the agent pick | |

**User's choice:** Flat array [9]
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Board + current player + status | Minimal: board cells, current player, game status (InProgress/Won/Draw) | ✓ |
| Board + player + status + history | Above plus move history Vec. Enables undo — but undo is out of scope. | |
| You decide | Let the agent decide | |

**User's choice:** Board + current player + status
**Notes:** No history needed — undo is out of scope per REQUIREMENTS.md

---

## API Surface for Phase 2

| Option | Description | Selected |
|--------|-------------|----------|
| Game struct with methods | Game struct with new(), make_move(), computer_move(), get_board(), get_status(), reset(). Clean OOP-style. | |
| Free functions + ID | Standalone functions with GameId. Stateless API, state in global map. | |
| You decide | Let the agent decide the API shape | ✓ |

**User's choice:** You decide (agent's discretion)
**Notes:** Agent should choose what maps cleanly to wasm_bindgen in Phase 2

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, return winning cells | Return which cells formed the winning line (e.g., [0, 1, 2]). Enables UI highlighting in Phase 3. | ✓ |
| No, just win/loss/draw status | Minimal — frontend figures out winning line itself. | |
| You decide | Let the agent decide | |

**User's choice:** Yes, return winning cells
**Notes:** Cheap to add now, avoids rework in Phase 3

---

## Test Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Thorough coverage | All 8 winning lines, draw, occupied-cell rejection, turn alternation, 100-game AI batch test. | ✓ |
| Exhaustive edge cases | Above plus every terminal state, AI on all 9 openings, reset correctness. | |
| Minimal happy paths | One win, one draw, one illegal move, one AI game. | |
| You decide | Let the agent decide | |

**User's choice:** Thorough coverage
**Notes:** Matches success criteria exactly

---

## Agent's Discretion

- API shape (struct with methods vs free functions)
- Internal minimax implementation details
- Rust project structure (module layout, file organization)

## Deferred Ideas

None — discussion stayed within phase scope
