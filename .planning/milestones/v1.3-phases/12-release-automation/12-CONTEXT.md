# Phase 12: Release Automation - Context

**Gathered:** 2026-04-25
**Status:** Implementation complete, ready for documentation
**Source:** Post-implementation documentation of completed work

<domain>
## Phase Boundary

Phase 12 adds the final automation layer to the CI/CD pipeline established in Phase 11. This phase ensures that production releases generate human-friendly Docker tags and include comprehensive metadata for container registries.

**What this phase delivers:**
- Automatic generation of semantic version tags from git tags (v1.3.0 → 1.3.0, 1.3, 1, latest)
- OCI image labels providing metadata annotations for Docker Hub and other registries
- Updated documentation reflecting the fzoc Docker Hub username

**What this phase does NOT deliver:**
- GitHub Releases automation (out of scope)
- Changelog generation (handled manually)
- Automated testing before release (covered in Phase 11 workflow)

</domain>

<decisions>
## Implementation Decisions

### Semantic Version Tags (DIST-02)
**LOCKED:** All four tag variants must be generated automatically from a single git tag push.

Implementation completed:
- Uses `docker/metadata-action@v5` to parse git tags
- Pattern `type=semver,pattern={{version}}` generates full version (1.3.0)
- Pattern `type=semver,pattern={{major}}.{{minor}}` generates minor version (1.3)
- Pattern `type=semver,pattern={{major}}` generates major version (1)
- Pattern `type=raw,value=latest,enable={{is_default_branch}}` generates latest on default branch only

Location: `.github/workflows/docker.yml` lines 49-53

### OCI Image Labels (DIST-03)
**LOCKED:** Standard org.opencontainers.image.* labels must be attached to published images.

Implementation completed:
- `docker/metadata-action@v5` automatically generates OCI labels from repository metadata
- Labels passed to `docker/build-push-action@v6` via `labels: ${{ steps.meta.outputs.labels }}`
- Standard labels include: title, description, url, source, version, created, revision

Location: `.github/workflows/docker.yml` line 62

### Docker Hub Username
**LOCKED:** Docker Hub username is `fzoc`, documented in README for public reference.

Implementation completed:
- README updated with `fzoc/tic-tac-toe` in Docker Hub examples
- Workflow uses `${{ vars.DOCKERHUB_USERNAME }}` variable for repository-agnostic configuration
- Documentation shows all four tag variants with fzoc username

Location: `README.md` Docker Hub section, `.github/workflows/docker.yml` line 48

### the agent's Discretion

**Tag ordering in Docker Hub:** The order tags appear on Docker Hub is determined by Docker Hub's display logic, not the workflow. No action needed.

**latest tag branch restriction:** The `enable={{is_default_branch}}` condition ensures latest only applies to production releases from the main branch. Feature branch tags will not overwrite latest.

**Label schema version:** docker/metadata-action uses OCI Image Spec 1.0 labels by default. No override needed unless future spec versions require it.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CI/CD Configuration
- `.github/workflows/docker.yml` — GitHub Actions workflow with tag generation and label attachment

### Documentation
- `README.md` — User-facing release documentation with prerequisites and workflow details

### Requirements
- `.planning/REQUIREMENTS.md` (DIST-02, DIST-03) — Verification criteria for semver tags and OCI labels

No external specs — requirements fully captured in decisions above

</canonical_refs>

<specifics>
## Specific Ideas

**Verification approach:**
1. Push a production tag (e.g., v1.3.0)
2. Monitor GitHub Actions workflow execution
3. Check Docker Hub for all four tag variants (1.3.0, 1.3, 1, latest)
4. Inspect image metadata on Docker Hub to verify OCI labels are present

**Commits implementing Phase 12:**
- `3dd3062` — feat(phase-12): add major version tag and update Docker Hub username to fzoc
- `d834ae3` — docs: fix get-shit-done GitHub link

**What changed from Phase 11:**
- Added `type=semver,pattern={{major}}` to generate major version tag (line 52)
- Updated README with fzoc username in Docker Hub examples
- No changes to label handling — already correct in Phase 11 implementation

</specifics>

<deferred>
## Deferred Ideas

**GitHub Releases:** Creating GitHub Releases with changelog would require additional workflow steps. Deferred to future milestone if needed.

**Automated changelog generation:** conventional-changelog or similar tooling could generate changelogs from commit messages. Not required for Phase 12 scope.

**Tag protection rules:** GitHub tag protection could prevent accidental overwrites of production tags. Repository settings change, not workflow change.

**Pre-release tag support:** Handling alpha/beta/rc tags (e.g., v1.4.0-beta.1) would require additional semver patterns. No current need — all releases are stable.

</deferred>

---

*Phase: 12-release-automation*
*Context gathered: 2026-04-25 — post-implementation documentation*
