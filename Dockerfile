# Multi-stage Dockerfile for Ruchy v1.5.0
# The world's first self-hosting MCP-first programming language

# Build stage
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/ruchy

# Copy source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples

# Build the release binary
RUN cargo build --release

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/ruchy/target/release/ruchy /usr/local/bin/ruchy

# Copy examples and documentation
COPY --from=builder /usr/src/ruchy/examples /usr/share/ruchy/examples
COPY README.md /usr/share/ruchy/
COPY SELF_HOSTING_ACHIEVEMENT.md /usr/share/ruchy/
COPY LICENSE /usr/share/ruchy/

# Create non-root user
RUN useradd -m -u 1000 ruchy && \
    mkdir -p /workspace && \
    chown -R ruchy:ruchy /workspace

# Set working directory
WORKDIR /workspace

# Switch to non-root user
USER ruchy

# Verify installation
RUN ruchy --version

# Labels
LABEL org.opencontainers.image.title="Ruchy" \
      org.opencontainers.image.description="The world's first self-hosting MCP-first programming language" \
      org.opencontainers.image.version="1.5.0" \
      org.opencontainers.image.authors="Ruchy Contributors" \
      org.opencontainers.image.source="https://github.com/paiml/ruchy" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.documentation="https://docs.ruchy-lang.org"

# Default command
ENTRYPOINT ["ruchy"]
CMD ["--help"]