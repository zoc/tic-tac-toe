# Domain Pitfalls — v1.2 Docker Multi-Arch Deployment

**Domain:** Docker multi-architecture deployment — Rust/WASM + Vite static site containerization
**Codebase:** Existing Rust/wasm-pack + Vite 8 + vanilla JS/CSS game, adding Docker packaging
**Researched:** 2026-04-14
**Confidence:** HIGH — Docker official docs + GitHub Actions official docs + MDN (WASM MIME) + direct codebase inspection

---

## Context: Why This Document Exists

v1.1 PITFALLS covers browser-side polish (Web Audio, CSS animations, localStorage, dark mode).
This document covers the **Docker deployment** pitfalls specific to packaging a Rust/WASM + Vite
static site as a multi-architecture container image.

This is not a generic Docker guide. Every pitfall here is specific to one or more of:
- Rust + Node.js **dual ecosystem** in the same Dockerfile
- `wasm-pack` compilation behavior inside Docker (especially under QEMU/arm64 emulation)
- Vite's WASM output bundle and how nginx must serve it
- Multi-arch buildx setup in GitHub Actions with tag-on-push workflows
- Docker Hub credential handling and `.dockerignore` for a Rust+Node project

---

## Critical Pitfalls

Mistakes that produce broken images, failed builds, or silent serving errors.

---

### Pitfall 1: QEMU Emulation Makes `wasm-pack build` Catastrophically Slow (or Hang)

**What goes wrong:**
Building `linux/arm64` via QEMU emulation on a `linux/amd64` GitHub Actions runner compiles
Rust under full software CPU emulation. `wasm-pack build` involves:
1. `cargo` dependency resolution and compilation (many crates)
2. `wasm-bindgen-cli` code generation
3. `wasm-opt` binary optimizer pass (runs native arm64 binary under QEMU)

For non-trivial Rust projects, the QEMU-emulated Rust compile is **10–30× slower** than native.
The wasm-opt pass in particular can stall for 5+ minutes on a 33KB `.wasm` binary under emulation.
GitHub Actions jobs time out after 6 hours by default, but some teams set shorter limits.

**Why it happens:**
QEMU is software CPU emulation — every x86 instruction executed for arm64 is individually translated.
Rust compilation is CPU-intensive. Docker docs explicitly warn: *"Emulation with QEMU can be much
slower than native builds, especially for compute-heavy tasks like compilation."*

**How to avoid:**
Use the `--platform=$BUILDPLATFORM` trick to run the Rust **build stage** on the native runner
architecture (`linux/amd64`), cross-compiling only the **output** to `wasm32-unknown-unknown`:

```dockerfile
# Build stage always runs on the runner's native platform
FROM --platform=$BUILDPLATFORM rust:slim AS build

# wasm32 target is platform-independent — the WASM binary is the same on amd64 and arm64
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack
COPY . .
RUN wasm-pack build --target web --release

RUN npm ci
RUN npm run build
# dist/ now contains the .wasm and all assets

# Serve stage runs on the target platform (nginx:alpine is multi-arch)
FROM --platform=$TARGETPLATFORM nginx:alpine AS serve
COPY --from=build /app/dist /usr/share/nginx/html
```

**Key insight:** WebAssembly is platform-neutral bytecode. A `.wasm` file compiled on `linux/amd64`
is **identical** to one compiled on `linux/arm64`. The `wasm32-unknown-unknown` target doesn't
know or care about the host architecture. There is zero reason to compile Rust under QEMU for the
arm64 image — the build stage output is the same either way.

The nginx **serve stage** is multi-arch because `nginx:alpine` has native arm64 and amd64 layers.
`COPY --from=build` carries the identical static files into the correct arch-specific nginx layer.

**Warning signs:**
- Build for `linux/arm64` takes >5 minutes while `linux/amd64` takes <1 minute
- Build logs show `[linux/arm64 build...]` running full cargo compilation
- `wasm-opt` step appears twice in build logs (once per platform)

