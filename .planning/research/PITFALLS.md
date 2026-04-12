# Pitfalls Research

**Domain:** Rust/WASM browser game (tic-tac-toe)
**Researched:** 2026-04-12
**Confidence:** HIGH — based on official Rust WASM Book, wasm-bindgen docs, and wasm-pack documentation

## Critical Pitfalls

### Pitfall 1: Forgetting Async WASM Initialization

**What goes wrong:**
WASM modules must be asynchronously loaded and initialized before any exported functions can be called. Developers write JS that calls WASM functions synchronously at module load time, resulting in `TypeError: wasm is undefined` or similar cryptic errors. With `--target web`, you must call and `await` the `init()` function before using any exports.

**Why it happens:**
WASM isn't like a regular JS import — the binary must be fetched, compiled, and instantiated. This is inherently asynchronous. The generated JS wrapper hides some complexity, but the initialization step is still required and easy to forget, especially when not using a bundler.

**How to avoid:**
- Use `--target web` and always structure your entry point as an async `run()` function:
  ```js
  import init, { GameState } from './pkg/tic_tac_toe.js';
  async function run() {
    await init();
    // NOW you can use GameState
  }
  run();
  ```
- Never call WASM exports at the top level of a script without awaiting init first.
- Consider using a bundler (Webpack/Vite) which handles WASM initialization automatically with `--target bundler`.

**Warning signs:**
- `TypeError` or `undefined is not a function` errors in the browser console on page load.
- Errors that only appear on first load but not on hot-reload (because bundler caches the initialized module).

**Phase to address:**
Phase 1 (Project scaffolding) — get the WASM init pattern right from the start.

---

### Pitfall 2: Excessive JS↔WASM Boundary Crossing

**What goes wrong:**
Every call across the JS↔WASM boundary has overhead: data must be serialized/copied, function call indirection occurs, and the JS engine can't optimize across the boundary. If the game logic makes many small calls per frame (e.g., calling `get_cell(row, col)` 9 times per render), the overhead dominates and the app feels sluggish or the architecture becomes convoluted.

**Why it happens:**
Developers design fine-grained APIs that mirror how they'd structure a pure JS or pure Rust app. They expose individual getters for each piece of state instead of thinking about the boundary as a bulk data transfer point.

**How to avoid:**
- **Design coarse-grained APIs across the boundary.** Instead of `get_cell(row, col)` called 9 times, expose `get_board() -> *const u8` returning a pointer to the board array in WASM linear memory that JS reads directly.
- **Keep large, long-lived data in Rust (WASM linear memory).** Expose opaque handles and bulk operations.
- **Minimize copying:** Use `#[repr(u8)]` enums and flat arrays so JS can read WASM memory directly via `Uint8Array` views without serialization.
- For a 3x3 tic-tac-toe board, the overhead is negligible — but establishing the right pattern now prevents issues in any follow-up project.

**Warning signs:**
- More than 2-3 WASM function calls per frame for rendering.
- Passing complex objects (strings, structs) back and forth frequently.
- Using `serde_wasm_bindgen::to_value` for data that could be a flat array.

**Phase to address:**
Phase 1 (Rust game engine) — design the WASM API boundary before writing implementation.

---

### Pitfall 3: Cryptic Panic Messages in the Browser

**What goes wrong:**
When Rust code panics in WASM, the default behavior produces `RuntimeError: unreachable executed` — an utterly unhelpful error message with no stack trace, no panic message, no file/line info. Developers spend hours debugging what turns out to be a simple `unwrap()` on a `None` value.

**Why it happens:**
WASM's `unreachable` instruction is how panics manifest by default. Without `console_error_panic_hook`, the Rust panic infrastructure's formatted messages are silently swallowed. Developers don't install the hook because the default template may not set it up, or they forget to call `set_once()` early enough.

**How to avoid:**
- Add `console_error_panic_hook` as a dependency and call `console_error_panic_hook::set_once()` at the very start of your public API entry point (e.g., in a `new()` constructor or an explicit `init()` function):
  ```rust
  #[wasm_bindgen]
  pub fn init() {
      console_error_panic_hook::set_once();
  }
  ```
- Build with debug symbols during development: `wasm-pack build --dev` or set `debug = true` in `[profile.release]` in Cargo.toml.
- Without debug symbols, stack traces show `wasm-function[42]` instead of `my_crate::my_function`.

