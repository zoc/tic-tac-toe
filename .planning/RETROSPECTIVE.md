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

---

# Retrospective: Milestone v1.1

**Shipped:** 2026-04-13
**Duration:** 1 day (2026-04-13)
**Scope:** 5 phases (4-8), 5 plans, 25 commits, ~1,689 LOC total

---

## What Was Built

All six polish features from v1.1 scope: CSS-only dark/light theming, localStorage score persistence, pop-in piece animations (incremental DOM), cancelable computer thinking delay, Web Audio synthesized sounds with mute toggle, and animated win line for all 8 orientations.

---

## What Went Well

**Verify-first pattern dominated.** 3 of 5 phases (Phase 4, 7, 8) were verification-only — Phase 3 had pre-implemented all their requirements in commit `18a87a0`. The pattern is: run static grep checks → human browser approval. No wasted reimplementation.

**Pre-implementation efficiency.** Because Phase 3 shipped all implementation in one large commit, Phases 4/7/8 became cheap verification checkpoints. Total time for these three phases: ~1 hour combined. This is a strong argument for front-loading implementation when the design is clear.

**Incremental DOM update was a clean fix.** The Phase 5 bug (pop-in firing on all pieces) had a clear root cause (`innerHTML=''` wipe on every render) and a clean fix (children.length guard + patch path). One file changed, 15 minutes end-to-end.

**clearTimeout was the right tool.** Phase 6 FEEL-02 (cancel thinking delay on New Game) used `clearTimeout` rather than AbortController or a game-ID counter. Simpler, sufficient, readable. The pitfall documentation from earlier phases guided the choice.

**Web Audio synthesis over files.** Zero audio files shipped. No network requests. ~82 LOC for 5 distinct sounds. The OscillatorNode/GainNode pattern is compact and browser-native — fits a 9-cell game perfectly.

---

## What Could Have Gone Better

**REQUIREMENTS.md traceability not maintained during execution.** Same issue as v1.0 — three requirements (ANIM-01, FEEL-01, FEEL-02) were left unchecked in REQUIREMENTS.md even though their phases completed successfully. Discovered only at milestone archival. The checkbox state should be updated as part of the phase summary commit, not left for cleanup.

**ROADMAP.md progress table had stale plan counts.** Phase 6 showed `0/?` plans and Phase 8 showed `0/?` plans in the table even after plans were created. Minor but creates confusion when reading state mid-milestone.

**No milestone audit.** Proceeded without a `v1.1-MILESTONE-AUDIT.md`. In this case it was fine because the SUMMARY files made completeness obvious, but the audit would have caught the stale checkboxes earlier.

---

## Surprises

- Phase 3 had pre-implemented all 16 v1.1 requirements before v1.1 planning even started. The entire Polish milestone was essentially "verify what Phase 3 built, fix two bugs, ship."
- The `diag-rl` win-line CSS bug (`rotate(-45deg)` → `left: 95% + rotate(135deg)`) was subtle — an off-by-direction error that only shows in one of 8 possible win configurations.
- Lazy AudioContext creation (on first user gesture) was essential for incognito mode in Safari, not just autoplay policy compliance.

---

## Decisions That Aged Well

| Decision | Why It Held Up |
|----------|---------------|
| Pre-implement all v1.1 in Phase 3 | 3 of 5 subsequent phases became cheap verification passes |
| CSS custom properties established in Phase 3 | Phase 4 theming required zero new architecture |
| `@media (prefers-reduced-motion)` guard in CSS | ANIM-03 satisfied by construction — no JS needed |
| Verify-first plan structure | Catches pre-implementation without re-implementing |

---

## What to Carry Forward

- Update REQUIREMENTS.md checkboxes **during** phase execution, not at archival time
- Add `prefers-reduced-motion` guard to all CSS animations from the start
- Lazy AudioContext pattern is the correct default for any Web Audio feature — document it in CONVENTIONS.md
- Incremental DOM update (build once, patch later) is the right pattern for any animation-on-add UI feature

---

*Written: 2026-04-14*

---

## Cross-Milestone Trends

*Updated after each milestone. Patterns that persist across multiple milestones.*

### Velocity

| Milestone | Phases | Plans | LOC | Duration | Commits |
|-----------|--------|-------|-----|----------|---------|
| v1.0 MVP | 3 | 3 | ~1,373 | 2 days | 44 |
| v1.1 Polish & Feel | 5 | 5 | ~1,689 | 1 day | 25 |

### What Consistently Works

- Bottom-up phased approach (logic → bridge → UI) keeps each phase independently testable
- Code review after UI phases consistently finds XSS, a11y, and hover-state bugs
- Verify-first plan structure: static grep checks + human browser approval catches pre-implementation without wasted re-work
- Pre-implementing polish features alongside core features reduces later phase cost dramatically

### Watch Out For

- CSS Grid: always set `grid-template-rows` AND `grid-template-columns` for square grids
- Vite 8+: use `build.target: 'esnext'` instead of `vite-plugin-top-level-await`
- Requirements traceability: mark requirements Complete during phase execution, not after milestone archival
- Web Audio: always use lazy AudioContext (created on first user gesture) — required for Safari/incognito autoplay policy
