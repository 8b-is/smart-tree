# Smart Tree as a Public Rust Crate

## Current State Analysis

Smart-tree is already structured as a library with a binary! The `src/lib.rs` exposes the core functionality, making it ready for use as a crate with minimal changes.

## Publishing Plan

### 1. Rename Package for Crates.io
```toml
[package]
name = "smart-tree"  # Change from "st" to avoid conflicts
version = "3.2.0"
```

Keep the binary name as `st`:
```toml
[[bin]]
name = "st"
path = "src/main.rs"
```

### 2. Library Structure (Already Good!)
Current `src/lib.rs` already exposes:
- Scanner and ScannerConfig
- All formatters
- File analysis types
- MCP server components

### 3. Feature Flags for Optional Dependencies
```toml
[features]
default = ["cli", "mcp"]
cli = ["clap", "clap_complete", "clap_mangen", "colored", "indicatif"]
mcp = ["tokio", "async-trait", "futures", "dashmap"]
core = []  # Just the core scanning and formatting

# Allow users to pick formatters
all-formatters = ["quantum", "semantic", "mermaid"]
quantum = []
semantic = []
mermaid = ["termimad"]
```

### 4. Clean Library API

Create a cleaner high-level API in `src/lib.rs`:

```rust
// High-level convenience API
pub mod prelude {
    pub use crate::{
        Scanner, ScannerConfig, TreeStats,
        FileNode, FileCategory, FilesystemType,
        formatters::{Formatter, StreamingFormatter},
    };
}

// Easy-to-use functions
pub fn scan_directory(path: impl AsRef<Path>) -> Result<Vec<FileNode>> {
    Scanner::new(ScannerConfig::default()).scan(path)
}

pub fn quick_tree(path: impl AsRef<Path>, depth: usize) -> Result<String> {
    let config = ScannerConfig {
        max_depth: depth,
        ..Default::default()
    };
    let nodes = Scanner::new(config).scan(path)?;
    let formatter = SummaryAiFormatter::new();
    formatter.format(&nodes)
}

pub fn analyze_project(path: impl AsRef<Path>) -> Result<ProjectAnalysis> {
    // High-level project analysis
}
```

### 5. Example Usage in Other Projects

```rust
// Cargo.toml
[dependencies]
smart-tree = "3.3.5"
# Or with specific features
smart-tree = { version = "3.3.5", features = ["quantum", "semantic"] }

// main.rs
use smart_tree::prelude::*;
use smart_tree::formatters::ai::AiFormatter;

fn main() -> Result<()> {
    // Simple directory scan
    let files = smart_tree::scan_directory(".")?;
    
    // Custom formatting
    let formatter = AiFormatter::new();
    let output = formatter.format(&files)?;
    println!("{}", output);
    
    // Or use the high-level API
    let tree = smart_tree::quick_tree(".", 3)?;
    println!("{}", tree);
    
    Ok(())
}
```

### 6. MCP Server as Optional Feature

```rust
// Using smart-tree's MCP server in your app
#[cfg(feature = "mcp")]
use smart_tree::mcp::{McpServer, McpConfig};

#[cfg(feature = "mcp")]
async fn run_mcp_server() {
    let config = McpConfig::default();
    let server = McpServer::new(config);
    server.run().await.unwrap();
}
```

### 7. Documentation Requirements

1. **Crate-level docs** in `src/lib.rs`:
```rust
//! # Smart Tree
//! 
//! A blazingly fast, AI-friendly directory visualization library.
//! 
//! ## Quick Start
//! ```
//! use smart_tree::prelude::*;
//! 
//! let files = smart_tree::scan_directory(".")?;
//! let tree = smart_tree::quick_tree(".", 3)?;
//! ```
```

2. **Module docs** for each major component
3. **Examples** in `examples/` directory
4. **README** updates for crates.io

### 8. Breaking Changes Needed

1. **Separate CLI from Library**
   - Move CLI-specific code to `src/bin/st.rs` or keep in `main.rs`
   - Ensure library doesn't depend on CLI features

2. **Clean up Public API**
   - Hide internal implementation details
   - Use `pub(crate)` for internal items
   - Provide builder patterns for complex configs

3. **Error Handling**
   - Create library-specific error types
   - Don't expose `anyhow::Error` in public API

### 9. Publishing Checklist

- [ ] Rename package to `smart-tree`
- [ ] Add feature flags
- [ ] Clean up public API
- [ ] Add comprehensive docs
- [ ] Add examples
- [ ] Test as dependency in another project
- [ ] Update README for crates.io
- [ ] Add CI for testing all feature combinations
- [ ] Run `cargo publish --dry-run`
- [ ] Publish to crates.io: `cargo publish`

### 10. Versioning Strategy

- Current: 3.2.0
- After publishing: Follow semver strictly
- Major version bump for breaking API changes
- Consider 4.0.0 for the cleaned-up library API

## Benefits of Publishing

1. **Reusable in Other Projects**: Add directory analysis to any Rust project
2. **Custom Formatters**: Users can implement their own formatters
3. **Embedded MCP**: Add MCP directory tools to any application
4. **Building Blocks**: Use Scanner for custom directory operations
5. **AI Integration**: Easy directory analysis for AI applications

## Example: Using in a Build Tool

```rust
use smart_tree::{Scanner, ScannerConfig};
use smart_tree::formatters::digest::DigestFormatter;

pub fn verify_project_structure() -> Result<()> {
    let config = ScannerConfig {
        file_type_filter: Some(vec!["rs", "toml"]),
        ..Default::default()
    };
    
    let scanner = Scanner::new(config);
    let files = scanner.scan(".")?;
    
    // Get digest for reproducible builds
    let formatter = DigestFormatter::new();
    let digest = formatter.format(&files)?;
    
    println!("Project digest: {}", digest);
    Ok(())
}
```

This would make smart-tree a powerful building block for other Rust projects!