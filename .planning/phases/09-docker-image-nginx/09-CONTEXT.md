# Phase 9: Docker Image & nginx - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the 3 open gaps in the existing Docker/nginx implementation (DOCK-04 no-cache header, DOCK-05 gzip, DOCK-06 HEALTHCHECK), fix 2 Dockerfile reproducibility issues (wasm-pack version pin, Node.js version), then run all 5 success criteria verifications.

**In scope:** nginx.conf changes, Dockerfile changes, local smoke test verification.
**Not in scope:** CI/CD pipeline, multi-arch builds, Docker Hub publishing (those are Future Requirements).

</domain>

<decisions>
## Implementation Decisions

### HEALTHCHECK (DOCK-06)
- **D-01:** Add a dedicated `/healthz` location block in nginx.conf that returns 200 OK with no HTML body
- **D-02:** HEALTHCHECK in Dockerfile probes `curl -f http://localhost/healthz` (not the root `/`)
- **D-03:** Install `curl` in the serve stage: `RUN apk add --no-cache curl`
- **D-04:** Timing: `--interval=30s --timeout=3s --start-period=5s --retries=3` (Docker defaults, no custom tuning)

### gzip Compression (DOCK-05)
- **D-05:** Enable dynamic gzip for HTML, CSS, and JavaScript only — skip WASM (already compressed by wasm-opt during build; double-gzip provides no benefit)
- **D-06:** `gzip_min_length 1000;` — don't compress files smaller than 1KB
- **D-07:** `gzip_comp_level 6;` (nginx default, balance of speed vs ratio)
- **D-08:** `gzip_types` to include: `text/html text/css application/javascript text/javascript`

### Cache-Control for index.html (DOCK-04)
- **D-09:** Add `add_header Cache-Control "no-cache";` in the root `location /` block so `index.html` is never cached by browsers or CDNs between deployments
- **D-10:** The `/assets/` location block already has `Cache-Control: public, immutable` — leave unchanged

### wasm-pack Version Pin
- **D-11:** Replace `curl | sh` installer with `cargo install wasm-pack@0.14.0 --locked` to pin exact version matching STACK.md recommendation
- **D-12:** This eliminates the supply chain risk flagged in CONCERNS.md (unpinned installer URL)

### Node.js Version in Dockerfile
- **D-13:** Replace `apt-get install nodejs npm` with NodeSource setup to install Node 20 LTS explicitly: `curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt-get install -y nodejs`
- **D-14:** This resolves the README vs Dockerfile mismatch (README says Node 20+, apt-get provides Node 18)

### Agent's Discretion
- nginx.conf directive placement and ordering (within the decided blocks above)
- Whether to add `gzip_vary on;` (standard best practice for proxies — agent can include)
- Exact nginx `/healthz` response format (empty body, or a minimal `ok` text — agent decides)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Implementation (already committed)
- `Dockerfile` — multi-stage build, currently missing HEALTHCHECK and using unpinned wasm-pack/Node
- `nginx.conf` — server configuration, currently missing gzip and Cache-Control no-cache
- `.dockerignore` — build context exclusions (complete, no changes needed)

### Requirements
- `.planning/REQUIREMENTS.md` — DOCK-01 through DOCK-08 with exact acceptance text
- `.planning/ROADMAP.md` §Phase 9 — 5 success criteria with exact curl/inspect commands

### Codebase Analysis
- `.planning/codebase/CONCERNS.md` §"Missing Critical Features" — exact descriptions of DOCK-04, DOCK-05, DOCK-06 gaps
- `.planning/codebase/CONCERNS.md` §"Dependencies at Risk" — wasm-pack and Node.js version concerns with recommended fixes
- `.planning/phases/09-01-dockerfile-nginx.md` — prior plan with verification steps and key decisions already made

### Stack Reference
- `AGENTS.md` §Technology Stack — wasm-pack 0.14.0 is the recommended version

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Dockerfile` (lines 1–46): Complete multi-stage build — only needs wasm-pack and Node.js install method changes
- `nginx.conf`: Complete server config — needs gzip block added, Cache-Control no-cache for `/`, and `/healthz` location

### Established Patterns
- nginx `location /assets/` already uses `add_header Cache-Control "public, immutable"` — the no-cache pattern for `/` mirrors this
- Security headers already use `add_header ... always` on all location blocks — `/healthz` and any new block should follow the same pattern if headers are needed

### Integration Points
- Serve stage (`FROM nginx:alpine`): `apk add --no-cache curl` goes here, not in the build stage
- nginx.conf `location /` block: add `add_header Cache-Control "no-cache";` here
- nginx.conf server block (outside location blocks): add `gzip` directives here

</code_context>

<specifics>
## Specific Ideas

- User explicitly wants a dedicated `/healthz` endpoint rather than probing root `/` — keep it lightweight with no response body
- wasm-pack pin: `cargo install wasm-pack@0.14.0 --locked` is the exact form preferred
- Node 20 install: NodeSource `setup_20.x` script is the preferred approach

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-docker-image-nginx*
*Context gathered: 2026-04-14*
