---
phase: 13
slug: rust-ai-parameterization-wasm-api
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-28
---

# Phase 13 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| JS → WASM (`set_difficulty`) | Untrusted JS sends a u8 level value into Rust game state | u8 integer (0–255), game preference — not sensitive |
| JS → WASM (`computer_move`) | Reads `self.difficulty` set by prior boundary crossing | u8 integer consumed internally — not returned to JS |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-13-01 | Tampering | `WasmGame.difficulty` via `set_difficulty(level: u8)` | accept | u8 type constrains input to 0–255; `_ => 0.25` wildcard arm in `mistake_rate_for_level` (src/ai.rs:16) silently falls back to Medium for any out-of-range level. No panic, no invalid state. | closed |
| T-13-02 | Elevation of Privilege | AI difficulty state | accept | `difficulty` field not exposed as getter — JS can only write via `set_difficulty`, never read back. Consistent with `inner: Game` field encapsulation. No privilege escalation path exists in a single-player game. | closed |
| T-13-03 | Information Disclosure | `difficulty` field visibility | accept | `difficulty` field carries no sensitive data (game preference only). No `#[wasm_bindgen(getter)]` annotation and no `fn difficulty()` exposed. Low-value target. | closed |
| T-13-04 | Denial of Service | `random_bool(0.0)` for level=3 | accept | `Bernoulli::new(0.0)` in rand-0.10.x stores `p_int = 0`, always returns false — no panic, no infinite loop. minimax terminates because the board is finite (9 cells max). Verified at src/ai.rs:45. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-13-01 | T-13-01 | Out-of-range u8 values (4–255) silently fall back to Medium difficulty via wildcard arm. This is intentional defensive design — no assertion or error is needed because Medium is a safe, functional default for any unknown level value. | gsd-security-auditor | 2026-04-28 |
| AR-13-02 | T-13-02 | JS cannot read `difficulty` back — write-only by design (D-01). Acceptable in a single-player game context where no server-side state exists to protect. | gsd-security-auditor | 2026-04-28 |
| AR-13-03 | T-13-03 | Difficulty preference is not sensitive data. No encryption or access control warranted. | gsd-security-auditor | 2026-04-28 |
| AR-13-04 | T-13-04 | rand-0.10.x behavior for `Bernoulli::new(0.0)` is a well-defined library guarantee. minimax board finiteness (9 cells) provides a structural termination proof. | gsd-security-auditor | 2026-04-28 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-28 | 4 | 4 | 0 | gsd-security-auditor |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-28
