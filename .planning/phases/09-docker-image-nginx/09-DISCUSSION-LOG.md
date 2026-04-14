# Phase 9: Docker Image & nginx - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 09-docker-image-nginx
**Areas discussed:** HEALTHCHECK design, gzip scope, wasm-pack pinning, Node.js version

---

## HEALTHCHECK design

| Option | Description | Selected |
|--------|-------------|----------|
| curl localhost/ (HTTP 200) | curl -f http://localhost/ — checks actual HTTP 200 from nginx. Standard for static sites. | |
| Dedicated /healthz endpoint | curl -f http://localhost/healthz with a dedicated nginx location block that returns 200 OK quickly, no HTML response body | ✓ |
| wget localhost/ (no curl needed) | wget -q -O /dev/null http://localhost/ — same as curl but uses wget, already in nginx:alpine base | |

**User's choice:** Dedicated `/healthz` endpoint

---

| Option | Description | Selected |
|--------|-------------|----------|
| Default timing (30s/3s/3/5s) | 30s interval, 3s timeout, 3 retries, 5s start period. The Docker default interval. | ✓ |
| Faster interval (10s/3s/3/10s) | 10s interval, 3s timeout, 3 retries, 10s start period. More responsive health signaling. | |

**User's choice:** Default timing (30s/3s/3/5s)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Install curl in serve stage | RUN apk add --no-cache curl to the serve stage | ✓ |
| Use wget instead | wget already in nginx:alpine — no additional package install | |

**User's choice:** Install curl in serve stage

---

## gzip scope

| Option | Description | Selected |
|--------|-------------|----------|
| HTML/CSS/JS only (skip WASM) | gzip on for HTML, CSS, JS only. WASM already compressed by wasm-opt. | ✓ |
| All four including WASM | gzip on for HTML, CSS, JS, and WASM. Simple config but WASM may already be compressed. | |
| Pre-compress in build stage (gzip_static) | Use gzip_static on to serve pre-gzipped files. More efficient at runtime, zero CPU at request time. | |

**User's choice:** HTML/CSS/JS only (skip WASM)

---

| Option | Description | Selected |
|--------|-------------|----------|
| 1KB minimum | gzip_min_length 1000 — don't bother compressing tiny files. A common sensible default. | ✓ |
| No minimum | Compress everything regardless of size | |
| 256 bytes minimum | gzip_min_length 256 — more aggressive compression | |

**User's choice:** 1KB minimum

---

## wasm-pack pinning

| Option | Description | Selected |
|--------|-------------|----------|
| cargo install --locked (pin 0.14.0) | Replace curl\|sh installer with: cargo install wasm-pack@0.14.0 --locked. Pins exact version. | ✓ |
| Keep curl\|sh (no change) | Keep curl\|sh pattern as-is. Acceptable risk for a personal project. | |
| Pin GitHub release URL | curl -L https://github.com/rustwasm/wasm-pack/releases/download/v0.14.0/wasm-pack-init.sh -sSf \| sh | |

**User's choice:** `cargo install wasm-pack@0.14.0 --locked`

---

## Node.js version in Dockerfile

| Option | Description | Selected |
|--------|-------------|----------|
| Install Node 20 via NodeSource | NodeSource setup_20.x script to install Node 20 LTS explicitly. Matches README's Node 20+ guidance. | ✓ |
| Keep apt-get (Node 18, accept mismatch) | Keep apt-get install nodejs. Debian bookworm provides Node 18 which meets Vite 8's minimum. | |
| Rebase on node:20-slim + install Rust | Switch base from rust:slim to node:20-slim and install Rust/rustup on top | |

**User's choice:** Install Node 20 via NodeSource

---

## Agent's Discretion

- nginx.conf directive placement and ordering
- Whether to add `gzip_vary on;`
- Exact `/healthz` response format (empty body or minimal `ok` text)

## Deferred Ideas

None — discussion stayed within phase scope.