**Phase to address:** Dockerfile creation phase — use `--platform=$BUILDPLATFORM` on the build
stage from the first line of the Dockerfile. Never patch this in after a slow build.

---

### Pitfall 2: Missing `application/wasm` MIME Type Causes Browser Rejection of `.wasm` Files

**What goes wrong:**
When nginx serves `tic_tac_toe_bg-BSmjV6BP.wasm`, it must send the response header:
```
Content-Type: application/wasm
```

If it sends `application/octet-stream` (the default fallback for unknown extensions), the browser
will **reject** the WASM module with a console error like:
```
WebAssembly.compileStreaming(): Response has unsupported MIME type 'application/octet-stream'
expected 'application/wasm'
```

This means the compiled game engine fails to load silently in production. The JS error handler
(`main().catch(...)`) will catch it, but the user sees a broken page.

**Why it happens:**
`nginx:alpine`'s default `mime.types` file **does include** `application/wasm wasm;` in recent
versions (nginx ≥ 1.11.4, 2017+). However:
1. The default `mime.types` is only loaded if your `nginx.conf` includes `include mime.types;`
2. If you write a custom `nginx.conf` that omits the `include` directive, WASM gets no MIME type
3. The `default_type application/octet-stream;` fallback kicks in silently

**How to avoid:**
Always verify the MIME type is set correctly. In your nginx config:
```nginx
http {
    include       /etc/nginx/mime.types;    # REQUIRED — includes application/wasm
    default_type  application/octet-stream;
    ...
}
```

And optionally add an explicit override as belt-and-suspenders:
```nginx
location ~* \.wasm$ {
    add_header Content-Type application/wasm;
    # Also add CORS if needed (see Pitfall 3)
}
```

**Warning signs:**
- Chrome DevTools Network tab shows `tic_tac_toe_bg-*.wasm` with `Content-Type: application/octet-stream`
- Console error: `WebAssembly.compileStreaming(): Response has unsupported MIME type`
- The `.wasm` request succeeds (200 OK) but the game fails to initialize — misleading

**Phase to address:** nginx configuration phase — always `include mime.types` first; verify in
final container smoke test with `curl -I http://localhost/assets/*.wasm`.

---

### Pitfall 3: nginx Missing CORS Headers for WASM `instantiateStreaming()` Cross-Origin Scenarios

**What goes wrong:**
`WebAssembly.instantiateStreaming()` requires the server to respond with the correct MIME type
AND (if the page and WASM file are on different origins) proper CORS headers. For the Docker
container serving everything on the same origin, CORS is not required. But if the image is later
deployed behind a CDN or if the WASM assets are served from a different subdomain, the lack of
CORS headers will cause `instantiateStreaming()` to fail even with the correct MIME type.

Additionally: some security scanners flag missing `Cross-Origin-Embedder-Policy` (COEP) and
`Cross-Origin-Opener-Policy` (COOP) headers, which are required for `SharedArrayBuffer` but also
sometimes required by hosting platforms' security policies.

**Why it happens:**
The static game works fine with Vite's dev server (which sets these headers automatically for
development). The Docker/nginx production configuration is written fresh and these headers are
omitted since they're not obviously required for a single-origin static site.

**How to avoid:**
Add security headers to the nginx config for production hardening:
```nginx
location / {
    add_header Cross-Origin-Opener-Policy "same-origin";
    add_header Cross-Origin-Embedder-Policy "require-corp";
    add_header X-Content-Type-Options "nosniff";
}
```

For WASM specifically, `X-Content-Type-Options: nosniff` ensures the browser respects the
`Content-Type` header and doesn't sniff the binary as something else.

**Warning signs:**
- WASM loads fine in the Docker container locally but fails when deployed to a CDN
- Browser console shows CORS errors referencing `.wasm` files
- Security scanner reports missing COEP/COOP headers

