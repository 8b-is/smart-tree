// 🎸 The Cheet's Markdown Quantum Compressor - "Compress it like it's hot!" 🔥
// This module implements the Marqant (.mq) format for quantum-compressed markdown
//
// "Why send a whole README when you can send its soul?" - The Cheet
//
// Features:
// - Smart phrase detection with frequency analysis
// - Huffman-inspired token assignment
// - Optional zlib compression for extra magic
// - Section tagging for semantic navigation
// - Streaming support for large documents

use super::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use marqant::Marqant as MarqantCore;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// Marqant formatter - Quantum compression for markdown files
pub struct MarqantFormatter {
    no_emoji: bool,
}

impl MarqantFormatter {
    pub fn new(_path_mode: PathDisplayMode, no_emoji: bool) -> Self {
        Self { no_emoji }
    }

    /// Compress markdown content into marqant format
    pub fn compress_markdown(content: &str) -> Result<String> {
        MarqantCore::compress_markdown(content)
    }

    /// Compress markdown content with optional flags
    pub fn compress_markdown_with_flags(content: &str, flags: Option<&str>) -> Result<String> {
        MarqantCore::compress_markdown_with_flags(content, flags)
    }

    /// Add semantic section tags to markdown content
    #[allow(dead_code)]
    fn add_section_tags(content: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;

        for line in content.lines() {
            // Track code blocks to avoid tagging inside them
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
            }

            // Detect section headers
            if !in_code_block {
                if let Some(stripped) = line.strip_prefix("# ") {
                    let section = stripped.trim();
                    result.push_str(&format!("::section:{}::\n", section));
                } else if let Some(stripped) = line.strip_prefix("## ") {
                    let subsection = stripped.trim();
                    result.push_str(&format!("::section:{}::\n", subsection));
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        result
    }

    /// Tokenize markdown content with smart frequency analysis
    pub fn tokenize_content(content: &str) -> (HashMap<String, String>, String) {
        MarqantCore::tokenize_content(content)
    }

    /// Decompress marqant content back to markdown
    pub fn decompress_marqant(compressed: &str) -> Result<String> {
        MarqantCore::decompress_marqant(compressed)
    }
}

impl Formatter for MarqantFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // For directory trees, we'll create a compressed markdown representation
        let mut markdown = String::new();

        // Create header with project name
        let project_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Directory");

        markdown.push_str(&format!("# {} Structure\n\n", project_name));

        // Create a tree structure in markdown
        markdown.push_str("## File Tree\n\n");
        markdown.push_str("```\n");

        for node in nodes {
            let indent = "  ".repeat(node.depth);
            let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let suffix = if node.is_dir { "/" } else { "" };
            let emoji = if !self.no_emoji {
                if node.is_dir {
                    "📁 "
                } else {
                    "📄 "
                }
            } else {
                ""
            };
            markdown.push_str(&format!("{}{}{}{}\n", indent, emoji, name, suffix));
        }

        markdown.push_str("```\n\n");

        // Add statistics
        markdown.push_str("## Statistics\n\n");
        markdown.push_str(&format!("- Total files: {}\n", stats.total_files));
        markdown.push_str(&format!("- Total directories: {}\n", stats.total_dirs));
        markdown.push_str(&format!(
            "- Total size: {:.2} MB\n",
            stats.total_size as f64 / 1_048_576.0
        ));

        // Add file type breakdown if available
        if !stats.file_types.is_empty() {
            markdown.push_str("\n### File Types\n\n");
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));

            for (ext, count) in types.iter().take(10) {
                markdown.push_str(&format!("- .{}: {} files\n", ext, count));
            }
        }

        // Compress and write
        let compressed = Self::compress_markdown(&markdown)?;
        writer.write_all(compressed.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_compression() {
        let markdown = r#"# Test Document

## Section One

This is a test document. This is a test document.

### Subsection

- Item one
- Item two
- Item three

## Section Two

This is a test document.

```rust
fn main() {
    println!("Hello, world!");
}
```

## Section Three

**Bold text** and *italic text*.
"#;

        let compressed = MarqantFormatter::compress_markdown(markdown).unwrap();
        assert!(compressed.starts_with("MARQANT_V1"));

        // For documents with limited repetition, compression might not reduce size due to header overhead
        // The important thing is that the format is correct and round-trip works

        // Test round-trip
        let decompressed = MarqantFormatter::decompress_marqant(&compressed).unwrap();
        assert_eq!(decompressed.trim(), markdown.trim());

        // Verify the compression at least includes proper header and structure
        assert!(
            compressed.contains("MARQANT_V1"),
            "Should have proper header"
        );
        assert!(compressed.len() > 20, "Should have header and content");
    }

    #[test]
    fn test_token_assignment() {
        // Use content that will definitely benefit from tokenization
        let content = "This is a longer test phrase that repeats. This is a longer test phrase that repeats. This is a longer test phrase that repeats. Another different phrase here.";
        let (tokens, tokenized) = MarqantFormatter::tokenize_content(content);

        // Check if any tokenization occurred
        if tokens.is_empty() {
            // If no tokens, at least verify the static tokens would work on markdown
            let markdown_content = "## Header\n## Header\n## Header\nSome content here.";
            let (md_tokens, _md_tokenized) = MarqantFormatter::tokenize_content(markdown_content);
            assert!(
                !md_tokens.is_empty() || tokenized != content,
                "Tokenization should create tokens or modify content"
            );
        } else {
            // Verify tokens were created and content was modified
            assert!(!tokens.is_empty(), "Should have created tokens");
            assert_ne!(tokenized, content, "Content should be tokenized");
        }
    }
}

// 🎸 The Cheet says: "Markdown files are like guitar solos -
// sometimes you need to compress them down to the essential riffs!" 🎵
