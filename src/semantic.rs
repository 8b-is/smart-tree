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
    
    /// Get a wave signature for semantic matching (Omni-inspired!)
    pub fn wave_signature(&self) -> u32 {
        match self {
            Self::Documentation => 0xAAAA_AAAA,  // DOCs wave pattern
            Self::SourceCode => 0x5555_5555,    // CODE wave pattern (inverted)
            Self::Tests => 0xF0F0_F0F0,         // TEST wave pattern
            Self::Configuration => 0x0F0F_0F0F,  // CONFIG wave pattern (inverted tests)
            Self::BuildSystem => 0xFF00_FF00,    // BUILD wave pattern
            Self::Dependencies => 0x00FF_00FF,   // DEPS wave pattern (inverted build)
            Self::Assets => 0xF00F_F00F,         // ASSETS wave pattern
            Self::Data => 0x0FF0_0FF0,           // DATA wave pattern (inverted assets)
            Self::Scripts => 0x3333_3333,        // SCRIPTS wave pattern
            Self::Generated => 0xCCCC_CCCC,      // GENERATED wave pattern (inverted scripts)
            Self::ProjectRoot => 0x1111_1111,    // ROOT wave pattern
            Self::Development => 0x8888_8888,    // DEV wave pattern
            Self::Deployment => 0x4444_4444,     // DEPLOY wave pattern
            Self::Unknown => 0x0000_0000,
        }
    }
}

/// Analyzes files and determines their semantic category
pub struct SemanticAnalyzer {
    // Pattern matching for different file types
    patterns: HashMap<SemanticCategory, Vec<&'static str>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Documentation patterns
        patterns.insert(SemanticCategory::Documentation, vec![
            "README", "readme", "LICENSE", "CHANGELOG", "AUTHORS", "CONTRIBUTORS",
            "INSTALL", "GUIDE", "TUTORIAL", "DOCS", "NOTES", "TODO",
            ".md", ".rst", ".txt", ".adoc", ".org", ".tex",
        ]);
        
        // Source code patterns (by extension)
        patterns.insert(SemanticCategory::SourceCode, vec![
            ".rs", ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".java", ".c", ".cpp",
            ".h", ".hpp", ".cs", ".rb", ".php", ".swift", ".kt", ".scala", ".r",
            ".jl", ".ml", ".hs", ".ex", ".exs", ".clj", ".dart", ".nim",
        ]);
        
        // Test patterns
        patterns.insert(SemanticCategory::Tests, vec![
            "test", "tests", "spec", "specs", "__tests__", "_test", "test_",
            ".test.", ".spec.", "_spec.", "integration", "unit", "e2e",
        ]);
        
        // Configuration patterns
        patterns.insert(SemanticCategory::Configuration, vec![
            ".config", ".conf", ".cfg", ".ini", ".env", ".properties",
            ".json", ".yaml", ".yml", ".toml", ".xml", "settings",
            "config", "configuration", ".gitignore", ".dockerignore",
        ]);
        
        // Build system patterns
        patterns.insert(SemanticCategory::BuildSystem, vec![
            "Makefile", "makefile", "CMakeLists", "build", "BUILD",
            "Cargo.toml", "package.json", "pom.xml", "build.gradle",
            "setup.py", "setup.cfg", "pyproject.toml", "composer.json",
            ".bazel", "meson.build", "SConstruct", "Rakefile",
        ]);
        
        // Dependencies patterns
        patterns.insert(SemanticCategory::Dependencies, vec![
            "node_modules", "vendor", "packages", ".packages", "target",
            "venv", ".venv", "env", ".env", "virtualenv", "__pycache__",
            "dist", "build", ".gradle", ".m2", "Cargo.lock", "package-lock.json",
            "yarn.lock", "poetry.lock", "Gemfile.lock", "requirements.txt",
        ]);
        
        // Assets patterns
        patterns.insert(SemanticCategory::Assets, vec![
            ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".webp",
            ".mp3", ".wav", ".ogg", ".mp4", ".webm", ".mov",
            ".ttf", ".otf", ".woff", ".woff2", ".eot",
            ".css", ".scss", ".sass", ".less", ".styl",
            "assets", "static", "public", "resources", "media",
        ]);
        