**Phase to address:** nginx configuration phase — include security headers from the start;
test with a security header scanner before calling deployment complete.

---

### Pitfall 4: Vite SPA Routing — Direct URL Access Returns nginx 404

**What goes wrong:**
This project is a single-page application with a single `index.html`. nginx serves static files,
and a request to `/` returns `index.html` correctly. But if any path is accessed directly (e.g.,
a cached bookmark, a reload on a non-root path), nginx returns 404 because there's no file at
that path on disk.

For *this specific game* (no client-side routing, single HTML file), this pitfall is LOW severity —
the user always lands on `/` and there are no sub-routes. But if the Docker image is ever used
as a template or extended, the missing `try_files` directive will cause confusion.

**Why it happens:**
Default nginx config tries to serve files at the literal request path. SPAs use client-side
routing and don't have corresponding files. Vite's build output is always `index.html` at root.

**How to avoid:**
Add `try_files` to the nginx location block:
```nginx
location / {
    root   /usr/share/nginx/html;
    index  index.html;
    try_files $uri $uri/ /index.html;
}
```

This serves real files (like `assets/tic_tac_toe_bg-*.wasm`) when they exist, and falls back to
`index.html` for everything else. This is safe for Vite's content-hashed asset files because they
always exist in `dist/assets/`.

**Warning signs:**
- `curl http://localhost/something` returns nginx 404 instead of `index.html`
- Hard refresh in browser on any non-root URL shows nginx 404 page

**Phase to address:** nginx configuration phase — include `try_files` in the initial config
even if this game doesn't have sub-routes.

---

### Pitfall 5: `target/` and `node_modules/` Included in Build Context, Bloating Context Transfer

**What goes wrong:**
`docker buildx build .` sends the entire working directory as the build context to the Docker
daemon. For a Rust + Node.js project, the working directory contains:
- `target/` — Rust compilation cache (can be **gigabytes** for a project with WASM targets)
- `node_modules/` — npm dependencies (typically 50–200MB)
- `pkg/` — previously built WASM output
- `dist/` — previously built Vite output

Sending gigabytes of build cache as context wastes time before the first `FROM` even executes.
On slow CI runners or when the Dockerfile doesn't use these directories (the Dockerfile installs
fresh), this is pure waste.

**Why it happens:**
`.dockerignore` is easy to forget for projects that didn't start Docker-first. Rust developers
know `target/` is large but may not realize Docker sees it. Node developers know `node_modules/`
but may focus on Rust-side ignores.

**How to avoid:**
Create a `.dockerignore` file covering both ecosystems:
```
# Rust build artifacts (can be gigabytes)
target/

# Previously built outputs (Dockerfile rebuilds these)
pkg/
dist/

# Node.js dependencies (installed fresh inside Docker)
node_modules/

# Development and test files
test.html
*.log
.git/
.gitignore
.github/

# Planning and documentation (not needed in image)
.planning/
AGENTS.md
README.md

# Editor and OS files
.DS_Store
.vscode/
*.swp
```

**Critical:** Both `target/` AND `node_modules/` must be ignored for a Rust+Node project.
Ignoring only one is a common mistake.

**Warning signs:**
- Docker build shows `Sending build context to Docker daemon  2.5GB` (or similar large size)
- Build is slow before any `RUN` step executes
- `docker build -f Dockerfile --no-cache .` takes 60+ seconds just on context transfer

**Phase to address:** `.dockerignore` creation phase — create BEFORE the first `docker build`.
Verify with: `docker build --dry-run .` or inspect context size in build output.

---

### Pitfall 6: `Cargo.lock` Not Copied into Build Stage — Non-Reproducible Rust Builds

