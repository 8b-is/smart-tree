//! Collaboration Module - Humans + AIs working together
//!
//! Each user (human or AI) brings their own skills to the project.
//! Users connect via i1.is or GitHub identity, get their own space
//! (optionally containerized), and collaborate in real-time.
//!
//! ## Philosophy
//! - **Work by default** - Don't block people, enable them
//! - **Bring your skills** - Each collaborator has unique abilities
//! - **Any machine** - Connect to any std daemon you have access to

pub mod identity;
pub mod permissions;
pub mod space;
pub mod templates;

pub use identity::{Identity, IdentityProvider};
pub use permissions::{AccessLevel, Permission, ProjectAccess};
pub use space::{UserSpace, SpaceConfig};
pub use templates::Template;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A collaborator - human or AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// Their identity (i1.is, GitHub, or local)
    pub identity: Identity,

    /// What kind of collaborator
    pub kind: CollaboratorKind,

    /// Their preferred template/environment
    pub template: Option<String>,

    /// Skills they bring (for AI: tools they can use, for humans: expertise)
    pub skills: Vec<String>,

    /// Current status
    pub status: CollaboratorStatus,
}

/// Type of collaborator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaboratorKind {
    /// Human developer
    Human,
    /// AI assistant (Claude, etc.)
    Ai { model: String },
    /// Automated system (CI, bots)
    System,
}

/// Current status of a collaborator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaboratorStatus {
    /// Currently active in the space
    Active,
    /// Connected but idle
    Idle,
    /// Not currently connected
    Offline,
    /// Invited but hasn't joined yet
    Invited,
}

/// A collaborative project/workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project identifier (e.g., "hue@i1.is/smart-tree")
    pub id: String,

    /// Display name
    pub name: String,

    /// Owner identity
    pub owner: Identity,

    /// Path on the host machine
    pub path: PathBuf,

    /// Collaborators with their access levels
    pub collaborators: Vec<(Collaborator, AccessLevel)>,

    /// Project-level settings
    pub settings: ProjectSettings,
}

/// Project settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    /// Allow container isolation for users
    pub allow_containers: bool,

    /// Default template for new collaborators
    pub default_template: Option<String>,

    /// Sync memories to i1.is cloud
    pub cloud_sync: bool,

    /// Specific denies (not allows - work by default!)
    pub deny_patterns: Vec<String>,
}

impl Project {
    /// Create a new project
    pub fn new(name: &str, owner: Identity, path: PathBuf) -> Self {
        let id = format!("{}/{}", owner, name);
        Project {
            id,
            name: name.to_string(),
            owner,
            path,
            collaborators: Vec::new(),
            settings: ProjectSettings::default(),
        }
    }

    /// Add a collaborator
    pub fn add_collaborator(&mut self, collab: Collaborator, access: AccessLevel) {
        self.collaborators.push((collab, access));
    }

    /// Check if an identity has access
    pub fn can_access(&self, identity: &Identity) -> bool {
        // Owner always has access
        if &self.owner == identity {
            return true;
        }

        // Check collaborators
        self.collaborators.iter().any(|(c, _)| &c.identity == identity)
    }

    /// Get access level for an identity
    pub fn access_level(&self, identity: &Identity) -> AccessLevel {
        if &self.owner == identity {
            return AccessLevel::Owner;
        }

        self.collaborators
            .iter()
            .find(|(c, _)| &c.identity == identity)
            .map(|(_, level)| level.clone())
            .unwrap_or(AccessLevel::None)
    }
}

/// Join a remote project
pub async fn join_project(project_id: &str) -> Result<Project> {
    // Parse project ID: "user@i1.is/project" or "github:user/project"
    let (owner_str, project_name) = project_id
        .rsplit_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid project ID format"))?;

    let owner = Identity::parse(owner_str)?;

    // TODO: Connect to remote daemon via STUN/relay
    // For now, return a placeholder
    Ok(Project::new(project_name, owner, PathBuf::from(".")))
}
