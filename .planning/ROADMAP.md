# Roadmap: Tic-Tac-Toe WASM

## Milestones

- ✅ **v1.0 MVP** — Phases 1-3 (shipped 2026-04-13)
- ✅ **v1.1 Polish & Feel** — Phases 4-8 (shipped 2026-04-13)
- ✅ **v1.2 Docker Deployment** — Phases 9-10 (shipped 2026-04-14)
- 🔄 **v1.3 CI/CD & Distribution** — Phases 11-12 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-3) — SHIPPED 2026-04-13</summary>

- [x] Phase 1: Rust Game Engine (1/1 plans) — completed 2026-04-12
- [x] Phase 2: WASM Bridge (1/1 plans) — completed 2026-04-12
- [x] Phase 3: Browser Game (1/1 plans) — completed 2026-04-13

See archive: `.planning/milestones/v1.0-ROADMAP.md`

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

<details>
<summary>✅ v1.2 Docker Deployment (Phases 9-10) — SHIPPED 2026-04-14</summary>

- [x] Phase 9: Docker Image & nginx (2/2 plans) — completed 2026-04-14
- [x] Phase 10: Documentation (1/1 plans) — completed 2026-04-14

See archive: `.planning/milestones/v1.2-ROADMAP.md`

</details>

## Current Milestone: v1.3 CI/CD & Distribution

### ✅ Phase 11: GitHub Actions Workflow — COMPLETED 2026-04-25

**Goal:** Create GitHub Actions workflow for multi-platform Docker builds and Docker Hub publishing

**Requirements:** ✅ All 6 requirements verified
- ✅ CICD-01: GitHub Actions workflow triggers on git tag push (tags matching `v*`)
- ✅ CICD-02: Workflow builds Docker image for linux/amd64 platform
- ✅ CICD-03: Workflow builds Docker image for linux/arm64 platform
- ✅ CICD-04: Multi-arch manifest created combining both platform images
- ✅ DIST-01: Images published to Docker Hub under user's repository
- ✅ DIST-04: Docker Hub credentials securely managed via GitHub Secrets

**Accomplishments:**
- Verified `.github/workflows/docker.yml` implements all Phase 11 requirements
- Multi-platform builds configured for linux/amd64 and linux/arm64
- Docker Hub publishing secured with GitHub Secrets (DOCKERHUB_TOKEN, DOCKERHUB_USERNAME)
- Added comprehensive `## Releasing` section to README with prerequisites and step-by-step guide
- Documented end-to-end test procedure using v1.3.0-test tag

**Plans:** 1/1 complete (11-01-SUMMARY.md)
**Duration:** 12 minutes

---

### Phase 12: Release Automation

**Goal:** Add semver tag automation and OCI image labels for production releases

**Requirements:**
- DIST-02: Semver tags generated automatically (v1.3.0 → tags: 1.3.0, 1.3, 1, latest)
- DIST-03: OCI image labels attached (org.opencontainers.image.* annotations)

**Success Criteria:**
1. Workflow uses `docker/metadata-action` to generate tags array
2. Tag pattern generates all variants: full version (1.3.0), minor (1.3), major (1), latest
3. OCI labels include: title, description, url, source, version, created, revision
4. Manual test push of `v1.3.0` tag results in Docker Hub showing all 4 tag variants
5. Docker Hub image metadata displays OCI labels correctly

**Estimated effort:** 1 plan

---

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
| 10. Documentation | v1.2 | 1/1 | Complete | 2026-04-14 |
| 11. GitHub Actions Workflow | v1.3 | 1/1 | Complete | 2026-04-25 |
| 12. Release Automation | v1.3 | 0/1 | Not started | — |

## Summary

**v1.3 CI/CD & Distribution:** 2 phases, 8 requirements, 2 plans total

**Progress:** Phase 11 complete (1/2 phases, 6/8 requirements validated). Phase 12 ready to begin.
