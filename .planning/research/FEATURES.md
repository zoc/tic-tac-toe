# Feature Landscape

**Domain:** Docker multi-architecture deployment of a static web game (Rust/WASM + Vite)
**Milestone scope:** v1.2 Docker Deployment — containerize, publish, document
**Researched:** 2026-04-14
**Confidence:** HIGH (Docker official docs + nginx official docs + GitHub Actions official docs)

---

## Context: What Already Exists

v1.1 is complete. The following are already built and validated:

| Already Built | Implementation |
|---------------|----------------|
| Vite 8 production build | `npm run build` → `dist/` containing HTML, CSS, JS, `.wasm` |
| Static file output | `dist/` is fully self-contained — no server-side processing needed |
| WASM-specific asset handling | `build.target: 'esnext'`, WASM files output alongside JS |
| Content-addressed filenames | Vite hashes JS/CSS: `assets/index-Dv3Lk8xT.js` — suitable for long-lived caching |
| `package.json` build scripts | `npm run build` is the single entry point for a production build |

**This milestone is pure infrastructure** — the game itself doesn't change. All features are about packaging, serving, publishing, and documenting the container.

---

## Table Stakes (Every Docker-Deployed Static Site Must Have These)

Features that operators and users expect. Missing any of these = broken or unprofessional deployment.

| Feature | Why Expected | Complexity | v1.2? | Notes |
|---------|--------------|------------|-------|-------|
| Multi-stage Dockerfile (build + serve) | The build stage installs Rust, wasm-pack, Node — tools that have no place in the final image. Final image must contain only nginx + `dist/`. Without multi-stage, the image would be 2–3 GB (Rust toolchain alone is ~1.5 GB). | MEDIUM | ✅ Yes | Build stage: `node:20-alpine` + install Rust + wasm-pack. Serve stage: `nginx:alpine`. |
| `nginx:alpine` as final base image | nginx:alpine is the canonical choice for static sites: ~5 MB base, battle-tested static file serving, gzip built-in, official multi-arch manifest (amd64/arm64/arm32v7/etc). The Debian nginx variant is 40+ MB for no benefit here. | LOW | ✅ Yes | `FROM nginx:alpine` — currently at 1.28.3-alpine or 1.29.8-alpine (mainline). |
| EXPOSE 80 declaration | Documents the container's listening port. Required for `docker run -P` to work. Consumers of the image expect to know the port without reading docs. | LOW | ✅ Yes | `EXPOSE 80` in Dockerfile. |
| `.dockerignore` file | Without it, the entire build context (including `target/`, `node_modules/`, `.git/`) is sent to the Docker daemon — slowing builds massively and risking secrets leaking into layers. `target/` alone can be 500 MB+ (Rust build artifacts). | LOW | ✅ Yes | Exclude: `target/`, `node_modules/`, `.git/`, `dist/`, `*.md`, `.planning/`. |
| Single-command `docker run` UX | Operators pulling from Docker Hub expect `docker run -p 8080:80 user/repo:tag` to Just Work. No volume mounts, no env vars, no config files needed — it's a static site. | LOW | ✅ Yes | Image must be fully self-contained. No mandatory configuration at runtime. |
| Semantic version tags on Docker Hub | `latest` alone is insufficient for production deployments. Users need `v1.2.0` pinned tags to know what they're running and reproduce deployments. | LOW | ✅ Yes | `docker/metadata-action` with semver pattern generates both `v1.2.0` and `latest` automatically from git tag. |
| Multi-arch manifest (amd64 + arm64) | arm64 is now the dominant architecture for VPS providers (AWS Graviton, Hetzner ARM, Oracle ARM Free Tier). An amd64-only image runs under emulation on arm64 with significant performance overhead. A multi-arch manifest transparently serves the right binary for each host. | MEDIUM | ✅ Yes | `docker buildx` + QEMU via `docker/setup-qemu-action`. `--platform linux/amd64,linux/arm64`. |
| GitHub Actions CI on tag push | Manual `docker buildx build --push` is error-prone and requires local credentials. Automated CI triggered by `git tag v1.2.0 && git push --tags` is the standard release workflow. | MEDIUM | ✅ Yes | Trigger: `on: push: tags: ['v*']`. Steps: QEMU → Buildx → Login → Build+Push. |
| README / usage docs | Operators pulling from Docker Hub need to know: how to run it, what port, how to update, how to put it behind a reverse proxy. Docker Hub README is the first thing users read. | LOW | ✅ Yes | README section with `docker pull`, `docker run`, compose snippet, reverse proxy note. |

