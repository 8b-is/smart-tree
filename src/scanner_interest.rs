//
// -----------------------------------------------------------------------------
//  INTEREST SCORING: Surfacing What Matters
//
//  This module is the heart of Smart Tree's intelligent scanning. Instead of
//  listing everything, we score each file/folder by "interest" - how relevant
//  is this to the developer or AI right now?
//
//  Key concepts:
//  - TraversalPath: How did we reach this location? (direct, symlink, mount, etc.)
//  - InterestScore: A 0.0-1.0 score with breakdown of contributing factors
//  - InterestLevel: Human-friendly categorization (Boring → Critical)
//
//  "The goal is signal, not noise." - Omni
// -----------------------------------------------------------------------------
//

use crate::scanner::FilesystemType;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// How we reached this location during traversal.
/// This context helps understand if a path is "real" or indirect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraversalPath {
    /// Directly under the scan root via normal directory traversal
    Direct,

    /// Reached via symbolic link
    Symlink {
        /// The actual target of the symlink
        target: PathBuf,
        /// Whether the target exists
        target_exists: bool,
    },

    /// Crossed a mount point boundary
    Mount {
        /// The filesystem type we're now on
        filesystem: FilesystemType,
        /// Mount point path
        mount_point: PathBuf,
    },

    /// Reached via recursive traversal into a nested structure
    Recursive {
        /// How deep we are from the original interesting location
        depth: usize,
        /// The original path that led us here
        original: PathBuf,
    },

    /// Inside a dependency/vendor directory (node_modules, vendor, etc.)
    Dependency {
        /// Type of dependency manager
        manager: DependencyManager,
        /// Root of the dependency tree
        dep_root: PathBuf,
    },
}

/// Types of dependency managers we recognize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyManager {
    /// npm/yarn/pnpm (node_modules)
    Npm,
    /// Cargo (target/debug, target/release)
    Cargo,
    /// pip/venv/virtualenv
    Python,
    /// Go modules (vendor)
    Go,
    /// Ruby gems (vendor/bundle)
    Ruby,
    /// Composer (vendor)
    Composer,
    /// Maven/Gradle (.m2, build)
    Java,
    /// Unknown dependency manager
    Unknown,
}

impl DependencyManager {
    /// Get the typical directory name for this dependency manager
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Npm => "node_modules",
            Self::Cargo => "target",
            Self::Python => ".venv",
            Self::Go => "vendor",
            Self::Ruby => "vendor",
            Self::Composer => "vendor",
            Self::Java => "build",
            Self::Unknown => "",
        }
    }

    /// Detect dependency manager from a directory name
    pub fn from_dir_name(name: &str) -> Option<Self> {
        match name {
            "node_modules" => Some(Self::Npm),
            "target" => Some(Self::Cargo),
            ".venv" | "venv" | ".virtualenv" | "virtualenv" => Some(Self::Python),
            "vendor" => Some(Self::Go), // Could also be Ruby/Composer - context needed
            ".m2" | "build" | "out" => Some(Self::Java),
            _ => None,
        }
    }
}

/// Full traversal context for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalContext {
    /// How we reached this location
    pub path: TraversalPath,

    /// Depth from scan root
    pub depth_from_root: usize,

    /// Is this path inside a git worktree?
    pub in_git_worktree: bool,

    /// Is this inside a submodule?
    pub in_submodule: bool,

    /// Parent directory interest level (for inheritance)
    pub parent_interest: Option<InterestLevel>,
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self {
            path: TraversalPath::Direct,
            depth_from_root: 0,
            in_git_worktree: false,
            in_submodule: false,
            parent_interest: None,
        }
    }
}

/// Interest level - human-readable categorization of importance
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InterestLevel {
    /// Not worth showing (generated files, caches, etc.)
    Boring = 0,

    /// Exists but rarely relevant (most dependencies, build artifacts)
    #[default]
    Background = 1,

    /// Worth knowing about but not urgent
    Notable = 2,

    /// Should be surfaced to the user
    Important = 3,

    /// Must be shown - security issues, breaking changes, critical files
    Critical = 4,
}

impl InterestLevel {
    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Boring => "💤",
            Self::Background => "📦",
            Self::Notable => "📝",
            Self::Important => "🔥",
            Self::Critical => "⚠️",
        }
    }

    /// Get color name for terminal output
    pub fn color(&self) -> &'static str {
        match self {
            Self::Boring => "bright_black",
            Self::Background => "white",
            Self::Notable => "cyan",
            Self::Important => "yellow",
            Self::Critical => "red",
        }
    }

    /// Convert from float score to level
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.8 => Self::Critical,
            s if s >= 0.6 => Self::Important,
            s if s >= 0.4 => Self::Notable,
            s if s >= 0.2 => Self::Background,
            _ => Self::Boring,
        }
    }
}

/// Risk level for security-related factors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No risk detected
    None = 0,
    /// Informational finding
    Info = 1,
    /// Low risk - worth noting
    Low = 2,
    /// Medium risk - should be reviewed
    Medium = 3,
    /// High risk - needs attention
    High = 4,
    /// Critical risk - immediate action needed
    Critical = 5,
}

