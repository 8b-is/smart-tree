# syntax=docker/dockerfile:1.7

# ============================================================================
# Stage 1: Base Builder Toolchain (CUDA 12.6.3 + Rust + cargo-chef)
# ============================================================================
FROM nvidia/cuda:12.6.3-devel-ubuntu24.04 AS chef

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:/usr/local/cuda/bin:${PATH}" \
    CUDA_PATH="/usr/local/cuda" \
    CMAKE_CUDA_ARCHITECTURES="80;86;89;90" \
    RUSTFLAGS="-C target-cpu=x86-64-v3"

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        libclang-dev \
        cmake \
        ninja-build \
        pkg-config \
        curl \
        ca-certificates \
        git \
        libgomp1 \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable \
    && cargo install cargo-chef --locked

WORKDIR /app

# ============================================================================
# Stage 2: Compute Cargo Dependency Recipe
# ============================================================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock* build.rs ./
COPY st-protocol ./st-protocol
COPY expert_prompt_engineer ./expert_prompt_engineer
COPY src ./src
COPY benches ./benches
COPY tests ./tests
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 3: Compile Cached Dependencies & MCP Server Binary with CUDA
# ============================================================================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Link against libcuda.so stub during container build when no physical GPU is attached to the build daemon
ENV LIBRARY_PATH="/usr/local/cuda/lib64/stubs:${LIBRARY_PATH}" \
    LD_LIBRARY_PATH="/usr/local/cuda/lib64:/usr/local/cuda/lib64/stubs:${LD_LIBRARY_PATH}"

RUN ln -sf /usr/local/cuda/lib64/stubs/libcuda.so /usr/local/cuda/lib64/stubs/libcuda.so.1 \
    && cargo chef cook --release --features "cuda" --recipe-path recipe.json

COPY Cargo.toml Cargo.lock* build.rs ./
COPY st-protocol ./st-protocol
COPY expert_prompt_engineer ./expert_prompt_engineer
COPY src ./src
RUN cargo build --release --bin st --features "cuda" \
    && strip /app/target/release/st \
    && cp /app/target/release/st /app/target/release/mcp-server

# ============================================================================
# Stage 4: Slim Production CUDA Runtime Image
# ============================================================================
FROM nvidia/cuda:12.6.3-runtime-ubuntu24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive \
    NVIDIA_VISIBLE_DEVICES=all \
    NVIDIA_DRIVER_CAPABILITIES=compute,utility \
    MCP_MODEL_CACHE_SIZE=4 \
    RUST_LOG=info

RUN apt-get update && apt-get install -y --no-install-recommends \
        libgomp1 \
        libcublas-12-6 \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 mcp \
    && useradd -u 10001 -g mcp -m -s /sbin/nologin mcp \
    && mkdir -p /models && chown -R mcp:mcp /models

COPY --from=builder /app/target/release/st /usr/local/bin/st
COPY --from=builder /app/target/release/mcp-server /usr/local/bin/mcp-server

USER mcp
WORKDIR /models
VOLUME ["/models"]

EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/health || exit 1

# Defaults to MCP mode
ENTRYPOINT ["/usr/local/bin/st"]
CMD ["--mcp"]