---

## Differentiators (Notably Better Than Minimal Setup)

Features not strictly required but that make the deployment meaningfully better — better performance, better operations, better security.

| Feature | Value Proposition | Complexity | v1.2? | Notes |
|---------|-------------------|------------|-------|-------|
| Custom nginx.conf with gzip compression | nginx default config has `gzip off`. Enabling gzip for HTML/CSS/JS reduces transfer sizes by 60–70%. For a WASM game, the `.wasm` file (33 KB) and JS bundle compress well. Mobile users on slow connections benefit meaningfully. Requires `gzip_types` to include `application/wasm application/javascript text/css`. | LOW | ✅ Yes | ~15-line nginx snippet. `gzip on; gzip_comp_level 6; gzip_min_length 1000; gzip_vary on; gzip_types text/html text/css application/javascript application/wasm;` |
| Cache-Control headers for Vite hashed assets | Vite content-hashes all JS/CSS/WASM filenames. These files are immutable — `main-Dv3Lk8xT.js` will never change contents while that hash exists. Setting `Cache-Control: public, max-age=31536000, immutable` (1 year) on `/assets/*` eliminates repeat download costs. `index.html` must stay `no-cache` so browsers fetch fresh asset references. | LOW | ✅ Yes | Two nginx `location` blocks: `/assets/` with `immutable` header, `/ ` with `no-cache`. |
| HEALTHCHECK instruction in Dockerfile | Allows container orchestrators (Docker Compose, Swarm, Kubernetes) to detect when nginx crashes silently. A failed health check triggers restart policies. Without it, a container with dead nginx shows as "running" indefinitely. | LOW | ✅ Yes | `HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO- http://localhost/ || exit 1` (wget is available in Alpine). |
| OCI image labels (org.opencontainers.image.*) | Provides machine-readable metadata: source repository URL, version, creation timestamp, description. `docker/metadata-action` generates these automatically. Enables tooling (Docker Scout, Renovate, Dependabot) to track image provenance and suggest updates. | LOW | ✅ Yes | `docker/metadata-action` outputs labels automatically. Pass to `build-push-action` via `labels: ${{ steps.meta.outputs.labels }}`. |
| Build cache in GitHub Actions | Rust/wasm-pack builds are slow (60–90 seconds cold). GitHub Actions can cache the `~/.cargo/registry` and `target/` directories between runs. Warm builds with unchanged Rust code take 10–15 seconds instead. | MEDIUM | ✅ Yes | `actions/cache` for `~/.cargo/registry`, `~/.cargo/git`, `target/`. Key: `cargo-${{ hashFiles('**/Cargo.lock') }}`. |
| `COPY --chown=nginx:nginx` for file ownership | Files COPYed into an nginx image default to root ownership. nginx worker processes run as the `nginx` user (uid 101). While static file serving works without correct ownership, it's a best-practice hygiene issue and prevents subtle permission errors if the nginx config is customized. | LOW | ✅ Yes | `COPY --chown=nginx:nginx --from=builder /app/dist /usr/share/nginx/html` |

---

## Differentiators: Future Consideration (v1.3+)

Features that have genuine value but are out of scope for the initial deployment milestone.

