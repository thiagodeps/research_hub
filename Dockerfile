# Reproducible Linux bundles for ResearchHub.
#
# The point of this image is the BUILD, not the run: it produces .deb, .rpm and
# .AppImage without installing webkit, Rust and Node on the host. ResearchHub is
# a desktop application — running it in a container would mean forwarding an X
# socket, which is strictly worse than the installer this image generates.
#
#   make docker-bundle
#
# Ubuntu 24.04 on purpose: building against an older glibc than the developer
# machine keeps the binaries runnable on more systems, never fewer.
#
# Written for the classic builder, so no BuildKit cache mounts or --output.
# Layer order is what provides caching here: system packages, then toolchains,
# then frontend dependencies, and only then the sources.

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/node/bin:$PATH

# Tauri's Linux dependencies, plus what the deb/rpm/AppImage bundlers shell out to.
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential ca-certificates curl file pkg-config \
      libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
      libayatana-appindicator3-dev librsvg2-dev \
      desktop-file-utils fakeroot rpm xz-utils \
    && rm -rf /var/lib/apt/lists/*

ARG RUST_VERSION=1.98.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --default-toolchain ${RUST_VERSION} --profile minimal

ARG NODE_VERSION=22.13.0
RUN curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
      | tar -xJ -C /usr/local \
    && mv /usr/local/node-v${NODE_VERSION}-linux-x64 /usr/local/node

RUN cargo install tauri-cli --version "^2" --locked

WORKDIR /app

# Frontend dependencies change far less often than sources, so this layer
# survives most rebuilds.
COPY frontend/package.json frontend/package-lock.json* frontend/
RUN cd frontend && (npm ci 2>/dev/null || npm install)

# Rust dependencies likewise: compile them against a stub before the real
# sources arrive, so editing our own code does not rebuild arrow and parquet.
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock* src-tauri/
RUN mkdir -p src-tauri/src \
    && echo 'fn main() {}' > src-tauri/src/main.rs \
    && echo '' > src-tauri/src/lib.rs \
    && cd src-tauri && (cargo build --release 2>/dev/null || true)

COPY . .

# `touch` defeats the stub's cached mtimes, which would otherwise convince
# cargo the real sources are already built.
RUN touch src-tauri/src/*.rs \
    && cd src-tauri && cargo tauri build --bundles deb,rpm,appimage \
    && mkdir -p /out \
    && find target/release/bundle -type f \
         \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) \
         -exec cp {} /out/ \;

# Extracted by `make docker-bundle`; the image is a build tool, not a service.
CMD ["sh", "-c", "ls -lh /out"]