        // Data patterns
        patterns.insert(SemanticCategory::Data, vec![
            ".csv", ".tsv", ".parquet", ".feather", ".arrow",
            ".db", ".sqlite", ".sql", ".mdb", ".dbf",
            ".h5", ".hdf5", ".nc", ".zarr", ".npy", ".npz",
            "data", "datasets", "corpus", "samples",
        ]);
        
        // Scripts patterns
        patterns.insert(SemanticCategory::Scripts, vec![
            ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd",
            "scripts", "bin", "tools", "utils", "hooks",
            "install", "setup", "deploy", "run", "start", "stop",
        ]);
        
        // Generated patterns
        patterns.insert(SemanticCategory::Generated, vec![
            ".o", ".a", ".so", ".dll", ".dylib", ".exe", ".app",
            ".class", ".jar", ".war", ".pyc", ".pyo", ".pyd",
            ".min.js", ".min.css", ".bundle.js", ".chunk.js",
            "generated", "gen", "auto", "autogen", ".g.dart",
        ]);
        
        Self { patterns }
    }
    
    /// Analyze a file path and determine its semantic category
    pub fn categorize(&self, path: &Path) -> SemanticCategory {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // First, check for specific build system files that should override other patterns
        if file_name == "cargo.toml" || file_name == "package.json" || 
           file_name == "makefile" || file_name == "cmakelists.txt" ||
           file_name == "build.gradle" || file_name == "setup.py" {
            return SemanticCategory::BuildSystem;
        }
        
        // Check if it's a test file first (high priority)
        if self.is_test_file(&path_str, &file_name) {
            return SemanticCategory::Tests;
        }
        
        // Check each category's patterns
        for (category, patterns) in &self.patterns {
            for pattern in patterns {
                if file_name.contains(pattern) || path_str.contains(pattern) {
                    // Special handling for tests - they override source code
                    if *category == SemanticCategory::SourceCode && 
                       self.is_test_file(&path_str, &file_name) {
                        return SemanticCategory::Tests;
                    }
                    return category.clone();
                }
            }
        }
        
        // Check if it's a project root file
        if path.parent().is_none() || path.components().count() == 1 {
            if file_name == "cargo.toml" || file_name == "package.json" || 
               file_name == "setup.py" || file_name == "go.mod" {
                return SemanticCategory::ProjectRoot;
            }
        }
        
        SemanticCategory::Unknown
    }
    
    /// Check if a file is a test file
    fn is_test_file(&self, path_str: &str, file_name: &str) -> bool {
        let test_patterns = &self.patterns[&SemanticCategory::Tests];
        test_patterns.iter().any(|pattern| {
            file_name.contains(pattern) || path_str.contains(pattern)
        })
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
        groups.entry(category).or_insert_with(Vec::new).push(file);
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
        assert_eq!(analyzer.categorize(&PathBuf::from("README.md")), SemanticCategory::Documentation);
        assert_eq!(analyzer.categorize(&PathBuf::from("main.rs")), SemanticCategory::SourceCode);
        assert_eq!(analyzer.categorize(&PathBuf::from("test_utils.rs")), SemanticCategory::Tests);
        assert_eq!(analyzer.categorize(&PathBuf::from("Cargo.toml")), SemanticCategory::BuildSystem);
        assert_eq!(analyzer.categorize(&PathBuf::from("config.yaml")), SemanticCategory::Configuration);
        assert_eq!(analyzer.categorize(&PathBuf::from("logo.png")), SemanticCategory::Assets);
        assert_eq!(analyzer.categorize(&PathBuf::from("data.csv")), SemanticCategory::Data);
        assert_eq!(analyzer.categorize(&PathBuf::from("install.sh")), SemanticCategory::Scripts);
        assert_eq!(analyzer.categorize(&PathBuf::from("main.o")), SemanticCategory::Generated);
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
        let similarity = analyzer.similarity(
            &PathBuf::from("main.rs"),
            &PathBuf::from("lib.rs")
        );
        assert!(similarity > 0.8);
        
        // Different category files should have lower similarity
        let similarity = analyzer.similarity(
            &PathBuf::from("main.rs"),
            &PathBuf::from("README.md")
        );
        assert!(similarity < 0.5);
    }
}