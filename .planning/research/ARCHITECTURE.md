# Architecture Research — Docker Multi-Arch Deployment

**Domain:** Static SPA (Rust/WASM + Vite) → Docker multi-platform image served by nginx
**Milestone:** v1.2 Docker Deployment
**Researched:** 2026-04-14
**Confidence:** HIGH — all patterns verified from official Docker docs, GitHub Actions docs, nginx Docker Hub

---

## Existing Project Structure

```
tic-tac-toe/
├── Cargo.toml              # Rust crate — cdylib + rlib
├── Cargo.lock
├── package.json            # type: module; scripts: dev/build/preview
├── package-lock.json
├── vite.config.js          # wasm() plugin, build.target: 'esnext'
├── index.html              # SPA entry; refs /src/main.js, /src/style.css
├── src/                    # Rust + JS/CSS co-located
│   ├── lib.rs              # wasm_bindgen exports
│   ├── wasm_api.rs         # WASM boundary
│   ├── ai.rs
│   ├── board.rs
│   ├── main.js             # ~400 LOC — DOM, events, WASM calls
│   ├── audio.js            # Web Audio synthesizer module
│   └── style.css           # ~449 LOC — dark/light theme, animations
├── pkg/                    # ← wasm-pack output (GITIGNORED — built in Docker)
│   └── tic_tac_toe*.{js,wasm,d.ts}
├── dist/                   # ← Vite production build (GITIGNORED — built in Docker)
│   ├── index.html
│   └── assets/
│       ├── index-*.js
│       ├── index-*.css
│       └── tic_tac_toe_bg-*.wasm   # Vite copies WASM into assets/
├── target/                 # Rust build cache (GITIGNORED)
└── node_modules/           # (GITIGNORED)
```

