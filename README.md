# Tic-Tac-Toe WASM

A browser-based tic-tac-toe game where a human player (X) plays against a computer opponent. The game logic is written in Rust and compiled to WebAssembly, with a polished frontend featuring smooth CSS animations, synthesized sound effects, system dark mode, score persistence, and an animated win line. The computer is beatable — it plays well but makes occasional mistakes.

## Play

Open `index.html` after building, or run a container:

```bash
docker run --rm -p 8080:80 <your-dockerhub-username>/tic-tac-toe:latest
```

Then open [http://localhost:8080](http://localhost:8080) in your browser.

## Docker

### Run from Docker Hub

```bash
# Latest release
docker run --rm -p 8080:80 <your-dockerhub-username>/tic-tac-toe:latest

# Specific version
docker run --rm -p 8080:80 <your-dockerhub-username>/tic-tac-toe:1.2.0
```

The image is a multi-arch manifest (`linux/amd64` + `linux/arm64`) — Docker will pull the correct variant for your machine automatically.

### Run in the background

```bash
docker run -d --name tic-tac-toe -p 8080:80 --restart unless-stopped \
  <your-dockerhub-username>/tic-tac-toe:latest
```

Stop it with:

```bash
docker stop tic-tac-toe && docker rm tic-tac-toe
```

### Build locally

```bash
# Single platform (amd64), load into local Docker
docker buildx build --platform linux/amd64 --load -t tic-tac-toe:local .
docker run --rm -p 8080:80 tic-tac-toe:local
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) 20+

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build and run

```bash
# Install JS dependencies
npm install

# Build Rust game engine → WASM
wasm-pack build --target web --release

# Start Vite dev server (hot reload for JS/CSS)
npm run dev
```

Open [http://localhost:5173](http://localhost:5173).

### Production build

```bash
wasm-pack build --target web --release
npm run build
# Output: dist/
```

## Tech

- **Rust** — game logic (board state, AI, win detection, ~927 LOC)
- **wasm-pack + wasm-bindgen** — Rust → WebAssembly compilation
- **Vite 8** — frontend build tool and dev server
- **Vanilla JS + CSS** — rendering and UI (~762 LOC, no framework)
- **nginx:alpine** — serves the static production build in Docker

The AI uses minimax with ~25% random mistake injection — tunable in `src/ai.rs`.

## Publish a new release

Tag a commit to trigger the GitHub Actions Docker workflow:

```bash
git tag v1.2.0
git push --tags
```

This builds `linux/amd64` + `linux/arm64` images and pushes them to Docker Hub with tags `1.2.0`, `1.2`, and `latest`.

> **Required GitHub setup** (one-time):
> - Create a Docker Hub access token: Hub → Account Settings → Security → New Access Token
> - Add GitHub secret `DOCKERHUB_TOKEN` with the token value
> - Add GitHub variable `DOCKERHUB_USERNAME` with your Docker Hub username
