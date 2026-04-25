# Phase 11: GitHub Actions Workflow - Research

**Gathered:** 2026-04-25
**Phase:** 11 - GitHub Actions Workflow
**Goal:** Create GitHub Actions workflow for multi-platform Docker builds and Docker Hub publishing

## Executive Summary

Research covers GitHub Actions multi-platform Docker builds, Docker Hub publishing, and secrets management. The project already has `.github/workflows/docker.yml` implementing the core workflow — this phase focuses on verification, documentation, and ensuring all requirements are met.

## Current State Analysis

### Existing Workflow File

**File:** `.github/workflows/docker.yml` (64 lines)
**Status:** Implemented with comprehensive multi-arch support

**Current implementation:**
- ✅ Triggers on `v*.*.*` tags (CICD-01)
- ✅ Builds for `linux/amd64` and `linux/arm64` (CICD-02, CICD-03)
- ✅ Uses `docker/setup-buildx-action@v3` with multi-platform support
- ✅ Uses `docker/build-push-action@v6` with `platforms: linux/amd64,linux/arm64` (CICD-04)
- ✅ Uses `docker/metadata-action@v5` for semver tag generation
- ✅ Pushes to Docker Hub (DIST-01)
- ✅ Uses GitHub Secrets for `DOCKERHUB_TOKEN` and GitHub Variables for `DOCKERHUB_USERNAME` (DIST-04)
- ✅ GitHub Actions cache layer optimization (`cache-from/cache-to: type=gha,mode=max`)
- ✅ QEMU setup for cross-platform emulation
- ✅ Inline documentation explains setup requirements

**Key patterns:**
1. **QEMU for serve stage only** — Comments clarify that Rust build runs on native platform via `--platform=$BUILDPLATFORM` in Dockerfile line 4
2. **Secrets vs Variables pattern** — Token in Secrets, username in Variables (public vs sensitive)
3. **Semver tag generation** — `type=semver,pattern={{version}}` and `pattern={{major}}.{{minor}}`
4. **Latest tag gating** — `type=raw,value=latest,enable={{is_default_branch}}`

### Dockerfile Multi-Arch Support

**File:** `Dockerfile` (61 lines)
**Key feature:** Line 4 — `FROM --platform=$BUILDPLATFORM rust:slim AS build`

This pattern ensures:
- Rust compilation runs natively (no QEMU penalty for CPU-intensive work)
- WASM bytecode is platform-neutral (same output on amd64/arm64)
- Only the nginx:alpine serve stage runs under QEMU for foreign architectures
- Typical build time: ~2-3 minutes per platform with layer caching

## GitHub Actions Best Practices

### Multi-Platform Build Architecture

**Recommended pattern for Rust + Docker:**
```yaml
platforms: linux/amd64,linux/arm64
```

**Why this matters:**
- GitHub Actions runners are `linux/amd64`
- ARM64 builds run under QEMU emulation (setup-qemu-action)
- Rust build stage uses `--platform=$BUILDPLATFORM` to avoid QEMU penalty
- Only the nginx:alpine serve stage cross-compiles (minimal impact)

**Performance considerations:**
- Native amd64 build: ~2-3 minutes
- QEMU arm64 build: ~3-5 minutes (only for nginx layer)
- Total workflow time: ~5-8 minutes with parallel builds

### Action Versions

**Current usage (all latest stable):**
- `actions/checkout@v4` — Standard, no security issues
- `docker/setup-qemu-action@v3` — Latest, supports all platforms
- `docker/setup-buildx-action@v3` — Buildx v0.12+, multi-platform support
- `docker/login-action@v3` — Secure credential handling
- `docker/metadata-action@v5` — Semver tag generation
- `docker/build-push-action@v6` — Latest, improved caching

**Pinning strategy:** Using major version tags (`@v4`, `@v3`) gets patch/minor updates automatically while preventing breaking changes. This is Docker's recommended approach for Actions workflows.

### Tag Patterns

**Implemented pattern:**
```yaml
tags: |
  type=semver,pattern={{version}}       # v1.2.0 → 1.2.0
  type=semver,pattern={{major}}.{{minor}} # v1.2.0 → 1.2
  type=raw,value=latest,enable={{is_default_branch}}
```

**Phase 12 requirement (DIST-02):** Add `type=semver,pattern={{major}}` for major-only tag (v1.2.0 → 1)

**Current gap:** Missing major-only tag pattern. Phase 11 delivers v1.2.0 → 1.2.0, 1.2, latest. Phase 12 adds the major tag.

## Docker Hub Publishing

### Authentication Pattern

