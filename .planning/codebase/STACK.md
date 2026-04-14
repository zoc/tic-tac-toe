# Technology Stack

**Analysis Date:** 2026-04-14

## Languages

**Primary:**
- Rust (edition 2021) — game logic, AI, WASM API layer (`src/board.rs`, `src/ai.rs`, `src/wasm_api.rs`, `src/lib.rs`)
- JavaScript (ES modules, ES2022+) — frontend rendering, DOM, event handling, audio (`src/main.js`, `src/audio.js`)

**Secondary:**
- CSS3 — styling, animations, responsive layout (`src/style.css`)
- HTML5 — single-page shell (`index.html`)

## Runtime

**Environment:**
- Browser (WebAssembly-capable): Chrome, Firefox, Safari, Edge (modern)
- Node.js 25.9.0 — build tooling only (not in production)

**Package Manager:**
- npm 11.12.1
- Lockfile: `package-lock.json` (lockfileVersion 3) — present and committed

## Frameworks

**Core:**
- None — vanilla JS + CSS only; no frontend framework

**Build/Dev:**
- Vite 8.0.8 — dev server, hot module reload, production bundler (`vite.config.js`)
- wasm-pack 0.14.0 — Rust → WASM compilation orchestrator (installed via installer script)
- wasm-bindgen 0.2.118 — Rust↔JS interop bindings (resolved in `Cargo.lock`)

**Testing:**
- Rust native test harness (`cargo test`) — unit tests embedded in `src/board.rs` and `src/ai.rs`
- wasm-bindgen-test 0.3.68 — WASM in-browser test runner (dev dependency in `Cargo.toml`)

## Key Dependencies

**Rust (resolved from `Cargo.lock`):**

| Crate | Version | Purpose |
|-------|---------|---------|
| `wasm-bindgen` | 0.2.118 | Generates JS/WASM bridge; `#[wasm_bindgen]` macros on exported types |
| `console_error_panic_hook` | 0.1.7 | Forwards Rust panics to `console.error()` with backtraces |
| `rand` | 0.10.1 | RNG for AI mistake injection (25% random move rate) |
| `getrandom` | 0.4.2 | WASM entropy backend for `rand`; requires `wasm_js` feature |
| `wasm-bindgen-test` | 0.3.68 | Dev-only: in-browser WASM unit testing |

**JavaScript (resolved from `package-lock.json`):**

| Package | Version | Purpose |
|---------|---------|---------|
| `vite` | 8.0.8 | Dev server + production bundler |
| `vite-plugin-wasm` | 3.6.0 | Enables `.wasm` ES module imports in Vite |

**Notable absent dependencies:**
- No `serde` / `serde-wasm-bindgen` — WASM API uses primitive types only (`Vec<u8>`, `Vec<usize>`, `u8`, `String`)
- No `vite-plugin-top-level-await` — replaced by `build.target: 'esnext'` in `vite.config.js`
- No `web-sys` / `js-sys` — Rust side does zero DOM interaction; all DOM work is JS

## Configuration

**Build — `vite.config.js`:**
```js
plugins: [wasm()]           // vite-plugin-wasm for .wasm ESM support
build.target: 'esnext'      // enables native top-level await; targets modern browsers
server.fs.allow: ['.']      // allows serving pkg/ directory (outside src/) during dev
```

**Cargo — `Cargo.toml`:**
```toml
crate-type = ["cdylib", "rlib"]   // cdylib for WASM, rlib for cargo test
edition = "2021"
getrandom: features = ["wasm_js"] // required for rand in WASM context
```

**Environment:**
- No `.env` files required — no runtime environment variables
- No secrets management needed — purely static client-side application

## Build Pipeline

**Development:**
```bash
wasm-pack build --target web      # Rust → pkg/ (tic_tac_toe.js + tic_tac_toe_bg.wasm)
npm run dev                        # Vite dev server with HMR
```

**Production (Docker):**
```bash
wasm-pack build --target web --release   # Optimised WASM (wasm-opt runs automatically)
npm run build                            # Vite → dist/ (content-hashed assets)
```

**WASM output (`pkg/`):**
- `tic_tac_toe.js` — JS glue module
- `tic_tac_toe_bg.wasm` — compiled WASM binary
- `tic_tac_toe.d.ts`, `tic_tac_toe_bg.wasm.d.ts` — TypeScript declarations
- `package.json` — npm-compatible package descriptor
- `pkg/` is `.gitignore`d; rebuilt at dev/build time

**Production output (`dist/`):**
- `index.html`
- `assets/index-*.js` — bundled JS (content-hashed)
- `assets/index-*.css` — bundled CSS (content-hashed)
- `assets/tic_tac_toe_bg-*.wasm` — WASM binary (content-hashed)
- `dist/` is `.gitignore`d; rebuilt in Docker

## Platform Requirements

**Development:**
- Rust stable toolchain (≥1.85 for edition 2021; 1.94.1 confirmed stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- wasm-pack 0.14.0
- Node.js 20+ (25.9.0 in use)
- npm 11+

**Production:**
- Docker multi-stage build defined in `Dockerfile`
- Serve stage: `nginx:alpine` (~8MB image)
- Final image serves only `dist/` — no Rust toolchain, no Node.js, no source
- Multi-arch: `linux/amd64` and `linux/arm64` (via GitHub Actions + Docker Buildx)

---

*Stack analysis: 2026-04-14*
