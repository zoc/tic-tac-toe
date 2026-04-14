# Requirements: Tic-Tac-Toe WASM

**Defined:** 2026-04-14
**Core Value:** The human player can play a complete, satisfying game of tic-tac-toe against the computer in their browser — with smooth interactions and clear visual feedback.

## v1.2 Requirements

Requirements for the Docker Deployment milestone.

### Docker Image

- [ ] **DOCK-01**: Developer can build the game image locally with a single `docker build` command
- [ ] **DOCK-02**: Image serves `.wasm` files with correct `Content-Type: application/wasm`
- [ ] **DOCK-03**: nginx serves Vite hashed assets with `Cache-Control: immutable` (1-year cache)
- [ ] **DOCK-04**: nginx serves `index.html` with `Cache-Control: no-cache`
- [ ] **DOCK-05**: gzip compression enabled for HTML, CSS, JS, and WASM
- [ ] **DOCK-06**: Container responds healthy to `HEALTHCHECK` and exposes port 80
- [ ] **DOCK-07**: `docker run -p 8080:80` starts the game at `http://localhost:8080`
- [ ] **DOCK-08**: Image build context excludes `target/`, `node_modules/`, `pkg/`, `dist/`, `.git/` (via `.dockerignore`)

### Documentation

- [ ] **DOCS-01**: README includes `docker build` and `docker run` one-liner
- [ ] **DOCS-02**: README includes notes on deploying behind a reverse proxy on a VPS

## Future Requirements

### CI/CD

- **CICD-01**: GitHub Actions workflow builds linux/amd64 + linux/arm64 on git tag push
- **CICD-02**: Multi-arch manifest published to Docker Hub on release tag
- **CICD-03**: Semver tags generated automatically from git tag (v1.2.0 → 1.2.0, 1.2, 1, latest)
- **CICD-04**: OCI image labels attached via docker/metadata-action
- **CICD-05**: GHA layer cache reduces warm build time

## Out of Scope

| Feature | Reason |
|---------|--------|
| Image size constraint | Removed — multi-stage nginx:alpine is naturally small; no hard limit needed |
| Docker Compose example | Low priority for v1.2; add if users request it |
| Security headers (CSP, X-Frame-Options) | Low priority for a game with no auth or user data |
| ARM v7 (linux/arm/v7) | Extremely slow under QEMU for Rust builds; add only on request |
| GitHub Container Registry mirror | Docker Hub sufficient for initial release |
| Volume mounts / runtime config | Static site — image must be fully self-contained, no runtime config needed |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOCK-01 | Phase 9 | Pending |
| DOCK-02 | Phase 9 | Pending |
| DOCK-03 | Phase 9 | Pending |
| DOCK-04 | Phase 9 | Pending |
| DOCK-05 | Phase 9 | Pending |
| DOCK-06 | Phase 9 | Pending |
| DOCK-07 | Phase 9 | Pending |
| DOCK-08 | Phase 9 | Pending |
| DOCS-01 | Phase 10 | Pending |
| DOCS-02 | Phase 10 | Pending |

**Coverage:**
- v1.2 requirements: 10 total
- Mapped to phases: 10 ✓
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-14*
*Last updated: 2026-04-14 — traceability filled in after roadmap creation*
