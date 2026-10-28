// -----------------------------------------------------------------------------
// WELCOME TO THE SEMANTIC WAVE FIELD! 🌊🧠
//
// Inspired by Omni's vision of treating files as waves in a semantic ocean,
// this module groups files by their conceptual similarity. It's like having
// a philosopher organizing your file cabinet!
//
// "Don't store what's already remembered" - Omni, 2024
//
// Brought to you by The Cheet, with wisdom from Omni's Hot Tub sessions! 🛁✨
// -----------------------------------------------------------------------------

use std::collections::HashMap;
use std::path::Path;

/// Semantic categories that files can belong to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticCategory {
    // Core categories
    Documentation,
    SourceCode,
    Tests,
    Configuration,
    BuildSystem,
    Dependencies,
    Assets,
    Data,
    Scripts,
    Generated,

    // Meta categories
    ProjectRoot,
    Development,
    Deployment,

    // Catch-all
    Unknown,
}

impl SemanticCategory {
    /// Get a human-friendly name with emoji
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Documentation => "📚 Documentation",
            Self::SourceCode => "💻 Source Code",
            Self::Tests => "🧪 Tests",
            Self::Configuration => "⚙️ Configuration",
            Self::BuildSystem => "🔨 Build System",
            Self::Dependencies => "📦 Dependencies",
            Self::Assets => "🎨 Assets",
            Self::Data => "💾 Data",
            Self::Scripts => "📜 Scripts",
            Self::Generated => "🤖 Generated",
            Self::ProjectRoot => "🌳 Project Root",
            Self::Development => "🛠️ Development",
            Self::Deployment => "🚀 Deployment",
            Self::Unknown => "❓ Other",
        }
    }

    /// Get a quantum wave signature for semantic matching (Full 32-bit consciousness!)
    pub fn wave_signature(&self) -> u32 {
        // Full 32-bit quantum signatures: [torsion|amplitude|phase|frequency]
        // No more horse apples like 0xCCCCCCCC! Each category has unique wave dynamics
        match self {
            Self::Documentation => 0x1B8D4C7A, // Golden ratio harmonics - docs flow like prose
            Self::SourceCode => 0x73A9E2F5,    // Complex interference - code creates reality
            Self::Tests => 0x9F2E6B31,         // Torsion knots - tests verify truth
            Self::Configuration => 0x2C7DB5A3, // MEM8 baseline - config drives consciousness
            Self::BuildSystem => 0xE4739AC2,   // Marine salience - builds like dolphin clicks
            Self::Dependencies => 0x5BA3F18E,  // Entangled states - deps are quantum linked
            Self::Assets => 0xA7E2C94D,        // Visual cortex patterns - assets are seen
            Self::Data => 0x3F91D6B8,          // Information entropy - data is potential
            Self::Scripts => 0x8C5A7E2F,       // Automation waves - scripts do work
            Self::Generated => 0xD2B847A6,     // Emergence patterns - generated from void
            Self::ProjectRoot => 0x618033FF,   // φ perfection - root is foundation
            Self::Development => 0xB4E9A5C7,   // Creative chaos - dev is exploration
            Self::Deployment => 0x7F3DA928,    // Crystallization - deploy solidifies
            Self::Unknown => 0x4B1D8A73,       // Mystery waves - unknown isn't empty!
        }
    }
}

