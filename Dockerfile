# Build Stage
FROM rust:1.88-slim-bookworm as builder
# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
# Create a dummy source file to pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/backend_rust*
# Copy actual source code
COPY . .
# Build the real binary
RUN cargo build --release
# Production Stage
FROM debian:bookworm-slim
# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy binary from builder
COPY --from=builder /app/target/release/backend-rust /app/backend-rust
# Create uploads directory if needed
RUN mkdir -p /app/uploads
EXPOSE 3000
CMD ["./backend-rust"]