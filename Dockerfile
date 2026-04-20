# Builder stage
FROM rust:1.80-slim as builder

WORKDIR /app

# Install dependencies required for sqlx and compilation
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build aw-api (the workspace REST server)
# Build in release mode
RUN cargo build --release -p aw-api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL is needed for Postgres connections)
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder environment
COPY --from=builder /app/target/release/aw-api /usr/local/bin/aw-api

# Set default configuration (can be overridden by docker-compose)
ENV PORT=4000
ENV STORAGE_BACKEND=postgres

# Expose the API port
EXPOSE 4000

# Start the API server
CMD ["aw-api"]
