// This is the main library file for `st`.
// It's like the table of contents for our awesome codebase,
// declaring the modules that make up the `st` library and
// re-exporting key items for convenient use.
// Think of it as the friendly librarian pointing you to the right sections!

// Declare the public modules that form the `st` library.
pub mod context;
pub mod formatters; // Home to all the different ways we can display the tree (Classic, JSON, AI, etc.).
pub mod scanner; // The heart of directory traversal and file metadata collection. // For intelligently detecting project context (e.g., Rust, Node.js).
pub mod quantum_scanner; // The native quantum format tree walker - no intermediate representation!
pub mod tokenizer; // Smart tokenization for semantic pattern recognition
pub mod decoders; // Decoders to convert quantum format to other representations

// Conditionally compile and declare the `mcp` module.
// This module is only included if the "mcp" feature flag is enabled during compilation.
// MCP stands for Model Context Protocol, enabling AI assistant integration.
#[cfg(feature = "mcp")]
pub mod mcp;

// Re-export key items from the `scanner` module for easier access.
// This means users of the `st` library can use `st::Scanner`
// instead of `st::scanner::Scanner`, for example. It's all about convenience!
pub use scanner::{
    parse_size, FileCategory, FileNode, FilesystemType, Scanner, ScannerConfig, TreeStats,
};
