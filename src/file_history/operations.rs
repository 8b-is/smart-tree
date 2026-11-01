//! File operation codes and definitions
//!
//! 🎸 The Cheet says: "Every operation has its own rhythm!"

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

/// File operation types with codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileOperation {
    /// Append content to end of file (least intrusive)
    Append,
    /// Prepend content to beginning of file
    Prepend,
    /// Insert content at specific position
    Insert,
    /// Delete content from file
    Delete,
    /// Replace content in file
    Replace,
    /// Create new file
    Create,
    /// Delete entire file
    Remove,
    /// Relocate/move file
    Relocate,
    /// Rename file
    Rename,
    /// Change permissions
    Chmod,
    /// Read file (for tracking access patterns)
    Read,
}

impl FileOperation {
    /// Get operation code (single character for compact logging)
    pub fn code(&self) -> char {
        match self {
            FileOperation::Append => 'A',
            FileOperation::Prepend => 'P',
            FileOperation::Insert => 'I',
            FileOperation::Delete => 'D',
            FileOperation::Replace => 'R',
            FileOperation::Create => 'C',
            FileOperation::Remove => 'X',
            FileOperation::Relocate => 'M',
            FileOperation::Rename => 'N',
            FileOperation::Chmod => 'H',
            FileOperation::Read => 'r',
        }
    }

    /// Parse operation from code
    pub fn from_code(code: char) -> Option<Self> {
        match code {
            'A' => Some(FileOperation::Append),
            'P' => Some(FileOperation::Prepend),
            'I' => Some(FileOperation::Insert),
            'D' => Some(FileOperation::Delete),
            'R' => Some(FileOperation::Replace),
            'C' => Some(FileOperation::Create),
            'X' => Some(FileOperation::Remove),
            'M' => Some(FileOperation::Relocate),
            'N' => Some(FileOperation::Rename),
            'H' => Some(FileOperation::Chmod),
            'r' => Some(FileOperation::Read),
            _ => None,
        }
    }

    /// Check if operation is non-destructive
    pub fn is_safe(&self) -> bool {
        matches!(self, FileOperation::Append | FileOperation::Read)
    }

    /// Get operation description
    pub fn description(&self) -> &'static str {
        match self {
            FileOperation::Append => "Appended content to file",
            FileOperation::Prepend => "Prepended content to file",
            FileOperation::Insert => "Inserted content into file",
            FileOperation::Delete => "Deleted content from file",
            FileOperation::Replace => "Replaced content in file",
            FileOperation::Create => "Created new file",
            FileOperation::Remove => "Removed file",
            FileOperation::Relocate => "Relocated file",
            FileOperation::Rename => "Renamed file",
            FileOperation::Chmod => "Changed file permissions",
            FileOperation::Read => "Read file",
        }
    }
}

impl fmt::Display for FileOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Operation context with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    /// Type of operation
    pub operation: FileOperation,
    /// Position in file (for insert operations)
    pub position: Option<usize>,
    /// Number of bytes affected
    pub bytes_affected: usize,
    /// Old content hash (before operation)
    pub old_hash: Option<String>,
    /// New content hash (after operation)
    pub new_hash: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl OperationContext {
    /// Create new operation context
    pub fn new(operation: FileOperation) -> Self {
        Self {
            operation,
            position: None,
            bytes_affected: 0,
            old_hash: None,
            new_hash: None,
            metadata: None,
        }
    }

    /// Set position for insert operations
    pub fn with_position(mut self, pos: usize) -> Self {
        self.position = Some(pos);
        self
    }

    /// Set bytes affected
    pub fn with_bytes(mut self, bytes: usize) -> Self {
        self.bytes_affected = bytes;
        self
    }

    /// Set hashes
    pub fn with_hashes(mut self, old: Option<String>, new: Option<String>) -> Self {
        self.old_hash = old;
        self.new_hash = new;
        self
    }
}

/// Suggest best operation for a given change
pub fn suggest_operation(
    original: Option<&str>,
    modified: &str,
    prefer_append: bool,
) -> FileOperation {
    match original {
        None => FileOperation::Create,
        Some("") => FileOperation::Append,
        Some(orig) => {
            // Check if it's a simple append
            if prefer_append && modified.starts_with(orig) {
                FileOperation::Append
            }
            // Check if it's a prepend
            else if modified.ends_with(orig) {
                FileOperation::Prepend
            }
            // Otherwise it's a more complex change
            else {
                FileOperation::Replace
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_codes() {
        assert_eq!(FileOperation::Append.code(), 'A');
        assert_eq!(FileOperation::from_code('A'), Some(FileOperation::Append));
    }

    #[test]
    fn test_suggest_operation() {
        // Test append preference
        let op = suggest_operation(Some("hello"), "hello world", true);
        assert_eq!(op, FileOperation::Append);

        // Test prepend detection
        let op = suggest_operation(Some("world"), "hello world", false);
        assert_eq!(op, FileOperation::Prepend);

        // Test create for new file
        let op = suggest_operation(None, "new content", true);
        assert_eq!(op, FileOperation::Create);
    }
}
