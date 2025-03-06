# Build stage: Use Rust to compile a static binary
FROM --platform=linux/amd64 rust:latest AS builder

WORKDIR /app

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools

# Set Rust to use musl
RUN rustup target add x86_64-unknown-linux-musl

# Copy Cargo files separately to optimize caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build a fully static Rust binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage: Use minimal Alpine base image
FROM --platform=linux/amd64 alpine:latest

WORKDIR /app

# Copy the compiled binary with the updated name
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/stress-test /usr/local/bin/stress-test

# Set the entrypoint to run the stress test
ENTRYPOINT ["/usr/local/bin/stress-test"]
