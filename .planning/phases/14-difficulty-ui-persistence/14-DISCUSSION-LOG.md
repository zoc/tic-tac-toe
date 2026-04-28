# Phase 14: Difficulty UI & Persistence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 14-difficulty-ui-persistence
**Areas discussed:** Dropdown placement, Dropdown styling, Game-over behavior

---

## Dropdown Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Title row | Right of mute button in existing .title-row flex row — no new HTML wrapper | ✓ |
| Below scoreboard | Dedicated row between scoreboard and status message | |
| Below status message | Just above the board, below "Your turn" status line | |

**User's choice:** Title row

| Sub-question | Options | Selected |
|--------------|---------|----------|
| Label presence | "Difficulty:" label + select / Select only | Label + select ✓ |

**Notes:** Visible "Difficulty:" label chosen for clarity on first visit.

---

## Dropdown Styling

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal dark theme | Dark background (var(--surface)), light text, 1px border; native <select> rendering | ✓ |
| Accent-bordered | Dark background with red accent border on focus | |
| You decide | Claude picks whatever integrates most cleanly | |

**User's choice:** Minimal dark theme

| Sub-question | Options | Selected |
|--------------|---------|----------|
| Disabled appearance | Browser default :disabled (opacity + not-allowed cursor) / Match board--disabled pattern | Browser default ✓ |

**Notes:** Keep CSS simple — native browser disabled styling is sufficient.

---

## Game-over Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-reset immediately | Call resetGame() on any difficulty change regardless of game state | ✓ |
| Requires New Game click | Difficulty staged; player still clicks New Game to apply | |

**User's choice:** Auto-reset immediately

| Sub-question | Options | Selected |
|--------------|---------|----------|
| Empty board edge case | Always reset / Skip if board empty | Always reset ✓ |

**Notes:** Unconditional reset keeps code simple and behavior consistent. On empty board, reset() is cheap with no visual change.

---

## Claude's Discretion

None — all areas had explicit user decisions.

## Deferred Ideas

None — discussion stayed within phase scope.
