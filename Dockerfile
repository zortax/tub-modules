# Multi-stage Dockerfile for Leptos app with sqlx offline mode

# Builder stage
FROM rustlang/rust:nightly-bookworm AS builder

# Install Node.js for Tailwind CSS compilation
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for faster binary installations
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install build dependencies and wasm tooling using binstall (much faster)
RUN rustup target add wasm32-unknown-unknown && \
    cargo binstall -y cargo-leptos@0.2.45 && \
    cargo binstall -y wasm-bindgen-cli@0.2.103

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY Cargo.lock ./

# Copy all crate directories
COPY app ./app
COPY db ./db
COPY moses-scraper ./moses-scraper

# Copy sqlx offline query data for compilation without database
COPY .sqlx ./.sqlx

# Install npm dependencies for Tailwind CSS
WORKDIR /app/app
RUN npm install

# Build the application in release mode with SSR features
# This builds both the server binary and WASM client
WORKDIR /app
RUN cargo leptos build --release -vv

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy the server binary from builder
COPY --from=builder /app/target/release/app /app/app

# Copy the site directory with compiled WASM and assets
COPY --from=builder /app/target/site /app/site

# Copy migrations (in case they're needed at runtime)
COPY --from=builder /app/db/migrations /app/migrations

# Set ownership to non-root user
RUN chown -R appuser:appuser /app

USER appuser

# Expose the port the app runs on
EXPOSE 3000

# Set environment variables for production
ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV RUST_LOG=info

# Run the server binary
CMD ["/app/app"]
