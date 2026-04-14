# Build stage: compiles Rust → WASM and JS → static bundle
# Always runs on the native runner platform (avoids QEMU for Rust compilation).
# wasm32-unknown-unknown bytecode is platform-neutral — identical on amd64 and arm64.
FROM --platform=$BUILDPLATFORM rust:slim AS build

WORKDIR /app

# Install Node.js (needed to run wasm-pack's bundled tools and for Vite build)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack (puts binary in ~/.cargo/bin)
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Ensure cargo bin dir is on PATH for subsequent RUN layers
ENV PATH="/root/.cargo/bin:${PATH}"

# Add the WebAssembly compilation target
RUN rustup target add wasm32-unknown-unknown

# Copy Rust manifests first — enables Docker layer caching for cargo fetch.
# Changes to src/ won't invalidate the cargo dependency layer.
COPY Cargo.toml Cargo.lock ./

# Create a stub lib.rs so cargo can parse/fetch deps without the real source
RUN mkdir -p src && echo '#[allow(dead_code)] fn main() {}' > src/lib.rs

# Pre-fetch Rust dependencies (cached layer; invalidated only when Cargo.lock changes)
RUN cargo fetch

# Copy the real source and build WASM
COPY src/ ./src/
RUN wasm-pack build --target web --release

# Copy JS/CSS manifests for npm install (cached layer separate from source changes)
COPY package.json package-lock.json ./
RUN npm ci

# Copy remaining frontend source and build the Vite production bundle
COPY index.html vite.config.js ./
RUN npm run build
# Output: dist/ containing index.html, assets/*.js, assets/*.css, assets/*.wasm

# Serve stage: minimal nginx:alpine image (~8MB) with only the static files
FROM nginx:alpine AS serve

# Replace the default nginx site config with our custom one
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy only the production build output — no Rust toolchain, no node_modules, no source
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80
