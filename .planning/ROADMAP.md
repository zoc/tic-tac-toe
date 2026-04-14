# Roadmap: Tic-Tac-Toe WASM — v1.2 Docker Deployment

**Milestone:** v1.2 Docker Deployment
**Phases:** 2 (Phases 9–10)
**Requirements:** 10 (DOCK-01–08, DOCS-01–02)
**Previous milestone ended at:** Phase 8

## Milestones

- ✅ **v1.0 MVP** — Phases 1-3 (shipped 2026-04-13)
- ✅ **v1.1 Polish & Feel** — Phases 4-8 (shipped 2026-04-13)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-3) — SHIPPED 2026-04-13</summary>

- [x] Phase 1: Rust Game Engine (1/1 plans) — completed 2026-04-12
- [x] Phase 2: WASM Bridge (1/1 plans) — completed 2026-04-12
- [x] Phase 3: Browser Game (1/1 plans) — completed 2026-04-13

</details>

<details>
<summary>✅ v1.1 Polish & Feel (Phases 4-8) — SHIPPED 2026-04-13</summary>

- [x] Phase 4: CSS Foundation & Persistence (1/1 plans) — completed 2026-04-13
- [x] Phase 5: CSS Piece Animations (1/1 plans) — completed 2026-04-13
- [x] Phase 6: Thinking Delay (1/1 plans) — completed 2026-04-13
- [x] Phase 7: Sound Effects & Mute (1/1 plans) — completed 2026-04-13
- [x] Phase 8: Animated Win Line (1/1 plans) — completed 2026-04-13

See archive: `.planning/milestones/v1.1-ROADMAP.md`

</details>

## v1.2 Docker Deployment (Phases 9–10)

- [x] **Phase 9: Docker Image & nginx** — Build and locally verify a multi-stage Docker image serving the game
- [x] **Phase 10: Documentation** — Write README usage docs after image is verified working (completed 2026-04-14)

## Phase Overview

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|-----------------|
| 9 | Docker Image & nginx | Developer can build, run, and verify a correct Docker image locally | DOCK-01, DOCK-02, DOCK-03, DOCK-04, DOCK-05, DOCK-06, DOCK-07, DOCK-08 | 5 |
| 10 | Documentation | 1/1 | Complete    | 2026-04-14 |

## Phase Details

### Phase 9: Docker Image & nginx
**Goal:** Developer can build, run, and verify a correct multi-stage Docker image that serves the game locally with proper MIME types, caching headers, and gzip compression.
**Requirements:** DOCK-01, DOCK-02, DOCK-03, DOCK-04, DOCK-05, DOCK-06, DOCK-07, DOCK-08
**Success criteria:**
1. `docker build -t tic-tac-toe:test .` completes successfully from a clean checkout
2. `docker run --rm -p 8080:80 tic-tac-toe:test` starts the game at `http://localhost:8080` and gameplay works
3. `curl -I http://localhost:8080/assets/*.wasm` returns `Content-Type: application/wasm` (not `octet-stream`)
4. Response headers show `Cache-Control: public, max-age=31536000, immutable` for `/assets/*` and `Cache-Control: no-cache` for `index.html`
5. `docker inspect tic-tac-toe:test` shows a HEALTHCHECK configured and port 80 exposed; build context excludes `target/`, `node_modules/`, `pkg/`, `dist/`, `.git/`
**Plans:** 2 plans

Plans:
- [x] 09-01-PLAN.md — Fix Dockerfile (wasm-pack pin, Node 20, HEALTHCHECK) and nginx.conf (gzip, Cache-Control no-cache, /healthz)
- [x] 09-02-PLAN.md — Build image, run all 5 ROADMAP success criteria verifications, human verify gameplay

### Phase 10: Documentation
**Goal:** Developer can find a clear README section explaining how to run the container locally and deploy it behind a reverse proxy on a VPS.
**Requirements:** DOCS-01, DOCS-02
**Success criteria:**
1. README contains a copy-pasteable `docker build` + `docker run -p 8080:80` one-liner that works without prior context
2. README includes a note on deploying behind a reverse proxy (Caddy or nginx upstream) on a VPS
**Plans:** 1/1 plans complete

Plans:
- [x] 10-01-PLAN.md — Add Quick Start section and reverse proxy note to README.md

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rust Game Engine | v1.0 | 1/1 | Complete | 2026-04-12 |
| 2. WASM Bridge | v1.0 | 1/1 | Complete | 2026-04-12 |
| 3. Browser Game | v1.0 | 1/1 | Complete | 2026-04-13 |
| 4. CSS Foundation & Persistence | v1.1 | 1/1 | Complete | 2026-04-13 |
| 5. CSS Piece Animations | v1.1 | 1/1 | Complete | 2026-04-13 |
| 6. Thinking Delay | v1.1 | 1/1 | Complete | 2026-04-13 |
| 7. Sound Effects & Mute | v1.1 | 1/1 | Complete | 2026-04-13 |
| 8. Animated Win Line | v1.1 | 1/1 | Complete | 2026-04-13 |
| 9. Docker Image & nginx | v1.2 | 2/2 | Complete | 2026-04-14 |
| 10. Documentation | v1.2 | 0/1 | Not started | — |