**Key observation:** `src/` is flat — Rust files and JS/CSS files coexist in the same directory.  
The Vite `server.fs.allow: ['.']` config explicitly allows serving `pkg/` from the project root.

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Docker Build Context                        │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Stage 1: builder  (FROM rust:1-slim-bookworm)           │   │
│  │                                                          │   │
│  │  apt-get: curl, nodejs, npm                              │   │
│  │  cargo install wasm-pack                                 │   │
│  │  wasm-pack build --target web --release                  │   │
│  │    → pkg/   (tic_tac_toe.js, tic_tac_toe_bg.wasm, ...)  │   │
│  │  npm ci                                                  │   │
│  │  npm run build   (vite build)                            │   │
│  │    → dist/  (index.html, assets/*.{js,css,wasm})         │   │
│  └──────────────────┬───────────────────────────────────────┘   │
│                     │  COPY --from=builder /app/dist/           │
│  ┌──────────────────▼───────────────────────────────────────┐   │
│  │  Stage 2: runtime  (FROM nginx:alpine)                   │   │
│  │                                                          │   │
│  │  COPY nginx.conf → /etc/nginx/conf.d/default.conf        │   │
│  │  COPY dist/     → /usr/share/nginx/html/                 │   │
│  │  EXPOSE 80                                               │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Multi-platform output: linux/amd64 + linux/arm64              │
│  Via: docker buildx build --platform linux/amd64,linux/arm64   │
│       (QEMU emulation on amd64 runner for arm64 target)        │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Notes |
|-----------|----------------|-------|
| Stage 1: builder | Compile Rust→WASM; install JS deps; run Vite build | Rust + Node.js toolchain; discarded after build |
| Stage 2: runtime | Serve static files via nginx | ~50MB final image (nginx:alpine base ~25MB + assets) |
| nginx.conf | SPA routing (try_files), WASM MIME type, gzip | Custom config replaces nginx default |
| GitHub Actions workflow | Tag-triggered build + push to Docker Hub | Uses QEMU + buildx for multi-arch |
| .dockerignore | Exclude target/, node_modules/, pkg/, dist/ | Prevents cache-busting + reduces build context |

---

## Recommended Project Structure (New Files)

```
tic-tac-toe/
├── Dockerfile              # NEW — multi-stage build
├── .dockerignore           # NEW — build context exclusions
├── nginx.conf              # NEW — custom nginx server config
└── .github/
    └── workflows/
        └── docker-publish.yml   # NEW — GitHub Actions CI/CD
```

**All existing files remain unchanged.** No modifications to `Cargo.toml`, `vite.config.js`, `package.json`, `src/`, or `index.html`.

---

## Architectural Patterns

### Pattern 1: Multi-Stage Dockerfile

**What:** Two stages — `builder` (fat: Rust + Node.js) and `runtime` (lean: nginx:alpine). Only the `dist/` output is copied to runtime.

**When to use:** Any static site with a heavy build toolchain.

**Trade-offs:** Larger layer cache for builder stage (wasm-pack install is slow: ~3–5 min). Runtime image stays small (~50MB vs ~2GB builder).

**Recommended Dockerfile:**

```dockerfile
# syntax=docker/dockerfile:1

# ─── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install Node.js (LTS) and npm — needed for Vite build
# Install curl for wasm-pack installer
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    nodejs \
    npm \
  && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
# Using cargo install so the binary works for the builder's native arch
RUN cargo install wasm-pack

# Add WASM compilation target
RUN rustup target add wasm32-unknown-unknown

# Copy dependency manifests first (layer-cache optimization)
COPY Cargo.toml Cargo.lock ./
COPY package.json package-lock.json ./

# Install JS dependencies (before copying source — cache-friendly)
RUN npm ci

# Copy all source files
COPY src/ ./src/
COPY index.html vite.config.js ./

# Build WASM package (outputs to pkg/)
RUN wasm-pack build --target web --release

# Build frontend (Vite reads pkg/ via vite.config.js, outputs to dist/)
RUN npm run build

# ─── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM nginx:alpine AS runtime

# Custom nginx config: WASM MIME type + SPA routing
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy production build artifacts
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
```

**Why `rust:1-slim-bookworm` not `rust:alpine`:**
- wasm-pack's installer script and some crates need glibc (bookworm = Debian 12)
- Alpine uses musl libc — can cause subtle linking issues with wasm-pack
- `slim-bookworm` is the smaller Debian variant (~100MB vs 250MB full)
- HIGH confidence: this is the community-recommended base for Rust Docker builds

**Why not `--platform=$BUILDPLATFORM` cross-compilation:**
- Rust→WASM compilation always targets `wasm32-unknown-unknown` — completely architecture-independent. The WASM binary is the same regardless of build platform.
- The final artifact (HTML/CSS/JS/WASM) is architecture-neutral static content.
- nginx:alpine is the only architecture-specific component — and Docker's multi-arch manifest handles that automatically.
- QEMU emulation on the builder stage is fine because `wasm32-unknown-unknown` is compiled via cross-compilation anyway (not native arch).
- **Simpler:** no `ARG TARGETPLATFORM` / `ARG BUILDPLATFORM` needed.

---

### Pattern 2: nginx Configuration for SPA + WASM

**What:** nginx serving a Vite SPA with WASM MIME type and SPA fallback routing.

**When to use:** Any Vite-built SPA deployed via nginx.

**Required capabilities:**
1. `application/wasm` MIME type for `.wasm` files — **critical**: without this, some browsers refuse WASM compilation with `TypeError: Response has unsupported MIME type`
2. `try_files $uri $uri/ /index.html` — SPA routing fallback (for any client-side routes, though this app has only one route)
3. Gzip compression — Vite's production build is already minified; gzip further reduces transfer

**Recommended `nginx.conf`:**

```nginx
server {
    listen 80;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    # WASM MIME type — required for browser WASM compilation
    # nginx:alpine includes this in the default mime.types, but explicit is safer
    include /etc/nginx/mime.types;
    types {
        application/wasm wasm;
    }

    # SPA routing — serve index.html for all paths not matching a file
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets aggressively (Vite content-hashes filenames)
    location ~* \.(?:js|css|wasm|ico|png|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Gzip for text assets
    gzip on;
    gzip_types text/plain text/css application/javascript application/wasm;
    gzip_min_length 1024;

    # Security headers
    add_header X-Content-Type-Options "nosniff";
    add_header X-Frame-Options "DENY";
}
```

**Why `include /etc/nginx/mime.types` + explicit `types { application/wasm wasm; }` block:**
- The `nginx:alpine` image ships with `/etc/nginx/mime.types` which includes `application/wasm` as of nginx 1.25+
- The explicit `types` block inside `server {}` is additive — it merges with the included mime.types
- Being explicit prevents breakage if an older nginx base image is used

**WASM MIME type requirement — HIGH confidence:**
- The Fetch spec requires `Content-Type: application/wasm` for `WebAssembly.instantiateStreaming()` to work
- Without correct MIME type, browsers fall back to `WebAssembly.instantiate()` (non-streaming, slower) or throw a TypeError
- Source: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiateStreaming

---

### Pattern 3: docker buildx + QEMU for Multi-Arch

**What:** Build `linux/amd64` and `linux/arm64` images from a single amd64 runner using QEMU CPU emulation.

**When to use:** Multi-arch image publishing when native multi-arch runners are unavailable or unneeded.

**How QEMU emulation works:**
1. `docker/setup-qemu-action` installs QEMU binaries into the Docker daemon's VM via `tonistiigi/binfmt --install all`
2. This registers QEMU as a `binfmt_misc` binary interpreter for non-native ELF formats
3. When `docker buildx` builds for `linux/arm64` on an `amd64` runner, arm64 binaries are transparently executed through QEMU
4. The `docker-container` buildx driver creates an isolated BuildKit container with QEMU support

**Performance note:** QEMU emulation is 3–10× slower than native. For this project:
- The `wasm-pack build` step compiles Rust to `wasm32-unknown-unknown` — this is a cross-compile regardless of host arch, so QEMU doesn't slow it down
- The `npm run build` (Vite) is pure JS — QEMU doesn't affect it on arm64
- Only `cargo install wasm-pack` (which installs a native binary for the target arch) is emulated — this is the slow step (~5 min under QEMU on arm64)
- **Acceptable:** The total arm64 build time is ~8–12 min; amd64 is ~3–5 min. For a tag-triggered workflow, this is fine.

---

### Pattern 4: GitHub Actions Tag-Triggered Workflow

**What:** CI/CD workflow that builds and pushes the multi-arch image to Docker Hub only on git tag pushes matching `v*.*.*`.

**When to use:** Release-gated deployments. Tags trigger Docker Hub pushes; branch pushes do not.

**Recommended `.github/workflows/docker-publish.yml`:**

```yaml
name: Publish Docker image

on:
  push:
    tags:
      - "v*.*.*"

jobs:
  build-and-push:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v4

      - name: Log in to Docker Hub
        uses: docker/login-action@v4
        with:
          username: ${{ vars.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Extract metadata (tags, labels)
        id: meta
        uses: docker/metadata-action@v6
        with:
          images: ${{ vars.DOCKERHUB_USERNAME }}/tic-tac-toe
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}
            type=raw,value=latest

      - name: Build and push multi-arch image
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

**Secrets and variables needed:**

| Secret/Variable | Type | Value | Purpose |
|-----------------|------|-------|---------|
| `DOCKERHUB_USERNAME` | Repository Variable (`vars.*`) | Docker Hub username | Image name prefix and login |
| `DOCKERHUB_TOKEN` | Repository Secret (`secrets.*`) | Docker Hub access token | Authenticates pushes (not account password) |

**Why use a Docker Hub access token, not account password:**
- Docker Hub access tokens are scoped (read/write), revocable, and don't expose account credentials
- Create at: hub.docker.com → Account Settings → Security → New Access Token

**Tag output from `docker/metadata-action`:**
Given tag `v1.2.0`, the action generates:
- `username/tic-tac-toe:1.2.0`
- `username/tic-tac-toe:1.2`
- `username/tic-tac-toe:1`
- `username/tic-tac-toe:latest`

**GitHub Actions cache (`cache-from/cache-to: type=gha`):**
- Caches BuildKit layers in GitHub Actions cache
- Subsequent tag pushes reuse the `npm ci` and wasm-pack install layers if Cargo.lock/package-lock.json are unchanged
- Reduces build time from ~10 min to ~3 min on cache hit

---

## Data Flow: Build Pipeline

```
git tag push v1.2.0
    ↓
GitHub Actions: ubuntu-latest runner
    ↓
actions/checkout@v4
    → repository at GITHUB_WORKSPACE
    ↓
docker/setup-qemu-action@v4
    → QEMU binfmt_misc registered for arm64, riscv64, etc.
    ↓
docker/setup-buildx-action@v4
    → docker-container driver buildx builder created
    ↓
docker/login-action@v4
    → authenticated to docker.io/DOCKERHUB_USERNAME
    ↓
docker/metadata-action@v6
    → tags: ["user/tic-tac-toe:1.2.0", "...:1.2", "...:1", "...:latest"]
    ↓
docker/build-push-action@v7 (platforms: linux/amd64,linux/arm64)
    ↓
  ┌─ amd64 build ───────────────────────────────────┐
  │  Stage 1 (builder / native amd64):              │
  │    apt-get: curl, nodejs, npm                   │
  │    cargo install wasm-pack          ~2 min      │
  │    rustup target add wasm32         ~10s        │
  │    npm ci                           ~30s        │
  │    wasm-pack build --release        ~90s        │
  │    npm run build (vite)             ~10s        │
  │  Stage 2 (runtime / nginx:alpine amd64):        │
  │    COPY nginx.conf + dist/          ~1s         │
  └─────────────────────────────────────────────────┘
  ┌─ arm64 build ───────────────────────────────────┐
  │  Stage 1 (builder / arm64 via QEMU):            │
  │    apt-get: curl, nodejs, npm       ~3 min      │
  │    cargo install wasm-pack          ~8 min (QEMU slow)│
  │    wasm-pack build --release        ~3 min (cross-compile, same output)│
  │    npm run build (vite)             ~15s        │
  │  Stage 2 (runtime / nginx:alpine arm64):        │
  │    COPY nginx.conf + dist/          ~1s         │
  └─────────────────────────────────────────────────┘
    ↓
  Multi-arch manifest list pushed to Docker Hub
  docker.io/user/tic-tac-toe:1.2.0 → {amd64: sha256:..., arm64: sha256:...}
```

**Key insight:** The `dist/` output (HTML/CSS/JS/WASM) is **identical** for both architectures — WASM is arch-neutral and Vite produces pure JS. Only the nginx binary in the runtime stage differs. Both builds run in parallel by buildx.

---

## File Details: `.dockerignore`

```
# Rust build cache — large (can be gigabytes), not needed in build context
target/

# Node.js dependencies — will be reinstalled by npm ci
node_modules/

# Generated by wasm-pack — will be regenerated by wasm-pack build
pkg/

# Vite production build — will be regenerated by npm run build
dist/

# Git history — not needed
.git/
.gitignore

# Planning and documentation
.planning/
AGENTS.md

# macOS artifacts
.DS_Store

# Editor config
.vscode/
.idea/
```

**Why `target/` is critical to exclude:** Rust's build cache (`target/`) can exceed 1–3GB. Including it bloats the build context transfer from host to Docker daemon significantly, adding minutes to every `docker build` invocation.

---

## Integration Points

### New Files to Create

| File | New/Modified | Description |
|------|-------------|-------------|
| `Dockerfile` | **NEW** | Multi-stage: `rust:1-slim-bookworm` builder → `nginx:alpine` runtime |
| `.dockerignore` | **NEW** | Excludes target/, node_modules/, pkg/, dist/, .git/ |
| `nginx.conf` | **NEW** | SPA routing, WASM MIME type, gzip, 1y cache headers |
| `.github/workflows/docker-publish.yml` | **NEW** | Tag-triggered build+push to Docker Hub |

### Unchanged Files

| File | Why Unchanged |
|------|--------------|
| `Cargo.toml` | No new Rust dependencies |
| `vite.config.js` | Vite config is already correct for production builds |
| `package.json` | `npm run build` script already calls `vite build` |
| `src/*.{rs,js,css}` | Game logic and frontend untouched |
| `index.html` | HTML entry point unchanged |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Dockerfile Stage 1 → Stage 2 | `COPY --from=builder /app/dist /usr/share/nginx/html` | Only `dist/` crosses the stage boundary |
| wasm-pack → Vite | `pkg/` directory at project root | Vite imports `./pkg/tic_tac_toe.js` via `src/main.js`; `vite build` bundles it |
| GitHub Actions → Docker Hub | `docker/build-push-action` with `push: true` | Authenticated via `DOCKERHUB_TOKEN` secret |

---

## Build Order

```
Stage 1 (builder):
  1. apt-get install curl nodejs npm       # System deps
  2. cargo install wasm-pack               # WASM toolchain
  3. rustup target add wasm32-unknown-unknown
  4. npm ci                                # JS deps (package-lock.json)
  5. COPY src/ index.html vite.config.js   # Source files
  6. wasm-pack build --target web --release → pkg/
  7. npm run build                         → dist/

Stage 2 (runtime):
  8. COPY nginx.conf
  9. COPY --from=builder /app/dist/ → /usr/share/nginx/html/
```

**Why this order matters:**
- Steps 1–3 are toolchain setup — cached until base image changes
- Step 4 (`npm ci`) is cached until `package-lock.json` changes
- Steps 5–7 invalidate cache on any source file change (intentional)
- wasm-pack (step 6) **must** run before Vite (step 7) — Vite's `vite-plugin-wasm` imports `pkg/tic_tac_toe.js`; if `pkg/` doesn't exist, `npm run build` fails with module resolution error

**Cache optimization note:** Separating `COPY Cargo.toml Cargo.lock ./` and `COPY package.json package-lock.json ./` before `npm ci` enables caching JS deps independently of source changes. However, Rust dependency pre-caching (via `cargo fetch` or dummy `lib.rs`) is complex and not worth adding — wasm-pack already builds incrementally with the `target/` cache during local dev; in Docker, `target/` is rebuilt fresh each time (or restored via GHA cache).

---

## Scaling Considerations

This is a static site. Scaling is handled entirely at the infrastructure level, not the application level.

| Scale | Approach |
|-------|----------|
| Single server | `docker run -p 80:80 user/tic-tac-toe:latest` — nginx handles thousands of concurrent static file requests |
| Multi-server | Place nginx behind a load balancer; no sticky sessions needed (stateless SPA) |
| CDN | Put CloudFlare/CloudFront in front; the Docker image serves as origin |
| Kubernetes | Standard nginx deployment — Deployment + Service + Ingress |

---

## Anti-Patterns

### Anti-Pattern 1: Missing `application/wasm` MIME Type

**What people do:** Use `nginx:alpine` with its default config and no custom `nginx.conf`.

**Why it's wrong:** nginx's default `mime.types` may not include `application/wasm` on older versions. Browsers enforce MIME type checking for WASM modules loaded via `WebAssembly.instantiateStreaming()`. A missing or incorrect MIME type produces: `TypeError: Failed to execute 'compile' on 'WebAssembly': Incorrect response MIME type. Expected 'application/wasm'.`

**Do this instead:** Always include explicit `types { application/wasm wasm; }` in `nginx.conf`.

---

### Anti-Pattern 2: Copying All Source Into Runtime Stage

**What people do:** Single-stage Dockerfile where Rust, npm, and source files all end up in the final image.

**Why it's wrong:** Final image is ~2GB (full Rust toolchain + node_modules). Takes 5–10 minutes to pull on first `docker run`. Vastly increases attack surface.

**Do this instead:** Multi-stage build. Final image is nginx:alpine + static files only (~50MB).

---

### Anti-Pattern 3: Running `wasm-pack` After `npm ci` Without Copying Source

**What people do:** `COPY . .` at the top of the Dockerfile (before `npm ci`) to ensure all files are present.

**Why it's wrong:** Any change to any source file invalidates the Docker layer cache at the `COPY . .` point, causing `npm ci` to re-run even if `package-lock.json` hasn't changed. For a project with 200+ npm dependencies, this adds 30–90 seconds per build.

**Do this instead:** Structured copy order:
1. `COPY Cargo.toml Cargo.lock ./` (Rust manifest)
2. `COPY package.json package-lock.json ./` (JS manifest)
3. `RUN npm ci` (cached until package-lock.json changes)
4. `COPY src/ index.html vite.config.js ./` (source — invalidates at this point)
5. `RUN wasm-pack build && npm run build` (always re-runs on source change)

---

### Anti-Pattern 4: Using `--no-cache` in wasm-pack build

**What people do:** `wasm-pack build --no-cache --target web --release` to force a clean build.

**Why it's wrong:** wasm-pack's `--no-cache` flag clears the wasm-bindgen download cache. In a fresh Docker layer, the cache is already empty. The flag adds no correctness benefit and downloads wasm-bindgen-cli again unnecessarily (~30s).

**Do this instead:** `wasm-pack build --target web --release` — no `--no-cache` flag.

---

### Anti-Pattern 5: Hardcoding Docker Hub Username in Workflow

**What people do:** `tags: myusername/tic-tac-toe:latest` directly in the YAML file.

**Why it's wrong:** Username is committed to public source; harder to fork and reuse; requires changing the YAML to rename the image.

**Do this instead:** Use `${{ vars.DOCKERHUB_USERNAME }}` (repository variable, not secret) — readable in logs, not sensitive, easy to configure per-repo.

---

## Sources

- Docker official docs: Multi-platform builds — https://docs.docker.com/build/building/multi-platform/ (HIGH confidence)
- Docker official docs: GitHub Actions multi-platform — https://docs.docker.com/build/ci/github-actions/multi-platform/ (HIGH confidence)
- Docker official docs: Manage tags and labels — https://docs.docker.com/build/ci/github-actions/manage-tags-labels/ (HIGH confidence)
- Docker Hub: nginx official image — https://hub.docker.com/_/nginx — nginx 1.29.8-alpine, architectures confirmed: amd64, arm64v8 (HIGH confidence)
- nginx docs: `ngx_http_core_module` types directive — https://nginx.org/en/docs/http/ngx_http_core_module.html#types (HIGH confidence)
- Direct project inspection: `vite.config.js`, `package.json`, `Cargo.toml`, `src/` structure (HIGH confidence — primary source)

---
*Architecture research for: Docker multi-arch deployment of Rust/WASM + Vite SPA*
*Researched: 2026-04-14*