| Feature | Value Proposition | Complexity | Why Defer |
|---------|-------------------|------------|-----------|
| Docker Compose example in repo | Makes local VPS deployment a `docker compose up` one-liner | LOW | Useful but not blocking. Add in v1.3 if there's user demand. |
| Security scanning (Trivy/Snyk in CI) | Automatically reports CVEs in the final image | MEDIUM | Adds CI complexity and noise for a simple nginx:alpine image. Defer until image has users. |
| nginx `server_tokens off` + security headers | X-Content-Type-Options, X-Frame-Options, CSP | LOW | Game has no auth, no user data. Security headers add value for real apps but are low priority for a game. |
| ARM v7 platform (linux/arm/v7) | Supports 32-bit Raspberry Pi 3/4 | LOW | The Rust wasm-pack build stage requires arm64-compatible or x86 tools. arm/v7 QEMU emulation is extremely slow for Rust compilation. Add only if users request it. |
| GitHub Container Registry (ghcr.io) mirror | Publish to both Docker Hub and ghcr.io | LOW | Docker Hub is sufficient for the initial release. ghcr.io avoids rate limits for GitHub-hosted users. |

---

## Anti-Features (Commonly Done, Actively Harmful)

Features that seem reasonable but make the deployment worse.

| Anti-Feature | Why It Seems Reasonable | Why Problematic | What to Do Instead |
|--------------|------------------------|-----------------|-------------------|
| **Bloated final image** (shipping build tools) | "Easier to debug in production" | A Rust + Node build environment is 1.5–3 GB. nginx:alpine serving static files is ~25 MB. 100x size difference with zero benefit — Rust and npm are not needed at runtime for a static site. | Strict multi-stage: build stage keeps all tools; `COPY --from=builder dist/ /usr/share/nginx/html` in the serve stage copies only outputs. |
| **Running nginx as root** | "Simpler configuration" | Security vulnerability. If the nginx process is compromised, root access means full container control. | Use `nginx:alpine` official image — it already drops privileges: master process runs as root (for port 80 binding) but worker processes run as `nginx` user (uid 101). No manual `USER` instruction needed. |
| **No `.dockerignore`** | "I'll add it later" | The `target/` directory (500 MB+ Rust artifacts) and `node_modules/` are sent as build context without it. Slow builds, potential secret leakage from `.env` files, large layer cache invalidation. | Add `.dockerignore` in the first commit alongside the Dockerfile. Must exist before `docker build` is ever run. |
| **Pinning to `latest` tag only** | "Always get the latest nginx" | `nginx:latest` resolves to mainline (currently 1.29.8) — Debian-based, ~55 MB. `nginx:alpine` is only ~5 MB and equally stable. `latest` tag changes unpredictably, breaking reproducible builds. | Pin to `nginx:alpine` (tracks latest stable Alpine-based release). Consider pinning to a specific version like `nginx:1.28-alpine` (stable branch) for maximum reproducibility. |
| **Hardcoded Docker Hub credentials in workflow** | "Works immediately" | Credentials in YAML = credentials in git history = security incident. | Use `secrets.DOCKERHUB_TOKEN` (Docker Hub access token, not password) stored in GitHub repository secrets. |
| **Single-platform build only (amd64)** | "Simpler to start" | arm64 is now mainstream for VPS (AWS Graviton, Hetzner, Oracle Free Tier). An amd64-only image runs under QEMU emulation on arm64 — 3–10x slower. This is especially bad for a game (UI responsiveness). | Multi-arch from day one using `docker buildx` + QEMU. The CI setup cost is minimal (3 extra lines in GitHub Actions). |
| **COPY the entire repo into the final image** | "Simple COPY . ." | Copies `.git/`, `.planning/`, `src/` (Rust source), `Cargo.toml`, test files, everything into the final image. Increases image size, exposes source code unnecessarily, slows builds. | Multi-stage: COPY only `dist/` from the build stage. Final image contains only nginx binary + HTML/CSS/JS/WASM. |
| **`gzip_static` module without pre-compressed assets** | "Better performance" | `gzip_static` serves pre-compressed `.gz` files for zero CPU overhead. But Vite doesn't generate `.gz` files by default — `vite-plugin-compression` would need to be added. If `.gz` files are missing, nginx falls back to dynamic gzip anyway. Unnecessary complexity for this project. | Use standard `ngx_http_gzip_module` (dynamic compression). CPU cost for static files with nginx is negligible. |
| **`CMD` override for nginx config** | "More control" | Replacing nginx's default CMD with a custom entrypoint shell script adds complexity for no benefit on a static site. | Mount or COPY a custom `nginx.conf` to `/etc/nginx/conf.d/default.conf`. Let nginx's own entrypoint handle startup. |

