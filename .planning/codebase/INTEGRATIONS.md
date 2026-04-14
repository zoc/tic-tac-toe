# External Integrations

**Analysis Date:** 2026-04-14

## APIs & External Services

**None.** This is a fully self-contained static web application. There are no external API calls, no network requests at runtime, and no third-party services consumed by the running game.

## Data Storage

**Databases:**
- None. No server-side database.

**Client-side Persistence (localStorage):**
- Score data — key `ttt-score`, value: `{ wins, losses, draws }` JSON object
  - Read: `localStorage.getItem('ttt-score')` on module load in `src/main.js`
  - Write: `localStorage.setItem('ttt-score', ...)` after each game ends
  - Graceful fallback: `try/catch` handles `SecurityError` in private browsing mode
- Mute preference — key `ttt-muted`, value: `"true"` or `"false"` string
  - Read: `localStorage.getItem('ttt-muted')` on module load in `src/audio.js`
  - Write: `localStorage.setItem('ttt-muted', ...)` on toggle
  - Graceful fallback: `try/catch` handles `SecurityError` / quota exceeded

**File Storage:**
- None.

**Caching:**
- HTTP caching via nginx headers (production only):
  - `/assets/*` — `Cache-Control: public, immutable`, `Expires: 1y` (content-hashed filenames guarantee cache busting)
  - All other paths — default nginx cache headers
  - Configured in `nginx.conf`

## Authentication & Identity

- None. No user accounts, no login, no auth provider.

## Browser APIs Used

**Web Audio API** (`src/audio.js`):
- `AudioContext` — lazy singleton, created on first user gesture to satisfy browser autoplay policy
- `AudioContext.createOscillator()` — synthesises all sound effects (no audio files)
- `AudioContext.createGain()` — envelope/fade-out to prevent click/pop
- `AudioContext.resume()` — handles Safari's aggressive auto-suspend behaviour
- 5 named sounds: `move`, `computerMove`, `win`, `loss`, `draw`

**localStorage API** (`src/main.js`, `src/audio.js`):
- Score persistence: key `ttt-score`
- Mute preference: key `ttt-muted`
- Both guarded with `try/catch` for private browsing compatibility

**DOM APIs** (`src/main.js`):
- `document.getElementById()` — all DOM references queried once at startup
- `Element.addEventListener()` — click and keydown on board; click on restart/mute buttons
- `Element.setAttribute()` / `Element.classList` — ARIA attributes and CSS state
- `setTimeout()` / `clearTimeout()` — computer "thinking" delay (300–800ms randomised)
- `document.createElement()` / `Element.replaceChildren()` — error overlay on WASM load failure

**WebAssembly API** (`src/main.js`):
- `init()` — async WASM module initialisation (from `pkg/tic_tac_toe.js`)
- `WasmGame` — exported Rust class, instantiated as `new WasmGame()`
- WASM bridge methods called: `get_board()`, `get_status()`, `get_winner()`, `get_winning_positions()`, `current_player()`, `make_move()`, `computer_move()`, `reset()`
- WASM binary loaded from: `pkg/tic_tac_toe_bg.wasm` (dev) / `assets/tic_tac_toe_bg-*.wasm` (prod)

**CSS Features** (`src/style.css`):
- CSS Custom Properties (variables) — theme tokens (`--bg`, `--accent`, etc.)
- CSS Grid — 3×3 board layout
- CSS Animations — `cell-pop` (piece placement), `win-draw` / `win-draw-col` / `win-draw-diag-*` (win line)
- `@media (prefers-color-scheme: light)` — automatic light/dark theme switching
- `@media (prefers-reduced-motion: no-preference)` — animations gated on motion preference

## Monitoring & Observability

**Error Tracking:**
- None. No external error tracking service.

**Logging:**
- Rust panics forwarded to browser `console.error()` via `console_error_panic_hook` (initialised via `#[wasm_bindgen(start)]` in `src/wasm_api.rs`)
- WASM load failures rendered as an in-page error overlay (`src/main.js` `main().catch(...)`)
- No structured logging; no log aggregation service.

## CI/CD & Deployment

**Hosting:**
- Docker container running `nginx:alpine`
- Docker Hub registry: `$DOCKERHUB_USERNAME/tic-tac-toe`
- Image tags: semver (`1.2.0`, `1.2`), plus `latest` on default branch
- Multi-arch: `linux/amd64`, `linux/arm64`

**CI Pipeline — GitHub Actions** (`.github/workflows/docker.yml`):
- Trigger: push of version tags matching `v*.*.*`
- Runner: `ubuntu-latest`
- Steps: checkout → QEMU setup → Docker Buildx setup → Docker Hub login → metadata (tags/labels) → build & push
- Layer caching: GitHub Actions cache (`type=gha`, `mode=max`) for Rust deps and npm deps
- Actions used:
  - `actions/checkout@v4`
  - `docker/setup-qemu-action@v3`
  - `docker/setup-buildx-action@v3`
  - `docker/login-action@v3`
  - `docker/metadata-action@v5`
  - `docker/build-push-action@v6`

**Required GitHub secrets/variables:**
- `secrets.DOCKERHUB_TOKEN` — Docker Hub access token (read+write)
- `vars.DOCKERHUB_USERNAME` — Docker Hub username (public variable)

## Webhooks & Callbacks

**Incoming:** None.
**Outgoing:** None.

## Environment Configuration

**Required environment variables at runtime:** None — purely static client-side application.

**Required for CI only:**
- `DOCKERHUB_TOKEN` (GitHub secret)
- `DOCKERHUB_USERNAME` (GitHub variable)

**No `.env` file** is used or required at any stage.

---

*Integration audit: 2026-04-14*
