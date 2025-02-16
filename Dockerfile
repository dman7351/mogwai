# Build stage: Use Alpine with musl to create a static binary
FROM rust:latest AS builder

WORKDIR /app

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools

# Set Rust to use musl
RUN rustup target add x86_64-unknown-linux-musl

# Copy source files
COPY . .

# Build a fully static Rust binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage: Use minimal Alpine base image
FROM alpine:latest

WORKDIR /app

# Copy the statically linked Rust binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-stress-test /usr/local/bin/stress-test

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/stress-test"]