---

## Feature Dependencies

```
[Multi-stage Dockerfile]
    └──requires──> .dockerignore (must exist before first build)
    └──requires──> `npm run build` produces dist/ (already validated)
    └──outputs──> final nginx:alpine image with dist/ files

[Custom nginx.conf]
    └──requires──> Multi-stage Dockerfile (needs a serve stage to configure)
    └──provides──> gzip compression
    └──provides──> Cache-Control headers
    └──enables──> HEALTHCHECK (healthcheck wget hits nginx which serves custom config)

[HEALTHCHECK]
    └──requires──> nginx:alpine (wget available in Alpine base)
    └──requires──> EXPOSE 80 declared

[GitHub Actions CI]
    └──requires──> Multi-stage Dockerfile (builds it)
    └──requires──> Docker Hub account + access token (external dependency)
    └──requires──> QEMU setup (for arm64 emulation on amd64 runner)
    └──requires──> docker buildx (for multi-arch)
    └──outputs──> multi-arch manifest on Docker Hub

[OCI image labels]
    └──requires──> docker/metadata-action in CI
    └──requires──> GitHub Actions CI workflow
    └──enhances──> Docker Hub discoverability

[Build cache in CI]
    └──requires──> GitHub Actions CI workflow
    └──enhances──> GitHub Actions CI (reduces build time)
    └──no effect on final image

[README / usage docs]
    └──requires──> Published Docker Hub image (to have a real pull command)
    └──depends-on──> All above features being validated first
```

### Dependency Notes

- **`.dockerignore` must exist before any other Dockerfile work** — building without it on a Rust project means sending 500 MB+ build context.
- **Custom nginx.conf enables both gzip and cache headers** — these two features are implemented in the same file, so they should be built together.
- **GitHub Actions CI requires all Dockerfile work to be done first** — the workflow just runs `docker buildx build`. The Dockerfile must be correct before CI is wired up.
- **README is the last step** — it documents the final `docker pull` command with the real image name, which isn't known until the image is published.

---

## MVP Definition for v1.2

### Launch With (v1.2 — all required)

- [ ] **Multi-stage Dockerfile** — core deliverable; without this, nothing else works
- [ ] **.dockerignore** — must be created alongside Dockerfile (not after)
- [ ] **Custom nginx.conf** with gzip + Cache-Control headers — small cost, big win for users
- [ ] **HEALTHCHECK** — 1 line in Dockerfile, makes the image production-appropriate
- [ ] **Multi-arch build** (amd64 + arm64) — arm64 VPS users are the primary audience
- [ ] **GitHub Actions CI** triggered on git tag push — automates the release workflow
- [ ] **OCI image labels** — generated automatically by `docker/metadata-action`
- [ ] **Build cache in CI** — dramatically reduces CI time for Rust builds
- [ ] **README usage docs** — Docker Hub README + repo README section

### Add After Validation (v1.3+)

- [ ] **Docker Compose example** — add after confirming users deploy via docker run
- [ ] **`nginx:1.28-alpine` version pin** — add if stability over freshness is preferred
- [ ] **Security headers** (X-Content-Type-Options, CSP) — if the game gets real users

### Future Consideration (v2+)

- [ ] **GitHub Container Registry mirror** — if Docker Hub rate limits become a problem
- [ ] **ARM v7 platform** — only if Raspberry Pi users request it
- [ ] **Security scanning in CI** — when image complexity justifies it

---

