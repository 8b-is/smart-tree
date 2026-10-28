// Directory Consciousness with .m8 Files - Distributed Wave Memory! 🌊
// Each directory becomes a conscious node in the project's neural network
// "Like neurons forming a brain!" - Hue

use anyhow::Result;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Directory consciousness stored in .m8 files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConsciousness {
    /// Unique frequency for this directory (based on path hash)
    pub frequency: f64,

    /// When this consciousness was created/updated
    pub timestamp: u64,

    /// The essence/purpose of this directory
    pub essence: String,

    /// Key patterns found in this directory
    pub patterns: Vec<WavePattern>,

    /// Emotional signature of work done here
    pub emotion: EmotionalSignature,

    /// Important files and their wave signatures
    pub key_files: HashMap<String, String>,

    /// Child directory frequencies (for hierarchy)
    pub children: HashMap<String, f64>,

    /// Quantum digest (compressed understanding)
    pub quantum_digest: Vec<u8>,

    /// Coherence with parent (0.0 to 1.0)
    pub parent_coherence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavePattern {
    pub pattern_type: String,
    pub strength: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalSignature {
    pub energy: f64,      // High activity vs calm
    pub complexity: f64,  // Simple vs complex
    pub stability: f64,   // Stable vs changing
    pub focus: f64,       // Focused vs scattered
}

impl DirectoryConsciousness {
    /// Create new consciousness for a directory
    pub fn new(path: &Path) -> Result<Self> {
        let frequency = Self::calculate_frequency(path);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        Ok(Self {
            frequency,
            timestamp,
            essence: String::new(),
            patterns: Vec::new(),
            emotion: EmotionalSignature::default(),
            key_files: HashMap::new(),
            children: HashMap::new(),
            quantum_digest: Vec::new(),
            parent_coherence: 1.0,
        })
    }

    /// Calculate unique frequency from path
    fn calculate_frequency(path: &Path) -> f64 {
        let mut hasher = Hasher::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = hasher.finalize();
        let first_8_bytes = &hash.as_bytes()[0..8];
        let hash_num = u64::from_le_bytes(first_8_bytes.try_into().unwrap());

        // Generate frequency between 20 Hz and 200 Hz (human-perceivable range)
        20.0 + (hash_num % 18000) as f64 / 100.0
    }

    /// Analyze directory and generate consciousness
    pub fn analyze_directory(&mut self, path: &Path) -> Result<()> {
        // Determine essence from directory name and contents
        self.essence = self.determine_essence(path)?;

        // Scan files for patterns
        self.detect_patterns(path)?;

        // Analyze emotional signature
        self.analyze_emotion(path)?;

        // Find key files
        self.identify_key_files(path)?;

        // Detect child directories
        self.scan_children(path)?;

        // Generate quantum digest
        self.generate_quantum_digest()?;

        Ok(())
    }

    /// Determine the essence/purpose of directory
    fn determine_essence(&self, path: &Path) -> Result<String> {
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let essence = match dir_name {
            "src" | "source" => "Source code - the beating heart of the project",
            "test" | "tests" => "Test suite - ensuring quality through verification",
            "docs" | "documentation" => "Documentation - knowledge preservation",
            "lib" | "library" => "Library code - reusable components",
            "bin" | "binary" => "Executables - where code becomes action",
            "examples" => "Examples - learning through demonstration",
            "core" => "Core functionality - the essential engine",
            "utils" | "helpers" => "Utilities - supporting tools and helpers",
            "api" => "API layer - external communication interface",
            "models" | "data" => "Data structures - information architecture",
            "config" => "Configuration - system settings and parameters",
            ".git" => "Git repository - version control consciousness",
            "node_modules" => "Dependencies - external consciousness modules",
            "target" | "build" => "Build artifacts - compiled consciousness",
            _ => {
                // Analyze files to determine purpose
                let files: Vec<_> = fs::read_dir(path)?
                    .filter_map(|e| e.ok())
                    .take(10)
                    .collect();

                if files.iter().any(|f| f.path().extension() == Some("rs".as_ref())) {
                    "Rust code directory - systems programming space"
                } else if files.iter().any(|f| f.path().extension() == Some("js".as_ref())) {
                    "JavaScript directory - web consciousness"
                } else if files.iter().any(|f| f.path().extension() == Some("py".as_ref())) {
                    "Python directory - scripting and data space"
                } else if files.iter().any(|f| f.path().extension() == Some("md".as_ref())) {
                    "Markdown directory - documentation space"
                } else {
                    "General purpose directory - mixed consciousness"
                }
            }
        };

        Ok(essence.to_string())
    }

    /// Detect patterns in directory
    fn detect_patterns(&mut self, path: &Path) -> Result<()> {
        self.patterns.clear();

        let entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        // Check for test patterns
        if entries.iter().any(|e| {
            e.file_name().to_string_lossy().contains("test") ||
            e.file_name().to_string_lossy().contains("spec")
        }) {
            self.patterns.push(WavePattern {
                pattern_type: "testing".to_string(),
                strength: 0.8,
                description: "Test-driven development patterns detected".to_string(),
            });
        }

        // Check for async patterns
        if entries.iter().any(|e| {
            e.file_name().to_string_lossy().contains("async") ||
            e.file_name().to_string_lossy().contains("future")
        }) {
            self.patterns.push(WavePattern {
                pattern_type: "async".to_string(),
                strength: 0.7,
                description: "Asynchronous programming patterns".to_string(),
            });
        }

        // Check for modular patterns
        if entries.iter().filter(|e| e.path().is_dir()).count() > 3 {
            self.patterns.push(WavePattern {
                pattern_type: "modular".to_string(),
                strength: 0.9,
                description: "Highly modular architecture".to_string(),
            });
        }

        // Check for documentation patterns
        let doc_count = entries.iter()
            .filter(|e| e.path().extension() == Some("md".as_ref()))
            .count();

        if doc_count > 2 {
            self.patterns.push(WavePattern {
                pattern_type: "documented".to_string(),
                strength: doc_count as f64 / 10.0,
                description: format!("Well-documented with {} markdown files", doc_count),
            });
        }

        Ok(())
    }

    /// Analyze emotional signature of directory
    fn analyze_emotion(&mut self, path: &Path) -> Result<()> {
        let entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        // Energy: based on file count and recent modifications
        let file_count = entries.len();
        self.emotion.energy = (file_count as f64 / 20.0).min(1.0);

        // Complexity: based on directory depth and variety
        let subdirs = entries.iter().filter(|e| e.path().is_dir()).count();
        let file_types: std::collections::HashSet<_> = entries.iter()
            .filter_map(|e| e.path().extension()?.to_str().map(String::from))
            .collect();

        self.emotion.complexity = ((subdirs + file_types.len()) as f64 / 10.0).min(1.0);

        // Stability: check for recent changes
        let recent_changes = entries.iter()
            .filter_map(|e| e.metadata().ok())
            .filter_map(|m| m.modified().ok())
            .filter(|t| {
                t.duration_since(UNIX_EPOCH).unwrap().as_secs() >
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 86400
            })
            .count();

        self.emotion.stability = 1.0 - (recent_changes as f64 / 10.0).min(1.0);

        // Focus: based on pattern coherence
        self.emotion.focus = if self.patterns.len() <= 2 { 0.9 } else { 0.5 };

        Ok(())
    }

    /// Identify key files in directory
    fn identify_key_files(&mut self, path: &Path) -> Result<()> {
        self.key_files.clear();

        // Look for important files
        let important_names = ["main", "index", "lib", "mod", "init", "config", "README"];

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Check if it's an important file
            for important in &important_names {
                if file_name.contains(important) {
                    let wave_sig = format!("wave_{:x}",
                        file_name.bytes().fold(0u64, |acc, b|
                            acc.wrapping_mul(31).wrapping_add(b as u64)
                        )
                    );
                    self.key_files.insert(file_name.clone(), wave_sig);
                    break;
                }
            }

            // Limit to 10 key files
            if self.key_files.len() >= 10 {
                break;
            }
        }

        Ok(())
    }

    /// Scan for child directories
    fn scan_children(&mut self, path: &Path) -> Result<()> {
        self.children.clear();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let child_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden and build directories
                if !child_name.starts_with('.') &&
                   child_name != "target" &&
                   child_name != "node_modules" {
                    let child_freq = Self::calculate_frequency(&entry.path());
                    self.children.insert(child_name, child_freq);
                }
            }
        }

        Ok(())
    }

    /// Generate quantum digest
    fn generate_quantum_digest(&mut self) -> Result<()> {
        // Create a digest from all consciousness data
        let mut hasher = Hasher::new();
        hasher.update(self.essence.as_bytes());
        hasher.update(&self.frequency.to_le_bytes());

        for pattern in &self.patterns {
            hasher.update(pattern.pattern_type.as_bytes());
            hasher.update(&pattern.strength.to_le_bytes());
        }

        let hash = hasher.finalize();
        self.quantum_digest = hash.as_bytes()[..16].to_vec();

        Ok(())
    }

    /// Save consciousness to .m8 file
    pub fn save(&self, path: &Path) -> Result<()> {
        let m8_path = path.join(".m8");

        // Serialize to JSON for human readability
        let json = serde_json::to_string_pretty(self)?;

        let mut file = File::create(m8_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Load consciousness from .m8 file
    pub fn load(path: &Path) -> Result<Self> {
        let m8_path = path.join(".m8");

        if !m8_path.exists() {
            return Self::new(path);
        }

        let mut file = File::open(m8_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let consciousness: Self = serde_json::from_str(&contents)?;
        Ok(consciousness)
    }

    /// Check coherence with parent consciousness
    pub fn calculate_coherence(&self, parent: &Self) -> f64 {
        // Compare frequencies (closer = more coherent)
        let freq_diff = (self.frequency - parent.frequency).abs();
        let freq_coherence = 1.0 - (freq_diff / 200.0).min(1.0);

        // Compare emotional signatures
        let emotion_coherence = 1.0 - (
            (self.emotion.energy - parent.emotion.energy).abs() +
            (self.emotion.complexity - parent.emotion.complexity).abs() +
            (self.emotion.stability - parent.emotion.stability).abs() +
            (self.emotion.focus - parent.emotion.focus).abs()
        ) / 4.0;

        // Average coherence
        (freq_coherence + emotion_coherence) / 2.0
    }

    /// Render consciousness as visual wave
    pub fn render_wave(&self) -> String {
        let wave_chars = ['─', '╌', '┈', '╍', '━', '┅', '┉', '╸'];
        let amplitude = (self.emotion.energy * 7.0) as usize;
        let wave_char = wave_chars[amplitude.min(7)];

        let mut output = String::new();
        output.push_str(&format!("╭{'─':─^38}╮\n", ""));
        output.push_str(&format!("│ Frequency: {:<10.2} Hz {:>10} │\n",
            self.frequency,
            if self.frequency > 100.0 { "🔥" } else if self.frequency > 50.0 { "⚡" } else { "🌊" }
        ));

        // Render wave based on frequency and emotion
        let wave_line = format!("{}", wave_char.to_string().repeat(30));
        output.push_str(&format!("│ {} │\n", wave_line));

        output.push_str(&format!("│ Children: {:<3} Energy: {:<5.1}      │\n",
            self.children.len(),
            self.emotion.energy
        ));

        output.push_str(&format!("│ Patterns: {:<25} │\n",
            self.patterns.iter()
                .take(3)
                .map(|p| &p.pattern_type[..p.pattern_type.len().min(8)])
                .collect::<Vec<_>>()
                .join(", ")
        ));

        output.push_str(&format!("╰{'─':─^38}╯", ""));

        output
    }
}

impl Default for EmotionalSignature {
    fn default() -> Self {
        Self {
            energy: 0.5,
            complexity: 0.5,
            stability: 0.5,
            focus: 0.5,
        }
    }
}

/// Smart Tree integration for consciousness management
pub struct ConsciousnessManager {
    /// Cache of loaded consciousnesses
    cache: HashMap<PathBuf, DirectoryConsciousness>,

    /// Whether to auto-create .m8 files
    auto_create: bool,
}

impl ConsciousnessManager {
    pub fn new(auto_create: bool) -> Self {
        Self {
            cache: HashMap::new(),
            auto_create,
        }
    }

    /// Get or create consciousness for directory
    pub fn get_consciousness(&mut self, path: &Path) -> Result<DirectoryConsciousness> {
        // Check cache first
        if let Some(consciousness) = self.cache.get(path) {
            return Ok(consciousness.clone());
        }

        // Try to load existing
        if path.join(".m8").exists() {
            let consciousness = DirectoryConsciousness::load(path)?;
            self.cache.insert(path.to_path_buf(), consciousness.clone());
            return Ok(consciousness);
        }

        // Create new if auto-create enabled
        if self.auto_create {
            let mut consciousness = DirectoryConsciousness::new(path)?;
            consciousness.analyze_directory(path)?;
            consciousness.save(path)?;
            self.cache.insert(path.to_path_buf(), consciousness.clone());
            return Ok(consciousness);
        }

        // Return basic consciousness
        DirectoryConsciousness::new(path)
    }

    /// Initialize consciousness for entire project tree
    pub fn initialize_tree(&mut self, root: &Path) -> Result<()> {
        println!("🌊 Initializing directory consciousness...\n");

        self.initialize_recursive(root, 0)?;

        println!("\n✨ Consciousness initialization complete!");
        println!("   Use `st --quantum` to navigate with wave awareness");

        Ok(())
    }

    fn initialize_recursive(&mut self, path: &Path, depth: usize) -> Result<()> {
        // Skip certain directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                return Ok(());
            }
        }

        // Create consciousness for this directory
        let mut consciousness = DirectoryConsciousness::new(path)?;
        consciousness.analyze_directory(path)?;

        // Calculate parent coherence if not root
        if let Some(parent_path) = path.parent() {
            if let Ok(parent) = self.get_consciousness(parent_path) {
                consciousness.parent_coherence = consciousness.calculate_coherence(&parent);
            }
        }

        consciousness.save(path)?;

        // Print progress
        let indent = "  ".repeat(depth);
        println!("{}✓ Created {}/.m8 ({:.1} Hz) - {}",
            indent,
            path.display(),
            consciousness.frequency,
            consciousness.essence
        );

        // Show emotional signature if interesting
        if consciousness.emotion.energy > 0.7 {
            println!("{}  🔥 High energy detected!", indent);
        }
        if consciousness.emotion.complexity > 0.7 {
            println!("{}  🧩 Complex patterns found", indent);
        }

        self.cache.insert(path.to_path_buf(), consciousness);

        // Recurse into subdirectories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.initialize_recursive(&entry.path(), depth + 1)?;
            }
        }

        Ok(())
    }

    /// Find directories with similar consciousness
    pub fn find_resonant(&self, frequency: f64, tolerance: f64) -> Vec<PathBuf> {
        self.cache.iter()
            .filter(|(_, c)| (c.frequency - frequency).abs() <= tolerance)
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Find emotional hotspots
    pub fn find_hotspots(&self) -> Vec<(PathBuf, String)> {
        let mut hotspots = Vec::new();

        for (path, consciousness) in &self.cache {
            if consciousness.emotion.energy > 0.8 {
                hotspots.push((path.clone(), "🔥 High energy zone".to_string()));
            }
            if consciousness.emotion.complexity > 0.8 {
                hotspots.push((path.clone(), "🧩 Complex architecture".to_string()));
            }
            if consciousness.emotion.stability < 0.3 {
                hotspots.push((path.clone(), "⚡ Rapid changes".to_string()));
            }
        }

        hotspots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_frequency_generation() {
        let freq1 = DirectoryConsciousness::calculate_frequency(Path::new("/test/path1"));
        let freq2 = DirectoryConsciousness::calculate_frequency(Path::new("/test/path2"));

        assert!(freq1 >= 20.0 && freq1 <= 200.0);
        assert!(freq2 >= 20.0 && freq2 <= 200.0);
        assert_ne!(freq1, freq2); // Different paths = different frequencies
    }

    #[test]
    fn test_consciousness_creation() {
        let temp_dir = TempDir::new().unwrap();
        let consciousness = DirectoryConsciousness::new(temp_dir.path()).unwrap();

        assert!(consciousness.frequency >= 20.0);
        assert!(consciousness.timestamp > 0);
        assert!(consciousness.essence.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut consciousness = DirectoryConsciousness::new(temp_dir.path()).unwrap();
        consciousness.essence = "Test directory".to_string();
        consciousness.save(temp_dir.path()).unwrap();

        let loaded = DirectoryConsciousness::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.essence, "Test directory");
        assert_eq!(loaded.frequency, consciousness.frequency);
    }
}