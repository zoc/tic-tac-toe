# Plan 11-01: README Documentation

**Phase:** 11 — Docs
**Goal:** README explains how to run and deploy the Docker image
**Status:** complete

## What We're Building

A README.md (or update to an existing one) covering:
1. What the project is (brief)
2. How to run locally (two options: `npm run dev` for dev, Docker for prod)
3. How to deploy with Docker (pull from Hub, run command)
4. How to build from source (wasm-pack + vite steps)
5. Docker Hub image reference

## Requirements

- [x] `README.md` — created with Docker usage section
- [x] "Run with Docker" section: `docker run --rm -p 8080:80 <user>/tic-tac-toe:latest`
- [x] "Build from source" section: wasm-pack + npm commands
- [x] "Development" section: `npm run dev`
- [x] Publish release section: `git tag v1.2.0 && git push --tags`

## Files to Create/Update

| File | Purpose |
|------|---------|
| `README.md` | Project documentation |

## Verification Steps

1. README renders correctly on GitHub
2. `docker run` command in README actually works (copy-paste test)
3. Build from source steps work on a clean checkout
