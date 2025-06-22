// This is the main library file for `stree`.
// It's like the table of contents for our awesome codebase,
// declaring the modules that make up the `stree` library and
// re-exporting key items for convenient use.
// Think of it as the friendly librarian pointing you to the right sections!

// Declare the public modules that form the `stree` library.
pub mod scanner;    // The heart of directory traversal and file metadata collection.
pub mod formatters; // Home to all the different ways we can display the tree (Classic, JSON, AI, etc.).
pub mod context;    // For intelligently detecting project context (e.g., Rust, Node.js).

// Conditionally compile and declare the `mcp` module.
// This module is only included if the "mcp" feature flag is enabled during compilation.
// MCP stands for Model Context Protocol, enabling AI assistant integration.
#[cfg(feature = "mcp")]
pub mod mcp;

// Re-export key items from the `scanner` module for easier access.
// This means users of the `stree` library can use `stree::Scanner`
// instead of `stree::scanner::Scanner`, for example. It's all about convenience!
pub use scanner::{Scanner, ScannerConfig, FileNode, FileCategory, FilesystemType, TreeStats, parse_size};