## Feature Prioritization Matrix

| Feature | Operator Value | Implementation Cost | Priority |
|---------|----------------|---------------------|----------|
| Multi-stage Dockerfile | HIGH | MEDIUM | P1 |
| .dockerignore | HIGH | LOW | P1 |
| Multi-arch manifest (amd64+arm64) | HIGH | MEDIUM | P1 |
| GitHub Actions CI on tag push | HIGH | MEDIUM | P1 |
| gzip compression (nginx.conf) | HIGH | LOW | P1 |
| Cache-Control headers (nginx.conf) | HIGH | LOW | P1 |
| HEALTHCHECK | MEDIUM | LOW | P1 |
| OCI image labels | MEDIUM | LOW | P1 |
| Build cache in CI | MEDIUM | MEDIUM | P1 |
| README / usage docs | HIGH | LOW | P1 |
| Docker Compose example | MEDIUM | LOW | P2 |
| Security headers | LOW | LOW | P3 |
| ARM v7 platform | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1.2 launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Complexity Details by Feature

| Feature | Effort Estimate | Key Challenge |
|---------|----------------|---------------|
| Multi-stage Dockerfile | 60–90 min | Installing Rust + wasm-pack in Docker; correct stage boundary; arm64 Rust cross-compile in build stage |
| .dockerignore | 5 min | None — it's a simple text file |
| Custom nginx.conf | 15 min | Correct MIME type for `.wasm` (`application/wasm`); gzip_types list; location block specificity order |
| HEALTHCHECK | 5 min | None — copy from template |
| GitHub Actions workflow | 30–45 min | QEMU + Buildx setup order; secrets configuration; tag trigger pattern; metadata-action semver config |
| Build cache in CI | 15 min | Correct cache key based on `Cargo.lock` hash |
| OCI labels | 0 min extra | `docker/metadata-action` generates these automatically; just pass through to build-push-action |
| README docs | 20–30 min | Writing clear `docker run` examples; VPS reverse proxy note; compose snippet |

**Total estimated effort: 3–4 hours** for a complete, production-quality deployment setup.

---

## Key Technical Details

### Dockerfile Stage Boundary

The critical design question is: **where does the build stage end and the serve stage begin?**

```
BUILD STAGE (node:20-alpine + Rust + wasm-pack):
  1. Install Rust toolchain (rustup, wasm32-unknown-unknown target)
  2. Install wasm-pack
  3. COPY Cargo.toml, Cargo.lock, src/ → wasm-pack build → outputs pkg/
  4. COPY package.json, package-lock.json → npm ci
  5. COPY remaining source → npm run build → outputs dist/

SERVE STAGE (nginx:alpine):
  COPY --from=builder /app/dist /usr/share/nginx/html
  COPY nginx.conf /etc/nginx/conf.d/default.conf
  EXPOSE 80
  HEALTHCHECK ...
```

The serve stage contains **only** nginx binary, static files, and nginx config. No Rust, no Node, no npm, no wasm-pack.

### nginx.conf Minimum Required Config

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    # Correct MIME type for WebAssembly
    types {
        application/wasm wasm;
    }
    include /etc/nginx/mime.types;

    # Gzip compression
    gzip on;
    gzip_comp_level 6;
    gzip_min_length 1000;
    gzip_vary on;
    gzip_types text/html text/css application/javascript application/wasm
               application/json text/plain text/xml;

    # Long-lived cache for Vite hashed assets
    location /assets/ {
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    # No cache for index.html (asset references change on rebuild)
    location / {
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache";
    }
}
```

**Critical:** nginx:alpine's default MIME types file includes `application/wasm` since nginx 1.21.4. However, the custom `server {}` block may override the include. Explicitly adding the wasm type ensures correctness.

### GitHub Actions Workflow Pattern

```yaml
on:
  push:
    tags: ['v*']

