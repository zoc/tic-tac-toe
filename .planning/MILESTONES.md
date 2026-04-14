# Milestones

## v1.1 Polish & Feel (Shipped: 2026-04-13)

**Phases completed:** 5 phases (4-8), 5 plans, ~1,689 LOC, 25 commits, 1 day

**Key accomplishments:**

- CSS-only dark/light theming via `@media (prefers-color-scheme)` — no JS, no FOUC, theme from first paint
- localStorage score persistence with `try/catch` graceful degradation for private/incognito mode
- Incremental `renderBoard()` DOM update — pop-in animation fires only on the newly placed piece (not the whole board)
- Cancelable 300–800ms computer thinking delay via `clearTimeout` — no ghost moves after New Game
- Web Audio OscillatorNode synthesizer — 5 distinct game sounds, lazy AudioContext for autoplay compliance, zero audio files shipped
- Animated win line through all 8 orientations with `prefers-reduced-motion` accessibility guard

**Archive:** `.planning/milestones/v1.1-ROADMAP.md` · `.planning/milestones/v1.1-REQUIREMENTS.md`

---

## v1.0 MVP (Shipped: 2026-04-13)

**Phases completed:** 3 phases, 3 plans, ~1,373 LOC, 44 commits, 2 days

**Key accomplishments:**

- Complete Rust game engine with board state management, win/draw detection for all 8 lines, and beatable AI via imperfect minimax (~25% mistake rate) — 20 tests all green
- Rust game engine compiled to WASM via wasm-pack with WasmGame opaque handle exporting all game operations through scalar-type wasm_bindgen boundary
- Vite 8 + vite-plugin-wasm frontend wiring Phase 2 WASM to dark navy/red responsive CSS Grid game UI with score tracking, win highlighting, keyboard navigation, and XSS-safe error handling

---
