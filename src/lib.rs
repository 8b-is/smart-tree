// This is the main library file for `st`.
#![allow(dead_code)] // TODO: Clean up unused code in next refactor
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::collapsible_match)]
// It's like the table of contents for our awesome codebase,
// declaring the modules that make up the `st` library and
// re-exporting key items for convenient use.
// Think of it as the friendly librarian pointing you to the right sections!

// Declare the public modules that form the `st` library.
pub mod activity_logger; // Transparent activity logging in JSONL format
pub mod cli; // Command-line argument definitions (extracted from main.rs)
pub mod compression_manager; // Smart global compression for all outputs
pub mod content_detector; // Content type detection - "Understanding what's in your directories" - Omni
pub mod context;
pub mod decoders; // Decoders to convert quantum format to other representations
pub mod dynamic_tokenizer;
pub mod feature_flags; // Enterprise-friendly feature control and compliance
pub mod formatters; // Home to all the different ways we can display the tree (Classic, JSON, AI, etc.).
pub mod inputs; // 🌊 Universal input adapters - QCP, SSE, OpenAPI, MEM8, and more!
pub mod m8_backwards_reader; // Backwards reading - C64 tape style!
pub mod m8_context_aware; // Context-aware progressive loading
pub mod mega_session_manager; // Mega session persistence in ~/.mem8/
pub mod memory_manager; // Real memory management for consciousness!
pub mod quantum_scanner; // The native quantum format tree walker - no intermediate representation!
pub mod relations; // Code relationship analyzer - "Semantic X-ray vision for codebases" - Omni
pub mod scanner; // The heart of directory traversal and file metadata collection. // For intelligently detecting project context (e.g., Rust, Node.js).
pub mod scanner_safety; // Safety mechanisms to prevent crashes on large directories
pub mod semantic; // Semantic analysis inspired by Omni's wave-based wisdom!
pub mod smart; // 🧠 Smart Tools - Context-aware AI collaboration features with 70-90% token reduction!
#[cfg(feature = "tui")]
pub mod terminal; // 🚀 Smart Tree Terminal Interface - Your coding companion that anticipates your needs!
pub mod tokenizer; // Smart tokenization for semantic pattern recognition
pub mod tree_sitter_quantum;
pub mod universal_chat_scanner; // Finds conversations everywhere!
pub mod universal_format_detector; // Detects format by structure! // Semantic-aware quantum compression - "AST meets compression!" - Omni // Dynamic pattern learning - "Every project has its own language!" - Omni

// The `mcp` module for Model Context Protocol integration.
// MCP stands for Model Context Protocol, enabling AI assistant integration.
pub mod mcp;

// Feedback API client for sending feedback to f.8b.is
pub mod feedback_client;

// Claude integration initializer - auto-configures optimal .claude directory
pub mod claude_init;

// AI integration installer - unified setup for all AI platforms
pub mod ai_install;

// Integration helpers for easier usage in other applications
pub mod integration;

// Project renaming - elegant identity transition
pub mod rename_project;

// Smart Tips System - helpful hints without the hassle!
pub mod tips;

// LLM Proxy - Unified AI interface
pub mod proxy;

// Code Review - AI-powered code review with Grok, OpenRouter, and more
pub mod code_review;

// Daemon and client for always-on AI context service
pub mod daemon;
pub mod daemon_client;

// Self-update mechanism - check and install updates from GitHub releases
pub mod updater;

// Project tags management - tag and categorize projects
pub mod project_tags;

// Spicy TUI - cyberpunk-style terminal interface! (requires `tui` feature)
#[cfg(feature = "tui")]
pub mod spicy_fuzzy;
#[cfg(feature = "tui")]
pub mod spicy_tui_enhanced;

// Emoji mapping - bringing life to file types!
pub mod emoji_mapper;
// pub mod emotional_depth; // 🎭 Smart Tree has feelings about directories! TODO: Fix implementation

// Re-export key items from the `scanner` module for easier access.
// This means users of the `st` library can use `st::Scanner`
// instead of `st::scanner::Scanner`, for example. It's all about convenience!
pub use scanner::{
    parse_size, FileCategory, FileNode, FilesystemType, Scanner, ScannerConfig, TreeStats,
};

// Re-export context detection for easy access
pub use context::detect_project_context;

// Re-export integration helpers for convenient usage
pub use integration::{analyze_project, quick_project_overview, ProjectAnalysis, ProjectAnalyzer};

// File history tracking - The ultimate context-driven system!
pub mod file_history;

// MEM8 - Wave-based cognitive architecture for consciousness simulation
pub mod mem8;

// Tree Agent - Living forest orchestrator for AI-human development
pub mod tree_agent;

// Context Gatherer - Searches AI tool directories for project context
pub mod context_gatherer;

// AI Output Discipline - Omni's efficiency manifesto implementation
pub mod ai_output;

// ST Unified System - One tool to rule them all!
pub mod st_context_aware;
pub mod st_unified;
pub mod tools_st_only;

// Smart Edit Diff Storage
pub mod smart_edit_diff;

// Rust Shell - Ultimate collaborative interface with casting support
pub mod rust_shell;

// Q8-Caster Bridge - Integration with quantum casting system
pub mod q8_caster_bridge;

// VAD with Marine Algorithm - Voice Activity Detection from MEM8
pub mod vad_marine;

// egui Dashboard - Real-time collaborative dashboard (requires `dashboard` feature)
#[cfg(feature = "dashboard")]
pub mod dashboard_egui;

// Dashboard WebSocket Server - Real-time bidirectional communication for telepathic pair programming!
#[cfg(feature = "dashboard")]
pub mod dashboard_ws;

// MEM8 Binary Format - The REAL wave-based .m8 format
pub mod mem8_binary;

// M8 Format Converter - Convert between .m8, .m8j, .m8z, and .mq formats
pub mod m8_format_converter;

// Quantum Wave Signatures - Full 32-bit consciousness patterns (not horse apples!)
pub mod quantum_wave_signature;

// Wave Compass - Omni's consciousness drift visualizer with resonance detection! (requires `dashboard` feature)
#[cfg(feature = "dashboard")]
pub mod wave_compass;

// Claude Hook Handler - Comprehensive context provider for conversations
pub mod claude_hook;

// Marqant - Quantum-compressed markdown format (external crate from ./marqant submodule)
// Now accessed via: use marqant::Marqant;

// ST Tokenizer - Advanced semantic pattern recognition with frequency tracking
pub mod st_tokenizer;

// SmartPastCode Registry Integration - Auto-indexing for universal code discovery
pub mod registry;

// Security Scanner - Detect supply chain attack patterns
pub mod security_scan;