**Warning signs:**
- `RuntimeError: unreachable executed` in the browser console.
- Stack traces with only numeric function identifiers.
- Errors that are impossible to correlate with Rust source code.

**Phase to address:**
Phase 1 (Project scaffolding) — install panic hook before writing any game logic.

---

### Pitfall 4: Wrong `crate-type` Breaks Native Tests

**What goes wrong:**
WASM crates require `crate-type = ["cdylib"]` in `Cargo.toml` to produce `.wasm` output. But if you _only_ specify `cdylib`, you can't run native `#[test]` functions with `cargo test` — the linker fails because `cdylib` doesn't produce an `rlib` that test harnesses can link against. You lose the ability to test game logic natively (which is far easier to debug than WASM).

**Why it happens:**
Developers follow WASM tutorials that set `crate-type = ["cdylib"]` and never add `"rlib"`. They don't discover the problem until they try to write their first `#[test]`.

**How to avoid:**
- Always set both crate types in `Cargo.toml`:
  ```toml
  [lib]
  crate-type = ["cdylib", "rlib"]
  ```
- `cdylib` = produces the `.wasm` binary for the browser.
- `rlib` = produces a Rust library that `cargo test` and `cargo bench` can link against.

**Warning signs:**
- `cargo test` fails with linker errors like `cannot find -l<crate_name>`.
- All tests are written as `wasm-bindgen-test` when they don't actually need browser APIs.

**Phase to address:**
Phase 1 (Project scaffolding) — set this in `Cargo.toml` from day one.

---

### Pitfall 5: Ownership Confusion with `#[wasm_bindgen]` Exported Structs

**What goes wrong:**
When a `#[wasm_bindgen]` struct is passed by value to a JS function, ownership transfers and the Rust-side memory is consumed. If JS code tries to use the object again after passing it to another function, it gets an error about the object being "already freed" or "null pointer". This is especially confusing because JS developers don't think about ownership.

Similarly, public fields on exported structs must implement `Copy` (for auto-generated getters/setters), or you need `#[wasm_bindgen(getter_with_clone)]`. A `pub value: String` field on a `#[wasm_bindgen]` struct will fail to compile without this annotation.

**Why it happens:**
Rust's ownership model is faithfully represented in the JS bindings. When you pass a struct by value, it's moved — the JS wrapper invalidates the old handle. Developers coming from JS or even Rust (where the compiler catches use-after-move) are caught off-guard because JS has no compile-time check.

**How to avoid:**
- Use `&self` and `&mut self` methods instead of consuming `self` wherever possible.
- For the tic-tac-toe game, keep a single `GameState` struct alive in JS and call methods on it via `&mut self` — never pass it by value.
- Use `#[wasm_bindgen(getter_with_clone)]` for structs with `String` or non-`Copy` public fields.
- Prefer returning primitive values (u8, u32, bool) or pointers to linear memory over returning complex structs.

**Warning signs:**
- JS runtime errors about "null pointer passed to Rust" or "attempt to use a moved value".
- Compilation errors about `Copy` not being implemented for a field type.

**Phase to address:**
Phase 1 (Rust game engine API design) — decide on API shape before implementation.

---

### Pitfall 6: WASM Binary Size Bloat from Formatting and Panics

**What goes wrong:**
The `.wasm` binary becomes unexpectedly large (hundreds of KB for a simple app). The formatting machinery (`std::fmt`, `Display`, `Debug`), panic infrastructure, and the default allocator (`dlmalloc`, ~10KB) are the usual culprits. A "hello world" WASM can easily be 50KB+ unoptimized.

**Why it happens:**
Any use of `format!()`, `println!()`, `.to_string()`, `panic!()` (even implicit panics from `unwrap()`, array indexing, division) pulls in the full formatting and panic infrastructure. Generics cause monomorphization that multiplies code size. Developers don't realize how much code these seemingly simple operations generate.

**How to avoid:**
- Configure release profile in `Cargo.toml`:
  ```toml
  [profile.release]
  lto = true        # Link-time optimization — inlines aggressively, removes dead code
  opt-level = 's'   # Optimize for size (try 'z' too, but measure — 's' is sometimes smaller)
  ```
- Run `wasm-opt` post-build (wasm-pack does this automatically in release mode).
- Avoid `format!()` in production paths — use it only in debug/logging code.
- Use `.get(i)` instead of `[i]` indexing to avoid implicit panic code paths.
- Use `twiggy` to profile binary size: `twiggy top -n 20 pkg/your_crate_bg.wasm`.
- For tic-tac-toe specifically: the binary will be small enough that this isn't a blocker, but good habits matter.

