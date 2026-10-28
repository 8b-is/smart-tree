//! Identity Resolution - Who are you?
//!
//! Supports multiple identity providers:
//! - i1.is - Our identity service (primary)
//! - GitHub - OAuth + SSH keys
//! - Local - Machine users (for standalone)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identity provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IdentityProvider {
    /// i1.is identity service
    I1is,
    /// GitHub OAuth
    GitHub,
    /// Local machine user
    Local,
}

/// A user identity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Identity {
    /// The provider
    pub provider: IdentityProvider,
    /// Username/identifier
    pub username: String,
    /// Optional display name
    pub display_name: Option<String>,
    /// Public key (for verification)
    pub public_key: Option<String>,
}

impl Identity {
    /// Create an i1.is identity
    pub fn i1is(username: &str) -> Self {
        Identity {
            provider: IdentityProvider::I1is,
            username: username.to_string(),
            display_name: None,
            public_key: None,
        }
    }

    /// Create a GitHub identity
    pub fn github(username: &str) -> Self {
        Identity {
            provider: IdentityProvider::GitHub,
            username: username.to_string(),
            display_name: None,
            public_key: None,
        }
    }

    /// Create a local identity
    pub fn local(username: &str) -> Self {
        Identity {
            provider: IdentityProvider::Local,
            username: username.to_string(),
            display_name: None,
            public_key: None,
        }
    }

    /// Parse an identity string
    ///
    /// Formats:
    /// - "user@i1.is" -> i1.is identity
    /// - "github:user" -> GitHub identity
    /// - "user" -> Local identity (fallback)
    pub fn parse(s: &str) -> Result<Self> {
        if s.ends_with("@i1.is") {
            let username = s.trim_end_matches("@i1.is");
            Ok(Identity::i1is(username))
        } else if s.starts_with("github:") {
            let username = s.trim_start_matches("github:");
            Ok(Identity::github(username))
        } else if s.contains('@') {
            // Assume i1.is format: user@i1.is
            let username = s.split('@').next().unwrap_or(s);
            Ok(Identity::i1is(username))
        } else {
            // Local user
            Ok(Identity::local(s))
        }
    }

    /// Get the canonical string representation
    pub fn canonical(&self) -> String {
        match self.provider {
            IdentityProvider::I1is => format!("{}@i1.is", self.username),
            IdentityProvider::GitHub => format!("github:{}", self.username),
            IdentityProvider::Local => format!("local:{}", self.username),
        }
    }

    /// Check if this is an AI identity
    pub fn is_ai(&self) -> bool {
        // AI identities typically have special prefixes
        self.username.starts_with("claude-")
            || self.username.starts_with("ai-")
            || self.username.starts_with("assistant-")
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical())
    }
}

/// Resolve an identity from a provider
pub struct IdentityResolver;

impl IdentityResolver {
    /// Resolve an i1.is identity (fetch public key, display name, etc.)
    pub async fn resolve_i1is(username: &str) -> Result<Identity> {
        // TODO: Call i1.is API to get user info
        // For now, return basic identity
        Ok(Identity {
            provider: IdentityProvider::I1is,
            username: username.to_string(),
            display_name: None,
            public_key: None,
        })
    }

    /// Resolve a GitHub identity via API
    pub async fn resolve_github(username: &str) -> Result<Identity> {
        // TODO: Call GitHub API to get user info + SSH keys
        // For now, return basic identity
        Ok(Identity {
            provider: IdentityProvider::GitHub,
            username: username.to_string(),
            display_name: None,
            public_key: None,
        })
    }

    /// Get current user's identity from environment
    pub fn current() -> Result<Identity> {
        // Check for i1.is identity first
        if let Ok(id) = std::env::var("I1IS_USER") {
            return Ok(Identity::i1is(&id));
        }

        // Check for GitHub user
        if let Ok(user) = std::env::var("GITHUB_USER") {
            return Ok(Identity::github(&user));
        }

        // Fall back to local user
        let username = whoami::username();
        Ok(Identity::local(&username))
    }

    /// Verify an identity signature
    pub fn verify(_identity: &Identity, _message: &[u8], _signature: &[u8]) -> Result<bool> {
        // TODO: Implement signature verification using public key
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity() {
        let i1 = Identity::parse("hue@i1.is").unwrap();
        assert_eq!(i1.provider, IdentityProvider::I1is);
        assert_eq!(i1.username, "hue");

        let gh = Identity::parse("github:8b-is").unwrap();
        assert_eq!(gh.provider, IdentityProvider::GitHub);
        assert_eq!(gh.username, "8b-is");

        let local = Identity::parse("alice").unwrap();
        assert_eq!(local.provider, IdentityProvider::Local);
        assert_eq!(local.username, "alice");
    }

    #[test]
    fn test_canonical() {
        assert_eq!(Identity::i1is("hue").canonical(), "hue@i1.is");
        assert_eq!(Identity::github("8b-is").canonical(), "github:8b-is");
        assert_eq!(Identity::local("alice").canonical(), "local:alice");
    }
}
