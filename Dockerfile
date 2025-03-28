# Build stage: Use Rust to compile a static binary
FROM rust:latest AS builder

WORKDIR /app

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools

# Set Rust to use musl target for static binary
RUN rustup target add x86_64-unknown-linux-musl

# --------------------------------------------
# Step 1: Cache Dependencies Efficiently
# --------------------------------------------

# Copy only Cargo files first (ensures dependencies are cached)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to allow dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Pre-build dependencies and cache them
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------------------------------------
# Step 2: Copy the Actual Source Code
# --------------------------------------------

# Now copy the actual project files
COPY . .

# Rebuild with the actual source code
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------------------------------------
# Step 3: Create Minimal Final Image
# --------------------------------------------

# Final stage: Use minimal Alpine base image
FROM alpine:latest

# Copy the static binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/stress-test /usr/local/bin/stress-test

# Set the entrypoint to run the stress test
ENTRYPOINT ["/usr/local/bin/stress-test"]
