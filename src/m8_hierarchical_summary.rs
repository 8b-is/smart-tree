// Hierarchical .m8 Summary System - Cascading Consciousness! 🌊
// Each level summarizes everything below it, creating richer context as you go up
// "Like zooming out on a fractal - each level contains the essence of all below!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Enhanced consciousness that summarizes all subdirectories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalConsciousness {
    /// This directory's own consciousness
    pub local: DirectoryEssence,

    /// Summarized consciousness from ALL subdirectories
    pub aggregated: AggregatedConsciousness,

    /// Depth in the hierarchy (root = 0)
    pub depth: usize,

    /// Total directories summarized below
    pub total_descendants: usize,

    /// Condensed wisdom from the entire subtree
    pub unified_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEssence {
    pub path: PathBuf,
    pub frequency: f64,
    pub essence: String,
    pub key_patterns: Vec<String>,
    pub file_count: usize,
    pub primary_language: Option<String>,
    pub last_modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedConsciousness {
    /// Combined patterns from all descendants
    pub unified_patterns: HashMap<String, PatternStrength>,

    /// Project types discovered in subtree
    pub project_types: Vec<ProjectType>,

    /// Aggregated emotional signature
    pub collective_emotion: CollectiveEmotion,

    /// Key insights from the entire subtree
    pub insights: Vec<String>,

    /// Technology stack detected
    pub tech_stack: Vec<String>,

    /// Important files from anywhere in subtree
    pub key_files_global: Vec<KeyFile>,

    /// Quantum coherence of the entire subtree
    pub subtree_coherence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStrength {
    pub occurrences: usize,
    pub strength: f64,
    pub directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectType {
    pub name: String,
    pub confidence: f64,
    pub root_path: String,
    pub characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveEmotion {
    pub average_energy: f64,
    pub peak_energy_zones: Vec<String>,
    pub stability_score: f64,
    pub complexity_gradient: f64,  // How complexity changes with depth
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFile {
    pub path: String,
    pub importance: f64,
    pub reason: String,
}

impl HierarchicalConsciousness {
    /// Create consciousness that summarizes entire subtree
    pub fn create_with_summary(path: &Path) -> Result<Self> {
        let mut consciousness = Self {
            local: DirectoryEssence::analyze(path)?,
            aggregated: AggregatedConsciousness::default(),
            depth: path.components().count(),
            total_descendants: 0,
            unified_summary: String::new(),
        };

        // Recursively gather consciousness from all subdirectories
        consciousness.aggregate_subtree(path)?;

        // Generate unified summary
        consciousness.generate_unified_summary()?;

        Ok(consciousness)
    }

    /// Aggregate consciousness from entire subtree
    fn aggregate_subtree(&mut self, path: &Path) -> Result<()> {
        let mut all_patterns: HashMap<String, PatternStrength> = HashMap::new();
        let mut all_projects = Vec::new();
        let mut all_tech = Vec::new();
        let mut energy_samples = Vec::new();
        let mut complexity_samples = Vec::new();

        // Recursive helper to traverse entire subtree
        self.traverse_and_aggregate(
            path,
            &mut all_patterns,
            &mut all_projects,
            &mut all_tech,
            &mut energy_samples,
            &mut complexity_samples,
            0
        )?;

        // Process aggregated data
        self.aggregated.unified_patterns = all_patterns;
        self.aggregated.project_types = Self::identify_projects(all_projects);
        self.aggregated.tech_stack = Self::deduplicate_tech(all_tech);
        self.aggregated.collective_emotion = Self::calculate_collective_emotion(
            &energy_samples,
            &complexity_samples
        );

        // Generate insights from patterns
        self.aggregated.insights = self.generate_insights();

        // Calculate subtree coherence
        self.aggregated.subtree_coherence = self.calculate_coherence(&energy_samples);

        Ok(())
    }

    /// Traverse directory tree and aggregate data
    fn traverse_and_aggregate(
        &mut self,
        path: &Path,
        patterns: &mut HashMap<String, PatternStrength>,
        projects: &mut Vec<(PathBuf, Vec<String>)>,
        tech: &mut Vec<String>,
        energy: &mut Vec<(usize, f64)>,  // (depth, energy)
        complexity: &mut Vec<(usize, f64)>,  // (depth, complexity)
        current_depth: usize,
    ) -> Result<()> {
        // Skip hidden and build directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                return Ok(());
            }
        }

        self.total_descendants += 1;

        // Check if this directory has its own .m8 file
        let m8_path = path.join(".m8");
        if m8_path.exists() {
            // Load and aggregate existing consciousness
            if let Ok(existing) = Self::load_m8_file(&m8_path) {
                // Merge patterns
                for (pattern, _) in &existing.aggregated.unified_patterns {
                    patterns.entry(pattern.clone())
                        .or_insert(PatternStrength {
                            occurrences: 0,
                            strength: 0.0,
                            directories: Vec::new(),
                        })
                        .occurrences += 1;
                }

                // Use existing summaries to avoid re-scanning
                return Ok(());
            }
        }

        // Analyze this directory
        let local_patterns = Self::detect_patterns(path)?;
        for pattern in local_patterns {
            let entry = patterns.entry(pattern.clone())
                .or_insert(PatternStrength {
                    occurrences: 0,
                    strength: 0.0,
                    directories: Vec::new(),
                });
            entry.occurrences += 1;
            entry.directories.push(path.to_string_lossy().to_string());
        }

        // Detect project type
        let characteristics = Self::detect_project_characteristics(path)?;
        if !characteristics.is_empty() {
            projects.push((path.to_path_buf(), characteristics));
        }

        // Detect technology
        tech.extend(Self::detect_technology(path)?);

        // Sample emotional metrics
        let (e, c) = Self::measure_directory_emotion(path)?;
        energy.push((current_depth, e));
        complexity.push((current_depth, c));

        // Find important files
        self.scan_key_files(path)?;

        // Recurse into subdirectories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.traverse_and_aggregate(
                    &entry.path(),
                    patterns,
                    projects,
                    tech,
                    energy,
                    complexity,
                    current_depth + 1,
                )?;
            }
        }

        Ok(())
    }

    /// Generate unified summary of entire subtree
    fn generate_unified_summary(&mut self) -> Result<()> {
        let mut summary = String::new();

        // Start with local essence
        summary.push_str(&format!("📍 {}: {}\n",
            self.local.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root"),
            self.local.essence
        ));

        // Add project types if found
        if !self.aggregated.project_types.is_empty() {
            summary.push_str("\n🎯 Projects Found:\n");
            for project in &self.aggregated.project_types {
                summary.push_str(&format!("  • {} ({:.0}% confidence) at {}\n",
                    project.name,
                    project.confidence * 100.0,
                    project.root_path
                ));
            }
        }

        // Add technology stack
        if !self.aggregated.tech_stack.is_empty() {
            summary.push_str(&format!("\n💻 Technology Stack: {}\n",
                self.aggregated.tech_stack.join(", ")
            ));
        }

        // Add dominant patterns
        let mut patterns: Vec<_> = self.aggregated.unified_patterns.iter().collect();
        patterns.sort_by(|a, b| b.1.occurrences.cmp(&a.1.occurrences));

        if !patterns.is_empty() {
            summary.push_str("\n🔍 Key Patterns:\n");
            for (pattern, strength) in patterns.iter().take(5) {
                summary.push_str(&format!("  • {} (found in {} locations)\n",
                    pattern,
                    strength.occurrences
                ));
            }
        }

        // Add emotional summary
        summary.push_str(&format!("\n🌈 Collective Consciousness:\n"));
        summary.push_str(&format!("  • Energy: {:.0}%\n",
            self.aggregated.collective_emotion.average_energy * 100.0
        ));
        summary.push_str(&format!("  • Stability: {:.0}%\n",
            self.aggregated.collective_emotion.stability_score * 100.0
        ));
        summary.push_str(&format!("  • Coherence: {:.0}%\n",
            self.aggregated.subtree_coherence * 100.0
        ));

        // Add insights
        if !self.aggregated.insights.is_empty() {
            summary.push_str("\n💡 Insights:\n");
            for insight in &self.aggregated.insights {
                summary.push_str(&format!("  • {}\n", insight));
            }
        }

        // Add statistics
        summary.push_str(&format!("\n📊 Statistics:\n"));
        summary.push_str(&format!("  • Total directories: {}\n", self.total_descendants));
        summary.push_str(&format!("  • Depth: {} levels\n", self.depth));
        summary.push_str(&format!("  • Files in this directory: {}\n", self.local.file_count));

        self.unified_summary = summary;
        Ok(())
    }

    /// Detect patterns in a directory
    fn detect_patterns(path: &Path) -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        let entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        // Check for various patterns
        if entries.iter().any(|e| e.file_name().to_string_lossy().contains("test")) {
            patterns.push("testing".to_string());
        }

        if entries.iter().any(|e| e.file_name().to_string_lossy().contains("Cargo.toml")) {
            patterns.push("rust-project".to_string());
        }

        if entries.iter().any(|e| e.file_name().to_string_lossy().contains("package.json")) {
            patterns.push("node-project".to_string());
        }

        if entries.iter().any(|e| e.file_name().to_string_lossy().contains("requirements.txt")) {
            patterns.push("python-project".to_string());
        }

        if entries.iter().any(|e| e.file_name().to_string_lossy().contains(".git")) {
            patterns.push("git-repository".to_string());
        }

        if entries.iter().any(|e| e.file_name().to_string_lossy().contains("README")) {
            patterns.push("documented".to_string());
        }

        // Check for source code patterns
        let has_src = entries.iter().any(|e| {
            e.file_name().to_string_lossy() == "src" && e.path().is_dir()
        });
        if has_src {
            patterns.push("structured-project".to_string());
        }

        // Check for documentation
        if entries.iter().any(|e| {
            e.file_name().to_string_lossy() == "docs" && e.path().is_dir()
        }) {
            patterns.push("well-documented".to_string());
        }

        Ok(patterns)
    }

    /// Detect project characteristics
    fn detect_project_characteristics(path: &Path) -> Result<Vec<String>> {
        let mut characteristics = Vec::new();

        // Check for project indicators
        if path.join("Cargo.toml").exists() {
            characteristics.push("Rust project".to_string());
            if path.join("src/main.rs").exists() {
                characteristics.push("Binary crate".to_string());
            }
            if path.join("src/lib.rs").exists() {
                characteristics.push("Library crate".to_string());
            }
        }

        if path.join("package.json").exists() {
            characteristics.push("Node.js project".to_string());
            if path.join("pages").exists() || path.join("app").exists() {
                characteristics.push("Next.js app".to_string());
            }
        }

        if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
            characteristics.push("Python project".to_string());
        }

        if path.join("go.mod").exists() {
            characteristics.push("Go project".to_string());
        }

        if path.join(".github/workflows").exists() {
            characteristics.push("CI/CD configured".to_string());
        }

        if path.join("Dockerfile").exists() {
            characteristics.push("Containerized".to_string());
        }

        Ok(characteristics)
    }

    /// Detect technology stack
    fn detect_technology(path: &Path) -> Result<Vec<String>> {
        let mut tech = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Detect by file extensions and names
            match name.as_str() {
                "Cargo.toml" => tech.push("Rust".to_string()),
                "package.json" => tech.push("Node.js".to_string()),
                "requirements.txt" | "Pipfile" => tech.push("Python".to_string()),
                "go.mod" => tech.push("Go".to_string()),
                "pom.xml" => tech.push("Java/Maven".to_string()),
                "build.gradle" => tech.push("Java/Gradle".to_string()),
                "Gemfile" => tech.push("Ruby".to_string()),
                "composer.json" => tech.push("PHP".to_string()),
                "tsconfig.json" => tech.push("TypeScript".to_string()),
                "webpack.config.js" => tech.push("Webpack".to_string()),
                "docker-compose.yml" => tech.push("Docker Compose".to_string()),
                ".eslintrc.json" => tech.push("ESLint".to_string()),
                _ => {}
            }

            // Check by extension
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => tech.push("Rust".to_string()),
                    "js" | "jsx" => tech.push("JavaScript".to_string()),
                    "ts" | "tsx" => tech.push("TypeScript".to_string()),
                    "py" => tech.push("Python".to_string()),
                    "go" => tech.push("Go".to_string()),
                    "java" => tech.push("Java".to_string()),
                    "cpp" | "cc" | "cxx" => tech.push("C++".to_string()),
                    "c" | "h" => tech.push("C".to_string()),
                    "rb" => tech.push("Ruby".to_string()),
                    "php" => tech.push("PHP".to_string()),
                    "swift" => tech.push("Swift".to_string()),
                    "kt" => tech.push("Kotlin".to_string()),
                    "scala" => tech.push("Scala".to_string()),
                    "r" => tech.push("R".to_string()),
                    "jl" => tech.push("Julia".to_string()),
                    _ => {}
                }
            }
        }

        Ok(tech)
    }

    /// Measure directory emotion
    fn measure_directory_emotion(path: &Path) -> Result<(f64, f64)> {
        let entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        // Energy based on file count and activity
        let energy = (entries.len() as f64 / 20.0).min(1.0);

        // Complexity based on variety
        let extensions: std::collections::HashSet<_> = entries.iter()
            .filter_map(|e| e.path().extension()?.to_str().map(String::from))
            .collect();
        let complexity = (extensions.len() as f64 / 10.0).min(1.0);

        Ok((energy, complexity))
    }

    /// Scan for key files in directory
    fn scan_key_files(&mut self, path: &Path) -> Result<()> {
        let important_files = [
            ("README", "Project documentation", 0.9),
            ("LICENSE", "Legal information", 0.8),
            ("Cargo.toml", "Rust project manifest", 0.9),
            ("package.json", "Node.js project manifest", 0.9),
            ("main", "Entry point", 0.85),
            ("index", "Index file", 0.8),
            ("config", "Configuration", 0.75),
            (".env", "Environment variables", 0.7),
        ];

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            for (pattern, reason, importance) in &important_files {
                if name.contains(pattern) {
                    self.aggregated.key_files_global.push(KeyFile {
                        path: entry.path().to_string_lossy().to_string(),
                        importance: *importance,
                        reason: reason.to_string(),
                    });
                    break;
                }
            }
        }

        // Sort by importance and limit
        self.aggregated.key_files_global.sort_by(|a, b|
            b.importance.partial_cmp(&a.importance).unwrap()
        );
        self.aggregated.key_files_global.truncate(20);

        Ok(())
    }

    /// Identify distinct projects in the tree
    fn identify_projects(mut candidates: Vec<(PathBuf, Vec<String>)>) -> Vec<ProjectType> {
        let mut projects = Vec::new();

        // Sort by path depth (shallower = more likely to be root)
        candidates.sort_by_key(|(path, _)| path.components().count());

        for (path, characteristics) in candidates {
            // Calculate confidence based on characteristics
            let confidence = characteristics.len() as f64 / 5.0;

            // Determine project type
            let name = if characteristics.iter().any(|c| c.contains("Rust")) {
                "Rust Project"
            } else if characteristics.iter().any(|c| c.contains("Node")) {
                "Node.js Project"
            } else if characteristics.iter().any(|c| c.contains("Python")) {
                "Python Project"
            } else if characteristics.iter().any(|c| c.contains("Go")) {
                "Go Project"
            } else {
                "Generic Project"
            };

            // Check if this is a subproject of an existing project
            let is_subproject = projects.iter().any(|p: &ProjectType| {
                path.starts_with(&p.root_path)
            });

            if !is_subproject && confidence > 0.3 {
                projects.push(ProjectType {
                    name: name.to_string(),
                    confidence: confidence.min(1.0),
                    root_path: path.to_string_lossy().to_string(),
                    characteristics,
                });
            }
        }

        projects
    }

    /// Deduplicate technology stack
    fn deduplicate_tech(tech: Vec<String>) -> Vec<String> {
        let mut unique: std::collections::HashSet<String> = std::collections::HashSet::new();
        for t in tech {
            unique.insert(t);
        }
        let mut result: Vec<String> = unique.into_iter().collect();
        result.sort();
        result
    }

    /// Calculate collective emotion
    fn calculate_collective_emotion(
        energy: &[(usize, f64)],
        complexity: &[(usize, f64)]
    ) -> CollectiveEmotion {
        if energy.is_empty() {
            return CollectiveEmotion::default();
        }

        let avg_energy = energy.iter().map(|(_, e)| e).sum::<f64>() / energy.len() as f64;

        // Find peak energy zones
        let mut peak_zones = Vec::new();
        for (depth, e) in energy {
            if *e > 0.7 {
                peak_zones.push(format!("Level {}", depth));
            }
        }

        // Calculate stability (inverse of variance)
        let energy_variance = energy.iter()
            .map(|(_, e)| (e - avg_energy).powi(2))
            .sum::<f64>() / energy.len() as f64;
        let stability = 1.0 - energy_variance.min(1.0);

        // Calculate complexity gradient (how it changes with depth)
        let gradient = if complexity.len() > 1 {
            let deep_complexity = complexity.iter()
                .filter(|(d, _)| *d > 2)
                .map(|(_, c)| c)
                .sum::<f64>() / complexity.len().max(1) as f64;

            let shallow_complexity = complexity.iter()
                .filter(|(d, _)| *d <= 2)
                .map(|(_, c)| c)
                .sum::<f64>() / complexity.len().max(1) as f64;

            deep_complexity - shallow_complexity
        } else {
            0.0
        };

        CollectiveEmotion {
            average_energy: avg_energy,
            peak_energy_zones: peak_zones,
            stability_score: stability,
            complexity_gradient: gradient,
        }
    }

    /// Calculate subtree coherence
    fn calculate_coherence(&self, energy: &[(usize, f64)]) -> f64 {
        if energy.len() < 2 {
            return 1.0;
        }

        // Coherence is based on how similar energy levels are
        let avg = energy.iter().map(|(_, e)| e).sum::<f64>() / energy.len() as f64;
        let variance = energy.iter()
            .map(|(_, e)| (e - avg).powi(2))
            .sum::<f64>() / energy.len() as f64;

        // Low variance = high coherence
        1.0 - variance.min(1.0)
    }

    /// Generate insights from patterns
    fn generate_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // Pattern-based insights
        if self.aggregated.unified_patterns.contains_key("testing") {
            insights.push("✅ Testing infrastructure detected".to_string());
        }

        if self.aggregated.unified_patterns.contains_key("documented") {
            insights.push("📚 Well-documented codebase".to_string());
        }

        if self.aggregated.project_types.len() > 1 {
            insights.push(format!("🎯 Polyglot repository with {} distinct projects",
                self.aggregated.project_types.len()
            ));
        }

        // Emotional insights
        if self.aggregated.collective_emotion.average_energy > 0.7 {
            insights.push("🔥 High-energy development zone".to_string());
        }

        if self.aggregated.collective_emotion.stability_score > 0.8 {
            insights.push("🏔️ Stable and mature codebase".to_string());
        }

        if self.aggregated.collective_emotion.complexity_gradient > 0.3 {
            insights.push("📈 Complexity increases with depth".to_string());
        } else if self.aggregated.collective_emotion.complexity_gradient < -0.3 {
            insights.push("📉 Simpler at deeper levels".to_string());
        }

        if self.aggregated.subtree_coherence > 0.8 {
            insights.push("🎼 Highly coherent architecture".to_string());
        } else if self.aggregated.subtree_coherence < 0.4 {
            insights.push("🌪️ Diverse energy patterns - consider refactoring".to_string());
        }

        // Technology insights
        if self.aggregated.tech_stack.len() > 5 {
            insights.push("🛠️ Rich technology ecosystem".to_string());
        }

        insights
    }

    /// Load existing .m8 file
    fn load_m8_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let consciousness: Self = serde_json::from_str(&content)?;
        Ok(consciousness)
    }

    /// Save consciousness to .m8 file
    pub fn save(&self, path: &Path) -> Result<()> {
        let m8_path = path.join(".m8");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(m8_path, json)?;
        Ok(())
    }
}

