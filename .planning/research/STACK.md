# Stack Research

**Domain:** Docker multi-arch deployment — Rust/WASM + Vite static site
**Researched:** 2026-04-14
**Confidence:** HIGH — all versions verified against Docker Hub and GitHub Releases

---

## Context

This is a **subsequent milestone** stack addendum. The core Rust/WASM/Vite stack is already validated (see prior research). This document covers only the **new additions** required for Docker multi-architecture deployment:

- Multi-stage Dockerfile (Rust/wasm-pack build → nginx serve)
- Multi-arch manifest (linux/amd64 + linux/arm64) via docker buildx + QEMU
- GitHub Actions CI triggered on git tag push, publishing to Docker Hub

---

## Key Architectural Insight: WASM Is Architecture-Neutral

**This is the most important fact for the Dockerfile design.**

WebAssembly `.wasm` binaries are not compiled to a native ISA — they are portable bytecode. A `.wasm` file built on `linux/amd64` runs identically on `linux/arm64` because the browser's WASM runtime handles the native translation at runtime (confirmed: https://webassembly.org/docs/portability/).

**Consequence:** The Rust/wasm-pack compilation step does NOT need to run twice (once per target arch). It only needs to run once on the host architecture (`linux/amd64`, which is the GitHub Actions runner). The final artifact is a pure-static `dist/` directory of HTML + CSS + JS + `.wasm` — all architecture-neutral files.

**This means the build stage does NOT need cross-compilation.** Only the `COPY` into nginx needs to produce arch-specific nginx binaries. Docker buildx handles that automatically by pulling the correct nginx binary for each target platform.

---

## Recommended Stack

### Docker Base Images

| Image | Tag | Purpose | Why Recommended |
|-------|-----|---------|-----------------|
| `rust` | `1.94.1-slim` | Rust/wasm-pack build stage | Official Rust Docker image, `slim` variant removes docs/examples but keeps cargo and glibc. Matches the project's validated Rust version. Multi-arch on Docker Hub (amd64, arm64, etc.) but build runs native on amd64. |
| `node` | `22-alpine3.23` | npm/Vite build sub-stage | Node 22 LTS (current LTS as of 2026-04), Alpine keeps image small (~54 MB compressed). Used to run `npm ci && npm run build`. Supports linux/amd64 and linux/arm64. |
| `nginx` | `1.29.8-alpine3.23` | Production serve stage | Current stable nginx (1.29.8) on Alpine 3.23. ~24 MB compressed. Supports linux/amd64, linux/arm64, linux/arm/v7, linux/386, linux/s390x, linux/ppc64le — all from a single image tag. Confirmed on Docker Hub 2026-04-14. |

**Tag pinning rationale:**
- `rust:1.94.1-slim` — fully pinned to match project's validated Rust version; prevents silent toolchain drift
- `node:22-alpine3.23` — LTS major + specific Alpine patch; `22-alpine` (floating to latest LTS patch) is also acceptable
- `nginx:1.29.8-alpine3.23` — fully pinned for deterministic deployments; `nginx:alpine` (floating to latest stable) is acceptable for this project's scale

### GitHub Actions Actions

| Action | Version | Purpose | Why |
|--------|---------|---------|-----|
| `docker/setup-qemu-action` | `v4` | Install QEMU binfmt handlers on runner | Required to emulate `linux/arm64` on `linux/amd64` GitHub runners. v4.0.0 released 2026-03-04 (Node 24 runtime). Latest. |
| `docker/setup-buildx-action` | `v4` | Create and configure BuildKit builder | Activates `docker buildx` with multi-platform support. v4.0.0 released 2026-03-05 (Node 24 runtime). Latest. |
| `docker/login-action` | `v4` | Authenticate to Docker Hub | v4.1.0 released 2026-04-02. Latest. Node 24 runtime. |
| `docker/build-push-action` | `v7` | Build multi-arch image and push manifest | v7.1.0 released 2026-04-10. Latest. Handles `platforms: linux/amd64,linux/arm64`, creates the multi-arch manifest automatically. |
| `docker/metadata-action` | `v6` | Generate Docker tags from git refs | v6.0.0 released 2026-03-05. Latest. Converts git tag `v1.2.0` → Docker Hub tags `1.2.0`, `1.2`, `1`, `latest` via semver rules. |

**Version pinning strategy:** Use major version tags (`v4`, `v7`) — the Docker team maintains backward compatibility within major versions and applies security patches. All four core Docker actions (`setup-qemu`, `setup-buildx`, `login`, `build-push`) were updated together in March 2026 to Node 24 runtime (required Actions Runner ≥ v2.327.1, which `ubuntu-latest` satisfies).

### Development Tools (local cross-arch testing)

| Tool | Purpose | Notes |
|------|---------|-------|
| Docker Desktop (Mac/Linux) | Bundles buildx and QEMU; enables `docker buildx build --platform linux/arm64` locally | Verify with `docker buildx inspect` |
| `docker buildx ls` | List available builders and their platform support | Run after `setup-buildx-action` in CI to debug platform issues |