**Implemented pattern:**
```yaml
username: ${{ vars.DOCKERHUB_USERNAME }}    # Public GitHub Variable
password: ${{ secrets.DOCKERHUB_TOKEN }}     # Secret GitHub Secret
```

**Why this works:**
- **Username is public** (visible in Docker Hub URLs) — no need to hide it as a secret
- **Token is sensitive** — Docker Hub Personal Access Token with Read & Write scope
- **Token ≠ Password** — PATs are revocable, have limited scope, and are CI-friendly

**Setup instructions (already documented in workflow comments):**
1. Docker Hub → Account Settings → Security → New Access Token
2. Scope: Read & Write for target repository
3. GitHub → Repository Settings → Secrets and variables → Actions
4. Add Secret: `DOCKERHUB_TOKEN` (value = token)
5. Add Variable: `DOCKERHUB_USERNAME` (value = username)

### Multi-Arch Manifest

**How it works:**
- `docker/build-push-action@v6` with `platforms: linux/amd64,linux/arm64` automatically creates a multi-arch manifest
- Docker Hub stores separate image layers for each platform
- Clients pulling the image get the correct architecture automatically
- No manual `docker manifest create` commands needed

**Verification:**
```bash
docker buildx imagetools inspect <username>/tic-tac-toe:1.2.0
```

Expected output:
```
MediaType: application/vnd.docker.distribution.manifest.list.v2+json
Digest:    sha256:abc123...
           
Manifests: 
  Name:      <username>/tic-tac-toe:1.2.0@sha256:def456...
  Platform:  linux/amd64
  
  Name:      <username>/tic-tac-toe:1.2.0@sha256:ghi789...
  Platform:  linux/arm64
```

### Image Naming Convention

**Current pattern:**
```yaml
images: ${{ vars.DOCKERHUB_USERNAME }}/tic-tac-toe
```

**Result:** Images published to `<username>/tic-tac-toe:1.2.0`, `<username>/tic-tac-toe:1.2`, etc.

**Best practice alignment:**
- ✅ Lowercase repository name (Docker Hub requirement)
- ✅ Hyphenated slug (tic-tac-toe) matches project name
- ✅ No organization prefix (publishing to user namespace)

## Security Patterns

### Secrets Management (DIST-04)

**GitHub Secrets best practices:**

1. **Use Personal Access Tokens, not passwords**
   - Docker Hub supports fine-grained PATs with Read & Write scope
   - Revocable without changing account password
   - Can be scoped to specific repositories

2. **Separate public and sensitive data**
   - Username → GitHub Variable (public, appears in logs)
   - Token → GitHub Secret (redacted in logs, encrypted at rest)

3. **Least-privilege principle**
   - PAT scope: Read & Write for target repository only
   - No admin permissions needed for image publishing

4. **Rotation strategy**
   - Tokens should be rotated every 90-180 days
   - GitHub doesn't enforce expiry, but it's a security best practice
   - Workflow will fail if token is revoked → immediate feedback

**Current implementation:** ✅ Follows all best practices. Uses `secrets.DOCKERHUB_TOKEN` and `vars.DOCKERHUB_USERNAME`.

### Supply Chain Security

**Current protections:**

1. **Action version pinning** — `@v4`, `@v3` major version tags
   - Gets security patches automatically
   - Protects against breaking changes
   - Trade-off: doesn't prevent supply chain attacks on patch releases

2. **Dockerfile pinning** — `wasm-pack@0.14.0 --locked`
   - Eliminates `curl|sh` supply chain risk
   - Reproducible builds
   - Validated in Phase 9

3. **Docker image provenance** — Not yet implemented
   - `docker/build-push-action@v6` supports `provenance: true`
   - Generates SLSA attestations for supply chain verification
   - Phase 12 consideration (out of scope for v1.3)

## OCI Image Labels (Phase 12 Preview)

**Current state:** Workflow uses `labels: ${{ steps.meta.outputs.labels }}` but metadata action needs OCI configuration.

**Phase 12 requirement (DIST-03):**
```yaml
- name: Docker metadata
  id: meta
  uses: docker/metadata-action@v5
  with:
    images: ${{ vars.DOCKERHUB_USERNAME }}/tic-tac-toe
    tags: |
      type=semver,pattern={{version}}
      type=semver,pattern={{major}}.{{minor}}
      type=semver,pattern={{major}}           # NEW in Phase 12
      type=raw,value=latest,enable={{is_default_branch}}
    labels: |                                 # NEW in Phase 12
      org.opencontainers.image.title=Tic-Tac-Toe WASM
      org.opencontainers.image.description=Browser-based tic-tac-toe with Rust/WASM
      org.opencontainers.image.url=https://github.com/${{ github.repository }}
      org.opencontainers.image.source=https://github.com/${{ github.repository }}
      org.opencontainers.image.version={{version}}
      org.opencontainers.image.created={{date 'YYYY-MM-DDTHH:mm:ssZ'}}
      org.opencontainers.image.revision=${{ github.sha }}
```

