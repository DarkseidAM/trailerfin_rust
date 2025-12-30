# Multi-stage build for trailerfin_rust
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Set working directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached unless Cargo.lock changes)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    rm -rf target/release/deps/trailerfin_rust*

# Copy source code
COPY src/ ./src/

# Build application with dependency cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && \
    strip target/release/trailerfin_rust && \
    cp target/release/trailerfin_rust /tmp/trailerfin_rust

# Runtime stage - use specific version for reproducibility
FROM alpine:3.21

# Install runtime dependencies in a single layer
RUN apk add --no-cache ca-certificates tzdata && \
    addgroup -g 1001 -S trailerfin && \
    adduser -u 1001 -S trailerfin -G trailerfin && \
    mkdir -p /config /mnt/plex

# Set working directory
WORKDIR /app

# Copy stripped binary from builder stage
COPY --from=builder /tmp/trailerfin_rust /app/trailerfin_rust

# Set default user (can be overridden)
USER trailerfin

# Expose any necessary ports (if needed in the future)
# EXPOSE 8080

# Set default environment variables
ENV TRAILERFIN_SCAN_PATH=/mnt/plex \
    TRAILERFIN_CACHE_PATH=/config \
    TRAILERFIN_THREADS=1 \
    TRAILERFIN_SHOULD_SCHEDULE=false \
    TRAILERFIN_DATA_SOURCE=imdb \
    TRAILERFIN_VIDEO_FILENAME=video1.strm \
    TRAILERFIN_USER_AGENT="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0.0.0" \
    TRAILERFIN_IMDB_RATE_LIMIT=30/minute \
    TRAILERFIN_TMDB_RATE_LIMIT=50/second \
    TRAILERFIN_IMDB_ID_REGEX='\{imdb-(tt\d+)\}' \
    TRAILERFIN_TMDB_ID_REGEX='\{tmdb-(\d+)\}'

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ps aux | grep trailerfin_rust || exit 1

# Run the application
ENTRYPOINT ["/app/trailerfin_rust"]
