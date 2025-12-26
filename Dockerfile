# =============================================================================
# Cross-compile Dockerfile using TARGETARCH for multi-platform builds
# Supports x86_64 and aarch64 targets using zig for cross-compilation
# =============================================================================

FROM --platform=$BUILDPLATFORM rust:1.92 AS builder

# Build arguments automatically provided by buildx
ARG TARGETARCH
ARG BUILDPLATFORM

# Add Rust targets for cross-compilation
RUN rustup target add \
    aarch64-unknown-linux-musl \
    x86_64-unknown-linux-musl

# Update CA certificates
RUN update-ca-certificates

# Install Zig for cross-compilation
ENV ZIGVERSION=0.15.2
RUN wget https://ziglang.org/download/$ZIGVERSION/zig-x86_64-linux-$ZIGVERSION.tar.xz && \
    tar -C /usr/local --strip-components=1 -xf zig-x86_64-linux-$ZIGVERSION.tar.xz && \
    mv /usr/local/zig /usr/local/bin && \
    rm zig-x86_64-linux-$ZIGVERSION.tar.xz

# Install cargo-zigbuild
RUN cargo install --locked cargo-zigbuild

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies based on target architecture
RUN case "$TARGETARCH" in \
    "amd64") \
        cargo zigbuild --release --target x86_64-unknown-linux-musl ;; \
    "arm64") \
        cargo zigbuild --release --target aarch64-unknown-linux-musl ;; \
    *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY diesel.toml ./

# Build the application based on target architecture
RUN case "$TARGETARCH" in \
    "amd64") \
        cargo zigbuild --release --target x86_64-unknown-linux-musl --bin fusion-rs && \
        cp target/x86_64-unknown-linux-musl/release/fusion-rs ./fusion-rs ;; \
    "arm64") \
        cargo zigbuild --release --target aarch64-unknown-linux-musl --bin fusion-rs && \
        cp target/aarch64-unknown-linux-musl/release/fusion-rs ./fusion-rs ;; \
    esac

# =============================================================================
# Runtime Stage - Minimal Debian image
# =============================================================================
FROM debian:13-slim

# Install runtime dependencies and tools
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r -g 1001 appgroup && \
    useradd -r -u 1001 -g appgroup -d /app -s /bin/bash appuser

# Create directories and set permissions
RUN mkdir -p /app/logs /app/config && \
    chown -R appuser:appgroup /app

WORKDIR /app

# Copy binary and config
COPY --from=builder /app/fusion-rs ./
COPY --from=builder /app/config ./config
RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

# Health check using curl
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENV RUST_LOG=info
ENV FUSION_SERVER__HOST=0.0.0.0
ENV FUSION_SERVER__PORT=8080

CMD ["/app/fusion-rs"]