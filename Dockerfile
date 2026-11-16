# Multi-stage Dockerfile for TaskMaster Rust Todo Application
# Stage 1: Build binary
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/todo

# Install required build tools and GUI/X11/Wayland libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxinerama-dev \
    libxkbcommon-dev \
    libwayland-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies and cache them
RUN mkdir src tests && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub mod dummy;" > src/lib.rs && \
    touch src/dummy.rs && \
    cargo build --release || true && \
    rm -rf src tests

# Copy actual source files and static assets
COPY src ./src
COPY static ./static
COPY tests ./tests

# Build optimized release binary
RUN cargo build --release --bin todo

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libx11-6 \
    libxcursor1 \
    libxrandr2 \
    libxi6 \
    libxinerama1 \
    libxkbcommon0 \
    libwayland-client0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/todo/target/release/todo /usr/local/bin/todo

# Create persistent data directory
RUN mkdir -p /data && chown -R 1000:1000 /data /app

ENV TODO_STORAGE_PATH=/data/todos.json
ENV RUST_LOG=info

VOLUME ["/data"]
EXPOSE 8000

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/api/health || exit 1

ENTRYPOINT ["todo"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8000"]
