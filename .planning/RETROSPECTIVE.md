# Retrospective: Milestone v1.0

**Shipped:** 2026-04-13
**Duration:** 2 days (2026-04-12 → 2026-04-13)
**Scope:** 3 phases, 3 plans, 44 commits, ~1,373 LOC

---

## What Was Built

A complete browser-based tic-tac-toe game: Rust game engine compiled to WebAssembly via wasm-pack, served by a Vite 8 frontend with dark navy/red UI, responsive CSS Grid board, win highlighting, score tracking, keyboard navigation, and an imperfect minimax AI that's actually beatable.

---

## What Went Well

**Rust/WASM architecture was clean.** Splitting game logic (Rust) from rendering (JS) kept both sides simple and independently testable. The scalar-type WASM boundary (`u8`/`bool`/`i32`) avoided serialization complexity — no serde needed at the boundary.

**Phased approach paid off.** Starting with pure Rust + native tests meant the game engine was provably correct before any WASM or browser complexity was introduced. Phase 3 wiring "just worked" because Phase 1 logic was solid.

**Imperfect minimax was the right call.** The 25% flat mistake rate (tunable constant in `src/ai.rs:AI_MISTAKE_RATE`) makes the game genuinely fun — a perfect opponent turns tic-tac-toe into a memorization exercise. The AI wins consistently but humans can beat it.

**Vite 8 + vite-plugin-wasm was seamless.** WASM ESM import worked out of the box. Hot reload for CSS/JS changes without WASM recompilation made frontend iteration fast.

**Code review caught real issues.** The 5 post-phase fixes (XSS via innerHTML, hover during disabled state, keyboard navigation, stale status message, redundant border) were all legitimate improvements — not noise. The review paid for itself.

---

## What Could Have Gone Better

**vite-plugin-top-level-await compatibility.** STACK.md listed this plugin but it's incompatible with Vite 8 (bundles rollup internally, plugin expects standalone rollup). Cost one debug cycle. `build.target: 'esnext'` is the correct Vite 8 approach — STACK.md should have noted this.

**CSS Grid row sizing omission.** Cells resized on first move because `grid-template-rows` wasn't set (only `grid-template-columns` was). This is a well-known CSS Grid gotcha — worth a reminder in future UI phases: always set both row and column templates for square grids.

**REQUIREMENTS.md traceability drifted.** During phase execution many requirements were marked "Pending" in REQUIREMENTS.md even after being validated. The requirements file ended up out of sync with actual phase completion. PROJECT.md was the authoritative source but REQUIREMENTS.md was confusing to read mid-project.

---

## Surprises

- `wasm-pack build --target web` produced a 33KB `.wasm` binary without any manual optimization passes — `wasm-opt` runs automatically on release builds. Smaller than expected.
- The `grid-template-rows` bug only manifested after X/O text was rendered (not at page load), making it a tricky intermittent-looking visual glitch.
- `vite-plugin-top-level-await` fails at build time (not dev time), so the mismatch wasn't caught until the first production build run.

---

## Decisions That Aged Well

| Decision | Why It Held Up |
|----------|---------------|
| Scalar WASM boundary (no serde) | Zero serialization bugs; easy to test |
| Vanilla JS (no framework) | ~160 LOC game loop; zero framework overhead |
| 25% mistake rate as constant | Easy to tune; makes the game winnable but not trivial |
| CSS custom properties for theme | Theme changes require one variable edit, not a refactor |

---

## What to Carry Forward

- Set both `grid-template-rows` and `grid-template-columns` when building square CSS Grid layouts
- Prefer `build.target: 'esnext'` over `vite-plugin-top-level-await` with Vite 8+
- Keep WASM boundaries scalar — avoid passing complex types across the JS/Rust boundary unless necessary
- Code review after Phase 3 (UI) consistently finds XSS, a11y, and disabled-state hover issues — make these standard checklist items

---

## v2 Ideas (Backlog)

- CSS entry/exit animations on cell placement and win highlight
- Artificial computer "thinking" delay (300–800ms) for more natural feel
- localStorage score persistence across page refreshes
- Difficulty selector (easy / medium / hard = lower/medium/higher mistake rates)
- Sound effects toggle

---

*Written: 2026-04-13*
