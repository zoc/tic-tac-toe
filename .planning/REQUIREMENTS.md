# Requirements: v1.3 CI/CD & Distribution

## Milestone Goal

Automate multi-arch Docker image builds and publish to Docker Hub on release tags.

## v1.3 Requirements

### CI/CD Automation

- [ ] **CICD-01**: GitHub Actions workflow triggers on git tag push (tags matching `v*`)
- [ ] **CICD-02**: Workflow builds Docker image for linux/amd64 platform
- [ ] **CICD-03**: Workflow builds Docker image for linux/arm64 platform
- [ ] **CICD-04**: Multi-arch manifest created combining both platform images

### Distribution

- [ ] **DIST-01**: Images published to Docker Hub under user's repository
- [ ] **DIST-02**: Semver tags generated automatically (v1.3.0 → tags: 1.3.0, 1.3, 1, latest)
- [ ] **DIST-03**: OCI image labels attached (org.opencontainers.image.* annotations)
- [ ] **DIST-04**: Docker Hub credentials securely managed via GitHub Secrets

## Future Requirements

None identified — this milestone completes the core CI/CD pipeline.

## Out of Scope

- **GitHub Container Registry (ghcr.io)** — Docker Hub only for v1.3; adding ghcr.io would double the matrix complexity with minimal value
- **Automated testing in CI** — existing Docker build already validates the app compiles and runs; separate test jobs add latency with no new coverage
- **Release notes automation** — manual release notes sufficient for low-frequency releases; automation overhead not justified
- **Version bumping automation** — manual git tagging workflow is simple and explicit; tools like semantic-release add complexity for minimal benefit

## Traceability

This section maps requirements to phases and will be populated by the roadmapper.

| Requirement | Phase | Status |
|-------------|-------|--------|
| (pending roadmap) | — | — |

---
*Created: 2026-04-25*
