# Plan 09-01: Multi-Stage Dockerfile + nginx Configuration

**Phase:** 09 — Docker & nginx
**Goal:** Package the game as a multi-arch-ready Docker image served by nginx
**Status:** complete

## What We're Building

A production-ready multi-stage Dockerfile that:
1. Compiles Rust → WASM → Vite production bundle (build stage, `--platform=$BUILDPLATFORM`)
2. Serves the static `dist/` output via nginx:alpine (serve stage, minimal ~10MB image)

Supporting files:
- `.dockerignore` — excludes `target/`, `node_modules/`, `pkg/`, `dist/` from build context
- `nginx.conf` — correct MIME type, `try_files`, security headers, no default nginx cruft

## Requirements

- [x] `Dockerfile` — multi-stage, `--platform=$BUILDPLATFORM` on build stage
- [x] `.dockerignore` — covers Rust + Node dual ecosystem
- [x] `nginx.conf` — `include mime.types`, `try_files`, security headers
- [x] Local smoke test: `docker buildx build --platform linux/amd64 --load -t tic-tac-toe:test .`
- [x] Image size: arm64 native = 24.7MB (actual bytes via inspect); amd64 = ~26MB reported
- [x] WASM MIME type: `Content-Type: application/wasm` confirmed via curl
- [ ] Game works: browser opens `http://localhost:8080`, game plays correctly (manual verification pending)

## Files to Create

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build |
| `.dockerignore` | Build context exclusions |
| `nginx.conf` | nginx server configuration |

## Key Decisions (from PITFALLS.md)

- `FROM --platform=$BUILDPLATFORM rust:slim AS build` — avoids QEMU for Rust compile
- `wasm-pack --target web --release` output → `pkg/` → Vite reads it
- Node.js installed in build stage only — `apt-get install -y curl nodejs npm`
- `wasm-pack` installed via curl installer → `ENV PATH="/root/.cargo/bin:${PATH}"`
- `COPY Cargo.toml Cargo.lock ./` before source — reproducible + Docker layer caching
- `COPY package.json package-lock.json ./` + `npm ci` before COPY . — npm layer caching
- Serve stage: `FROM nginx:alpine` + `COPY --from=build /app/dist /usr/share/nginx/html`
- nginx: `include /etc/nginx/mime.types;` present — ensures `application/wasm` MIME type
- Security headers: `X-Content-Type-Options`, `Cross-Origin-Opener-Policy`, `Cross-Origin-Embedder-Policy`

## Verification Steps

1. Build for amd64: `docker buildx build --platform linux/amd64 --load -t tic-tac-toe:test .`
2. Run container: `docker run --rm -p 8080:80 tic-tac-toe:test`
3. Check image size: `docker image ls tic-tac-toe:test`
4. Check WASM MIME: `curl -s -I "http://localhost:8080/assets/$(ls dist/assets/*.wasm | xargs basename)" | grep content-type`
5. Browser: open `http://localhost:8080` and play a game
6. Check no build artifacts in container: `docker run --rm tic-tac-toe:test ls /usr/share/nginx/html`
