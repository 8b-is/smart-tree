// Test Rust file for Smart Tree
// Testing syntax highlighting and AST parsing

use std::collections::HashMap;
use anyhow::{Result, Context};

/// A test struct for demonstrating Rust code
#[derive(Debug, Clone)]
pub struct SmartTreeTest {
    name: String,
    files: Vec<String>,
    metadata: HashMap<String, String>,
}

impl SmartTreeTest {
    /// Creates a new test instance
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            files: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a file to the test
    pub fn add_file(&mut self, path: impl Into<String>) -> Result<()> {
        let file_path = path.into();
        if !file_path.is_empty() {
            self.files.push(file_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Empty file path"))
        }
    }

    /// Quantum compression simulation
    pub fn quantum_compress(&self) -> Vec<u8> {
        // Simulating MEM|8 wave-based compression
        let mut result = Vec::new();
        result.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        result.extend_from_slice(self.name.as_bytes());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_tree_creation() {
        let mut tree = SmartTreeTest::new("test");
        assert!(tree.add_file("test.rs").is_ok());
        assert_eq!(tree.files.len(), 1);
    }

    #[test]
    fn test_quantum_compression() {
        let tree = SmartTreeTest::new("quantum");
        let compressed = tree.quantum_compress();
        assert!(compressed.starts_with(&[0xDE, 0xAD, 0xBE, 0xEF]));
    }
}

fn main() {
    println!("Smart Tree Test File - Aye! =€");
    let mut test = SmartTreeTest::new("demo");
    let _ = test.add_file("example.txt");
    println!("Files: {:?}", test.files);
}