impl DirectoryEssence {
    fn analyze(path: &Path) -> Result<Self> {
        let entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        let file_count = entries.iter()
            .filter(|e| e.path().is_file())
            .count();

        // Detect primary language
        let mut lang_counts: HashMap<String, usize> = HashMap::new();
        for entry in &entries {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => *lang_counts.entry("Rust".to_string()).or_insert(0) += 1,
                    "js" | "jsx" => *lang_counts.entry("JavaScript".to_string()).or_insert(0) += 1,
                    "ts" | "tsx" => *lang_counts.entry("TypeScript".to_string()).or_insert(0) += 1,
                    "py" => *lang_counts.entry("Python".to_string()).or_insert(0) += 1,
                    "go" => *lang_counts.entry("Go".to_string()).or_insert(0) += 1,
                    _ => {}
                }
            }
        }

        let primary_language = lang_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang);

        // Determine essence based on directory name and contents
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let essence = match dir_name {
            "source" | "src" => "Source code repository",
            "documents" | "docs" => "Documentation collection",
            "projects" => "Project portfolio",
            "user" | "users" => "User workspace",
            "home" => "Home directory",
            _ => "General directory",
        }.to_string();

        Ok(Self {
            path: path.to_path_buf(),
            frequency: 42.0, // Simplified for example
            essence,
            key_patterns: Vec::new(),
            file_count,
            primary_language,
            last_modified: 0,
        })
    }
}