**Phase 11 scope:** Workflow already has the plumbing (`labels: ${{ steps.meta.outputs.labels }}`). Phase 12 adds the label configuration.

## Testing Strategy

### Local Testing (Pre-Push)

**Single-platform test:**
```bash
docker build -t tic-tac-toe:test .
docker run --rm -p 8080:80 tic-tac-toe:test
# Verify http://localhost:8080 in browser
```

**Multi-platform test (requires buildx):**
```bash
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t tic-tac-toe:test .
```

### CI Testing (GitHub Actions)

**Workflow test strategy:**
1. Push a test tag: `git tag v1.3.0-test && git push origin v1.3.0-test`
2. Workflow triggers automatically
3. Monitor Actions tab for build progress
4. Verify images on Docker Hub:
   - Image shows linux/amd64 and linux/arm64 architectures
   - Tags: `1.3.0-test` appears
   - Image size ~26MB (matches local build)
5. Pull and test: `docker run --rm -p 8080:80 <username>/tic-tac-toe:1.3.0-test`

**Success criteria for Phase 11:**
- ✅ Workflow runs without errors
- ✅ Both platforms build successfully
- ✅ Multi-arch manifest created
- ✅ Images appear on Docker Hub with correct tags
- ✅ Pulled image runs correctly on local machine

## Codebase Patterns

### Existing Workflow Location

**Path:** `.github/workflows/docker.yml`
**Convention:** GitHub Actions standard location
**Pattern:** Single workflow file for Docker publishing (no separate CI/test workflows)

**Project conventions:**
- No existing test workflow (Docker build validates compilation)
- No linting workflow (PROJECT.md notes test jobs add latency with no new coverage)
- Docker workflow is the only CI automation

### Documentation Pattern

**Current docs:**
- Workflow file has inline comments explaining QEMU, secrets setup, and tag patterns
- README.md (Phase 10) documents Docker build/run but not GitHub Actions workflow

**Phase 11 gap:** README doesn't document the CI/CD workflow or how to trigger releases. Phase 11 should add a "Releasing" section.

## Implementation Checklist

### Phase 11 Requirements Mapping

**CICD-01: Workflow triggers on `v*` tags**
- ✅ Implemented: Line 16-17 `tags: - 'v*.*.*'`
- ✅ Pattern matches `v1.2.0`, `v1.3.0`, etc.
- 📝 Action: Verify in test

**CICD-02: Build for linux/amd64**
- ✅ Implemented: Line 58 `platforms: linux/amd64,linux/arm64`
- 📝 Action: Verify in test

**CICD-03: Build for linux/arm64**
- ✅ Implemented: Line 58 `platforms: linux/amd64,linux/arm64`
- 📝 Action: Verify in test

**CICD-04: Multi-arch manifest**
- ✅ Implemented: `docker/build-push-action@v6` automatically creates manifest for multiple platforms
- 📝 Action: Verify with `docker buildx imagetools inspect`

**DIST-01: Publish to Docker Hub**
- ✅ Implemented: Line 36-40 `docker/login-action` + Line 59 `push: true`
- 📝 Action: Verify images appear on Docker Hub

**DIST-04: Secrets management**
- ✅ Implemented: Line 39-40 uses `vars.DOCKERHUB_USERNAME` and `secrets.DOCKERHUB_TOKEN`
- 📋 Action: Document setup instructions in README
- 📋 Action: Verify secrets exist in repository settings

### Out of Scope for Phase 11

**DIST-02: Semver major tag (v1.2.0 → 1)**
- 🔄 Phase 12 requirement
- Current: Generates 1.2.0, 1.2, latest
- Missing: Major-only tag (1)

**DIST-03: OCI labels**
- 🔄 Phase 12 requirement
- Current: Plumbing exists (`labels: ${{ steps.meta.outputs.labels }}`)
- Missing: Label configuration in metadata action

## Technical Constraints

### GitHub Actions Runner Limits

**Standard runners:**
- Platform: `ubuntu-latest` (currently Ubuntu 22.04)
- Architecture: `linux/amd64` (x86_64)
- CPU: 2 cores
- RAM: 7GB
- Storage: 14GB SSD

**Build time considerations:**
- Rust compilation: CPU-intensive (~2-3 min)
- WASM-pack: ~30 sec
- npm ci + vite build: ~20 sec
- Docker layer caching reduces repeat builds to ~1 min

