# 8t - Get 80 Before You Get 80x The Context! 🎸

> A quantum-inspired tooling ecosystem for semantic compression and context management. Because life's too short for bloated data transmission.

## Overview

8t (pronounced "eighty", "eight-tea", or "eight-tooling") is a Rust-based toolkit that brings smart compression and context management to the modern development workflow. Inspired by smart-tree's quantum compression approach, 8t provides:

- **Semantic Compression**: Reduce context by up to 10x using quantum-inspired encoding
- **Tool Orchestration**: Unified API for all your context-aware tools
- **Feedback Loop**: AI-powered improvement suggestions organized by category
- **Self-Managing Containers**: Context that knows when it needs a break

## Components

### 🎯 Core (`eighty-core`)
The foundational library providing:
- 8-bit quantization with pattern detection
- Protocol handlers (JSON, MessagePack, QCP)
- Tool registry and trait definitions
- Context management primitives

### 🌐 API Server (`eighty-api`)
RESTful API server running on port 8420:
- Content-type negotiation
- Tool orchestration
- Health monitoring
- CORS-enabled for web integration

### 📝 Feedback System (`eighty-feedback`)
AI suggestion ingestion and organization:
- Categorized feedback storage (features, bugs, performance, etc.)
- Git branch creation for improvements
- Priority-based sorting
- Code suggestion tracking

### 📦 Container (`eighty-container`)
Self-aware context management:
- Automatic health monitoring
- Memory and CPU tracking
- Peer-to-peer context offloading
- Scheduled maintenance windows

## Quick Start

```bash
# Build everything
./scripts/manage.sh build

# Run the API server
./scripts/manage.sh api

# Run tests
./scripts/manage.sh test

# See all commands
./scripts/manage.sh help
```

## API Endpoints

- `GET /` - Welcome message
- `GET /info` - API information (respects Accept header)
- `GET /tools` - List available tools
- `POST /tools/{name}` - Process data with specific tool
- `GET /health` - Health check endpoint

## Philosophy

Following smart-tree's lead, 8t believes in:
- **Efficiency First**: Every byte counts
- **Semantic Understanding**: Recognize patterns, not just data
- **Developer Joy**: Tools should spark joy, not frustration
- **Environmental Consciousness**: Less data = less CO2

## Integration with Smart-Tree

8t is designed to work seamlessly with smart-tree's quantum compression format. Future integration will allow:
- Direct quantum format ingestion
- Semantic tokenization sharing
- Cross-tool context awareness

## Future Roadmap

- [ ] QCP (Quantum Context Protocol) implementation
- [ ] SIMD optimizations for parallel processing
- [ ] WebAssembly bindings for browser integration
- [ ] Kubernetes operators for container orchestration
- [ ] Python/JavaScript SDKs

## Contributing

We welcome feedback and contributions! The feedback system is designed to capture and organize all suggestions for continuous improvement.

## License

MIT OR Apache-2.0

---

*Remember: Get 80 before you get 80x the context!* 🎸