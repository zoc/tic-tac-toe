# Stack Research

**Domain:** Rust-to-WebAssembly browser game (tic-tac-toe)
**Researched:** 2026-04-12
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | 1.94.1 | Game logic language | Memory-safe, compiles to compact WASM; the dominant language for WASM targeting browsers. Installed via `rustup`. |
| wasm-bindgen | 0.2.118 | Rust↔JS interop bindings | The canonical bridge between Rust/WASM and JavaScript. Generates TypeScript definitions, handles type marshalling. Every Rust-WASM project uses this. |
| wasm-pack | 0.14.0 | Build tool (Rust → WASM pkg) | Wraps `cargo build --target wasm32-unknown-unknown` + `wasm-bindgen-cli` + `wasm-opt` into a single command. Outputs a ready-to-import `pkg/` directory with `.wasm`, JS glue, and `package.json`. |
| web-sys | 0.3.95 | Browser API bindings for Rust | Provides typed Rust bindings to DOM, Canvas, Events, etc. Feature-gated — only import what you use. For this project we need minimal DOM features (the JS side handles rendering). |
| js-sys | 0.3.95 | JavaScript built-in bindings | Typed access to `Math`, `Array`, `Date`, etc. from Rust. Companion to web-sys. Useful if any JS built-ins are needed from the Rust side. |
| Vite | 8.0.8 | Frontend dev server & bundler | Fast HMR, native ESM, zero-config for HTML/CSS/JS. Handles WASM import with plugin. The standard choice for modern frontend tooling — replaces Webpack. |
| HTML/CSS/JS (vanilla) | ES2022+ | Frontend rendering & UI | PROJECT.md specifies "no heavy framework needed." For a tic-tac-toe game, vanilla JS with CSS animations is the right call — no React/Vue overhead for 9 squares. |

### Supporting Libraries (Rust side)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | Serialization framework | Derive `Serialize`/`Deserialize` on game state structs so they can cross the WASM boundary cleanly. |
| serde-wasm-bindgen | 0.6.5 | Serde ↔ JsValue bridge | Convert Rust structs directly to native JS objects (not JSON strings). Faster and smaller than `serde_json` for WASM. Use `to_value()` / `from_value()`. |
| rand | 0.10.1 | Random number generation | Powers the "beatable AI" — minimax with random mistake injection. Requires `getrandom` WASM feature. |
| getrandom | 0.4.2 | Platform RNG backend | Required by `rand` for entropy in WASM. Must enable the `wasm_js` feature to use browser `crypto.getRandomValues()`. |
| console_error_panic_hook | 0.1.7 | WASM panic debugging | Forwards Rust panics to `console.error()` with full backtraces. Essential for development — WASM panics are otherwise silent `unreachable` traps. |
| console_log | 1.0.0 | Rust `log` → browser console | Bridges the Rust `log` crate to `console.log()`. Optional but helpful during development. |
| wasm-bindgen-test | 0.3.68 | WASM unit testing | Run `#[wasm_bindgen_test]` functions in headless browsers via `wasm-pack test`. Dev dependency only. |

### Supporting Libraries (JS side)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| vite-plugin-wasm | 3.6.0 | WASM ESM import support | Lets Vite import `.wasm` files as ES modules. Required for `import init from './pkg'` pattern to work. |
| vite-plugin-top-level-await | 1.6.0 | Top-level `await` in modules | WASM init is async. This plugin enables `await init()` at module top level for broader browser compat. Not needed if `build.target = "esnext"`. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| rustup | Rust toolchain manager | Install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| wasm32-unknown-unknown target | WASM compilation target | Add with `rustup target add wasm32-unknown-unknown` |
| wasm-pack | Build orchestrator | Install with `cargo install wasm-pack` or `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf \| sh` |
| cargo | Rust package manager | Comes with rustup. Manages Rust dependencies. |
| npm / Node.js | JS package manager | For Vite, dev server, and JS dependencies. Node 20+ recommended. |
| wasm-opt (via wasm-pack) | WASM binary optimizer | wasm-pack runs this automatically on release builds. Shrinks `.wasm` by 10-30%. No separate install needed. |

## Installation

```bash
# 1. Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 2. wasm-pack
cargo install wasm-pack

# 3. JS dependencies (from project root, after npm init)
npm install -D vite vite-plugin-wasm vite-plugin-top-level-await
```

### Cargo.toml (Rust project)

```toml
[package]
name = "tic-tac-toe"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
rand = { version = "0.10", features = ["std", "std_rng"] }
getrandom = { version = "0.4", features = ["wasm_js"] }
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"    # Optimize for size (small .wasm)
lto = true         # Link-time optimization
```

### vite.config.js

```javascript
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  build: {
    target: 'esnext',
  },
});
```

### Build command