**Current workflow optimization:**
- ✅ `cache-from/cache-to: type=gha,mode=max` caches all layers
- ✅ Dockerfile layer ordering minimizes cache invalidation
- ✅ `--platform=$BUILDPLATFORM` avoids QEMU for Rust stage

### Docker Hub Rate Limits

**Authenticated pulls/pushes:**
- Personal accounts: Unlimited pulls, 50 pushes per day
- Rate limit headers: `RateLimit-Limit`, `RateLimit-Remaining`

**Workflow impact:**
- Each workflow run = 1 push (multi-arch manifest counts as single push)
- Multiple tags on same image = 1 push (tags are references, not copies)
- Current usage: ~1-5 releases per month → well within limits

## Risk Assessment

### Potential Issues

**1. Secrets not configured**
- **Symptom:** Workflow fails at login step with "401 Unauthorized"
- **Prevention:** Document setup in README with clear instructions
- **Mitigation:** GitHub Actions UI shows clear error message

**2. Tag pattern mismatch**
- **Symptom:** Workflow doesn't trigger on tag push
- **Current pattern:** `v*.*.*` (requires three version segments)
- **Risk:** Tags like `v1.0` or `v2` won't trigger workflow
- **Mitigation:** Document required tag format in README

**3. Multi-arch build timeout**
- **Symptom:** Workflow times out after 6 hours (GitHub Actions limit)
- **Likelihood:** Very low (current builds: 5-8 minutes)
- **Mitigation:** Layer caching keeps builds fast

**4. Docker Hub push failure**
- **Symptom:** Build succeeds but push fails (disk quota, rate limit, token revoked)
- **Prevention:** Monitor Docker Hub quota and token expiry
- **Mitigation:** Workflow fails loudly, retryable

### Validation Requirements

**Manual test plan (Phase 11 success criteria):**

1. **Verify secrets configuration:**
   ```bash
   # GitHub UI: Settings → Secrets and variables → Actions
   # Check: DOCKERHUB_TOKEN exists (value hidden)
   # Check: DOCKERHUB_USERNAME exists (value visible)
   ```

2. **Push test tag:**
   ```bash
   git tag v1.3.0-test
   git push origin v1.3.0-test
   ```

3. **Monitor workflow:**
   - GitHub Actions tab shows workflow triggered
   - All steps complete successfully (green checks)
   - Build time: 5-8 minutes expected

4. **Verify Docker Hub:**
   - Image: `<username>/tic-tac-toe:1.3.0-test` exists
   - Architectures: linux/amd64, linux/arm64 shown
   - Image size: ~26MB (matches local build)

5. **Pull and test:**
   ```bash
   docker pull <username>/tic-tac-toe:1.3.0-test
   docker run --rm -p 8080:80 <username>/tic-tac-toe:1.3.0-test
   # Browser: http://localhost:8080 loads game
   ```

## References

### Official Documentation

- [GitHub Actions: Publishing Docker images](https://docs.github.com/en/actions/publishing-packages/publishing-docker-images)
- [Docker: Multi-platform builds](https://docs.docker.com/build/building/multi-platform/)
- [Docker Hub: Personal Access Tokens](https://docs.docker.com/security/for-developers/access-tokens/)
- [Docker metadata-action: Tag patterns](https://github.com/docker/metadata-action#tags-input)

### Action Documentation

- [docker/setup-qemu-action](https://github.com/docker/setup-qemu-action)
- [docker/setup-buildx-action](https://github.com/docker/setup-buildx-action)
- [docker/login-action](https://github.com/docker/login-action)
- [docker/metadata-action](https://github.com/docker/metadata-action)
- [docker/build-push-action](https://github.com/docker/build-push-action)

### Project Files

- `.github/workflows/docker.yml` (64 lines) — Main workflow file
- `Dockerfile` (61 lines) — Multi-arch build configuration
- `README.md` — User documentation (needs CI/CD section)

---

## Research Summary

**Status:** ✅ Workflow already implemented with all Phase 11 requirements

**Key findings:**
1. `.github/workflows/docker.yml` exists and implements CICD-01 through CICD-04, DIST-01, and DIST-04
2. Workflow uses current best practices (Buildx, QEMU, GitHub Actions cache, semver tags)
3. Dockerfile has multi-arch support via `--platform=$BUILDPLATFORM`
4. Documentation gap: README doesn't explain how to trigger releases or configure secrets
5. Phase 11 is verification + documentation, not net-new implementation

**Planning guidance:**
- Primary task: Verify workflow works end-to-end with test tag push
- Secondary task: Document release process and secrets setup in README
- Tertiary task: Add troubleshooting section for common CI failures
- Out of scope: Phase 12 features (major tag, OCI labels)

---

*Phase: 11-github-actions-workflow*
*Research completed: 2026-04-25*
