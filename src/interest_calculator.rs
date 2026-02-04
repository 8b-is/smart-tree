//
// -----------------------------------------------------------------------------
//  INTEREST CALCULATOR: The Scoring Engine
//
//  This is where the magic happens. We take raw file metadata and turn it into
//  actionable intelligence: "Is this file interesting right now?"
//
//  The calculator weighs multiple factors:
//  - Recency: Modified in the last 24h? That's hot.
//  - Security: Suspicious patterns? Critical.
//  - Key files: README, Cargo.toml, package.json? Important.
//  - Changes: Different from last scan? Notable.
//  - Context: Inside node_modules? Probably boring (unless suspicious).
//
//  "Interest is contextual. A config file is boring until it changes." - Omni
// -----------------------------------------------------------------------------
//

use crate::scanner::{FileCategory, FileNode, FilesystemType};
use crate::scanner_interest::{
    ChangeType, DependencyManager, InterestFactor, InterestLevel, InterestScore,
    InterestWeights, KeyFileType, RiskLevel, TraversalContext, TraversalPath,
};
use crate::scanner_state::{FileSignature, ScanState};
use crate::security_scan::{SecurityFinding, SecurityScanner};
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

/// The Interest Calculator - determines what's worth showing
pub struct InterestCalculator {
    /// Weights for different interest factors
    weights: InterestWeights,

    /// Previous scan state for change detection
    previous_state: Option<ScanState>,

    /// Directories marked as "hot" (frequent changes)
    hot_dirs: HashSet<std::path::PathBuf>,

    /// Security scanner for detecting suspicious patterns
    security_scanner: Option<SecurityScanner>,

    /// Current time (cached for consistency during scan)
    now: SystemTime,
}

impl InterestCalculator {
    /// Create a new interest calculator with default weights
    pub fn new() -> Self {
        Self {
            weights: InterestWeights::default(),
            previous_state: None,
            hot_dirs: HashSet::new(),
            security_scanner: Some(SecurityScanner::new()),
            now: SystemTime::now(),
        }
    }

    /// Create with custom weights
    pub fn with_weights(weights: InterestWeights) -> Self {
        Self {
            weights,
            previous_state: None,
            hot_dirs: HashSet::new(),
            security_scanner: Some(SecurityScanner::new()),
            now: SystemTime::now(),
        }
    }

    /// Set the previous state for change detection
    pub fn with_previous_state(mut self, state: ScanState) -> Self {
        self.previous_state = Some(state);
        self
    }

    /// Set hot directories to watch
    pub fn with_hot_dirs(mut self, dirs: HashSet<std::path::PathBuf>) -> Self {
        self.hot_dirs = dirs;
        self
    }

    /// Disable security scanning (for performance)
    pub fn without_security(mut self) -> Self {
        self.security_scanner = None;
        self
    }

    /// Calculate the interest score for a file node
    pub fn calculate(&self, node: &FileNode) -> InterestScore {
        let mut factors = Vec::new();

        // Factor 1: Recently modified
        if let Some(factor) = self.check_recency(node) {
            factors.push(factor);
        }

        // Factor 2: Key project file
        if let Some(factor) = self.check_key_file(node) {
            factors.push(factor);
        }

        // Factor 3: Changed since last scan
        if let Some(factor) = self.check_changed(node) {
            factors.push(factor);
        }

        // Factor 4: In hot directory
        if let Some(factor) = self.check_hot_dir(node) {
            factors.push(factor);
        }

        // Factor 5: Inside dependency tree (negative weight)
        if let Some(factor) = self.check_dependency_context(node) {
            factors.push(factor);
        }

        // Factor 6: Virtual filesystem (usually boring)
        if let Some(factor) = self.check_filesystem_type(node) {
            factors.push(factor);
        }

        // Factor 7: File category boost
        if let Some(factor) = self.check_category_boost(node) {
            factors.push(factor);
        }

        InterestScore::from_factors(factors)
    }

