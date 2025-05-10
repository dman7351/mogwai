# Build stage: Use Rust to compile a static binary
FROM rust:alpine AS builder

WORKDIR /app

# Allow specifying the target triple at build time (default: x86_64-unknown-linux-musl)
ARG TARGET_TRIPLE=x86_64-unknown-linux-musl
ENV CARGO_TARGET=${TARGET_TRIPLE}

# Install musl tools for static linking using apk (instead of apt)
RUN apk add --no-cache musl-dev musl-utils build-base

# Set Rust to use musl target for static binary
RUN rustup target add ${CARGO_TARGET}

# Copy only Cargo files first (ensures dependencies are cached)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to allow dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Pre-build dependencies and cache them
RUN cargo build --release --target ${CARGO_TARGET}

# Now copy the actual project files
COPY . .

# Rebuild with the actual source code
RUN cargo build --release --target ${CARGO_TARGET}

# Final stage: Use minimal Alpine base image
FROM alpine:latest

ARG TARGET_TRIPLE=x86_64-unknown-linux-musl

# Copy the static binary from the builder stage
COPY --from=builder /app/target/${TARGET_TRIPLE}/release/controller /usr/local/bin/controller

# Set the entrypoint to run the stress test
ENTRYPOINT ["/usr/local/bin/controller"]
