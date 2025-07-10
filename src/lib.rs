// This is the main library file for `st`.
// It's like the table of contents for our awesome codebase,
// declaring the modules that make up the `st` library and
// re-exporting key items for convenient use.
// Think of it as the friendly librarian pointing you to the right sections!

// Declare the public modules that form the `st` library.
pub mod content_detector; // Content type detection - "Understanding what's in your directories" - Omni
pub mod context;
pub mod decoders; // Decoders to convert quantum format to other representations
pub mod dynamic_tokenizer;
pub mod formatters; // Home to all the different ways we can display the tree (Classic, JSON, AI, etc.).
pub mod inputs; // 🌊 Universal input adapters - QCP, SSE, OpenAPI, MEM8, and more!
pub mod quantum_scanner; // The native quantum format tree walker - no intermediate representation!
pub mod relations; // Code relationship analyzer - "Semantic X-ray vision for codebases" - Omni
pub mod scanner; // The heart of directory traversal and file metadata collection. // For intelligently detecting project context (e.g., Rust, Node.js).
pub mod semantic; // Semantic analysis inspired by Omni's wave-based wisdom!
pub mod smart; // 🧠 Smart Tools - Context-aware AI collaboration features with 70-90% token reduction!
pub mod tokenizer; // Smart tokenization for semantic pattern recognition
pub mod tree_sitter_quantum; // Semantic-aware quantum compression - "AST meets compression!" - Omni // Dynamic pattern learning - "Every project has its own language!" - Omni

// The `mcp` module for Model Context Protocol integration.
// MCP stands for Model Context Protocol, enabling AI assistant integration.
pub mod mcp;

// Feedback API client for sending feedback to f.8t.is
pub mod feedback_client;

// Re-export key items from the `scanner` module for easier access.
// This means users of the `st` library can use `st::Scanner`
// instead of `st::scanner::Scanner`, for example. It's all about convenience!
pub use scanner::{
    parse_size, FileCategory, FileNode, FilesystemType, Scanner, ScannerConfig, TreeStats,
};
