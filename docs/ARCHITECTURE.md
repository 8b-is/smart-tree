# Smart Tree Architecture Documentation

## Overview

Smart Tree is a high-performance, AI-native directory intelligence and computation platform written in Rust. It combines filesystem scanning, graph-theoretic algorithms, ML inference, and persistent memory into a unified CLI/daemon/MCP server.

## Core Components

### 1. Scanner (`src/scanner.rs`)
Filesystem traversal with configurable depth, filtering, and interest scoring. Produces `TreeStats` and `FileNode` trees.

### 2. Formatters (`src/formatters/`)
Multiple output formats:
- **Classic**: Traditional tree view
- **JSON/CSV/TSV**: Structured data
- **AI/AI_JSON**: AI-optimized compressed output
- **Quantum/Quantum-Semantic**: Wave-based semantic compression
- **LOUDS/SMTR**: Bit-packed 64B capsules
- **Szemerédi/Erdős/Steiner**: Graph-theoretic views

### 3. MCP Server (`src/mcp/`)
Model Context Protocol server with 30+ tools:
- **Combinatorics/LLM**: 11 tools (Erdős-Rényi, Steiner, Moser-Tardos, Szemerédi, Ramsey, MLX Quant, Llama Rust, Ultra Zeno, Gaia Omni, Ternary Deck, Kintsugi Ledger)
- **Tools Registry**: ToolExecutor trait-based registry
- **Progress Streaming**: SSE-based progress emission

### 4. Daemon (`src/daemon.rs`)
Persistent background service:
- HTTP API on port 28428
- WebSocket for real-time updates
- LLM Proxy (OpenAI-compatible)
- Collaboration Station (Hot Tub mode)
- The Custodian (security scanner)
- GitHub OAuth

### 5. Memory System
- **MEM8**: Wave-based consciousness pyramids
- **M8**: Bottom-up hierarchical capsules
- **ContextMemory**: Real-time directory intelligence
- **ProxyMemory**: Conversation persistence

### 6. Graph-Theoretic Engine (`src/mcp/combinatorics_llm.rs`)
Bit-parallel / Rayon-parallel solvers:
- **Szemerédi Regularity**: Graph compression
- **Erdős-Rényi**: Phase transition scanning
- **Erdős-Steiner**: Steiner tree pruning
- **Moser-Tardos**: Constructive LLL
- **Ramsey Clique**: Bron-Kerbosch with degeneracy ordering

### 7. ML Inference (`src/mcp/combinatorics_llm.rs`)
- **Burn**: Local LLM inference (replaced Candle)
- **Speculative Decoding**: Tree-attention with KV radix pool
- **MLX Quant**: Apple Silicon Metal quantization
- **SIMD Kernels**: NEON/AVX2 bent 5-trit GEMV

### 6. Distributed Systems
- **Galaxia Cluster**: CRDT-based membership, gossip, leader election
- **Kintsugi Ledger**: Append-only consensus ledger with FNV-1a seals

## Data Flow

```
CLI (st) → HTTP → Daemon → Scanner → Formatters → Output
                    ↓
               MCP Server → Tools → Results
                    ↓
               Memory (MEM8/M8) → Persistence
                    ↓
               Graph Engine → Combinatorics
                    ↓
               ML Inference (Burn) → Inference
```

## Key Algorithms

### Bent 5-trit SIMD Packing (NEON/AVX2)
```
5 balanced trits/byte, base-3 packing (3^5 = 243 ≤ 256)
Position-dependent cyclic bend: r_i = (k * i) mod 3
Fused unpack-unbend-GEMV in registers
```

### Szemerédi Regularity Compression
```
ε-regular partition with energy increment
Rayon-parallel BitSet operations
StackBitSet<N> with SIMD POPCNT
```

### Erdos-Rényi Phase Transitions
```
G(n, p) scanning for giant component (1/n) and connectivity (ln n / n)
SplitMix64 deterministic scanning
```

### Burn ML Inference
- Backend: flex (ndarray), wgpu, autodiff, train
- Speculative decoding with KV radix pool
- NEON/AVX2 SIMD for token processing

## Hardware Targets
- **Primary**: Apple Silicon (M1/M2/M3) - NEON SIMD, Metal/MLX
- **Secondary**: x86_64 AVX2 - AVX2 SIMD
- **Tertiary**: ARM64 - NEON SIMD
- **GPU**: wgpu (Vulkan/Metal/DX12/WebGPU)

## Build System
```toml
# Cargo.toml features
default = ["full"]
full = ["burn", "google", "voice"]
burn = ["burn-core", "burn-import", "burn-ndarray", "tokenizers"]
metal = ["mlx-ffi"]
cuda = []
google = ["google-gmail1", "google-drive3", ...]
voice = []
```

## Testing
```bash
# All tests
cargo test --workspace

# Integration tests
cargo test --test mcp_integration
cargo test --test latent_moe_simd_test
cargo test --test latent_moe_layer_test
cargo test --test hybrid_attention_test
cargo test --test galaxia_kintsugi_e2e_test
cargo test --test vaked_foken_e2e_test

# Benchmarks
cargo bench --bench combinatorics_llm_bench
```

## Deployment
```bash
# Install
cargo install --path .

# Run daemon
st --http-daemon

# MCP mode
st --mcp

# CLI scan
st --mode ai --compress /path/to/dir
```

## Extending Smart Tree

### Adding a New MCP Tool
1. Create module in `src/mcp/combinatorics_llm/tools/`
2. Implement `ToolExecutor` trait
3. Register in `tools/mod.rs`

### Adding a New Formatter
1. Create module in `src/formatters/`
2. Implement `Formatter` trait
2. Register in `formatters/mod.rs`

### Adding a Sovereign Model
1. Add alias to `ModelConfig::default()` in `config.rs`
2. Model file goes in `models/` directory
3. Preload at daemon startup via `model_cache.preload_named_model()`

## Security
- **MagiSCanner**: AST & supply-chain threat detection
- **The Custodian**: Runtime operation monitoring
- **CAPSULE**: 64B LOUDS + 8BIS seal
- **Jury PID**: Stability certificates for adaptive control

## Performance Targets
- **Scanning**: 10-24x faster than `tree`
- **SIMD GEMV**: 160 trits/iter (AVX2), 80 trits/iter (NEON)
- **Scanning**: ~500K files/sec
- **Memory**: <50MB base, <200MB with ML models
- **Startup**: <100ms daemon, <10ms CLI