impl Default for AggregatedConsciousness {
    fn default() -> Self {
        Self {
            unified_patterns: HashMap::new(),
            project_types: Vec::new(),
            collective_emotion: CollectiveEmotion::default(),
            insights: Vec::new(),
            tech_stack: Vec::new(),
            key_files_global: Vec::new(),
            subtree_coherence: 1.0,
        }
    }
}

impl Default for CollectiveEmotion {
    fn default() -> Self {
        Self {
            average_energy: 0.5,
            peak_energy_zones: Vec::new(),
            stability_score: 0.5,
            complexity_gradient: 0.0,
        }
    }
}

/// Example of how Smart Tree would use this
pub fn demonstrate_hierarchical_summary() {
    println!("\n🌊 Hierarchical .m8 Consciousness Demo\n");
    println!("=====================================\n");

    // Simulate a deep directory structure
    println!("Imagine this directory structure:");
    println!("~/source/                    (Your source folder)");
    println!("  └── projects/              (All your projects)");
    println!("      └── smart-tree/        (This project)");
    println!("          ├── src/           (Source code)");
    println!("          ├── docs/          (Documentation)");
    println!("          └── tests/         (Test files)\n");

    println!("Each level's .m8 file summarizes everything below:\n");

    println!("📁 ~/source/.m8 contains:");
    println!("  • Summary of ALL projects (10 Rust, 5 Python, 3 Node.js)");
    println!("  • Combined tech stack: Rust, Python, Node.js, Docker, K8s");
    println!("  • Total: 50,000 files across 18 projects");
    println!("  • Collective energy: 75% (very active development)");
    println!("  • Key insight: 'Primarily systems programming focus'\n");

    println!("📁 ~/source/projects/.m8 contains:");
    println!("  • Direct project summaries (less abstract than parent)");
    println!("  • Project health scores");
    println!("  • Cross-project patterns (shared libraries, common tools)");
    println!("  • Active vs dormant projects\n");

    println!("📁 ~/source/projects/smart-tree/.m8 contains:");
    println!("  • Specific Smart Tree architecture");
    println!("  • Local patterns (MCP server, tree visualization)");
    println!("  • Direct file references");
    println!("  • Detailed emotional signature\n");

    println!("The magic: Each level provides the RIGHT level of detail!");
    println!("  • Top level: Bird's eye view of everything");
    println!("  • Middle levels: Category summaries");
    println!("  • Deep levels: Specific implementation details\n");

    println!("🎯 Query Examples:");
    println!("  st --m8-summary ~/source");
    println!("    → 'You have 18 active projects, mostly Rust, high energy'\n");

    println!("  st --m8-summary ~/source/projects");
    println!("    → 'Smart Tree: Active, MEM8: Experimental, Others: Stable'\n");

    println!("  st --m8-summary ~/source/projects/smart-tree");
    println!("    → 'MCP server with 30+ tools, 147 Rust files, testing coverage 65%'\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hierarchical_consciousness() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let projects_dir = temp_dir.path().join("projects");
        let smart_tree_dir = projects_dir.join("smart-tree");
        let src_dir = smart_tree_dir.join("src");

        fs::create_dir_all(&src_dir).unwrap();

        // Create some files
        fs::write(smart_tree_dir.join("Cargo.toml"), "test").unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        // Create consciousness
        let consciousness = HierarchicalConsciousness::create_with_summary(&smart_tree_dir).unwrap();

        assert!(consciousness.total_descendants > 0);
        assert!(!consciousness.unified_summary.is_empty());
        assert!(consciousness.aggregated.tech_stack.contains(&"Rust".to_string()));
    }
}