---

## Dockerfile Structure

The correct multi-stage Dockerfile for this project:

```dockerfile
# ── Stage 1: Rust build (wasm-pack) ─────────────────────────────────────
# Runs on the builder's native arch (linux/amd64 on GitHub Actions).
# Output: pkg/*.wasm + pkg/*.js — both arch-neutral.
FROM --platform=$BUILDPLATFORM rust:1.94.1-slim AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack (also installs wasm32-unknown-unknown target)
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# --release: wasm-opt runs automatically; --target web: ES module output
RUN wasm-pack build --target web --release

# ── Stage 2: npm/Vite build ──────────────────────────────────────────────
# Also arch-neutral: produces dist/ of HTML + CSS + JS + .wasm
FROM --platform=$BUILDPLATFORM node:22-alpine3.23 AS node-builder

WORKDIR /app
COPY package.json package-lock.json vite.config.js index.html ./
COPY src/ ./src/
COPY --from=rust-builder /app/pkg ./pkg

RUN npm ci && npm run build

# ── Stage 3: nginx serve ─────────────────────────────────────────────────
# THIS stage instantiates for each target platform.
# BuildKit pulls nginx:1.29.8-alpine3.23 for linux/amd64 AND linux/arm64.
# The dist/ content is identical for both — only nginx binary differs.
FROM nginx:1.29.8-alpine3.23

COPY --from=node-builder /app/dist /usr/share/nginx/html

EXPOSE 80
```

**Why `--platform=$BUILDPLATFORM` on stages 1 and 2:**
- Forces these expensive stages to always run on the builder's native arch (amd64), even when building for arm64
- Without it, BuildKit would emulate them via QEMU for the arm64 target — making a 3-minute Rust compile take 15-20 minutes
- Since WASM output is arch-neutral, running stages 1 and 2 natively on amd64 is always correct

**Why `rust:1.94.1-slim` (Debian) not `rust:alpine` (musl):**
wasm-pack downloads pre-built `wasm-bindgen-cli` binaries that link against glibc. On Alpine (musl libc), these binaries fail with dynamic linker errors. Debian slim uses glibc — the standard ABI these binaries target. Build stage image size doesn't matter since it's discarded.

---

## GitHub Actions Workflow