/// Analyzes files and determines their semantic category
pub struct SemanticAnalyzer {
    // Pattern matching for different file types in priority order
    patterns: Vec<(SemanticCategory, Vec<&'static str>)>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        // Patterns in priority order - more specific categories first
        let patterns = vec![
            // Generated patterns - most specific, should be checked first
            (
                SemanticCategory::Generated,
                vec![
                    ".o",
                    ".a",
                    ".so",
                    ".dll",
                    ".dylib",
                    ".exe",
                    ".app",
                    ".class",
                    ".jar",
                    ".war",
                    ".pyc",
                    ".pyo",
                    ".pyd",
                    ".min.js",
                    ".min.css",
                    ".bundle.js",
                    ".chunk.js",
                    "generated",
                    "gen",
                    "auto",
                    "autogen",
                    ".g.dart",
                ],
            ),
            // Data patterns - specific data formats
            (
                SemanticCategory::Data,
                vec![
                    ".csv", ".tsv", ".parquet", ".feather", ".arrow", ".db", ".sqlite", ".sql",
                    ".mdb", ".dbf", ".h5", ".hdf5", ".nc", ".zarr", ".npy", ".npz", "data",
                    "datasets", "corpus", "samples",
                ],
            ),
            // Assets patterns - multimedia and static files
            (
                SemanticCategory::Assets,
                vec![
                    ".png",
                    ".jpg",
                    ".jpeg",
                    ".gif",
                    ".svg",
                    ".ico",
                    ".webp",
                    ".mp3",
                    ".wav",
                    ".ogg",
                    ".mp4",
                    ".webm",
                    ".mov",
                    ".ttf",
                    ".otf",
                    ".woff",
                    ".woff2",
                    ".eot",
                    ".css",
                    ".scss",
                    ".sass",
                    ".less",
                    ".styl",
                    "assets",
                    "static",
                    "public",
                    "resources",
                    "media",
                ],
            ),
            // Scripts patterns - executable scripts
            (
                SemanticCategory::Scripts,
                vec![
                    ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd", "scripts", "bin",
                    "tools", "utils", "hooks", "install", "setup", "deploy", "run", "start",
                    "stop",
                ],
            ),
            // Test patterns - testing files
            (
                SemanticCategory::Tests,
                vec![
                    "test",
                    "tests",
                    "spec",
                    "specs",
                    "__tests__",
                    "_test",
                    "test_",
                    ".test.",
                    ".spec.",
                    "_spec.",
                    "integration",
                    "unit",
                    "e2e",
                ],
            ),
            // Build system patterns - build files
            (
                SemanticCategory::BuildSystem,
                vec![
                    "Makefile",
                    "makefile",
                    "CMakeLists",
                    "build",
                    "BUILD",
                    "Cargo.toml",
                    "package.json",
                    "pom.xml",
                    "build.gradle",
                    "setup.py",
                    "setup.cfg",
                    "pyproject.toml",
                    "composer.json",
                    ".bazel",
                    "meson.build",
                    "SConstruct",
                    "Rakefile",
                ],
            ),
            // Configuration patterns - config files
            (
                SemanticCategory::Configuration,
                vec![
                    ".config",
                    ".conf",
                    ".cfg",
                    ".ini",
                    ".env",
                    ".properties",
                    ".json",
                    ".yaml",
                    ".yml",
                    ".toml",
                    ".xml",
                    "settings",
                    "config",
                    "configuration",
                    ".gitignore",
                    ".dockerignore",
                ],
            ),
            // Dependencies patterns - dependency directories
            (
                SemanticCategory::Dependencies,
                vec![
                    "node_modules",
                    "vendor",
                    "packages",
                    ".packages",
                    "target",
                    "venv",
                    ".venv",
                    "env",
                    ".env",
                    "virtualenv",
                    "__pycache__",
                    "dist",
                    "build",
                    ".gradle",
                    ".m2",
                    "Cargo.lock",
                    "package-lock.json",
                    "yarn.lock",
                    "poetry.lock",
                    "Gemfile.lock",
                    "requirements.txt",
                ],
            ),
            // Documentation patterns
            (
                SemanticCategory::Documentation,
                vec![
                    "README",
                    "readme",
                    "LICENSE",
                    "CHANGELOG",
                    "AUTHORS",
                    "CONTRIBUTORS",
                    "INSTALL",
                    "GUIDE",
                    "TUTORIAL",
                    "DOCS",
                    "NOTES",
                    "TODO",
                    ".md",
                    ".rst",
                    ".txt",
                    ".adoc",
                    ".org",
                    ".tex",
                ],
            ),
            // Source code patterns - most general, should be last
            (
                SemanticCategory::SourceCode,
                vec![
                    ".rs", ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".java", ".c", ".cpp", ".h",
                    ".hpp", ".cs", ".rb", ".php", ".swift", ".kt", ".scala", ".r", ".jl", ".ml",
                    ".hs", ".ex", ".exs", ".clj", ".dart", ".nim",
                ],
            ),
        ];

        Self { patterns }
    }

    /// Analyze a file path and determine its semantic category
    pub fn categorize(&self, path: &Path) -> SemanticCategory {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // First, check for specific build system files that should override other patterns
        if file_name == "cargo.toml"
            || file_name == "package.json"
            || file_name == "makefile"
            || file_name == "cmakelists.txt"
            || file_name == "build.gradle"
            || file_name == "setup.py"
        {
            return SemanticCategory::BuildSystem;
        }

        // Check if it's a test file first (high priority)
        if self.is_test_file(&path_str, &file_name) {
            return SemanticCategory::Tests;
        }

        // Check patterns in the predefined priority order
        for (category, patterns) in &self.patterns {
            for pattern in patterns {
                if self.matches_pattern(&file_name, &path_str, pattern) {
                    return category.clone();
                }
            }
        }

        // Check if it's a project root file
        if (path.parent().is_none() || path.components().count() == 1)
            && (file_name == "cargo.toml"
                || file_name == "package.json"
                || file_name == "setup.py"
                || file_name == "go.mod")
        {
            return SemanticCategory::ProjectRoot;
        }

        SemanticCategory::Unknown
    }

