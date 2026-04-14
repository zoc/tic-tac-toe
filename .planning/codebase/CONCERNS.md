# Codebase Concerns

**Analysis Date:** 2026-04-14

---

## Tech Debt

**Duplicated `WIN_LINES` constant:**
- Issue: The 8-element `WIN_LINES` array is defined independently in both `src/ai.rs` (line 130) and `src/board.rs` (line 28). The AI has its own private `check_winner()` that re-implements the same logic already in `board.rs`.
- Files: `src/ai.rs:130–139`, `src/board.rs:28–37`
- Impact: If win conditions ever changed (e.g., larger board variant), both must be updated in sync. Currently zero risk for a fixed 3×3 game, but a code smell.
- Fix approach: Export `WIN_LINES` from `board.rs` and remove the duplicate from `ai.rs`. Have the AI call `game.status()` to check for terminal states instead of re-checking with its own `check_winner()`.

**`Cargo.toml` uses major-only version pins:**
- Issue: All Rust dependencies use major-only semver constraints (e.g., `wasm-bindgen = "0.2"`, `rand = "0.10"`). This allows automatic minor/patch updates within a major version on a fresh `cargo fetch` without a lockfile.
- Files: `Cargo.toml:10–14`
- Impact: `Cargo.lock` is committed and tracked in git, which mitigates this for reproducible builds. However, any operation that runs `cargo update` (or a new dev environment without the lockfile) can pull in newer patch versions silently.
- Fix approach: Acceptable as-is given the committed `Cargo.lock`. No immediate action needed.

**`package.json` uses `^` caret version ranges:**
- Issue: `vite: "^8.0.8"` and `vite-plugin-wasm: "^3.6.0"` allow minor version updates. `npm ci` pins to `package-lock.json`, so Docker builds are reproducible, but `npm install` (used in local development) will silently upgrade within the caret range.
- Files: `package.json:11–13`
- Impact: Low — Vite 8.x is stable. No known breaking minor releases in the range.
- Fix approach: Pin exact versions in `package.json` for stricter control, or accept the caret range since `package-lock.json` provides the pin.

**`src/main.rs` CLI binary exists but is out of scope:**
- Issue: `src/main.rs` is a functional CLI-based tic-tac-toe implementation (109 LOC) that uses `unwrap()` on stdin/stdout I/O (lines 79, 82, 102). This binary is never built in the web/Docker workflow (`wasm-pack build` builds only the lib target), but it compiles as part of `cargo test` and `cargo build`.
- Files: `src/main.rs:79`, `src/main.rs:82`, `src/main.rs:102`
- Impact: The `unwrap()` calls on stdin would panic on an I/O error in the CLI context. Harmless for the browser game, but the binary adds compilation overhead. No `[[bin]]` section in `Cargo.toml` — it auto-detects `src/main.rs`.
- Fix approach: No action needed for the browser game. If the CLI binary is truly dead code, adding `[[bin]] name = "tic-tac-toe-cli"` and documenting it — or removing `src/main.rs` — would clarify intent.

---

## Known Bugs

**Duplicate `role` attribute set on cells during initialization:**
- Symptoms: Each cell is created with `role="gridcell"` (line 94), then immediately overwritten with `role="button"` (line 96). After a piece is placed, `role="gridcell"` is set again (lines 114, 120). The final state has `role="gridcell"` for taken cells, but during cell construction, `role="button"` is the last value set.
- Files: `src/main.js:94–97`
- Trigger: Happens on every `renderBoard()` call that creates cells from scratch (new game or first load).
- Impact: Minor accessibility concern — screen readers may announce cells inconsistently during the brief render window. Empty cells end up as `role="button"` (correct for interactive elements) but transition to `role="gridcell"` after a move. The intent appears inconsistent.
- Workaround: None currently. The game is playable; the ARIA semantics are slightly confused.

