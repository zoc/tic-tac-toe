# Tic-Tac-Toe WASM

A browser-based tic-tac-toe game where a human player (X) plays against a computer opponent. The game logic is written in Rust and compiled to WebAssembly, with a polished frontend featuring smooth CSS animations, synthesized sound effects, system dark mode, score persistence, and an animated win line. The computer is beatable — it plays well but makes occasional mistakes.

## Quick Start

```bash
docker build -t tic-tac-toe .
docker run --rm -p 8080:80 tic-tac-toe
```

Then open http://localhost:8080 in your browser.

## Play

Open `index.html` after building, or run a container:

```bash
docker run --rm -p 8080:80 <your-dockerhub-username>/tic-tac-toe:latest
```

Then open [http://localhost:8080](http://localhost:8080) in your browser.

## Docker

> **Note:** Replace `<your-dockerhub-username>` with your Docker Hub username in all commands below.

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

### Deploy behind a reverse proxy

To serve the game publicly, proxy port 80 of the container through your web server.
Replace `yourdomain.com` with your domain and adjust the upstream port if needed (default: 8080 mapped to container's 80).

**nginx** (`/etc/nginx/sites-available/tic-tac-toe`):

```nginx
server {
    listen 80;
    server_name yourdomain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

Run `docker run -d -p 8080:80 --restart unless-stopped tic-tac-toe` to start the container in the background, then reload nginx.

## Releasing

This project uses GitHub Actions to automatically build and publish multi-platform Docker images to Docker Hub on every version tag push.

### Prerequisites

The repository maintainer must configure two GitHub settings:

**Secret: DOCKERHUB_TOKEN**
- Navigate to: Repository → Settings → Secrets and variables → Actions → New repository secret
- Name: `DOCKERHUB_TOKEN`
- Value: Docker Hub Personal Access Token (create at Docker Hub → Account Settings → Security → New Access Token with Read & Write scope)

**Variable: DOCKERHUB_USERNAME**
- Navigate to: Repository → Settings → Secrets and variables → Actions → Variables → New repository variable
- Name: `DOCKERHUB_USERNAME`
- Value: Your Docker Hub username (e.g., `johndoe`)

### Creating a release

1. **Tag the release:**
   ```bash
   git tag v1.3.0
   git push origin v1.3.0
   ```

2. **Monitor the build:**
   - GitHub Actions tab shows "Docker" workflow running
   - Build takes 5-8 minutes
   - All steps must show green checkmarks

3. **Verify on Docker Hub:**
   - Visit https://hub.docker.com/r/your-username/tic-tac-toe (replace `your-username` with your Docker Hub username)
   - Tag `1.3.0` and `1.3` appear (Phase 12 will add major tag `1`)
   - Both `linux/amd64` and `linux/arm64` architectures shown

### Workflow details

**Workflow file:** `.github/workflows/docker.yml`
**Trigger:** Git tags matching `v*.*.*` (e.g., v1.2.0, v1.3.0)
**Platforms:** linux/amd64, linux/arm64 (multi-arch manifest created automatically)
**Cache:** GitHub Actions cache (speeds up subsequent builds)
**Build time:** ~5-8 minutes with warm cache

**Technical notes:**
- Rust compilation runs natively (uses `--platform=$BUILDPLATFORM` in Dockerfile to avoid QEMU)
- WASM bytecode is platform-neutral (identical output on both architectures)
- Only the nginx:alpine serve stage cross-compiles under QEMU
- Multi-arch manifest means users automatically get the correct architecture for their platform

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
