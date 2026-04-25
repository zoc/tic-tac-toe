# Milestones

## v1.3 CI/CD & Distribution (Shipped: 2026-04-25)

**Phases completed:** 2 phases (11-12), 2 plans, ~14 commits, 1 day (2026-04-25)

**Key accomplishments:**

- GitHub Actions workflow with multi-platform Docker builds for linux/amd64 + linux/arm64 triggered on v*.*.* tags
- Docker Hub publishing with secure GitHub Secrets management (DOCKERHUB_TOKEN secret, DOCKERHUB_USERNAME variable)
- Semver tag automation: v1.3.0 → generates tags 1.3.0, 1.3, 1, latest — all pointing to same multi-arch manifest
- OCI image labels (org.opencontainers.image.* annotations) automatically generated via docker/metadata-action v5
- README Releasing section with prerequisites, step-by-step release process, workflow details, and technical notes
- Published to Docker Hub as fzoc/tic-tac-toe with full multi-arch support

**Archive:** `.planning/milestones/v1.3-ROADMAP.md` · `.planning/milestones/v1.3-REQUIREMENTS.md`

---

## v1.2 Docker Deployment (Shipped: 2026-04-14)

**Phases completed:** 2 phases (9-10), 3 plans, ~42 commits, 2 days (2026-04-13 → 2026-04-14)

**Key accomplishments:**

- Production-ready multi-stage Dockerfile: Rust/Node build stage → nginx:alpine serve stage (25.9MB image), wasm-pack 0.14.0 pinned via `cargo install --locked`, Node 20 via NodeSource
- All 5 ROADMAP success criteria passed on first build: correct WASM MIME type, `Cache-Control: immutable` for assets + `no-cache` for index.html, gzip for text assets, HEALTHCHECK on `/healthz`, port 80 exposed
- README updated with copy-pasteable `docker build` + `docker run` Quick Start and nginx reverse proxy deployment section for VPS hosting

**Archive:** `.planning/milestones/v1.2-ROADMAP.md` · `.planning/milestones/v1.2-REQUIREMENTS.md`

---

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
