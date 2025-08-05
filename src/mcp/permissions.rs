//! Permission-based tool gating system for MCP
//! 
//! This module implements a smart permission checking system that:
//! 1. Requires digest/verification before other tools can be used
//! 2. Only exposes tools that are relevant based on permissions
//! 3. Saves context by hiding unavailable operations
//! 4. Provides helpful comments about why tools are unavailable

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Permission state for a path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPermissions {
    /// Path that was checked
    pub path: PathBuf,
    /// Whether the path exists
    pub exists: bool,
    /// Whether we can read the path
    pub readable: bool,
    /// Whether we can write to the path
    pub writable: bool,
    /// Whether it's a directory
    pub is_directory: bool,
    /// Whether it's a file
    pub is_file: bool,
    /// When this was last verified
    pub verified_at: SystemTime,
    /// Any error messages
    pub error: Option<String>,
}

/// Permission cache for verified paths
#[derive(Debug, Default)]
pub struct PermissionCache {
    /// Cached permissions by path
    permissions: HashMap<PathBuf, PathPermissions>,
    /// How long to cache permissions (default: 5 minutes)
    cache_duration: Duration,
}

impl PermissionCache {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            cache_duration: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Check if a path has been verified recently
    pub fn is_verified(&self, path: &Path) -> bool {
        if let Some(perms) = self.permissions.get(path) {
            if let Ok(elapsed) = perms.verified_at.elapsed() {
                return elapsed < self.cache_duration;
            }
        }
        false
    }
    
    /// Get cached permissions for a path
    pub fn get(&self, path: &Path) -> Option<&PathPermissions> {
        self.permissions.get(path).filter(|p| {
            p.verified_at.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
        })
    }
    
    /// Verify and cache permissions for a path
    pub fn verify(&mut self, path: &Path) -> Result<PathPermissions> {
        // Check if path exists
        let exists = path.exists();
        if !exists {
            let perms = PathPermissions {
                path: path.to_path_buf(),
                exists: false,
                readable: false,
                writable: false,
                is_directory: false,
                is_file: false,
                verified_at: SystemTime::now(),
                error: Some("Path does not exist".to_string()),
            };
            self.permissions.insert(path.to_path_buf(), perms.clone());
            return Ok(perms);
        }
        
        // Get metadata
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                let perms = PathPermissions {
                    path: path.to_path_buf(),
                    exists: true,
                    readable: false,
                    writable: false,
                    is_directory: false,
                    is_file: false,
                    verified_at: SystemTime::now(),
                    error: Some(format!("Cannot read metadata: {}", e)),
                };
                self.permissions.insert(path.to_path_buf(), perms.clone());
                return Ok(perms);
            }
        };
        
        let is_directory = metadata.is_dir();
        let is_file = metadata.is_file();
        
        // Check read permission
        let readable = if is_directory {
            fs::read_dir(path).is_ok()
        } else {
            fs::File::open(path).is_ok()
        };
        
        // Check write permission
        let writable = !metadata.permissions().readonly();
        
        let perms = PathPermissions {
            path: path.to_path_buf(),
            exists,
            readable,
            writable,
            is_directory,
            is_file,
            verified_at: SystemTime::now(),
            error: None,
        };
        
        self.permissions.insert(path.to_path_buf(), perms.clone());
        Ok(perms)
    }
    
    /// Clear expired entries
    pub fn cleanup(&mut self) {
        let now = SystemTime::now();
        self.permissions.retain(|_, p| {
            p.verified_at.elapsed().unwrap_or(Duration::MAX) < self.cache_duration
        });
    }
}

/// Tool availability based on permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAvailability {
    /// Tool name
    pub name: String,
    /// Whether the tool is available
    pub available: bool,
    /// Reason why the tool is unavailable (if applicable)
    pub reason: Option<String>,
    /// Required permissions for this tool
    pub requires: Vec<String>,
}

