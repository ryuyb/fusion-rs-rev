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

# Create non-root user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    "app"

# =============================================================================
# Runtime Stage - Minimal scratch image
# =============================================================================
FROM scratch

# Copy user and group files
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy CA certificates for HTTPS requests
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app

# Copy binary and config
COPY --from=builder /app/fusion-rs ./
COPY --from=builder /app/config ./config

USER app:app

EXPOSE 8080

ENV RUST_LOG=info
ENV FUSION_SERVER__HOST=0.0.0.0
ENV FUSION_SERVER__PORT=8080

CMD ["/app/fusion-rs"]