impl RiskLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Info => "ℹ️",
            Self::Low => "🔵",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::Critical => "🔴",
        }
    }
}

/// Factors that contribute to a node's interest score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterestFactor {
    /// File was recently modified
    RecentlyModified {
        /// Hours since last modification
        hours_ago: f32,
        /// Contribution to score (0.0-1.0)
        weight: f32,
    },

    /// Security pattern detected
    SecurityPattern {
        /// Risk level of the finding
        risk: RiskLevel,
        /// Brief description
        description: String,
        /// Contribution to score
        weight: f32,
    },

    /// This is a key project file (README, Cargo.toml, package.json, etc.)
    KeyProjectFile {
        /// Type of key file
        file_type: KeyFileType,
        /// Contribution to score
        weight: f32,
    },

    /// Changed since last scan
    ChangedSinceLastScan {
        /// Type of change
        change: ChangeType,
        /// Contribution to score
        weight: f32,
    },

    /// In a "hot" directory with frequent changes
    HotDirectory {
        /// Number of changes in recent period
        change_count: u32,
        /// Contribution to score
        weight: f32,
    },

    /// Suspicious dependency detected
    SuspiciousDependency {
        /// Reason for suspicion
        reason: String,
        /// Contribution to score
        weight: f32,
    },

    /// Git-related interest (uncommitted changes, conflicts, etc.)
    GitStatus {
        /// Type of git status
        status: GitStatusType,
        /// Contribution to score
        weight: f32,
    },

    /// Code complexity or size concern
    Complexity {
        /// Description of the complexity factor
        description: String,
        /// Contribution to score
        weight: f32,
    },

    /// Inside a dependency tree (usually reduces interest)
    InDependencyTree {
        /// Depth inside dependency tree
        depth: usize,
        /// Negative contribution to score
        weight: f32,
    },

    /// Custom user-defined interest factor
    Custom {
        /// Name of the custom factor
        name: String,
        /// Contribution to score
        weight: f32,
    },
}

impl InterestFactor {
    /// Get the weight contribution of this factor
    pub fn weight(&self) -> f32 {
        match self {
            Self::RecentlyModified { weight, .. } => *weight,
            Self::SecurityPattern { weight, .. } => *weight,
            Self::KeyProjectFile { weight, .. } => *weight,
            Self::ChangedSinceLastScan { weight, .. } => *weight,
            Self::HotDirectory { weight, .. } => *weight,
            Self::SuspiciousDependency { weight, .. } => *weight,
            Self::GitStatus { weight, .. } => *weight,
            Self::Complexity { weight, .. } => *weight,
            Self::InDependencyTree { weight, .. } => *weight,
            Self::Custom { weight, .. } => *weight,
        }
    }

    /// Get a short description of this factor
    pub fn description(&self) -> String {
        match self {
            Self::RecentlyModified { hours_ago, .. } => {
                format!("Modified {:.1}h ago", hours_ago)
            }
            Self::SecurityPattern { description, .. } => description.clone(),
            Self::KeyProjectFile { file_type, .. } => {
                format!("Key file: {:?}", file_type)
            }
            Self::ChangedSinceLastScan { change, .. } => {
                format!("Changed: {:?}", change)
            }
            Self::HotDirectory { change_count, .. } => {
                format!("{} recent changes", change_count)
            }
            Self::SuspiciousDependency { reason, .. } => reason.clone(),
            Self::GitStatus { status, .. } => format!("Git: {:?}", status),
            Self::Complexity { description, .. } => description.clone(),
            Self::InDependencyTree { depth, .. } => {
                format!("Dependency depth: {}", depth)
            }
            Self::Custom { name, .. } => name.clone(),
        }
    }
}

/// Types of key project files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyFileType {
    /// README, CHANGELOG, etc.
    Documentation,
    /// Cargo.toml, package.json, etc.
    BuildConfig,
    /// .env, config.toml, etc.
    Configuration,
    /// main.rs, index.js, etc.
    EntryPoint,
    /// LICENSE, COPYING
    License,
    /// .github/workflows, .gitlab-ci.yml
    CiConfig,
    /// Dockerfile, docker-compose.yml
    Container,
    /// CLAUDE.md, .cursorrules, etc.
    AiConfig,
}

/// Types of changes detected between scans
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was added
    Added,
    /// File content was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File permissions changed
    PermissionChanged,
    /// File was renamed/moved
    Renamed,
    /// File type changed (e.g., regular file to symlink)
    TypeChanged,
}

impl ChangeType {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Modified => "~",
            Self::Deleted => "-",
            Self::PermissionChanged => "🔐",
            Self::Renamed => "→",
            Self::TypeChanged => "⚡",
        }
    }
}

/// Git status types that affect interest
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitStatusType {
    /// Uncommitted changes
    Uncommitted,
    /// Merge conflict
    Conflict,
    /// Staged for commit
    Staged,
    /// Untracked file
    Untracked,
    /// Ahead of remote
    Ahead,
    /// Behind remote
    Behind,
}