**`audioCtx.resume()` result not awaited in `getCtx()`:**
- Symptoms: In `src/audio.js:24`, `audioCtx.resume()` returns a `Promise` that is not awaited. On Safari (which is strict about AudioContext suspension), the first sound after a long idle period may fail to play or be clipped.
- Files: `src/audio.js:22–26`
- Trigger: Safari-specific; occurs when the tab has been idle and the browser auto-suspends the AudioContext.
- Impact: Low — the comment in the code acknowledges this. The oscillator nodes are scheduled slightly after the resume call, and the context is usually running by then. Occasional missed sounds on Safari.
- Workaround: Sound is optional and mutable. User can mute to avoid the issue.

**`test.html` is committed and served by Vite dev server:**
- Symptoms: `test.html` at the project root is excluded from the Docker build context via `.dockerignore` but is served by `npm run dev` at `http://localhost:5173/test.html`. It imports directly from `./pkg/tic_tac_toe.js` (not through Vite's module resolution), so it only works after `wasm-pack build` has been run.
- Files: `test.html:22`
- Trigger: Running `npm run dev` without having built the WASM package first.
- Impact: Confusing for new contributors — `test.html` exists but may fail with a module-not-found error.
- Workaround: Run `wasm-pack build --target web --release` before `npm run dev`.

---

## Security Considerations

**`curl | sh` pattern for wasm-pack installation in Dockerfile:**
- Risk: Line 16 of the Dockerfile fetches and executes an installer script from `rustwasm.github.io` over HTTPS at build time: `RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`. This is a standard wasm-pack installation method but represents a supply chain dependency on an external URL being available and uncompromised during the build.
- Files: `Dockerfile:16`
- Current mitigation: The URL uses HTTPS (`-sSf` does not skip TLS verification). The domain is the official Rust WASM working group domain.
- Recommendations: For maximum build reproducibility and supply chain hardness, prefer `cargo install wasm-pack --locked` (pins to the version in Cargo.lock) or pin the installer URL to a specific release tag. The `curl | sh` pattern is currently the officially recommended wasm-pack install method, making it acceptable for this project's risk profile.

**GitHub Actions uses mutable action version tags (not SHA pins):**
- Risk: The workflow references `actions/checkout@v4`, `docker/setup-qemu-action@v3`, `docker/setup-buildx-action@v3`, `docker/login-action@v3`, `docker/metadata-action@v5`, `docker/build-push-action@v6`. These are mutable tags — a compromised upstream action maintainer could change what `@v3` points to.
- Files: `.github/workflows/docker.yml:26–60`
- Current mitigation: All actions are from `actions/` and `docker/` — well-maintained official repositories with low compromise risk. Docker official actions are widely used.
- Recommendations: For stricter supply chain security, pin to SHA: e.g., `actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683` (v4). This is optional hardening for a personal project.

**Docker Hub credentials configuration:**
- Risk: The workflow correctly uses `vars.DOCKERHUB_USERNAME` (GitHub variable, not secret) and `secrets.DOCKERHUB_TOKEN` (GitHub secret, access token). However, these must be configured manually in GitHub before the workflow runs. A misconfiguration (e.g., using the account password instead of a token) would expose full Docker Hub account access.
- Files: `.github/workflows/docker.yml:39–40`
- Current mitigation: Workflow comments clearly document the correct setup (access token, not password; variable for username, not secret). The pattern follows Docker Hub best practices.
- Recommendations: Verify credentials are repository-scoped in Docker Hub settings when setting up for the first time.

**No Content Security Policy (CSP) header:**
- Risk: `nginx.conf` sets `X-Content-Type-Options`, `X-Frame-Options`, `Cross-Origin-Opener-Policy`, and `Cross-Origin-Embedder-Policy`, but no `Content-Security-Policy` header. For a static game with no user-generated content or auth, the risk is minimal.
- Files: `nginx.conf:14–17`
- Current mitigation: No user input is ever rendered as HTML. XSS is explicitly guarded in `src/main.js:308–317` (DOM text nodes, not innerHTML). The game has no login, no cookies, no sensitive data.
- Recommendations: Out of scope per REQUIREMENTS.md. Would be a good addition if the project ever adds user features.

---

## Performance Bottlenecks

**Minimax runs to full depth on every AI turn:**
- Problem: The minimax function in `src/ai.rs:76–127` explores the full game tree without alpha-beta pruning. For a 3×3 board, the maximum search space is 9! = 362,880 leaf nodes, but the effective search space is much smaller (~5,478 unique positions). This is fast enough in WASM — benchmarks typically show <1ms — but it is unoptimized.
- Files: `src/ai.rs:76–127`
- Cause: Alpha-beta pruning was not implemented. The 300–800ms thinking delay in `src/main.js:77–78` completely masks the actual computation time.
- Improvement path: Not needed for this project. Adding alpha-beta pruning would be the standard optimization if the board were larger.

**`renderBoard()` builds all 9 cells from scratch on reset:**
- Problem: `resetGame()` in `src/main.js:267` sets `boardEl.innerHTML = ''`, which causes `renderBoard()` to rebuild all 9 `div.cell` elements from scratch. This is intentional to reset animation state, but it forces 9 DOM node creations per reset.
- Files: `src/main.js:88–99`, `src/main.js:267`
- Cause: Deliberate design choice to trigger pop-in animations only on newly placed pieces (documented in ARCHITECTURE decision). Full board rebuild was chosen over per-cell animation state tracking.
- Improvement path: Not needed. 9 DOM nodes is negligible. The current approach is well-commented and intentional.

---

## Fragile Areas

**Dockerfile stub `src/lib.rs` may mask `main.rs` compilation:**
- Files: `Dockerfile:29`
- Why fragile: The Docker layer caching pattern creates a stub `src/lib.rs` with `echo '#[allow(dead_code)] fn main() {}' > src/lib.rs` before `cargo fetch`. Since `Cargo.toml` has `crate-type = ["cdylib", "rlib"]` with no explicit `[[bin]]` section, Cargo auto-detects `src/main.rs` as a binary target. The stub only creates `lib.rs`, not `main.rs`. Running `cargo fetch` against this stub should succeed because fetch only resolves dependencies, not compile targets. However, `cargo build` would fail without `src/main.rs`. The subsequent `COPY src/ ./src/` overwrites the stub with the real source before `wasm-pack build` runs.
- Safe modification: Always keep the stub layer before the `COPY src/` layer. Do not add `RUN cargo build` between the stub creation and the real source copy.
- Test coverage: The `COPY src/ ./src/` step overwrites the stub, so `wasm-pack build` always compiles real code. The stub is only used for `cargo fetch`.

**JavaScript DOM queries are unchecked at module load time:**
- Files: `src/main.js:37–44`
- Why fragile: All 8 DOM element references (`boardEl`, `statusEl`, `restartBtn`, `muteBtn`, `scoreWinsEl`, `scoreLossEl`, `scoreDrawEl`, `winLineEl`) are queried at module top level with `getElementById()`. If any element is missing from `index.html` (e.g., accidentally deleted), the reference is `null` and downstream code that calls methods on it will throw a TypeError at runtime, not at load time.
- Safe modification: When editing `index.html`, ensure all 8 IDs remain: `board`, `status-message`, `restart-btn`, `mute-btn`, `score-wins`, `score-losses`, `score-draws`, `win-line`.
- Test coverage: No automated test covers the DOM wiring. Manual smoke test required after HTML edits.

**Win line CSS uses hardcoded `127.28%` diagonal width:**
- Files: `src/style.css:319`, `src/style.css:327`
- Why fragile: The diagonal win line length is hardcoded as `127.28%` (calculated as `√2 × 90%`). This is derived from the board's usable area being `5%–95%` on each axis. If the board padding percentages change (`left: 5%`, `width: 90%` on row win lines at lines 285–287), the diagonal width must also be recalculated. There is no CSS variable tying these values together.
- Safe modification: If adjusting board padding for win lines, recalculate diagonal width as `sqrt(2) * width_percentage`. Document the formula in a comment.
- Test coverage: Visual-only; no automated test.

**`computer_move()` returns `u8` with `255` as sentinel:**
- Files: `src/wasm_api.rs:81–88`
- Why fragile: The `computer_move()` function returns a `u8` with `255` as the "game over" sentinel value. The JavaScript side checks `compPos === NO_MOVE` where `NO_MOVE = 255` (defined in `src/main.js:11`). This works correctly because valid positions are 0–8, well within `u8` range. However, if the AI ever returned a value > 127 (impossible with a 9-cell board), the `pos as u8` cast on line 85 would silently truncate, not panic. The sentinel value is a magic number that requires coordinated changes across the Rust/JS boundary.
- Safe modification: Do not change the `255` sentinel without updating `NO_MOVE` in `src/main.js`. Consider adding a doc comment cross-referencing both files.
- Test coverage: `test.html:109–110` verifies the 255 sentinel. No automated JS test.

---

## Scaling Limits

**Not applicable:** The game is a static single-player application with fixed 3×3 board state. No server-side resources, no user data, no sessions. The nginx:alpine container serves static files; scaling is handled by adding replicas behind a load balancer if ever needed.

---

## Dependencies at Risk

**`wasm-pack` installed via curl installer — no version pin:**
- Risk: The Dockerfile installs whatever version of `wasm-pack` the installer script resolves to at build time. If `wasm-pack` releases a breaking change or the installer URL resolves differently over time, builds may break silently.
- Files: `Dockerfile:16`
- Impact: Build failure or changed behavior in CI. This could cause the GitHub Actions workflow to break without any code changes.
- Migration plan: Pin `wasm-pack` to a specific version with `cargo install wasm-pack@0.14.0 --locked`, or use the GitHub release URL directly: `curl -L https://github.com/rustwasm/wasm-pack/releases/download/v0.14.0/wasm-pack-init.sh -sSf | sh`.

**Node.js installed via `apt-get` without version pin:**
- Risk: `apt-get install nodejs` installs whatever Node.js version is in the Debian package repositories for the `rust:slim` base image (Debian bookworm). This is typically an older LTS version (Node 18 as of Debian bookworm). The project README specifies Node 20+. If the apt-provided Node.js is <20, Vite 8's build may fail.
- Files: `Dockerfile:10–13`
- Impact: Silent version mismatch. Vite 8 requires Node 18+ minimum, so Debian bookworm's Node 18 is technically compatible, but the README's "Node 20+" guidance is not enforced in Docker.
- Migration plan: Replace `apt-get install nodejs npm` with the NodeSource setup script to install a specific Node.js version (e.g., Node 20 LTS), or switch the base image to `node:20-slim` and install Rust/wasm-pack on top.

**`rust:slim` base image is not version-pinned:**
- Risk: `FROM --platform=$BUILDPLATFORM rust:slim` pulls the latest stable Rust in the `slim` tag. Rust editions and API changes are generally backward-compatible within the 2021 edition, but a major Rust release that changes the `rand` or `wasm-bindgen` API surface could break the build.
- Files: `Dockerfile:4`
- Impact: Builds are not bit-for-bit reproducible over time. `Cargo.lock` pins Rust crate versions but not the Rust compiler version.
- Migration plan: Pin to `rust:1.77-slim` or a specific stable release to guarantee compiler version. For maximum reproducibility, add a `rust-toolchain.toml` file pinning the toolchain version — this is respected by both `rustup` and the `rust:slim` Docker image.

---

## Missing Critical Features

**No `HEALTHCHECK` in Dockerfile (DOCK-06 open):**
- Problem: The `Dockerfile` has no `HEALTHCHECK` instruction. DOCK-06 requires the container to respond healthy to a health check.
- Files: `Dockerfile`
- Blocks: DOCK-06 requirement validation; Phase 9 success criterion 5.

**No gzip compression in `nginx.conf` (DOCK-05 open):**
- Problem: `nginx.conf` has no `gzip` directive. DOCK-05 requires gzip compression for HTML, CSS, JS, and WASM responses.
- Files: `nginx.conf`
- Blocks: DOCK-05 requirement validation.

**No `Cache-Control: no-cache` for `index.html` (DOCK-04 open):**
- Problem: `nginx.conf` sets `Cache-Control: public, immutable` for `/assets/` only. There is no explicit cache header for `index.html` at the root `location /` block. Without `no-cache`, browsers may cache a stale `index.html` after a Docker image update, preventing users from seeing the new version.
- Files: `nginx.conf:21–23`
- Blocks: DOCK-04 requirement validation.

**No CI test step in GitHub Actions workflow:**
- Problem: `.github/workflows/docker.yml` only builds and pushes the Docker image. There is no step that runs `cargo test` or validates the game logic before the image is built. A regression in Rust game logic would not be caught by CI until manual testing.
- Files: `.github/workflows/docker.yml`
- Blocks: No current requirement, but a notable gap for confidence on releases.

**No favicon:**
- Problem: No `favicon.ico` or `<link rel="icon">` tag in `index.html`. Browsers make a request to `/favicon.ico` automatically, and nginx returns `index.html` (due to `try_files`) for this path, which is not a valid favicon. This generates a 404-equivalent in browser DevTools (nginx logs an empty response for the `access_log off` location).
- Files: `index.html`, `nginx.conf:37–40`
- Blocks: Nothing functional. Minor polish gap.

---

## Test Coverage Gaps

**No automated tests for the WASM API layer (`src/wasm_api.rs`):**
- What's not tested: `WasmGame::new()`, `make_move()`, `get_board()`, `current_player()`, `get_status()`, `get_winner()`, `get_winning_positions()`, `computer_move()`, `reset()` — all exported functions.
- Files: `src/wasm_api.rs`
- Risk: A serialization bug at the JS/WASM boundary (e.g., wrong u8 encoding, wrong Vec length) would not be caught by `cargo test`. The manual `test.html` covers this informally but is not automated.
- Priority: Medium. `test.html` provides manual coverage. `wasm-bindgen-test` is in `[dev-dependencies]` but unused.

**No automated tests for JavaScript UI logic (`src/main.js`, `src/audio.js`):**
- What's not tested: `handleCellClick()`, `resetGame()`, `renderBoard()`, `handleGameOver()`, `showWinLine()`, score persistence, mute toggle, thinking delay cancellation.
- Files: `src/main.js`, `src/audio.js`
- Risk: Regressions in event handling, score logic, or UI state are only caught by manual play. The `isProcessing` guard (double-click prevention) and `thinkingTimer` cancellation on reset are particularly hard to validate manually.
- Priority: Low for this project scope. The JS layer is ~318 LOC of DOM manipulation — a browser integration test (e.g., Playwright) would provide coverage but adds significant toolchain complexity.

**AI randomness makes `test_ai_beatable_in_100_games` potentially flaky:**
- What's not tested: The test at `src/ai.rs:253` asserts "at least 1 human win in 100 random-play games." With 25% mistake rate and random human play, the probability of 0 human wins in 100 games is astronomically low, but the test is technically non-deterministic. `rand::rng()` is seeded from OS entropy.
- Files: `src/ai.rs:253–295`
- Risk: Extremely unlikely to fail (<0.001% per run). Not a practical concern.
- Priority: Low. A deterministic seed (`StdRng::seed_from_u64(42)`) would make the test fully reproducible.

---

*Concerns audit: 2026-04-14*
