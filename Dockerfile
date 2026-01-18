# Multi-stage Dockerfile for Vulcan CI services
# Usage: docker build --build-arg SERVICE=vulcan-worker-orchestrator -t vulcan-worker-orchestrator .

FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /app

# Install dependencies for diesel (PostgreSQL client)
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build argument to specify which service to build
ARG SERVICE=vulcan-worker-orchestrator

# Build the specified service in release mode
RUN cargo build --release --package ${SERVICE}

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build argument to specify which service binary to copy
ARG SERVICE=vulcan-worker-orchestrator

# Copy the binary from builder
COPY --from=builder /app/target/release/${SERVICE} /app/service

# Run the service
CMD ["/app/service"]
