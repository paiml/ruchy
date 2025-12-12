# Ruchy Build Environment - Reproducible Dockerfile
# Version: 1.0.0
# Purpose: Ensure deterministic builds across all environments

# Build stage with pinned Rust version
FROM rust:1.83.0-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set up cargo configuration for reproducibility
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV RUSTFLAGS="-C target-cpu=generic"
ENV SOURCE_DATE_EPOCH=1704067200

# Install additional Rust components
RUN rustup component add clippy rustfmt

# Create working directory
WORKDIR /build

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY ruchy-wasm/Cargo.toml ./ruchy-wasm/

# Create dummy src for dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN mkdir -p src/lib.rs && echo "" > src/lib.rs
RUN mkdir -p ruchy-wasm/src && echo "" > ruchy-wasm/src/lib.rs

# Build dependencies only
RUN cargo build --release --locked 2>/dev/null || true

# Remove dummy sources
RUN rm -rf src ruchy-wasm/src

# Copy actual source code
COPY src ./src
COPY ruchy-wasm ./ruchy-wasm
COPY benches ./benches
COPY tests ./tests
COPY examples ./examples

# Build the actual project
RUN cargo build --release --locked

# Run tests to verify build
RUN cargo test --release --locked

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/ruchy /usr/local/bin/ruchy

# Set reproducibility metadata
LABEL org.opencontainers.image.title="Ruchy"
LABEL org.opencontainers.image.description="A programming language with Python syntax and Rust performance"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/noahgift/ruchy"
LABEL org.opencontainers.image.created="2024-01-01T00:00:00Z"

ENTRYPOINT ["ruchy"]
CMD ["--help"]