/// Interest score with breakdown of contributing factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestScore {
    /// Overall interest score (0.0 = boring, 1.0 = critical)
    pub score: f32,

    /// Factors that contributed to this score
    pub factors: Vec<InterestFactor>,

    /// Human-friendly interest level
    pub level: InterestLevel,

    /// When this score was calculated
    pub calculated_at: SystemTime,
}

impl InterestScore {
    /// Create a new interest score from factors
    pub fn from_factors(factors: Vec<InterestFactor>) -> Self {
        let score = factors.iter().map(|f| f.weight()).sum::<f32>().clamp(0.0, 1.0);
        let level = InterestLevel::from_score(score);

        Self {
            score,
            factors,
            level,
            calculated_at: SystemTime::now(),
        }
    }

    /// Create a default "boring" score
    pub fn boring() -> Self {
        Self {
            score: 0.0,
            factors: vec![],
            level: InterestLevel::Boring,
            calculated_at: SystemTime::now(),
        }
    }

    /// Create a critical score with a single reason
    pub fn critical(reason: String) -> Self {
        Self {
            score: 1.0,
            factors: vec![InterestFactor::SecurityPattern {
                risk: RiskLevel::Critical,
                description: reason,
                weight: 1.0,
            }],
            level: InterestLevel::Critical,
            calculated_at: SystemTime::now(),
        }
    }

    /// Check if this score indicates the node should be shown by default
    pub fn should_show(&self) -> bool {
        self.level >= InterestLevel::Notable
    }

    /// Get a summary of why this is interesting
    pub fn summary(&self) -> String {
        if self.factors.is_empty() {
            return String::from("No notable factors");
        }

        self.factors
            .iter()
            .map(|f| f.description())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for InterestScore {
    fn default() -> Self {
        Self {
            score: 0.1,
            factors: vec![],
            level: InterestLevel::Background,
            calculated_at: SystemTime::now(),
        }
    }
}

/// Default weights for interest calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestWeights {
    /// Weight for recently modified files (within 24h)
    pub recent_modification: f32,

    /// Weight for critical security findings
    pub security_critical: f32,

    /// Weight for key project files
    pub key_file: f32,

    /// Weight for files changed since last scan
    pub changed_since_scan: f32,

    /// Weight for files in hot directories
    pub hot_directory: f32,

    /// Negative weight for dependency tree depth
    pub dependency_depth_penalty: f32,

    /// Base interest for files with git changes
    pub git_changes: f32,
}

impl Default for InterestWeights {
    fn default() -> Self {
        Self {
            recent_modification: 0.3,
            security_critical: 1.0,
            key_file: 0.5,
            changed_since_scan: 0.4,
            hot_directory: 0.3,
            dependency_depth_penalty: -0.1,
            git_changes: 0.35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_level_from_score() {
        assert_eq!(InterestLevel::from_score(0.0), InterestLevel::Boring);
        assert_eq!(InterestLevel::from_score(0.1), InterestLevel::Boring);
        assert_eq!(InterestLevel::from_score(0.3), InterestLevel::Background);
        assert_eq!(InterestLevel::from_score(0.5), InterestLevel::Notable);
        assert_eq!(InterestLevel::from_score(0.7), InterestLevel::Important);
        assert_eq!(InterestLevel::from_score(0.9), InterestLevel::Critical);
        assert_eq!(InterestLevel::from_score(1.0), InterestLevel::Critical);
    }

    #[test]
    fn test_interest_score_from_factors() {
        let factors = vec![
            InterestFactor::RecentlyModified {
                hours_ago: 2.0,
                weight: 0.3,
            },
            InterestFactor::KeyProjectFile {
                file_type: KeyFileType::BuildConfig,
                weight: 0.5,
            },
        ];

        let score = InterestScore::from_factors(factors);
        assert!((score.score - 0.8).abs() < 0.01);
        assert_eq!(score.level, InterestLevel::Critical);
    }

    #[test]
    fn test_interest_score_clamping() {
        let factors = vec![
            InterestFactor::SecurityPattern {
                risk: RiskLevel::Critical,
                description: "Bad thing".to_string(),
                weight: 1.0,
            },
            InterestFactor::HotDirectory {
                change_count: 100,
                weight: 0.5,
            },
        ];

        let score = InterestScore::from_factors(factors);
        // Should clamp to 1.0
        assert_eq!(score.score, 1.0);
    }

    #[test]
    fn test_dependency_manager_detection() {
        assert_eq!(
            DependencyManager::from_dir_name("node_modules"),
            Some(DependencyManager::Npm)
        );
        assert_eq!(
            DependencyManager::from_dir_name("target"),
            Some(DependencyManager::Cargo)
        );
        assert_eq!(
            DependencyManager::from_dir_name(".venv"),
            Some(DependencyManager::Python)
        );
        assert_eq!(DependencyManager::from_dir_name("src"), None);
    }
}