    /// Calculate interest and include security findings
    pub fn calculate_with_security(
        &self,
        node: &FileNode,
        content: Option<&str>,
    ) -> (InterestScore, Vec<SecurityFinding>) {
        let mut factors = Vec::new();
        let mut findings = Vec::new();

        // Run security scan if enabled and we have content
        if let (Some(scanner), Some(content)) = (&self.security_scanner, content) {
            let file_findings = scanner.scan_file_content(&node.path, content);
            for finding in &file_findings {
                let risk_level = match finding.risk_level {
                    crate::security_scan::RiskLevel::Critical => RiskLevel::Critical,
                    crate::security_scan::RiskLevel::High => RiskLevel::High,
                    crate::security_scan::RiskLevel::Medium => RiskLevel::Medium,
                    crate::security_scan::RiskLevel::Low => RiskLevel::Low,
                };

                factors.push(InterestFactor::SecurityPattern {
                    risk: risk_level,
                    description: finding.description.clone(),
                    weight: match finding.risk_level {
                        crate::security_scan::RiskLevel::Critical => 1.0,
                        crate::security_scan::RiskLevel::High => 0.8,
                        crate::security_scan::RiskLevel::Medium => 0.5,
                        crate::security_scan::RiskLevel::Low => 0.2,
                    },
                });
            }
            findings = file_findings;
        }

        // Add all other factors
        if let Some(factor) = self.check_recency(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_key_file(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_changed(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_hot_dir(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_dependency_context(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_filesystem_type(node) {
            factors.push(factor);
        }
        if let Some(factor) = self.check_category_boost(node) {
            factors.push(factor);
        }

        (InterestScore::from_factors(factors), findings)
    }

    /// Check if file was recently modified
    fn check_recency(&self, node: &FileNode) -> Option<InterestFactor> {
        let duration = self.now.duration_since(node.modified).ok()?;
        let hours = duration.as_secs_f32() / 3600.0;

        // Interest decays over time
        let weight = if hours < 1.0 {
            self.weights.recent_modification * 1.5 // Very recent boost
        } else if hours < 24.0 {
            self.weights.recent_modification * (1.0 - hours / 48.0)
        } else if hours < 168.0 {
            // Within a week
            self.weights.recent_modification * 0.3 * (1.0 - hours / 336.0)
        } else {
            return None; // Too old to matter
        };

        if weight > 0.05 {
            Some(InterestFactor::RecentlyModified {
                hours_ago: hours,
                weight,
            })
        } else {
            None
        }
    }

    /// Check if this is a key project file
    fn check_key_file(&self, node: &FileNode) -> Option<InterestFactor> {
        if node.is_dir {
            return None;
        }

        let file_name = node.path.file_name()?.to_str()?;
        let file_name_lower = file_name.to_lowercase();

        let key_type = match file_name_lower.as_str() {
            // Documentation
            "readme.md" | "readme" | "readme.txt" | "changelog.md" | "changelog" | "history.md" => {
                Some(KeyFileType::Documentation)
            }

            // Build configs
            "cargo.toml" | "package.json" | "pyproject.toml" | "go.mod" | "gemfile"
            | "build.gradle" | "pom.xml" | "makefile" | "cmakelists.txt" => {
                Some(KeyFileType::BuildConfig)
            }

            // Configuration
            ".env" | ".env.local" | ".env.example" | "config.toml" | "config.yaml"
            | "config.json" | "settings.toml" | "settings.yaml" => Some(KeyFileType::Configuration),

            // Entry points
            "main.rs" | "lib.rs" | "mod.rs" | "index.js" | "index.ts" | "main.py" | "__init__.py"
            | "app.py" | "main.go" | "main.java" => Some(KeyFileType::EntryPoint),

            // License
            "license" | "license.md" | "license.txt" | "copying" => Some(KeyFileType::License),

            // CI/CD
            ".gitlab-ci.yml" | "jenkinsfile" | ".travis.yml" | "azure-pipelines.yml" => {
                Some(KeyFileType::CiConfig)
            }

            // Container
            "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" | "containerfile" => {
                Some(KeyFileType::Container)
            }

            // AI config
            "claude.md" | ".cursorrules" | ".aider" | "copilot.md" => Some(KeyFileType::AiConfig),

            _ => None,
        };

        // Also check for GitHub workflows
        let key_type = key_type.or_else(|| {
            if node.path.to_string_lossy().contains(".github/workflows") {
                Some(KeyFileType::CiConfig)
            } else {
                None
            }
        });

        key_type.map(|file_type| InterestFactor::KeyProjectFile {
            file_type,
            weight: self.weights.key_file,
        })
    }

    /// Check if file changed since last scan
    fn check_changed(&self, node: &FileNode) -> Option<InterestFactor> {
        let prev_state = self.previous_state.as_ref()?;
        let prev_sig = prev_state.signatures.get(&node.path);

        match prev_sig {
            None => {
                // File is new
                Some(InterestFactor::ChangedSinceLastScan {
                    change: ChangeType::Added,
                    weight: self.weights.changed_since_scan,
                })
            }
            Some(old_sig) => {
                // Check if changed
                let new_sig = FileSignature::from_path(&node.path).ok()?;

                if new_sig.changed(old_sig) {
                    let change_type = if old_sig.permissions != new_sig.permissions {
                        ChangeType::PermissionChanged
                    } else {
                        ChangeType::Modified
                    };

                    Some(InterestFactor::ChangedSinceLastScan {
                        change: change_type,
                        weight: self.weights.changed_since_scan,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Check if file is in a hot directory
    fn check_hot_dir(&self, node: &FileNode) -> Option<InterestFactor> {
        // Check if any ancestor is a hot directory
        for ancestor in node.path.ancestors() {
            if self.hot_dirs.contains(ancestor) {
                return Some(InterestFactor::HotDirectory {
                    change_count: 0, // We don't track exact count here
                    weight: self.weights.hot_directory,
                });
            }
        }
        None
    }

    /// Check if inside a dependency tree (reduces interest)
    fn check_dependency_context(&self, node: &FileNode) -> Option<InterestFactor> {
        let path_str = node.path.to_string_lossy();

        // Check for common dependency directories
        let dep_indicators = [
            ("node_modules", DependencyManager::Npm),
            ("target/debug", DependencyManager::Cargo),
            ("target/release", DependencyManager::Cargo),
            (".venv", DependencyManager::Python),
            ("venv", DependencyManager::Python),
            ("__pycache__", DependencyManager::Python),
            ("vendor", DependencyManager::Go), // Could also be Ruby/PHP
            (".m2", DependencyManager::Java),
            ("build/classes", DependencyManager::Java),
        ];

        for (indicator, _manager) in &dep_indicators {
            if path_str.contains(indicator) {
                // Calculate depth inside dependency tree
                let depth = path_str
                    .split(indicator)
                    .nth(1)
                    .map(|s| s.matches('/').count())
                    .unwrap_or(0);

                return Some(InterestFactor::InDependencyTree {
                    depth,
                    weight: self.weights.dependency_depth_penalty * (depth as f32 + 1.0),
                });
            }
        }

        None
    }

    /// Check filesystem type (virtual filesystems are less interesting)
    fn check_filesystem_type(&self, node: &FileNode) -> Option<InterestFactor> {
        match node.filesystem_type {
            FilesystemType::Procfs | FilesystemType::Sysfs | FilesystemType::Devfs => {
                Some(InterestFactor::InDependencyTree {
                    depth: 0,
                    weight: -0.5, // Strong negative for virtual filesystems
                })
            }
            FilesystemType::Tmpfs => Some(InterestFactor::InDependencyTree {
                depth: 0,
                weight: -0.2, // Mild negative for temp filesystems
            }),
            _ => None,
        }
    }

    /// Boost interest based on file category
    fn check_category_boost(&self, node: &FileNode) -> Option<InterestFactor> {
        if node.is_dir {
            return None;
        }

        // Source code files are generally more interesting
        let boost: f32 = match node.category {
            FileCategory::Rust
            | FileCategory::Python
            | FileCategory::JavaScript
            | FileCategory::TypeScript
            | FileCategory::Go
            | FileCategory::Java
            | FileCategory::Cpp
            | FileCategory::C => 0.1,

            // Config and build files
            FileCategory::Toml
            | FileCategory::Yaml
            | FileCategory::Json
            | FileCategory::Makefile
            | FileCategory::Dockerfile => 0.15,

            // Documentation
            FileCategory::Markdown | FileCategory::Readme => 0.1,

            // Tests are interesting
            FileCategory::Test => 0.1,

            // Archives and binaries less interesting
            FileCategory::Archive | FileCategory::Binary | FileCategory::DiskImage => -0.1,

            // Temp and backup files not interesting
            FileCategory::Temp | FileCategory::Backup => -0.2,

            _ => 0.0,
        };

        if boost.abs() > 0.01 {
            Some(InterestFactor::Custom {
                name: format!("Category: {:?}", node.category),
                weight: boost,
            })
        } else {
            None
        }
    }

    /// Build traversal context for a node
    pub fn build_traversal_context(
        &self,
        node: &FileNode,
        parent_interest: Option<InterestLevel>,
    ) -> TraversalContext {
        let path_str = node.path.to_string_lossy();

        // Determine traversal path type
        let traversal_path = if node.is_symlink {
            TraversalPath::Symlink {
                target: std::fs::read_link(&node.path).unwrap_or_default(),
                target_exists: node.path.exists(),
            }
        } else if let Some((indicator, manager)) = self.find_dependency_indicator(&path_str) {
            TraversalPath::Dependency {
                manager,
                dep_root: node
                    .path
                    .to_string_lossy()
                    .split(indicator)
                    .next()
                    .map(|s| std::path::PathBuf::from(format!("{}{}", s, indicator)))
                    .unwrap_or_default(),
            }
        } else {
            TraversalPath::Direct
        };

        // Check for git worktree
        let in_git_worktree = node.path.join(".git").exists()
            || node
                .path
                .ancestors()
                .any(|p| p.join(".git").exists());

        // Check for submodule
        let in_submodule = node
            .path
            .ancestors()
            .any(|p| p.join(".git").is_file()); // Submodules have .git as file

        TraversalContext {
            path: traversal_path,
            depth_from_root: node.depth,
            in_git_worktree,
            in_submodule,
            parent_interest,
        }
    }

    /// Find dependency indicator in path
    fn find_dependency_indicator(&self, path: &str) -> Option<(&'static str, DependencyManager)> {
        let indicators = [
            ("node_modules", DependencyManager::Npm),
            ("target/debug", DependencyManager::Cargo),
            ("target/release", DependencyManager::Cargo),
            (".venv", DependencyManager::Python),
            ("venv", DependencyManager::Python),
            ("vendor", DependencyManager::Go),
            (".m2", DependencyManager::Java),
        ];

        for (indicator, manager) in indicators {
            if path.contains(indicator) {
                return Some((indicator, manager));
            }
        }
        None
    }
}

impl Default for InterestCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick helper to determine if a path is likely interesting
pub fn quick_interest_check(path: &Path) -> InterestLevel {
    let path_str = path.to_string_lossy();

    // Critical paths
    if path_str.contains(".env") && !path_str.contains(".env.example") {
        return InterestLevel::Critical;
    }

    // Boring paths
    let boring_patterns = [
        "node_modules",
        "target/debug",
        "target/release",
        "__pycache__",
        ".git/objects",
        ".venv",
        "venv/lib",
    ];

    for pattern in boring_patterns {
        if path_str.contains(pattern) {
            return InterestLevel::Boring;
        }
    }

    // Key files
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let name_lower = name.to_lowercase();
        if matches!(
            name_lower.as_str(),
            "readme.md"
                | "cargo.toml"
                | "package.json"
                | "main.rs"
                | "lib.rs"
                | "index.js"
                | "index.ts"
        ) {
            return InterestLevel::Important;
        }
    }

    InterestLevel::Background
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::Duration;

    fn make_test_node(path: &str, is_dir: bool, hours_old: f32) -> FileNode {
        let modified = SystemTime::now() - Duration::from_secs_f32(hours_old * 3600.0);

        FileNode {
            path: PathBuf::from(path),
            is_dir,
            size: 1000,
            permissions: 0o644,
            uid: 1000,
            gid: 1000,
            modified,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            depth: path.matches('/').count(),
            file_type: if is_dir {
                FileType::Directory
            } else {
                FileType::RegularFile
            },
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        }
    }

    #[test]
    fn test_recency_scoring() {
        let calc = InterestCalculator::new();

        // Very recent file
        let recent = make_test_node("src/main.rs", false, 0.5);
        let score = calc.calculate(&recent);
        assert!(score.score > 0.3, "Recent file should have high score");

        // Old file
        let old = make_test_node("src/old.rs", false, 200.0);
        let score = calc.calculate(&old);
        assert!(score.score < 0.2, "Old file should have low score");
    }

    #[test]
    fn test_key_file_detection() {
        let calc = InterestCalculator::new();

        let readme = make_test_node("README.md", false, 100.0);
        let score = calc.calculate(&readme);
        assert!(
            score.score >= 0.4,
            "README should be important: {}",
            score.score
        );

        let cargo = make_test_node("Cargo.toml", false, 100.0);
        let score = calc.calculate(&cargo);
        assert!(
            score.score >= 0.4,
            "Cargo.toml should be important: {}",
            score.score
        );
    }

    #[test]
    fn test_dependency_penalty() {
        let calc = InterestCalculator::new();

        // File in node_modules (use old file to avoid recency boost)
        let node_mod = make_test_node("node_modules/lodash/index.js", false, 200.0);
        let score = calc.calculate(&node_mod);
        // Category boost (+0.1) minus dependency penalty (-0.1*depth) can be positive
        // Key assertion: it should be lower than files outside node_modules
        assert!(
            score.score < 0.3,
            "node_modules file should have reduced interest: {}",
            score.score
        );

        // Same file outside node_modules (also old)
        let normal = make_test_node("src/utils/index.js", false, 200.0);
        let score = calc.calculate(&normal);

        // The normal file should score higher than node_modules file
        let node_mod_score = calc.calculate(&make_test_node("node_modules/lodash/index.js", false, 200.0)).score;
        assert!(
            score.score > node_mod_score,
            "Normal source file ({}) should have higher interest than node_modules ({})",
            score.score,
            node_mod_score
        );
    }

    #[test]
    fn test_quick_interest_check() {
        assert_eq!(
            quick_interest_check(Path::new(".env")),
            InterestLevel::Critical
        );
        assert_eq!(
            quick_interest_check(Path::new("node_modules/foo/bar.js")),
            InterestLevel::Boring
        );
        assert_eq!(
            quick_interest_check(Path::new("README.md")),
            InterestLevel::Important
        );
        assert_eq!(
            quick_interest_check(Path::new("src/utils.rs")),
            InterestLevel::Background
        );
    }
}