```bash
# Build WASM package (outputs to pkg/)
wasm-pack build --target web

# Run Vite dev server
npx vite
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| wasm-pack + Vite | Trunk (Rust-centric bundler) | If the entire app were in Rust (e.g., Yew/Leptos SPA). For our split architecture (Rust logic + vanilla JS UI), wasm-pack + Vite gives better DX for the frontend side. |
| `--target web` | `--target bundler` | When using Webpack as the bundler. Vite works with both, but `--target web` produces standalone ES modules that also work without any bundler — simpler and more portable. |
| Vanilla JS | Yew / Leptos (Rust UI frameworks) | If building a complex SPA entirely in Rust. For 9 squares with CSS animations, vanilla JS is dramatically simpler with zero compile overhead for UI changes. |
| serde-wasm-bindgen | serde_json (string serialization) | Never for this project. serde-wasm-bindgen converts directly to JS objects, avoiding JSON.parse() overhead. serde_json makes sense only when you need JSON-formatted strings. |
| rand 0.10 | `js_sys::Math::random()` | If you want zero Rust-side RNG dependencies. But `rand` provides proper distributions and the `Rng` trait, making imperfect minimax cleaner to implement. |
| Vite | Webpack 5 | If you're locked into an existing Webpack config. Vite is faster, simpler, and has first-class WASM plugin support. No reason to choose Webpack for a greenfield project. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| stdweb | Abandoned since 2020; superseded by web-sys/js-sys. No updates in 5+ years. | web-sys + js-sys |
| wasm-bindgen `--target no-modules` | Legacy output format for `<script>` tags. No tree-shaking, no ES module support. | `--target web` (ES modules) |
| Webpack for new projects | Slower builds, more complex config, larger ecosystem footprint for zero benefit on a small project. | Vite 8 |
| Yew / Leptos / Dioxus | Full Rust UI frameworks — massive overkill for a tic-tac-toe grid. They add compile times, complexity, and a virtual DOM you don't need. | Vanilla JS + CSS |
| `wasm-bindgen` `serde-serialize` feature | Deprecated legacy approach. Uses JSON internally — slower and larger output than serde-wasm-bindgen. | serde-wasm-bindgen crate |
| TypeScript | Adds a compilation step for a trivial frontend. The JS side is ~100 lines of DOM manipulation and event handlers — TS overhead isn't justified. | Vanilla JS (ES modules) |
| getrandom 0.2.x `js` feature | Older API. rand 0.10 depends on getrandom 0.4.x which uses the `wasm_js` feature name. Using 0.2.x will cause version conflicts. | getrandom 0.4.x with `wasm_js` feature |

## Stack Patterns

**The Split Architecture (recommended for this project):**
- Rust/WASM owns: board state, move validation, AI logic, win detection, score tracking
- JavaScript owns: DOM rendering, CSS animations, event handling, WASM initialization
- Bridge: exported `#[wasm_bindgen]` functions returning simple types or `JsValue` via serde-wasm-bindgen

**Why this split:**
- UI changes (animations, colors) don't require Rust recompilation
- Rust side is pure logic — easy to unit test with `cargo test` (no WASM needed)
- The JS rendering layer is trivial (~100 LOC) — no framework needed
- Hot reload works for CSS/JS changes via Vite; only Rust changes need `wasm-pack build`

**If the project were more complex:**
- Use Yew/Leptos for the UI if it were a full SPA with routing, forms, etc.
- Use `web-sys` DOM manipulation from Rust if you wanted zero-JS architecture
- Use Trunk instead of Vite if the entire app lived in Rust

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| wasm-bindgen 0.2.118 | web-sys 0.3.95, js-sys 0.3.95 | These three share a version lockstep — web-sys/js-sys 0.3.x always depends on wasm-bindgen 0.2.x. Keep all three on latest minor. |
| rand 0.10.x | getrandom 0.4.x | rand 0.10 requires `getrandom ^0.4.0`. You must use getrandom 0.4+ with `wasm_js` feature — not the older 0.2.x `js` feature. |
| serde-wasm-bindgen 0.6.x | wasm-bindgen 0.2.x, serde 1.x | Stable bridge. No known compatibility issues. |
| wasm-pack 0.14.0 | wasm-bindgen 0.2.x | wasm-pack downloads the matching `wasm-bindgen-cli` automatically. |
| vite-plugin-wasm 3.6.0 | Vite 8.x | Compatible. Plugin uses standard Vite plugin API. |
| Rust edition 2024 | Rust 1.85+ | The 2024 edition was stabilized in Rust 1.85. Our target Rust 1.94.1 fully supports it. |

## Sources

- crates.io API — wasm-bindgen 0.2.118, web-sys 0.3.95, js-sys 0.3.95, wasm-pack 0.14.0, serde 1.0.228, serde-wasm-bindgen 0.6.5, rand 0.10.1, getrandom 0.4.2, console_error_panic_hook 0.1.7, console_log 1.0.0, wasm-bindgen-test 0.3.68 (HIGH confidence — primary source)
- npmjs.org API — Vite 8.0.8, vite-plugin-wasm 3.6.0, vite-plugin-top-level-await 1.6.0 (HIGH confidence — primary source)
- forge.rust-lang.org — Rust stable 1.94.1 (HIGH confidence — official)
- Context7: /drager/wasm-pack — project setup, build targets, Cargo.toml configuration (HIGH confidence)
- Context7: /websites/rs_serde-wasm-bindgen — JsValue serialization patterns (HIGH confidence)
- Context7: /menci/vite-plugin-wasm — Vite WASM integration, configuration (HIGH confidence)
- Context7: /websites/rs_js-sys_0_3_91_js_sys — js-sys/wasm-bindgen relationship (HIGH confidence)

---
*Stack research for: Rust/WASM tic-tac-toe browser game*
*Researched: 2026-04-12*
