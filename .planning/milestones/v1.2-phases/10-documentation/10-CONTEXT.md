# Phase 10: Documentation - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Update README.md to satisfy DOCS-01 (copy-pasteable `docker build` + `docker run` one-liner) and DOCS-02 (reverse proxy deployment note with a config snippet). The image is already built and locally verified (Phase 9 complete).

**In scope:** README.md edits only — Quick Start section, Docker Hub username placeholder instruction, nginx reverse proxy snippet.
**Not in scope:** CI/CD pipeline, multi-arch builds, Docker Hub publishing, GitHub Actions workflow.

</domain>

<decisions>
## Implementation Decisions

### Docker Hub Username (DOCS-01 context)
- **D-01:** Keep `<your-dockerhub-username>` as a template placeholder throughout the README — do not substitute a real username.
- **D-02:** Add a one-time setup note near the top of the Docker section: "Replace `<your-dockerhub-username>` with your Docker Hub username in all commands below."

### Quick Start Section (DOCS-01)
- **D-03:** Add a `## Quick Start` section at the very top of README (immediately after the intro paragraph, before the existing `## Play` section).
- **D-04:** The Quick Start contains a 2-command block: `docker build -t tic-tac-toe .` followed by `docker run --rm -p 8080:80 tic-tac-toe` — no `--platform` flag, no `buildx`.
- **D-05:** The one-liner must work without prior context — no prerequisite steps, no username substitution required (local build only).

### Reverse Proxy Note (DOCS-02)
- **D-06:** Add a brief reverse proxy section inside the existing `## Docker` section — a short callout block, not a full subsection.
- **D-07:** Provide a single **nginx** `proxy_pass` upstream snippet (not Caddy) — one config block, copy-pasteable.
- **D-08:** Keep it short: context sentence + nginx server block snippet + one-liner explaining what to change (domain name, port).

### Agent's Discretion
- Exact wording of the Quick Start intro sentence
- Whether to include `open http://localhost:8080` as a third line in the Quick Start block
- Exact placement of the "Replace your-dockerhub-username" note within the Docker section (before the first run command)
- nginx snippet style: whether to include the `upstream` block or inline `proxy_pass http://localhost:8080`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing README
- `README.md` — existing content to be amended; note existing sections: Play, Docker (Run from Hub, Run in background, Build locally), Development, Tech, Publish a new release.

### Requirements
- `.planning/REQUIREMENTS.md` §Documentation — DOCS-01 and DOCS-02 exact acceptance text
- `.planning/ROADMAP.md` §Phase 10 — 2 success criteria with exact wording

### Phase 9 Output (verified working state)
- `.planning/phases/09-docker-image-nginx/09-02-SUMMARY.md` (if exists) — Phase 9 verification results; confirms `docker build -t tic-tac-toe:test .` + `docker run --rm -p 8080:80 tic-tac-toe:test` is the verified one-liner form

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `README.md` (116 lines): Complete existing README — Phase 10 amends this file, does not replace it. All existing sections remain; Quick Start and reverse proxy note are additions.

### Established Patterns
- README already uses fenced code blocks for all commands — follow same style
- Placeholder pattern `<your-dockerhub-username>` is already consistent throughout — keep it, add the substitution instruction once at the top of the Docker section
- "Build locally" section already uses `docker buildx build --platform linux/amd64 --load` — the Quick Start uses the simpler `docker build` form; both coexist

### Integration Points
- New `## Quick Start` section inserts after line 3 (intro paragraph) and before existing `## Play` section
- Reverse proxy snippet inserts inside the existing `## Docker` section, after the "Build locally" subsection

</code_context>

<specifics>
## Specific Ideas

- Quick Start should be "first thing a reader sees" — immediately after the intro paragraph
- Reverse proxy: nginx upstream snippet, not Caddy — copy-pasteable `proxy_pass` block
- DOCS-01 one-liner uses plain `docker build` (no `--platform`, no `buildx`) for maximum simplicity

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 10-documentation*
*Context gathered: 2026-04-14*