**What goes wrong:**
The Dockerfile copies `Cargo.toml` to install dependencies before copying source code (a common
layer caching pattern). If `Cargo.lock` is omitted from the COPY instruction (or from the build
context via `.dockerignore`), Cargo performs dependency **resolution** on every build and may
pull newer patch versions of crates. This breaks build reproducibility and can introduce
surprising compilation failures in CI when a new crate version has a breaking change.

**Why it happens:**
Developers familiar with Node.js Docker caching use the `COPY package.json package-lock.json ./`
+ `RUN npm ci` pattern, which correctly pins to the lockfile. The Rust equivalent is less
well-known: both `Cargo.toml` AND `Cargo.lock` must be copied before `RUN cargo` to get pinned
dependencies.

**How to avoid:**
```dockerfile
# Copy both manifest and lockfile for reproducible dependency resolution
COPY Cargo.toml Cargo.lock ./
# Then copy source
COPY src/ ./src/
RUN wasm-pack build --target web --release
```

Also ensure `Cargo.lock` is **not** in `.dockerignore` and **is** committed to git.
For library crates, `Cargo.lock` is sometimes gitignored (Cargo convention for published libs),
but for applications and binaries (including WASM games), it must be committed and included.

**Warning signs:**
- Build succeeds locally but fails in CI with "failed to select a version for `xyz`"
- Different build runs produce different `.wasm` binary sizes
- `cargo update` was not run but a dependency version changed in the Docker image

**Phase to address:** Dockerfile creation phase — include `Cargo.lock` in the first COPY.

---

### Pitfall 7: Multi-Arch Build Fails to Push Without `--push` Flag (Images Not Accessible Locally)

**What goes wrong:**
Running:
```bash
docker buildx build --platform linux/amd64,linux/arm64 -t user/app:latest .
```
without `--push` (or `--output type=image,push=true`) succeeds but **does not load the multi-arch
manifest into the local Docker image store**. The output exists only in buildx's internal cache.
`docker images` shows nothing. `docker run user/app:latest` fails with "image not found."

Additionally, attempting `--load` with multi-arch builds fails because the local Docker image
store (classic storage drivers) cannot represent multi-platform manifests:
```
error: docker exporter does not currently support exporting manifest lists
```

**Why it happens:**
Multi-arch manifests require a registry to store the manifest list. The local Docker engine
traditionally stores single-platform images. This is a fundamental constraint of buildx multi-arch
builds — they must be pushed to a registry or use the containerd image store.

**How to avoid:**
In GitHub Actions, always use `--push` (via `docker/build-push-action` with `push: true`):
```yaml
- name: Build and push
  uses: docker/build-push-action@v7
  with:
    platforms: linux/amd64,linux/arm64
    push: true
    tags: ${{ steps.meta.outputs.tags }}
```

For local testing, build single-platform with `--load`:
```bash
# Local test: single platform
docker buildx build --platform linux/amd64 --load -t user/app:test .
docker run --rm -p 8080:80 user/app:test
```

**Warning signs:**
- `docker buildx build --platform ... .` exits 0 but `docker images` shows nothing
- `error: docker exporter does not currently support exporting manifest lists` when using `--load`
- Confusion about whether the build "worked" because it exited without error

**Phase to address:** GitHub Actions workflow phase — always use `push: true` in the workflow;
document local testing command with single-platform `--load`.

---

### Pitfall 8: Docker Hub Credentials Stored as Plaintext or Using Password Instead of Access Token

**What goes wrong:**
Using Docker Hub account **password** instead of an **access token** in GitHub Actions secrets:
- Passwords have full account access — if the secret leaks, the attacker owns the Docker Hub account
- Docker Hub access tokens can be scoped to specific repositories and revoked without changing the password
- GitHub Actions `docker/login-action` accepts both, making it easy to accidentally use the password

