---
phase: 13-rust-ai-parameterization-wasm-api
fixed_at: 2026-04-28T00:00:00Z
review_path: .planning/phases/13-rust-ai-parameterization-wasm-api/13-REVIEW.md
fix_scope: critical_warning
findings_in_scope: 5
fixed: 5
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 13: Code Review Fix Report

**Fixed at:** 2026-04-28
**Source review:** .planning/phases/13-rust-ai-parameterization-wasm-api/13-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: `computer_move` silently eats a failed internal `make_move` and returns a stale position

**Files modified:** `src/wasm_api.rs`
**Commit:** f331181
**Applied fix:** Replaced `let _ = self.inner.make_move(pos)` with an `if self.inner.make_move(pos).is_ok()` check. When `make_move` fails, the method now returns `255` (the sentinel) instead of `pos`, correctly signaling to the caller that no move was applied.

---

### WR-01: `set_difficulty` accepts and silently normalizes out-of-range values

**Files modified:** `src/wasm_api.rs`
**Commit:** 76e0a3b
**Applied fix:** Added `level.min(3)` clamping in `set_difficulty` so values above 3 are clamped to 3 (Unbeatable) rather than silently falling through to the wildcard arm in `mistake_rate_for_level`. Updated the doc comment to document this clamping behavior.

---

### WR-02: `computer_move` return type uses a magic sentinel (`255`) with no type-level enforcement

**Files modified:** `src/wasm_api.rs`
**Commit:** 3ff345c
**Applied fix:** Changed `computer_move` return type from `u8` to `Option<u8>`. On success returns `Some(pos as u8)`, on failure (no move available or `make_move` error) returns `None`. Updated doc comment to document JS caller pattern (`if (pos !== undefined)`). This fix builds on top of CR-01 — the `make_move` error path returns `None` instead of `255`.

---

### WR-03: CLI `main.rs` hardcodes difficulty and ignores the phase's parameterization goal

**Files modified:** `src/main.rs`
**Commit:** 17ccd35
**Applied fix:** Added `std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1).min(3)` to parse an optional difficulty argument (0-3) from the command line, defaulting to Medium (1). Added a startup message displaying the active difficulty level with usage hint. Replaced hardcoded `1` in the `get_computer_move` call with the parsed `difficulty` variable. Removed the `// D-07: hardcode Medium; no --difficulty flag` comment.

---

### WR-04: `get_winning_positions` return type is `Vec<usize>` — portability issue for non-WASM32 targets

**Files modified:** `src/wasm_api.rs`
**Commit:** 897ce70
**Applied fix:** Changed `get_winning_positions` return type from `Vec<usize>` to `Vec<u32>`, with an explicit `.map(|&p| p as u32)` conversion. This makes the board-position type explicit and portable across WASM32, WASM64, and native targets, avoiding potential wasm-bindgen serialization issues.

---

## Skipped Issues

None — all in-scope findings were fixed.

---

_Fixed: 2026-04-28_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
