# Multi-stage build for trailerfin_rust
FROM rust:1.87.0-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release

# Remove dummy main.rs and copy real source code
RUN rm src/main.rs
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create non-root user (optional - can be overridden in docker-compose)
RUN addgroup -g 1001 -S trailerfin && \
    adduser -u 1001 -S trailerfin -G trailerfin

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/trailerfin_rust /app/trailerfin_rust

# Create necessary directories with flexible ownership
RUN mkdir -p /config /mnt/plex

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