**Warning signs:**
- `.wasm` file > 100KB for a simple game.
- `twiggy` shows `dlmalloc`, `fmt`, or `panicking` as top contributors.
- Slow initial page load on mobile connections.

**Phase to address:**
Phase 2 (Build optimization) — after core logic works, optimize the release build.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using `to_string()` to pass board state to JS | Quick rendering, easy to debug | Allocates a String in WASM, copies to JS heap every frame; pulls in `fmt` code bloat | MVP only — switch to direct memory access for production |
| Using `unwrap()` freely in game logic | Faster to write, cleaner looking code | Each `unwrap()` adds panic infrastructure to the binary; crashes show `unreachable` without panic hook | During development with panic hook installed; replace with safe alternatives before release |
| Skipping `wasm-opt` during development | Faster build times | Larger binaries in dev; forgetting to enable for release | Always acceptable in dev; never skip in release |
| `--target bundler` with Webpack for serving | Familiar tooling, auto-handles WASM init | Heavy dependency chain for serving a static page; Webpack config complexity | Only if you already have a Webpack project; prefer `--target web` for simplicity |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| WASM ↔ JS data | Serializing board state as JSON with `serde_wasm_bindgen` | Use `#[repr(u8)]` flat arrays and expose a pointer to linear memory; JS reads via `new Uint8Array(wasm.memory.buffer, ptr, len)` |
| WASM module loading | Assuming `import` of WASM works like a regular ES module | With `--target web`, always `await init()` before calling any exports; WASM requires async instantiation |
| `web-sys` feature flags | Importing `web-sys` and wondering why DOM methods don't exist | Each Web API must be individually enabled via Cargo.toml features: `features = ["console", "Document", "Element"]` |
| Closure lifetime | Creating a `Closure<dyn FnMut()>` for `requestAnimationFrame` and letting it drop | Store the `Closure` in a struct field or use `Closure::forget()` (leaks memory) or `Closure::into_js_value()`. If the `Closure` drops, the JS callback throws on next invocation |
| Serving WASM files | Opening `index.html` directly with `file://` protocol | WASM requires proper MIME type (`application/wasm`) and CORS headers. Use a local HTTP server (`python3 -m http.server`, `miniserve`, or a dev server) |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| String-based rendering | Growing memory usage, GC pauses on each frame | Render to Canvas API or manipulate DOM directly; pass cell data as typed array, not string | Noticeable with grids > 20×20; not a real issue for 3×3 tic-tac-toe |
| Allocating per frame | Memory grows over time, eventual slowdown | Pre-allocate board arrays; reuse Vec instead of creating new ones per tick | Visible in long-running sessions or high-frequency updates |
| Debug builds in production | Binary is 5-10× larger, runs significantly slower | Always deploy release builds: `wasm-pack build --release` | Immediately — debug WASM is noticeably slow even for simple apps |
| Not using `instantiateStreaming` | WASM must fully download before compilation starts | Use `--target web` (the generated JS uses streaming compilation by default) or ensure your bundler supports streaming | Affects load time on slow connections; the generated code handles this correctly if you don't override |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting game state in JS | Player can modify game state via browser DevTools to cheat | For a single-player game this is fine — the only person cheated is the player. Don't over-engineer server-side validation for a client-only game. |
| Exposing internal WASM memory layout | Theoretically allows memory corruption via JS | Use opaque handles and well-defined API methods rather than raw pointer manipulation. For a tic-tac-toe game, this is extremely low risk. |
| Serving WASM without proper Content-Type | Browser may refuse to compile WASM | Ensure server sends `Content-Type: application/wasm` for `.wasm` files |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No loading state during WASM init | User sees blank page for a moment, may think app is broken | Show a lightweight HTML/CSS loading indicator, then hide it after `await init()` completes |
| Computer moves instantly | Feels robotic, not like playing against an opponent | Add a short artificial delay (200-500ms) before the computer's move for a more natural feel |
| No visual feedback on player click | User unsure if click registered, especially on mobile | Immediately update the UI on click before calling WASM; or ensure WASM call + DOM update happens within the same animation frame |
| Win/loss announced but board immediately resets | User can't see the winning line or appreciate the outcome | Highlight the winning line and show result for 1-2 seconds before offering "New Game" |
| Unbeatable AI | Player gets frustrated, stops playing | Minimax with random mistakes (as specified in PROJECT.md) — the AI should feel smart but beatable |