A separate mistake: using `${{ secrets.DOCKERHUB_USERNAME }}` for the username when it's not
actually a GitHub **secret** (it's a public username) — it should be a GitHub **variable**
(`${{ vars.DOCKERHUB_USERNAME }}`). Storing non-secret data as secrets obscures configuration
and doesn't provide security benefit.

**Why it happens:**
The Docker Hub "password" works in `docker login`, so developers use it. The distinction between
GitHub Actions `secrets` and `vars` is not prominent. Many tutorials use `${{ secrets.DOCKERHUB_USERNAME }}`
because it "works" — but the pattern leaks a public value through the secrets mechanism.

**How to avoid:**
1. Create a Docker Hub access token with scope: **Read & Write** for the specific repository
   (Settings → Security → Access Tokens)
2. Store the token in GitHub secrets as `DOCKERHUB_TOKEN`
3. Store the username as a GitHub **variable** (`vars.DOCKERHUB_USERNAME`), not a secret

```yaml
- name: Login to Docker Hub
  uses: docker/login-action@v4
  with:
    username: ${{ vars.DOCKERHUB_USERNAME }}    # variable, not secret
    password: ${{ secrets.DOCKERHUB_TOKEN }}     # access token, not password
```

**Warning signs:**
- GitHub Actions secrets list contains `DOCKERHUB_USERNAME` (should be a variable)
- Docker Hub access shows all-repositories access (should be repository-scoped)
- Security audit finds credentials with excessive permissions

**Phase to address:** GitHub Actions workflow phase — set up credentials before writing any
workflow code; use access tokens from day one.

---

### Pitfall 9: Tag `latest` Applied to Every Push, Including Work-in-Progress Commits

**What goes wrong:**
A naive CI workflow that pushes on every commit with `tags: user/app:latest` means every pushed
commit — including debugging commits, half-finished features, and commits that "accidentally" pass
CI — overwrites the `latest` tag on Docker Hub. Users who run `docker pull user/app` get whatever
was last pushed, not necessarily a real release.

For this project, the planned behavior is: **publish only on git tag push**. A manual `latest`
tag in a push-on-every-commit workflow defeats this entirely.

**Why it happens:**
Copy-pasting workflows from tutorials that show `tags: user/app:latest`. The `latest` tag in
Docker has special meaning (it's what `docker pull name` resolves to by default) but Docker Hub
doesn't enforce any semantics — anything can overwrite it at any time.

**How to avoid:**
Use `docker/metadata-action` with `type=semver` tags triggered only on `refs/tags/v*` pushes:

```yaml
on:
  push:
    tags:
      - 'v*.*.*'    # Only trigger on version tags like v1.2.0

jobs:
  docker:
    steps:
      - name: Docker meta
        id: meta
        uses: docker/metadata-action@v6
        with:
          images: user/tic-tac-toe
          tags: |
            type=semver,pattern={{version}}          # e.g. 1.2.0
            type=semver,pattern={{major}}.{{minor}}  # e.g. 1.2
            type=raw,value=latest,enable={{is_default_branch}}
```

With this configuration:
- `git tag v1.2.0 && git push --tags` → publishes `1.2.0`, `1.2`, and `latest`
- Regular commits → no Docker Hub push at all

**Warning signs:**
- GitHub Actions workflow triggers on `push:` to branches (not just tags)
- `tags: user/app:latest` is hardcoded in the workflow (not from metadata-action)
- Docker Hub shows recent push timestamps that don't correspond to releases

**Phase to address:** GitHub Actions workflow phase — use metadata-action from the start;
trigger on tag push only.

---

### Pitfall 10: Rust Multi-Stage Build Image Size Bloat from Build Dependencies

**What goes wrong:**
The Rust build stage image (`rust:slim` or `rust:alpine`) plus:
- Full Rust toolchain with std library (~600MB)
- wasm-pack binary and its dependencies (~50MB)
- Cargo build cache with all dependencies (~150MB for this project)
- wasm32-unknown-unknown target (~60MB)
- Node.js + npm (~100MB)
- node_modules (~50MB for this project)

... totals **~1GB+** in the build stage. Without multi-stage builds, this entire stack would
ship as the final image. With a correct multi-stage build, the **serve stage** is just:
- `nginx:alpine` base (~8MB)
- `dist/` static files (JS + CSS + WASM, ~80KB total for this project)

The final image should be **~8–10MB**. If it's larger, the multi-stage build has a mistake.

**Common mistakes that inflate the serve stage:**
1. `COPY . .` in the serve stage instead of `COPY --from=build /app/dist /usr/share/nginx/html`
2. Installing additional tools in the serve stage (curl, vim, build-essential)
3. Using `rust:latest` or `node:latest` as the serve stage base (not `nginx:alpine`)
4. Forgetting `--from=build` in the COPY — copies from the local filesystem into the serve layer

**How to avoid:**
```dockerfile
# Build stage — will NOT be included in the final image
FROM --platform=$BUILDPLATFORM rust:slim AS build
WORKDIR /app

# Install Node.js in the build stage only
RUN apt-get update && apt-get install -y curl nodejs npm && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

# Copy and build Rust
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN wasm-pack build --target web --release

# Copy and build JS
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html vite.config.js ./
RUN npm run build

# Serve stage — this is the final image
FROM nginx:alpine AS serve
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
```

Verify final image size:
```bash
docker buildx build --platform linux/amd64 --load -t tic-tac-toe:test .
docker image ls tic-tac-toe:test
# Should show ~10MB, not hundreds of MB
```

**Warning signs:**
- `docker image ls` shows the image as >50MB
- Build stage packages (cargo, rustup, npm, node_modules) appear in `docker exec` of the container
- `docker history` shows large layers in the final image that shouldn't be there

**Phase to address:** Dockerfile creation phase — verify image size immediately after first build;
fix before adding CI.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using `rust:latest` as build base (not pinned version) | Always uses newest Rust | Build may break when Rust updates (edition changes, API changes) | Never for CI; pin to `rust:1` or `rust:slim` |
| Building both platforms under QEMU (no `--platform=$BUILDPLATFORM`) | Simpler Dockerfile | arm64 build is 10–30× slower; may timeout in CI | Never — use `--platform=$BUILDPLATFORM` always |
| Using Docker Hub password instead of access token | Works immediately | Full account exposure if secret leaks | Never — access tokens are free and scoped |
| `latest` tag on every commit push | Simple workflow | Latest tag is unstable; users get WIP builds | Never for a public image |
| No `.dockerignore` | No extra file to maintain | Multi-GB build context; slow builds | Never — create `.dockerignore` with first Dockerfile |
| Single-stage Dockerfile (no multi-stage) | Simpler file | ~1GB final image instead of ~10MB | Never — multi-stage is the standard for compiled projects |
| `RUN npm install` instead of `npm ci` | Allows version range resolution | Non-reproducible builds; ignores package-lock.json | Never in Docker builds |

---

## Integration Gotchas

Common mistakes at the boundary between Rust and Node.js in the same Dockerfile.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| wasm-pack output → Vite input | `wasm-pack build` writes to `pkg/`; `COPY` only copies `src/` and misses `pkg/` for Vite | Run `wasm-pack build` before `npm run build`; both in the same stage; Vite reads from `pkg/` in the same directory |
| Two package managers | Installing Node.js in the same `RUN` as Rust tools, then cleaning up later | Install Node.js early, clean apt cache in the same `RUN` layer to avoid layer size inflation |
| COPY ordering for cache efficiency | `COPY . .` before `npm ci` — any source change invalidates the npm cache layer | Copy package files first: `COPY package.json package-lock.json ./` + `RUN npm ci`, then `COPY . .` |
| Cargo.lock in `.dockerignore` | Accidentally ignoring `Cargo.lock` makes builds non-reproducible | Explicitly list what to ignore; never add `*.lock` glob patterns |
| wasm-pack binary not on PATH | After `curl | sh` install, wasm-pack is in `~/.cargo/bin/` which may not be on PATH for subsequent RUN layers | Add `ENV PATH="/root/.cargo/bin:${PATH}"` after installing wasm-pack |

---

## Performance Traps

Patterns that slow CI significantly.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| No Cargo build cache between CI runs | Every build downloads + compiles all dependencies from scratch (2–5 min) | Use `docker/build-push-action` with `cache-from: type=gha` + `cache-to: type=gha,mode=max` | Every CI run without cache |
| No npm ci cache between CI runs | npm downloads all packages each run | Cache `/root/.npm` in the build stage via BuildKit cache mount: `--mount=type=cache,target=/root/.npm` | Every CI run without cache |
| Both platforms built sequentially on one runner | arm64 QEMU build adds 10–30× time | Use `--platform=$BUILDPLATFORM` so Rust only compiles once natively | Every multi-arch build |
| `mode=min` for GHA cache | Only caches the result layer, not intermediate layers | Use `cache-to: type=gha,mode=max` to cache all layers | On cache miss — full rebuild |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Using Docker Hub password instead of access token in GitHub secrets | Full Docker Hub account compromise if secret leaks | Create repository-scoped access token; store as `DOCKERHUB_TOKEN` |
| No `X-Content-Type-Options: nosniff` header in nginx | Browser MIME sniffing could misinterpret WASM binary | Add `add_header X-Content-Type-Options nosniff;` to nginx config |
| Running nginx as root (default) in container | Privilege escalation if nginx is compromised | Use `nginx:alpine` which drops to `nginx` user (uid 101) for worker processes by default |
| Serving sensitive build artifacts (Cargo.lock, package.json exposed) | Leaks dependency versions for vulnerability scanning | Multi-stage build ensures only `dist/` is in the serve stage — build files never ship |
| Public Docker Hub image with no README / description | Confused users may not know if it's an official release | Add Docker Hub description in workflow using `docker/hub-description-action` (optional but good practice) |

---

## "Looks Done But Isn't" Checklist

Things that appear to work in the happy path but have hidden gaps.

- [ ] **WASM MIME type:** Build succeeds and image runs — verify: `curl -I http://localhost/assets/*.wasm | grep content-type` shows `application/wasm`, not `application/octet-stream`
- [ ] **Multi-arch manifest:** Image is pushed — verify: `docker manifest inspect user/app:latest` shows both `linux/amd64` and `linux/arm64` manifests
- [ ] **Image size:** Build succeeds — verify: `docker image ls` shows <20MB for the serve image
- [ ] **Cargo.lock included:** Build is reproducible — verify: `Cargo.lock` is present in the build context (not in `.dockerignore`, not gitignored)
- [ ] **arm64 native in serve stage:** Multi-arch built — verify: `docker run --platform linux/arm64 --rm user/app uname -m` shows `aarch64`
- [ ] **Tag policy:** Workflow runs on tag push — verify: pushing a commit to `main` does NOT trigger a Docker Hub push
- [ ] **Credential type:** Docker Hub credentials set up — verify: secrets use access token, not password; username is a `vars` not a `secret`
- [ ] **SPA routing:** nginx config deployed — verify: `curl http://localhost/nonexistent-path` returns `index.html` content (200), not nginx 404
- [ ] **wasm-pack PATH:** Build runs in CI — verify: `wasm-pack --version` succeeds in the build stage without `command not found`
- [ ] **No build artifacts in final image:** Multi-stage complete — verify: `docker exec <container> ls /usr/share/nginx/html` shows only `index.html` and `assets/`, not `src/`, `target/`, or `Cargo.toml`

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| QEMU build timeout | LOW — add `--platform=$BUILDPLATFORM` to build stage FROM | Add `FROM --platform=$BUILDPLATFORM` prefix; re-push; new build completes in normal time |
| WASM MIME type wrong in production | LOW — nginx config fix, rebuild image | Add explicit MIME type to nginx config; rebuild and push; users just need to refresh |
| Wrong credentials in CI (password, not token) | LOW — update GitHub secret | Generate new Docker Hub access token; update `DOCKERHUB_TOKEN` secret; revoke old password usage |
| `latest` tag on wrong commit | MEDIUM — must push a correct release | Create correct version tag; push; metadata-action will regenerate `latest` pointing to correct release |
| Build context sending gigabytes | LOW — add `.dockerignore` | Create `.dockerignore`; re-run build; context drops to kilobytes immediately |
| Bloated final image (build deps in serve stage) | MEDIUM — rewrite Dockerfile | Fix multi-stage copy path; rebuild; push; existing pulled images need replacement |
| Multi-arch manifest not created (only single arch pushed) | MEDIUM — rebuild and push with `--platform` | Add both platforms to buildx command; push new manifest; docker pull will get multi-arch manifest |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| QEMU slowness / arm64 compile | Dockerfile creation | Build time for arm64 stage is <2× amd64 time |
| WASM MIME type | nginx configuration | `curl -I http://localhost/assets/*.wasm` shows `application/wasm` |
| CORS / security headers | nginx configuration | Security header scanner returns no critical findings |
| SPA routing / nginx 404 | nginx configuration | All paths return 200 with index.html content |
| `.dockerignore` missing | Before first `docker build` | Build context size is <1MB |
| `Cargo.lock` excluded | Dockerfile creation | Two successive builds produce identical `.wasm` binary checksums |
| Multi-arch not pushed | GitHub Actions workflow | `docker manifest inspect` shows amd64 + arm64 |
| Docker Hub password vs token | Credentials setup | GitHub secret name ends in `_TOKEN`, not `_PASSWORD` |
| Tag `latest` on every push | GitHub Actions workflow | Push to main branch; verify no Docker Hub push occurs |
| Image size bloat | Dockerfile creation | `docker image ls` shows <20MB |

---

## Sources

- [Docker: Multi-platform image docs](https://docs.docker.com/build/building/multi-platform/) — HIGH confidence (official, 2026)
  — `--platform=$BUILDPLATFORM` pattern, QEMU limitations warning
- [Docker: GitHub Actions multi-platform](https://docs.docker.com/build/ci/github-actions/multi-platform/) — HIGH confidence (official, 2026)
  — `docker/setup-qemu-action`, `docker/build-push-action` patterns
- [Docker: GitHub Actions cache backend](https://docs.docker.com/build/cache/backends/gha/) — HIGH confidence (official, 2026)
  — `type=gha,mode=max` cache configuration
- [docker/metadata-action README](https://github.com/docker/metadata-action) — HIGH confidence (official, 2026)
  — Tag semver patterns, `latest` tag management, `vars.DOCKERHUB_USERNAME` vs `secrets.DOCKERHUB_TOKEN`
- [MDN: WebAssembly Loading and Running](https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Loading_and_running) — HIGH confidence (official, updated 2025-08-26)
  — `instantiateStreaming()` requires correct MIME type
- [nginx Docker Hub official image](https://hub.docker.com/_/nginx) — HIGH confidence (official, 2026)
  — nginx:alpine architecture support, user/group IDs, configuration patterns
- Codebase inspection: `Cargo.toml`, `package.json`, `vite.config.js`, `dist/assets/` — HIGH confidence (direct)
  — wasm-pack output to `pkg/`, Vite output to `dist/`, `.wasm` file naming pattern

---

## Prior Version Reference

The v1.1 PITFALLS.md (browser-side polish: CSS animations, Web Audio, localStorage, dark mode)
is preserved for reference. That document is specific to browser-side JS/CSS concerns and does
not overlap with this Docker deployment document.

---
*Pitfalls research for: v1.2 Docker multi-architecture deployment milestone*
*Researched: 2026-04-14*
