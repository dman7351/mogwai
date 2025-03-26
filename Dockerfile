# Build stage: Use Rust to compile a static binary
FROM rust:latest AS builder

WORKDIR /app

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools

# Set Rust to use musl target for static binary
RUN rustup target add x86_64-unknown-linux-musl

# Copy Cargo files first (for caching dependencies)
COPY Cargo.toml Cargo.lock ./

# This step ensures that dependencies are cached
RUN cargo fetch

# Now copy the source code
COPY src ./src

# Build a fully static Rust binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage: Use minimal Alpine base image
FROM alpine:latest

# Copy the static binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/stress-test /usr/local/bin/stress-test

# Set the entrypoint to run the stress test
ENTRYPOINT ["/usr/local/bin/stress-test"]
