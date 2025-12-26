# =============================================================================
# Cross-compile Dockerfile supporting both x86_64-unknown-linux-musl and
# aarch64-unknown-linux-musl targets using zig to link against musl libc.
# =============================================================================

# --- Build Stage ---
FROM rust:1.92 AS builder

# Add Rust targets for cross-compilation
RUN rustup target add \
    aarch64-unknown-linux-musl \
    x86_64-unknown-linux-musl

# Update CA certificates
RUN update-ca-certificates

# Install Zig for cross-compilation
ENV ZIGVERSION=0.15.2
RUN wget https://ziglang.org/download/$ZIGVERSION/zig-linux-x86_64-$ZIGVERSION.tar.xz && \
    tar -C /usr/local --strip-components=1 -xf zig-linux-x86_64-$ZIGVERSION.tar.xz && \
    rm zig-linux-x86_64-$ZIGVERSION.tar.xz

# Install cargo-zigbuild
RUN cargo install --locked cargo-zigbuild

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies for both targets
RUN cargo zigbuild \
    --release \
    --target aarch64-unknown-linux-musl \
    --target x86_64-unknown-linux-musl && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY diesel.toml ./

# Build the application for both targets
RUN cargo zigbuild \
    --release \
    --target aarch64-unknown-linux-musl \
    --target x86_64-unknown-linux-musl \
    --bin fusion-rs

# Create non-root user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    "app"

# --- x86_64-unknown-linux-musl final image ---
FROM scratch AS amd64

# Copy user and group files
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy CA certificates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app

# Copy binary and config
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/fusion-rs ./
COPY --from=builder /app/config ./config

USER app:app

EXPOSE 8080

ENV RUST_LOG=info
ENV FUSION_SERVER__HOST=0.0.0.0
ENV FUSION_SERVER__PORT=8080

CMD ["/app/fusion-rs"]

# --- aarch64-unknown-linux-musl final image ---
FROM scratch AS arm64

# Copy user and group files
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy CA certificates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app

# Copy binary and config
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/fusion-rs ./
COPY --from=builder /app/config ./config

USER app:app

EXPOSE 8080

ENV RUST_LOG=info
ENV FUSION_SERVER__HOST=0.0.0.0
ENV FUSION_SERVER__PORT=8080

CMD ["/app/fusion-rs"]