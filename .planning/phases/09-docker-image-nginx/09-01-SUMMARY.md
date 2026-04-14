---
phase: "09-docker-image-nginx"
plan: "01"
subsystem: "docker"
tags: ["dockerfile", "nginx", "docker", "healthcheck", "gzip", "security"]
dependency_graph:
  requires: []
  provides: ["docker-image-fixes", "nginx-config-complete"]
  affects: ["09-02-PLAN.md"]
tech_stack:
  added: []
  patterns:
    - "cargo install --locked for pinned supply-chain-safe binary installs"
    - "NodeSource setup_20.x for explicit Node LTS version in Debian-based containers"
    - "nginx gzip block for text assets, WASM intentionally excluded"
    - "nginx return 200 for lightweight health endpoints"
key_files:
  created: []
  modified:
    - "Dockerfile"
    - "nginx.conf"
decisions:
  - "wasm-pack pinned via cargo install wasm-pack@0.14.0 --locked (eliminates curl|sh supply-chain risk)"
  - "Node 20 LTS via NodeSource setup_20.x (apt default is Node 18 on Debian bookworm)"
  - "gzip_types excludes application/wasm — WASM pre-compressed by wasm-opt, double-gzip wastes CPU"
  - "HEALTHCHECK probes /healthz (dedicated endpoint) not / (avoids logging noise from root path)"
metrics:
  duration: "~4 minutes"
  completed: "2026-04-14"
  tasks_completed: 2
  files_modified: 2
requirements_addressed:
  - DOCK-01
  - DOCK-02
  - DOCK-03
  - DOCK-04
  - DOCK-05
  - DOCK-06
  - DOCK-07
  - DOCK-08
---

# Phase 9 Plan 01: Docker & nginx Configuration Fixes Summary

**One-liner:** Pinned wasm-pack@0.14.0 via cargo install, Node 20 via NodeSource, and added HEALTHCHECK + gzip + Cache-Control no-cache to complete the production Docker/nginx configuration.

## What Was Built

Two files updated with five targeted changes to close all known gaps in the Docker/nginx implementation:

### Dockerfile (3 changes)

1. **Node.js 20 LTS via NodeSource** — Replaced `apt-get install nodejs npm` with a NodeSource `setup_20.x` pipeline that installs Node 20 explicitly. The Debian bookworm `apt` default is Node 18; this ensures the README promise ("Node 20+") matches reality inside the container.

2. **wasm-pack pinned to 0.14.0** — Replaced the `curl | sh` installer URL with `cargo install wasm-pack@0.14.0 --locked`. Eliminates the supply-chain risk of fetching and executing an unversioned shell script at build time.

3. **HEALTHCHECK + curl in serve stage** — Added `apk add --no-cache curl` to `nginx:alpine` and a HEALTHCHECK instruction probing `/healthz` with timing: `--interval=30s --timeout=3s --start-period=5s --retries=3`.

### nginx.conf (3 changes)

1. **gzip compression block** — Added `gzip on` with `gzip_types text/html text/css application/javascript text/javascript`. `application/wasm` intentionally excluded — WASM files are pre-optimized by `wasm-opt` during build; runtime gzip adds CPU cost with negligible benefit.

2. **Cache-Control: no-cache for location /** — Added `add_header Cache-Control "no-cache"` inside the root location block so `index.html` is always revalidated between deployments. Assets in `/assets/` retain `public, immutable` (content-hashed, unchanged).

3. **/healthz endpoint** — New `location = /healthz` block with `return 200 "ok"` and `access_log off`. Lightweight, no HTML body, keeps health check probes out of nginx access logs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix Dockerfile — pin wasm-pack, Node 20, HEALTHCHECK | bae28dd | Dockerfile |
| 2 | Fix nginx.conf — gzip, Cache-Control no-cache, /healthz | c38f7ea | nginx.conf |

## Deviations from Plan

None — plan executed exactly as written. All five changes implemented in the order and form specified by the plan.

## Verification Results

All acceptance criteria from the plan pass:

```
✓ grep "cargo install wasm-pack@0.14.0 --locked" Dockerfile  → exits 0
✓ grep "setup_20.x" Dockerfile                               → exits 0
✓ grep "apk add --no-cache curl" Dockerfile                  → exits 0
✓ grep "HEALTHCHECK" Dockerfile                              → exits 0
✓ grep "healthz" Dockerfile                                  → exits 0
✓ grep "curl https://rustwasm.github.io" Dockerfile          → exits 1 (removed)
✓ grep "gzip on;" nginx.conf                                 → exits 0
✓ grep "gzip_min_length 1000" nginx.conf                     → exits 0
✓ grep "gzip_comp_level 6" nginx.conf                        → exits 0
✓ grep "gzip_types.*text/html.*text/css.*application/javascript" nginx.conf → exits 0
✓ application/wasm NOT in gzip_types (only in comment)       → directive-free
✓ grep 'Cache-Control.*no-cache' nginx.conf                  → exits 0
✓ grep "location = /healthz" nginx.conf                      → exits 0
✓ grep "return 200" nginx.conf                               → exits 0
✓ .dockerignore: target/, node_modules/, pkg/, dist/, .git   → all present
```

## Self-Check: PASSED

- `Dockerfile` modified: ✓ exists, changes verified by grep
- `nginx.conf` modified: ✓ exists, changes verified by grep
- Task 1 commit `bae28dd`: ✓ exists in git log
- Task 2 commit `c38f7ea`: ✓ exists in git log
