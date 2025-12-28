# =============================================================================
# Optimized Multi-arch Dockerfile with Zig cross-compilation
# Supports: amd64 (x86_64) and arm64 (aarch64) without QEMU
# =============================================================================

# syntax=docker/dockerfile:1.4

# =============================================================================
# Stage 1: Builder - Build dependencies and application with Zig
# =============================================================================
FROM --platform=$BUILDPLATFORM rust:1.92 AS builder

# Build arguments for multi-arch support
ARG TARGETARCH
ARG BUILDPLATFORM

# Add Rust targets for cross-compilation
RUN rustup target add \
    aarch64-unknown-linux-musl \
    x86_64-unknown-linux-musl

# Update CA certificates
RUN update-ca-certificates

# Install Zig for cross-compilation (download appropriate version for build platform)
ENV ZIGVERSION=0.15.2
RUN case "$(uname -m)" in \
        "x86_64") \
            wget https://ziglang.org/download/${ZIGVERSION}/zig-x86_64-linux-${ZIGVERSION}.tar.xz && \
            tar -C /usr/local -xf zig-x86_64-linux-${ZIGVERSION}.tar.xz && \
            mv /usr/local/zig-x86_64-linux-${ZIGVERSION} /usr/local/zig ;; \
        "aarch64") \
            wget https://ziglang.org/download/${ZIGVERSION}/zig-aarch64-linux-${ZIGVERSION}.tar.xz && \
            tar -C /usr/local -xf zig-aarch64-linux-${ZIGVERSION}.tar.xz && \
            mv /usr/local/zig-aarch64-linux-${ZIGVERSION} /usr/local/zig ;; \
    esac && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig && \
    rm -f zig-*.tar.xz

# Install cargo-zigbuild
RUN cargo install --locked cargo-zigbuild

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-${TARGETARCH} \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git-${TARGETARCH} \
    case "$TARGETARCH" in \
        "amd64") \
            cargo zigbuild --release --target x86_64-unknown-linux-musl ;; \
        "arm64") \
            cargo zigbuild --release --target aarch64-unknown-linux-musl ;; \
    esac

# Remove dummy source and copy real source code
RUN rm -rf src
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY diesel.toml ./
COPY build.rs ./

# Build application with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-${TARGETARCH} \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git-${TARGETARCH} \
    --mount=type=cache,target=/app/target,id=target-${TARGETARCH} \
    case "$TARGETARCH" in \
        "amd64") \
            cargo zigbuild --release --target x86_64-unknown-linux-musl --bin fusion-rs && \
            cp target/x86_64-unknown-linux-musl/release/fusion-rs /tmp/fusion-rs ;; \
        "arm64") \
            cargo zigbuild --release --target aarch64-unknown-linux-musl --bin fusion-rs && \
            cp target/aarch64-unknown-linux-musl/release/fusion-rs /tmp/fusion-rs ;; \
    esac

# Strip binary to reduce size
RUN strip /tmp/fusion-rs

# =============================================================================
# Stage 2: Runtime - Minimal distroless image for security and size
# =============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy binary and config from builder
COPY --from=builder --chown=nonroot:nonroot /tmp/fusion-rs ./fusion-rs
COPY --from=builder --chown=nonroot:nonroot /app/config ./config

# Expose application port
EXPOSE 8080

# Environment variables
ENV RUST_LOG=info
ENV FUSION_SERVER__HOST=0.0.0.0
ENV FUSION_SERVER__PORT=8080

# Use nonroot user (uid=65532)
USER nonroot

# Run the application
ENTRYPOINT ["/app/fusion-rs"]
