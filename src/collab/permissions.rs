//! Permissions - Work by default, deny when necessary
//!
//! Philosophy: Enable people, don't block them.
//! Instead of whitelisting what users CAN do, we only specify
//! what they CAN'T do (deny patterns).

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Access level for a collaborator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum AccessLevel {
    /// No access
    None,
    /// Can view files and run read-only tools
    #[default]
    Read,
    /// Can edit files in their space
    Write,
    /// Can manage other collaborators, settings
    Admin,
    /// Project owner - full control
    Owner,
}

/// A specific permission grant or deny
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// What action (read, write, execute, admin)
    pub action: PermissionAction,
    /// Path pattern (glob) this applies to
    pub path_pattern: Option<String>,
    /// Whether this is a grant or deny
    pub effect: PermissionEffect,
}

/// Permission action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionAction {
    /// Read files
    Read,
    /// Write/edit files
    Write,
    /// Execute commands
    Execute,
    /// Run specific tools
    Tool(String),
    /// Administrative actions
    Admin,
    /// All actions
    All,
}

/// Grant or deny
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionEffect {
    /// Allow this action (rarely needed - work by default)
    Allow,
    /// Deny this action (primary mechanism)
    Deny,
}

/// Project access configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectAccess {
    /// Default access level for new collaborators
    pub default_level: AccessLevel,

    /// Specific deny rules (work by default, deny specific things)
    pub deny_rules: Vec<Permission>,

    /// Paths that are always off-limits (secrets, credentials)
    pub protected_paths: Vec<String>,

    /// Tools that require explicit approval
    pub restricted_tools: Vec<String>,
}

impl ProjectAccess {
    /// Create with sensible defaults
    pub fn new() -> Self {
        ProjectAccess {
            default_level: AccessLevel::Write,
            deny_rules: Vec::new(),
            protected_paths: vec![
                ".env".to_string(),
                ".env.*".to_string(),
                "**/secrets/**".to_string(),
                "**/*.pem".to_string(),
                "**/*.key".to_string(),
                "**/credentials*".to_string(),
            ],
            restricted_tools: vec![
                "execute_command".to_string(), // Needs approval
            ],
        }
    }

    /// Check if a path is accessible
    pub fn can_access_path(&self, path: &Path, level: &AccessLevel) -> bool {
        // Owner can access everything
        if *level == AccessLevel::Owner {
            return true;
        }

        let path_str = path.to_string_lossy();

        // Check protected paths
        for pattern in &self.protected_paths {
            if Self::matches_glob(pattern, &path_str) {
                return false;
            }
        }

        // Check deny rules
        for rule in &self.deny_rules {
            if rule.effect == PermissionEffect::Deny {
                if let Some(ref pattern) = rule.path_pattern {
                    if Self::matches_glob(pattern, &path_str) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if a tool can be used
    pub fn can_use_tool(&self, tool: &str, level: &AccessLevel) -> bool {
        // Owner/Admin can use all tools
        if *level >= AccessLevel::Admin {
            return true;
        }

        // Check if tool is restricted
        !self.restricted_tools.contains(&tool.to_string())
    }

    /// Simple glob matching (supports * and **)
    fn matches_glob(pattern: &str, path: &str) -> bool {
        // Simple implementation - could use glob crate for full support
        if pattern.contains("**") {
            // ** matches any path segments
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                // Empty prefix means match from start
                let prefix_ok = prefix.is_empty() || path.starts_with(prefix);

                // For suffix, need to handle *.ext patterns
                let suffix_ok = if suffix.is_empty() {
                    true
                } else if suffix.starts_with('*') {
                    // Handle *.ext pattern in suffix
                    let ext = suffix.trim_start_matches('*');
                    path.ends_with(ext)
                } else {
                    path.ends_with(suffix)
                };

                return prefix_ok && suffix_ok;
            }
        }

        // Simple * matching
        if pattern.contains('*') && !pattern.contains("**") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }

        // Exact match
        pattern == path
    }

    /// Add a deny rule
    pub fn deny(&mut self, action: PermissionAction, path_pattern: Option<&str>) {
        self.deny_rules.push(Permission {
            action,
            path_pattern: path_pattern.map(String::from),
            effect: PermissionEffect::Deny,
        });
    }

    /// Protect a path pattern
    pub fn protect_path(&mut self, pattern: &str) {
        self.protected_paths.push(pattern.to_string());
    }

    /// Restrict a tool (require approval)
    pub fn restrict_tool(&mut self, tool: &str) {
        if !self.restricted_tools.contains(&tool.to_string()) {
            self.restricted_tools.push(tool.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_access() {
        let access = ProjectAccess::new();
        assert_eq!(access.default_level, AccessLevel::Write);
    }

    #[test]
    fn test_protected_paths() {
        let access = ProjectAccess::new();
        let level = AccessLevel::Write;

        // Should deny access to .env files
        assert!(!access.can_access_path(Path::new(".env"), &level));
        assert!(!access.can_access_path(Path::new(".env.production"), &level));

        // Should allow normal files
        assert!(access.can_access_path(Path::new("src/main.rs"), &level));
    }

    #[test]
    fn test_owner_bypasses_all() {
        let access = ProjectAccess::new();
        let level = AccessLevel::Owner;

        // Owner can access protected paths
        assert!(access.can_access_path(Path::new(".env"), &level));
        assert!(access.can_access_path(Path::new("secrets/api.key"), &level));
    }

    #[test]
    fn test_glob_matching() {
        assert!(ProjectAccess::matches_glob("*.rs", "main.rs"));
        assert!(ProjectAccess::matches_glob("**/*.key", "secrets/api.key"));
        assert!(ProjectAccess::matches_glob(".env.*", ".env.production"));
        assert!(!ProjectAccess::matches_glob("*.rs", "main.py"));
    }
}