steps:
  - uses: actions/checkout@v5
  - uses: docker/setup-qemu-action@v4       # ARM64 emulation
  - uses: docker/setup-buildx-action@v4     # Multi-platform builder
  - uses: docker/login-action@v4            # Docker Hub auth
    with:
      username: ${{ vars.DOCKERHUB_USERNAME }}
      password: ${{ secrets.DOCKERHUB_TOKEN }}
  - uses: docker/metadata-action@v5         # Tags + labels from git tag
    id: meta
    with:
      images: user/tic-tac-toe-wasm
      tags: |
        type=semver,pattern={{version}}
        type=semver,pattern={{major}}.{{minor}}
  - uses: docker/build-push-action@v7
    with:
      platforms: linux/amd64,linux/arm64
      push: true
      tags: ${{ steps.meta.outputs.tags }}
      labels: ${{ steps.meta.outputs.labels }}
      cache-from: type=gha
      cache-to: type=gha,mode=max
```

**Note:** `cache-from: type=gha` / `cache-to: type=gha,mode=max` uses GitHub Actions cache for Docker layer caching — this is separate from the `actions/cache` approach for Cargo caches. Both should be used.

---

## Pitfall Preview

| Topic | Key Risk | Mitigation |
|-------|----------|-----------|
| Rust in Docker on arm64 | `node:20-alpine` + rustup on arm64 works, but `musl` target required for Alpine. Rust default target is `x86_64-unknown-linux-gnu`. Must add `wasm32-unknown-unknown` explicitly. | `rustup target add wasm32-unknown-unknown` in Dockerfile |
| wasm-pack install in Docker | `cargo install wasm-pack` compiles from source — takes 5+ minutes. Use the official install script instead: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf \| sh` | Prefer installer script over cargo install |
| Layer cache invalidation | Copying `Cargo.toml` + `Cargo.lock` before source files lets Docker cache the `cargo build` layer when only JS/CSS changes. Copying everything at once invalidates the Rust build cache on every source change. | Two-step COPY: first manifests, then source |
| QEMU build time | Rust compilation under QEMU emulation for arm64 on an amd64 runner can take 15–20 minutes. Since WASM is architecture-neutral, the Rust/wasm-pack build stage only needs to run on the native platform — but the final nginx stage must be built for both. | Use `--platform=$BUILDPLATFORM` on the build stage; only the serve stage needs QEMU |
| `.wasm` MIME type | Some nginx configurations serve `.wasm` as `application/octet-stream`. Browsers may refuse to execute it or show warnings. | Explicit `application/wasm` in nginx config |
| Docker Hub access token | Using Docker Hub password (not access token) in GitHub secrets is a security anti-pattern. Tokens can be scoped and revoked without changing the password. | Create a Docker Hub access token with "Read & Write" scope for the repository only |

---

## Sources

- Docker Official Docs: Multi-platform builds — https://docs.docker.com/build/building/multi-platform/ (HIGH confidence)
- Docker Official Docs: Building best practices — https://docs.docker.com/build/building/best-practices/ (HIGH confidence)
- Docker Official Docs: Dockerfile reference, HEALTHCHECK — https://docs.docker.com/reference/dockerfile/#healthcheck (HIGH confidence)
- Docker Hub: nginx official image — https://hub.docker.com/_/nginx (HIGH confidence)
- nginx Official Docs: ngx_http_gzip_module — https://nginx.org/en/docs/http/ngx_http_gzip_module.html (HIGH confidence)
- nginx Official Docs: ngx_http_headers_module — https://nginx.org/en/docs/http/ngx_http_headers_module.html (HIGH confidence)
- GitHub Actions Official Docs: Publishing Docker images — https://docs.github.com/en/actions/use-cases-and-examples/publishing-packages/publishing-docker-images (HIGH confidence)
- Docker Official Docs: Multi-platform with GitHub Actions — https://docs.docker.com/build/ci/github-actions/multi-platform/ (HIGH confidence)

---

*Feature research for: Docker multi-architecture deployment — Rust/WASM + Vite static site*
*Milestone: v1.2 Docker Deployment*
*Researched: 2026-04-14*