## "Looks Done But Isn't" Checklist

- [ ] **WASM init:** Module loads, but did you test what happens if WASM fails to load? (network error, old browser) — verify graceful fallback or error message
- [ ] **Panic hook:** Game works, but try triggering an edge case (invalid move, double-click) — verify you see a readable Rust panic message, not `unreachable`
- [ ] **Score persistence:** Scores display, but do they survive a page reload? — verify if that's intended (localStorage) or accepted (scores reset on reload)
- [ ] **Mobile clicks:** Game works with mouse, but tap events on mobile can fire differently — verify `click` events work on iOS Safari and Android Chrome
- [ ] **Draw detection:** Win/loss works, but does a full-board draw correctly trigger? — verify the draw condition when all 9 cells are filled with no winner
- [ ] **Browser back/forward:** User navigates away and back — verify game state is reasonable (reset is fine, but crashing is not)
- [ ] **Release build:** Everything works in `--dev`, but did you test the release build? — verify `wasm-pack build --release` produces a working artifact with reasonable size

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong crate-type (missing rlib) | LOW | Add `"rlib"` to `crate-type` array in Cargo.toml, rebuild |
| No panic hook installed | LOW | Add `console_error_panic_hook` dependency, call `set_once()` in init |
| Excessive boundary crossing | MEDIUM | Refactor API to use bulk data transfer (pointer + length) instead of individual calls; requires changing both Rust exports and JS consumers |
| Ownership confusion (use-after-move) | LOW | Change function signatures to take `&self` / `&mut self` instead of `self`; update JS call sites |
| Large WASM binary | LOW | Add release profile optimizations to Cargo.toml, enable LTO and size optimization |
| String-based rendering | MEDIUM | Refactor to expose board memory pointer, update JS to read typed array and render to Canvas/DOM |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Async WASM initialization | Phase 1: Scaffolding | Page loads without errors in browser console; WASM functions callable after init |
| Panic hook not installed | Phase 1: Scaffolding | Intentionally trigger a panic; verify readable message in browser console |
| Wrong crate-type | Phase 1: Scaffolding | `cargo test` runs successfully with a trivial test |
| API boundary design | Phase 1: Game engine | Review: no more than 2-3 cross-boundary calls per game action |
| Ownership confusion | Phase 1: Game engine | All public WASM methods use `&self` or `&mut self`; no pass-by-value of game state |
| WASM binary size | Phase 2: Polish/release | Release `.wasm` file < 50KB (generous for tic-tac-toe); verified with `ls -la pkg/*.wasm` |
| UX: computer move timing | Phase 2: UI polish | Playtest: computer move has visible delay, feels natural |
| UX: loading state | Phase 2: UI polish | Throttle network in DevTools; verify loading indicator appears |
| UX: win highlighting | Phase 2: UI polish | Win a game; verify winning line is highlighted for at least 1 second before reset option |

## Sources

- [Rust and WebAssembly Book — Debugging](https://rustwasm.github.io/docs/book/reference/debugging.html) — HIGH confidence
- [Rust and WebAssembly Book — Code Size](https://rustwasm.github.io/docs/book/reference/code-size.html) — HIGH confidence
- [Rust and WebAssembly Book — Game of Life Implementation](https://rustwasm.github.io/docs/book/game-of-life/implementing.html) — HIGH confidence (JS↔WASM interface design principles)
- [wasm-bindgen Reference — Exported Rust Types](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/exported-rust-types.html) — HIGH confidence
- [wasm-bindgen Reference — Passing Closures to JS](https://rustwasm.github.io/docs/wasm-bindgen/reference/passing-rust-closures-to-js.html) — HIGH confidence
- [wasm-bindgen Reference — Weak References](https://rustwasm.github.io/docs/wasm-bindgen/reference/weak-references.html) — HIGH confidence
- [wasm-bindgen Reference — Deployment](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html) — HIGH confidence
- [wasm-pack Build Command](https://rustwasm.github.io/docs/wasm-pack/commands/build.html) — HIGH confidence
- [serde-wasm-bindgen docs](https://docs.rs/serde-wasm-bindgen) via Context7 — HIGH confidence

---
*Pitfalls research for: Rust/WASM browser tic-tac-toe game*
*Researched: 2026-04-12*
