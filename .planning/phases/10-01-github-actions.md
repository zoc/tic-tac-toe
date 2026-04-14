# Plan 10-01: GitHub Actions CI Workflow

**Phase:** 10 — GitHub Actions
**Goal:** Publish a multi-arch Docker image to Docker Hub on every git version tag
**Status:** complete

## What We're Building

A GitHub Actions workflow that:
1. Triggers on `push` to `v*.*.*` tags only (no branch pushes)
2. Builds `linux/amd64` + `linux/arm64` using buildx (with `--platform=$BUILDPLATFORM` in Dockerfile)
3. Tags using `docker/metadata-action` — semver + `latest`
4. Pushes to Docker Hub with access token (not password)
5. Uses `type=gha` cache to avoid re-downloading all Rust/npm deps on every run

## Requirements

- [x] `.github/workflows/docker.yml` — trigger on `v*.*.*` tags
- [x] Uses `docker/setup-qemu-action` + `docker/setup-buildx-action`
- [x] Uses `docker/metadata-action` — semver tags + latest
- [x] Uses `docker/login-action` with `vars.DOCKERHUB_USERNAME` + `secrets.DOCKERHUB_TOKEN`
- [x] Uses `docker/build-push-action` with `push: true`, both platforms, GHA cache
- [x] Setup instructions documented in README and workflow comments

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/docker.yml` | CI workflow definition |

## Setup Required (human action)

Before the workflow can push to Docker Hub:
1. Create a Docker Hub access token: Hub → Account Settings → Security → New Access Token
   - Description: `tic-tac-toe-github-actions`
   - Access permissions: Read & Write
2. In GitHub repo → Settings → Secrets and variables → Actions:
   - **Secret:** `DOCKERHUB_TOKEN` = (the access token value)
   - **Variable:** `DOCKERHUB_USERNAME` = (your Docker Hub username)

## Key Decisions (from PITFALLS.md)

- Trigger on `tags: v*.*.*` only — no Docker push on regular commits
- `vars.DOCKERHUB_USERNAME` — username is public, not a secret
- `secrets.DOCKERHUB_TOKEN` — access token, not account password
- `cache-from: type=gha` + `cache-to: type=gha,mode=max` — full layer cache
- `platforms: linux/amd64,linux/arm64` — Dockerfile handles $BUILDPLATFORM optimization
- `docker/metadata-action` semver pattern: `{{version}}`, `{{major}}.{{minor}}`, `latest`

## Verification Steps

1. Commit all files + create tag: `git tag v1.2.0 && git push --tags`
2. Watch GitHub Actions → the `docker` workflow should trigger (not any branch push)
3. Verify both platforms in manifest: `docker manifest inspect <user>/tic-tac-toe:latest`
4. Pull and run: `docker run --rm -p 8080:80 <user>/tic-tac-toe:1.2.0`
5. Verify arm64 image: `docker run --rm --platform linux/arm64 <user>/tic-tac-toe:latest uname -m` → `aarch64`
6. Push a regular commit to `main` — confirm no Docker workflow runs