```yaml
name: Publish Docker Image

on:
  push:
    tags:
      - 'v*'              # Triggers on v1.2.0, v1.2.3, etc.

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Docker meta
        id: meta
        uses: docker/metadata-action@v6
        with:
          images: ${{ vars.DOCKERHUB_USERNAME }}/tic-tac-toe
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v4

      - name: Login to Docker Hub
        uses: docker/login-action@v4
        with:
          username: ${{ vars.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v7
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**Key workflow choices explained:**
- `tags: 'v*'` — triggers only on version tags, not every commit push; no accidental publishes
- `docker/metadata-action` — `v1.2.3` tag → Docker Hub publishes `1.2.3`, `1.2`, `1` (and `latest` via the major pattern)
- `cache-from/cache-to: type=gha` — GitHub Actions cache for BuildKit layers; Rust compilation takes 2–5 min cold; with layer cache subsequent builds are ~30s for CSS/JS changes
- `push: true` — multi-arch manifests can only be pushed to a registry, not loaded locally (without containerd image store setup)
- `${{ vars.DOCKERHUB_USERNAME }}` — use GitHub Actions variable (not secret) for the username; `${{ secrets.DOCKERHUB_TOKEN }}` for the access token

---

## WASM-Specific Cross-Compilation Notes

### What requires NO cross-compilation

| Artifact | Reason |
|----------|--------|
| `.wasm` binary | Architecture-neutral bytecode — runs on any WASM runtime |
| wasm-bindgen JS glue (`pkg/*.js`) | Pure JavaScript |
| Vite `dist/` output | HTML, CSS, JS, `.wasm` — all arch-neutral |

### What requires per-arch binaries (handled by BuildKit automatically)

| Artifact | How handled |
|----------|-------------|
| nginx server binary | BuildKit pulls `nginx:1.29.8-alpine3.23` manifest for the correct target arch |

### wasm-opt availability in Docker

`wasm-pack --release` runs `wasm-opt` automatically. The `rust:1.94.1-slim` base image (Debian Bookworm) has the necessary system libraries. If wasm-opt fails in CI (it does on some stripped environments), add `--no-opt` to the wasm-pack invocation — the binary will be ~10–30% larger but functionally identical.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Rust build base | `rust:1.94.1-slim` (Debian/glibc) | `rust:alpine` (musl) | wasm-bindgen-cli prebuilt binaries require glibc; fails on musl without source compilation |
| WASM cross-compilation | Single native build (amd64), arch-neutral WASM output | Compile Rust targeting aarch64 natively | Unnecessary complexity — WASM output is not ISA-specific |
| Vite build location | Separate Node stage in Dockerfile | Build `dist/` outside Docker, COPY in | Multi-stage keeps CI reproducible; no local artifacts to manage |
| nginx version | `nginx:1.29.8-alpine3.23` (mainline pinned) | `nginx:stable-alpine` (1.26.x floating) | Mainline (1.29.x) has all current security patches; `stable` is older |
| Multi-arch strategy | QEMU emulation on single amd64 runner | Native matrix (separate arm64 runner per platform) | QEMU is simpler for a static site. For compiled server apps with long builds, native runners win. |
| Tag trigger | `tags: 'v*'` | `push: branches: [main]` | Deploy on explicit version tags only; no accidental latest-tag updates on every commit |
| Caching | `cache-from: type=gha` | `cache-from: type=registry` | GitHub Actions cache is zero-config and free; registry cache requires an additional image push |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `FROM rust:alpine` as build stage | wasm-bindgen-cli prebuilt binary targets glibc; hits "No such file or directory" on musl Alpine | `FROM rust:1.94.1-slim` (Debian slim) |
| `--platform linux/arm64` on Rust/Node build stages | Forces QEMU-emulated Rust compilation: 15–20 min vs 3 min native. WASM output is arch-neutral — no benefit. | `--platform=$BUILDPLATFORM` on build stages |
| Building without layer cache | Cold `cargo build` takes 2–5 minutes; without cache, every CI run pays full cost | `cache-from: type=gha` + `cache-to: type=gha,mode=max` |
| `latest` as the only Docker Hub tag | Makes rollback impossible; users can't pin to a version | `docker/metadata-action` to also publish `1.2.3`, `1.2`, `1` tags |
| `docker/build-push-action@v6` or earlier | v6 uses Node 20 runtime; v7 is current (Node 24). All Docker actions updated together in March 2026. | Use `v7` for `build-push-action`, `v4` for the others |
| Hardcoding Docker Hub password in workflow | Leaks credentials | `${{ secrets.DOCKERHUB_TOKEN }}` — use a scoped Docker Hub Access Token, not the account password |
| `COPY . .` as first Dockerfile instruction | Invalidates build cache on every source change; Rust recompiles everything | Copy `Cargo.toml`/`Cargo.lock` first, then `src/` — allows cargo dependency layer caching |

---

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| docker/setup-qemu-action@v4 | docker/setup-buildx-action@v4, docker/build-push-action@v7 | All use @docker/actions-toolkit 0.77–0.87 range; released together March 2026 |
| docker/login-action@v4 | docker/build-push-action@v7 | Same actions-toolkit version family; v4.1.0 is latest |
| docker/metadata-action@v6 | docker/build-push-action@v7 | Metadata `outputs.tags` and `outputs.labels` consumed directly as build-push inputs |
| `rust:1.94.1-slim` | wasm-pack 0.14.0 | wasm-pack installer script works on Debian Bookworm (glibc 2.36). Confirmed. |
| `node:22-alpine3.23` | Vite 8.x, npm 10.x | Node 22 LTS is fully compatible with Vite 8. npm comes bundled. |
| `nginx:1.29.8-alpine3.23` | linux/amd64, linux/arm64, linux/arm/v7, linux/386, linux/s390x, linux/ppc64le | Multi-arch manifest verified on Docker Hub 2026-04-14 |
| `docker/build-push-action@v7` | Actions Runner ≥ v2.327.1 | Node 24 runtime requirement. `ubuntu-latest` on GitHub Actions satisfies this. |

---

## Sources

- Docker Hub `nginx` tags (live, 2026-04-14) — nginx 1.29.8-alpine3.23 confirmed multi-arch (HIGH confidence)
- Docker Hub `rust` tags (live, 2026-04-14) — rust:1.94.1-slim confirmed amd64+arm64 (HIGH confidence)
- Docker Hub `node` tags (live, 2026-04-14) — node:22-alpine3.23 LTS confirmed multi-arch (HIGH confidence)
- GitHub Releases: docker/setup-buildx-action v4.0.0 (2026-03-05) — https://github.com/docker/setup-buildx-action/releases (HIGH confidence)
- GitHub Releases: docker/build-push-action v7.1.0 (2026-04-10) — https://github.com/docker/build-push-action/releases (HIGH confidence)
- GitHub Releases: docker/login-action v4.1.0 (2026-04-02) — https://github.com/docker/login-action/releases (HIGH confidence)
- GitHub Releases: docker/setup-qemu-action v4.0.0 (2026-03-04) — https://github.com/docker/setup-qemu-action/releases (HIGH confidence)
- GitHub Releases: docker/metadata-action v6.0.0 (2026-03-05) — https://github.com/docker/metadata-action/releases (HIGH confidence)
- Docker official docs — Multi-platform images with GitHub Actions — https://docs.docker.com/build/ci/github-actions/multi-platform/ (HIGH confidence)
- WebAssembly.org — Portability spec — https://webassembly.org/docs/portability/ (HIGH confidence) — confirms WASM is arch-neutral bytecode

---

*Stack research for: Docker multi-arch deployment, Rust/WASM + Vite static site*
*Researched: 2026-04-14*