/// Get available tools based on path permissions
pub fn get_available_tools(perms: &PathPermissions) -> Vec<ToolAvailability> {
    let mut tools = vec![];
    
    // Always available tools (verification tools)
    tools.push(ToolAvailability {
        name: "get_digest".to_string(),
        available: true,
        reason: None,
        requires: vec![],
    });
    
    tools.push(ToolAvailability {
        name: "server_info".to_string(),
        available: true,
        reason: None,
        requires: vec![],
    });
    
    // Read-only tools
    if perms.readable {
        tools.extend(vec![
            ToolAvailability {
                name: "analyze_directory".to_string(),
                available: perms.is_directory,
                reason: if !perms.is_directory {
                    Some("Path is not a directory".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "directory".to_string()],
            },
            ToolAvailability {
                name: "quick_tree".to_string(),
                available: perms.is_directory,
                reason: if !perms.is_directory {
                    Some("Path is not a directory".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "directory".to_string()],
            },
            ToolAvailability {
                name: "find_files".to_string(),
                available: perms.is_directory,
                reason: if !perms.is_directory {
                    Some("Path is not a directory".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "directory".to_string()],
            },
            ToolAvailability {
                name: "search_in_files".to_string(),
                available: perms.is_directory,
                reason: if !perms.is_directory {
                    Some("Path is not a directory".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "directory".to_string()],
            },
            ToolAvailability {
                name: "get_statistics".to_string(),
                available: perms.is_directory,
                reason: if !perms.is_directory {
                    Some("Path is not a directory".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "directory".to_string()],
            },
            ToolAvailability {
                name: "get_function_tree".to_string(),
                available: perms.is_file,
                reason: if !perms.is_file {
                    Some("Path is not a file".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "file".to_string()],
            },
        ]);
    } else {
        // Add read tools as unavailable with reason
        tools.extend(vec![
            ToolAvailability {
                name: "analyze_directory".to_string(),
                available: false,
                reason: Some("No read permission for this path".to_string()),
                requires: vec!["read".to_string()],
            },
            ToolAvailability {
                name: "quick_tree".to_string(),
                available: false,
                reason: Some("No read permission for this path".to_string()),
                requires: vec!["read".to_string()],
            },
        ]);
    }
    
    // Write tools
    if perms.writable && perms.readable {
        tools.extend(vec![
            ToolAvailability {
                name: "smart_edit".to_string(),
                available: perms.is_file,
                reason: if !perms.is_file {
                    Some("Can only edit files, not directories".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "write".to_string(), "file".to_string()],
            },
            ToolAvailability {
                name: "insert_function".to_string(),
                available: perms.is_file,
                reason: if !perms.is_file {
                    Some("Can only edit files, not directories".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "write".to_string(), "file".to_string()],
            },
            ToolAvailability {
                name: "remove_function".to_string(),
                available: perms.is_file,
                reason: if !perms.is_file {
                    Some("Can only edit files, not directories".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "write".to_string(), "file".to_string()],
            },
            ToolAvailability {
                name: "track_file_operation".to_string(),
                available: perms.is_file,
                reason: if !perms.is_file {
                    Some("Can only track operations on files".to_string())
                } else {
                    None
                },
                requires: vec!["read".to_string(), "write".to_string(), "file".to_string()],
            },
        ]);
    } else if !perms.writable && perms.readable {
        // Add write tools as unavailable
        tools.extend(vec![
            ToolAvailability {
                name: "smart_edit".to_string(),
                available: false,
                reason: Some("File is read-only - no write permission".to_string()),
                requires: vec!["write".to_string()],
            },
            ToolAvailability {
                name: "insert_function".to_string(),
                available: false,
                reason: Some("File is read-only - no write permission".to_string()),
                requires: vec!["write".to_string()],
            },
            ToolAvailability {
                name: "remove_function".to_string(),
                available: false,
                reason: Some("File is read-only - no write permission".to_string()),
                requires: vec!["write".to_string()],
            },
        ]);
    }
    
    tools
}

/// Check if a specific tool is available for a path
pub fn is_tool_available(tool_name: &str, perms: &PathPermissions) -> (bool, Option<String>) {
    let tools = get_available_tools(perms);
    for tool in tools {
        if tool.name == tool_name {
            return (tool.available, tool.reason);
        }
    }
    // Tool not found in permission system - might be always available
    (true, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tempfile::TempDir;
    
    #[test]
    fn test_permission_cache() {
        let mut cache = PermissionCache::new();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Verify directory permissions
        let perms = cache.verify(path).unwrap();
        assert!(perms.exists);
        assert!(perms.readable);
        assert!(perms.is_directory);
        assert!(!perms.is_file);
        
        // Check caching
        assert!(cache.is_verified(path));
    }
    
    #[test]
    fn test_tool_availability() {
        // Test with readable directory
        let dir_perms = PathPermissions {
            path: PathBuf::from("/test"),
            exists: true,
            readable: true,
            writable: true,
            is_directory: true,
            is_file: false,
            verified_at: SystemTime::now(),
            error: None,
        };
        
        let tools = get_available_tools(&dir_perms);
        
        // Check that directory tools are available
        let analyze = tools.iter().find(|t| t.name == "analyze_directory").unwrap();
        assert!(analyze.available);
        
        // Check that file tools are not available for directories
        let edit = tools.iter().find(|t| t.name == "smart_edit").unwrap();
        assert!(!edit.available);
        assert_eq!(edit.reason, Some("Can only edit files, not directories".to_string()));
        
        // Test with read-only file
        let ro_file_perms = PathPermissions {
            path: PathBuf::from("/test.txt"),
            exists: true,
            readable: true,
            writable: false,
            is_directory: false,
            is_file: true,
            verified_at: SystemTime::now(),
            error: None,
        };
        
        let tools = get_available_tools(&ro_file_perms);
        
        // Check that edit tools are unavailable
        let edit = tools.iter().find(|t| t.name == "smart_edit").unwrap();
        assert!(!edit.available);
        assert_eq!(edit.reason, Some("File is read-only - no write permission".to_string()));
    }
}