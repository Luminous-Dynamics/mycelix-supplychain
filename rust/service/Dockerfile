# Multi-stage build for Rust service

FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY claim-model ./claim-model
COPY crypto ./crypto
COPY service ./service

# Build release
WORKDIR /app/service
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/provenance-service /app/provenance-service

# Create data directory
RUN mkdir -p /app/data

EXPOSE 8080

CMD ["/app/provenance-service"]