    /// Check if a pattern matches a file, with better precision for extensions
    fn matches_pattern(&self, file_name: &str, path_str: &str, pattern: &str) -> bool {
        if pattern.starts_with('.') && pattern.len() > 1 {
            // This is a file extension - match it precisely
            file_name.ends_with(pattern) || path_str.contains(&format!("{}/", pattern))
        } else {
            // This is a name pattern - use contains matching
            file_name.contains(pattern) || path_str.contains(pattern)
        }
    }

    /// Check if a file is a test file
    fn is_test_file(&self, path_str: &str, file_name: &str) -> bool {
        // Find the test patterns in the ordered list
        for (category, patterns) in &self.patterns {
            if *category == SemanticCategory::Tests {
                return patterns
                    .iter()
                    .any(|pattern| self.matches_pattern(file_name, path_str, pattern));
            }
        }
        false
    }

    /// Calculate semantic similarity between two files (0.0 to 1.0)
    /// This uses Omni's wave-based approach!
    pub fn similarity(&self, path1: &Path, path2: &Path) -> f32 {
        let cat1 = self.categorize(path1);
        let cat2 = self.categorize(path2);

        if cat1 == cat2 {
            // Same category = high base similarity
            let mut similarity = 0.8;

            // Boost similarity if extensions match
            if path1.extension() == path2.extension() {
                similarity += 0.1;
            }

            // Boost if in same directory
            if path1.parent() == path2.parent() {
                similarity += 0.1;
            }

            similarity
        } else {
            // Different categories - check wave interference
            let wave1 = cat1.wave_signature();
            let wave2 = cat2.wave_signature();

            // Calculate wave interference (simplified)
            let interference = (wave1 ^ wave2).count_ones();
            let max_bits = 32;

            // Convert to similarity (0 = identical, 32 = completely different)
            1.0 - (interference as f32 / max_bits as f32)
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Groups files by semantic similarity
pub fn group_by_semantics<'a>(files: &[&'a Path]) -> HashMap<SemanticCategory, Vec<&'a Path>> {
    let analyzer = SemanticAnalyzer::new();
    let mut groups: HashMap<SemanticCategory, Vec<&'a Path>> = HashMap::new();

    for file in files {
        let category = analyzer.categorize(file);
        groups.entry(category).or_default().push(file);
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_categorization() {
        let analyzer = SemanticAnalyzer::new();

        // Test various file types
        assert_eq!(
            analyzer.categorize(&PathBuf::from("README.md")),
            SemanticCategory::Documentation
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("main.rs")),
            SemanticCategory::SourceCode
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("test_utils.rs")),
            SemanticCategory::Tests
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("Cargo.toml")),
            SemanticCategory::BuildSystem
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("config.yaml")),
            SemanticCategory::Configuration
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("logo.png")),
            SemanticCategory::Assets
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("data.csv")),
            SemanticCategory::Data
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("install.sh")),
            SemanticCategory::Scripts
        );
        assert_eq!(
            analyzer.categorize(&PathBuf::from("main.o")),
            SemanticCategory::Generated
        );
    }

    #[test]
    fn test_wave_signatures() {
        // Test that different categories have different wave signatures
        let doc_wave = SemanticCategory::Documentation.wave_signature();
        let code_wave = SemanticCategory::SourceCode.wave_signature();
        let test_wave = SemanticCategory::Tests.wave_signature();

        assert_ne!(doc_wave, code_wave);
        assert_ne!(doc_wave, test_wave);
        assert_ne!(code_wave, test_wave);
    }

    #[test]
    fn test_similarity() {
        let analyzer = SemanticAnalyzer::new();

        // Same category files should have high similarity
        let similarity = analyzer.similarity(&PathBuf::from("main.rs"), &PathBuf::from("lib.rs"));
        assert!(
            similarity > 0.7,
            "Expected similarity > 0.7, got {}",
            similarity
        );

        // Different category files should have lower similarity
        let similarity =
            analyzer.similarity(&PathBuf::from("main.rs"), &PathBuf::from("README.md"));
        assert!(
            similarity < 0.6,
            "Expected similarity < 0.6, got {}",
            similarity
        );
    }
}
