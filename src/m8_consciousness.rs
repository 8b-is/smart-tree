// Directory Consciousness with .m8 Files - Distributed Wave Memory! 🌊
// Each directory becomes a conscious node in the project's neural network
// "Like neurons forming a brain!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<WavePattern>,

    /// Emotional signature of work done here
    #[serde(default)]
    pub emotion: EmotionalSignature,

    /// Important files and their wave signatures
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub key_files: HashMap<String, String>,

    /// Child directory frequencies (for hierarchy)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub children: HashMap<String, f64>,

    /// Rolled-up summaries of child directories (bottom-up hierarchy)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub child_summaries: HashMap<String, String>,

    /// Salient insights extracted from Markdown (.md) and manifest files at this level
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markdown_insights: Vec<String>,

    /// Built-in BitNet b1.58 (B+/B- ternary NO-FLOAT) multi-resolution summary & gist
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitnet_summary: Option<crate::bitnet_m8::BitNetDirectorySummary>,

    /// Extensive function/struct/enum consciousness (populated at leaf code folders like `src/.m8`;
    /// omitted at parent folders so parent `.m8` holds the rolled-up gist!)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<crate::bitnet_m8::FunctionConsciousness>,

    /// Quantum digest (compressed understanding)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quantum_digest: Vec<u8>,

    /// Coherence with parent (0.0 to 1.0)
    #[serde(default)]
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
    pub energy: f64,     // High activity vs calm
    pub complexity: f64, // Simple vs complex
    pub stability: f64,  // Stable vs changing
    pub focus: f64,      // Focused vs scattered
}

impl DirectoryConsciousness {
    /// Create new consciousness for a directory
    pub fn new(path: &Path) -> Result<Self> {
        let frequency = Self::calculate_frequency(path);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        Ok(Self {
            frequency,
            timestamp,
            essence: String::new(),
            patterns: Vec::new(),
            emotion: EmotionalSignature::default(),
            key_files: HashMap::new(),
            children: HashMap::new(),
            child_summaries: HashMap::new(),
            markdown_insights: Vec::new(),
            bitnet_summary: None,
            functions: Vec::new(),
            quantum_digest: Vec::new(),
            parent_coherence: 1.0,
        })
    }

    /// Calculate unique frequency from path
    fn calculate_frequency(path: &Path) -> f64 {
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = hasher.finalize();
        let first_8_bytes = &hash[0..8];
        let hash_num = u64::from_le_bytes(first_8_bytes.try_into().unwrap());

        // Generate frequency between 20 Hz and 200 Hz (human-perceivable range)
        20.0 + (hash_num % 18000) as f64 / 100.0
    }

    /// Analyze directory and generate consciousness (including Markdown insights and existing child .m8 summaries)
    pub fn analyze_directory(&mut self, path: &Path) -> Result<()> {
        // Extract Markdown (.md) insights at this level
        self.markdown_insights = Self::extract_markdown_insights(path);

        // Scan files for patterns
        self.detect_patterns(path)?;

        // Analyze emotional signature
        self.analyze_emotion(path)?;

        // Find key files
        self.identify_key_files(path)?;

        // Detect child directories and load any existing child .m8 summaries
        self.scan_children(path)?;

        // Determine essence from manifests, .md files, child .m8 summaries, and directory contents
        self.essence = self.determine_essence(path)?;

        // Run built-in BitNet b1.58 multi-resolution code & hierarchy summarizer
        let (leaf_fns, leaf_files) = crate::bitnet_m8::analyze_leaf_code_directory(path);
        let mut child_m8_map = std::collections::BTreeMap::new();
        for child_name in self.children.keys() {
            let cp = path.join(child_name);
            if cp.join(".m8").is_file() {
                if let Ok(cc) = Self::load(&cp) {
                    child_m8_map.insert(child_name.clone(), cc);
                }
            }
        }
        let bn_summary = crate::bitnet_m8::synthesize_bitnet_directory_summary(
            path,
            &leaf_fns,
            &leaf_files,
            &child_m8_map,
            &self.essence,
        );
        if bn_summary.zoom_level == "leaf-code" {
            self.functions = leaf_fns;
        } else {
            self.functions.clear();
        }
        self.bitnet_summary = Some(bn_summary);

        // Generate quantum digest
        self.generate_quantum_digest()?;

        Ok(())
    }

    /// Build a bottom-up hierarchical `.m8` pyramid starting from the deepest directories (`max_depth`)
    /// and rolling up child `.m8` summaries into each parent directory as we climb the ladder.
    pub fn build_bottom_up_pyramid(
        path: &Path,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<Self> {
        let mut child_dirs: Vec<PathBuf> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| !Self::should_skip_child_dir(name))
                    .unwrap_or(false)
            })
            .collect();
        child_dirs.sort();
        let total_child_dirs = child_dirs.len();
        let reuse_existing_child_m8 = total_child_dirs > 32;
        if total_child_dirs > 128 {
            let parent_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let step = (total_child_dirs / 128).max(1);
            let mut sampled = Vec::with_capacity(130);
            for (idx, p) in child_dirs.into_iter().enumerate() {
                let is_self_named = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == parent_name)
                    .unwrap_or(false);
                if is_self_named || idx % step == 0 || sampled.len() < 64 {
                    sampled.push(p);
                }
            }
            sampled.truncate(128);
            child_dirs = sampled;
        }

        let mut child_map = HashMap::new();
        let mut child_summaries = HashMap::new();
        let mut child_m8_objects = std::collections::BTreeMap::new();

        for child_path in child_dirs {
            let Some(child_name) = child_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let has_existing_m8 = child_path.join(".m8").is_file();
            if current_depth < max_depth && !(reuse_existing_child_m8 && has_existing_m8) {
                if let Ok(child_c) =
                    Self::build_bottom_up_pyramid(&child_path, current_depth + 1, max_depth)
                {
                    child_map.insert(child_name.clone(), child_c.frequency);
                    let child_gist = child_c
                        .bitnet_summary
                        .as_ref()
                        .map(|b| b.synthesized_gist.clone())
                        .unwrap_or_else(|| child_c.essence.clone());
                    if !child_gist.is_empty() {
                        child_summaries.insert(child_name.clone(), child_gist);
                    }
                    child_m8_objects.insert(child_name, child_c);
                }
            } else if has_existing_m8 {
                if let Ok(child_c) = Self::load(&child_path) {
                    child_map.insert(child_name.clone(), child_c.frequency);
                    let child_gist = child_c
                        .bitnet_summary
                        .as_ref()
                        .map(|b| b.synthesized_gist.clone())
                        .unwrap_or_else(|| child_c.essence.clone());
                    if !child_gist.is_empty() {
                        child_summaries.insert(child_name.clone(), child_gist);
                    }
                    child_m8_objects.insert(child_name, child_c);
                }
            } else {
                child_map.insert(child_name, Self::calculate_frequency(&child_path));
            }
        }

        let mut consciousness = Self::new(path)?;
        consciousness.markdown_insights = Self::extract_markdown_insights(path);
        consciousness.detect_patterns(path)?;
        consciousness.analyze_emotion(path)?;
        consciousness.identify_key_files(path)?;
        consciousness.children = child_map;
        consciousness.child_summaries = child_summaries;
        consciousness.essence = consciousness.determine_essence(path)?;

        let (leaf_fns, leaf_files) = crate::bitnet_m8::analyze_leaf_code_directory(path);
        let bn_summary = crate::bitnet_m8::synthesize_bitnet_directory_summary(
            path,
            &leaf_fns,
            &leaf_files,
            &child_m8_objects,
            &consciousness.essence,
        );
        if bn_summary.zoom_level == "leaf-code" {
            consciousness.functions = leaf_fns;
        } else {
            consciousness.functions.clear();
        }
        consciousness.bitnet_summary = Some(bn_summary);

        consciousness.generate_quantum_digest()?;
        let _ = consciousness.save(path);

        Ok(consciousness)
    }

    pub fn should_skip_child_dir(name: &str) -> bool {
        name.starts_with('.')
            || matches!(
                name,
                "target"
                    | "node_modules"
                    | "dist"
                    | "build"
                    | "__pycache__"
                    | "venv"
                    | "vendor"
                    | "backups"
                    | "nfs-cold-49"
                    | "borg-vault"
                    | "forgejo-massive"
            )
    }

    /// Extract salient headings and first-paragraph summaries from Markdown (`.md`) files in `path`.
    pub fn extract_markdown_insights(path: &Path) -> Vec<String> {
        let priority = [
            "README.md",
            "CLAUDE.md",
            "AGENTS.md",
            "ARCHITECTURE.md",
            "FAMILY_CONTEXT.md",
            "FAMILY_SEED.md",
            "DIFF_REPORT.md",
        ];
        let mut md_files: Vec<PathBuf> = Vec::new();
        for name in priority {
            let p = path.join(name);
            if p.is_file() {
                md_files.push(p);
            }
        }
        if let Ok(entries) = fs::read_dir(path) {
            let mut others: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                        && !md_files.contains(p)
                })
                .collect();
            others.sort();
            for other in others.into_iter().take(6) {
                md_files.push(other);
            }
        }

        let mut insights = Vec::new();
        for md_path in md_files.into_iter().take(8) {
            let Ok(content) = fs::read_to_string(&md_path) else {
                continue;
            };
            let fname = md_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("doc.md");
            let mut title = String::new();
            let mut body_line = String::new();
            let mut in_frontmatter = false;
            for (idx, raw) in content.lines().take(80).enumerate() {
                let line = raw.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
                if idx == 0 && line == "---" {
                    in_frontmatter = true;
                    continue;
                }
                if in_frontmatter {
                    if line == "---" {
                        in_frontmatter = false;
                    }
                    continue;
                }
                if line.is_empty()
                    || line.starts_with("![")
                    || line.starts_with("[![")
                    || line.starts_with("```")
                    || line.starts_with('<')
                {
                    continue;
                }
                if let Some(h) = line.strip_prefix('#') {
                    if title.is_empty() {
                        title = h.trim_start_matches('#').trim().to_string();
                    }
                    continue;
                }
                if body_line.is_empty() && line.len() >= 12 {
                    body_line = line.chars().take(160).collect();
                    break;
                }
            }
            let summary = match (!title.is_empty(), !body_line.is_empty()) {
                (true, true) => format!("{}: {} — {}", fname, title, body_line),
                (true, false) => format!("{}: {}", fname, title),
                (false, true) => format!("{}: {}", fname, body_line),
                (false, false) => continue,
            };
            insights.push(summary);
        }
        insights
    }

    /// Determine the essence/purpose of directory by combining manifest metadata,
    /// local `.md` insights, and rolled-up child `.m8` summaries.
    fn determine_essence(&self, path: &Path) -> Result<String> {
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut parts: Vec<String> = Vec::new();

        // 1. Check Cargo.toml / package.json / pyproject.toml for package description
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.is_file() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                let mut pkg_name = None;
                let mut desc = None;
                for line in content.lines().take(60) {
                    let trimmed = line.trim();
                    if pkg_name.is_none() && trimmed.starts_with("name") && trimmed.contains('=') {
                        pkg_name = trimmed.split_once('=').map(|(_, v)| {
                            v.trim().trim_matches('"').trim_matches('\'').to_string()
                        });
                    } else if desc.is_none()
                        && trimmed.starts_with("description")
                        && trimmed.contains('=')
                    {
                        desc = trimmed.split_once('=').map(|(_, v)| {
                            v.trim().trim_matches('"').trim_matches('\'').to_string()
                        });
                    }
                }
                match (pkg_name, desc) {
                    (Some(n), Some(d)) if !d.is_empty() => {
                        parts.push(format!("{} (Rust): {}", n, d))
                    }
                    (Some(n), _) => parts.push(format!("{} (Rust crate)", n)),
                    _ => {}
                }
            }
        }

        // 2. Include top Markdown insight at this level
        if let Some(first_md) = self.markdown_insights.first() {
            parts.push(first_md.clone());
        }

        // 3. Roll up child directory summaries (climbing the ladder!)
        if !self.child_summaries.is_empty() {
            let mut child_keys: Vec<&String> = self.child_summaries.keys().collect();
            child_keys.sort();
            let preview: Vec<String> = child_keys
                .iter()
                .take(6)
                .map(|k| {
                    let s = &self.child_summaries[*k];
                    let short: String = s.chars().take(60).collect();
                    format!("{}: {}", k, short)
                })
                .collect();
            if self.child_summaries.len() > 6 {
                parts.push(format!(
                    "Rolls up {} subfolders ({}; +{} more)",
                    self.child_summaries.len(),
                    preview.join("; "),
                    self.child_summaries.len() - 6
                ));
            } else {
                parts.push(format!(
                    "Rolls up {} subfolders ({})",
                    self.child_summaries.len(),
                    preview.join("; ")
                ));
            }
        }

        if !parts.is_empty() {
            return Ok(parts.join(" | "));
        }

        let fallback = match dir_name {
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

                if files
                    .iter()
                    .any(|f| f.path().extension() == Some("rs".as_ref()))
                {
                    "Rust code directory - systems programming space"
                } else if files
                    .iter()
                    .any(|f| f.path().extension() == Some("js".as_ref()))
                {
                    "JavaScript directory - web consciousness"
                } else if files
                    .iter()
                    .any(|f| f.path().extension() == Some("py".as_ref()))
                {
                    "Python directory - scripting and data space"
                } else if files
                    .iter()
                    .any(|f| f.path().extension() == Some("md".as_ref()))
                {
                    "Markdown directory - documentation space"
                } else {
                    "General purpose directory - mixed consciousness"
                }
            }
        };

        Ok(fallback.to_string())
    }

    /// Detect patterns in directory
    fn detect_patterns(&mut self, path: &Path) -> Result<()> {
        self.patterns.clear();

        let entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

        // Check for test patterns
        if entries.iter().any(|e| {
            e.file_name().to_string_lossy().contains("test")
                || e.file_name().to_string_lossy().contains("spec")
        }) {
            self.patterns.push(WavePattern {
                pattern_type: "testing".to_string(),
                strength: 0.8,
                description: "Test-driven development patterns detected".to_string(),
            });
        }

        // Check for async patterns
        if entries.iter().any(|e| {
            e.file_name().to_string_lossy().contains("async")
                || e.file_name().to_string_lossy().contains("future")
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
        let doc_count = entries
            .iter()
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
        let entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

        // Energy: based on file count and recent modifications
        let file_count = entries.len();
        self.emotion.energy = (file_count as f64 / 20.0).min(1.0);

        // Complexity: based on directory depth and variety
        let subdirs = entries.iter().filter(|e| e.path().is_dir()).count();
        let file_types: std::collections::HashSet<_> = entries
            .iter()
            .filter_map(|e| e.path().extension()?.to_str().map(String::from))
            .collect();

        self.emotion.complexity = ((subdirs + file_types.len()) as f64 / 10.0).min(1.0);

        // Stability: check for recent changes
        let recent_changes = entries
            .iter()
            .filter_map(|e| e.metadata().ok())
            .filter_map(|m| m.modified().ok())
            .filter(|t| {
                t.duration_since(UNIX_EPOCH).unwrap().as_secs()
                    > SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        - 86400
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
                    let wave_sig = format!(
                        "wave_{:x}",
                        file_name
                            .bytes()
                            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
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

    /// Scan for child directories and load any existing child `.m8` summaries
    fn scan_children(&mut self, path: &Path) -> Result<()> {
        self.children.clear();
        self.child_summaries.clear();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child_path = entry.path();
            if child_path.is_dir() {
                let child_name = entry.file_name().to_string_lossy().to_string();

                if !Self::should_skip_child_dir(&child_name) {
                    let child_freq = Self::calculate_frequency(&child_path);
                    self.children.insert(child_name.clone(), child_freq);
                    if child_path.join(".m8").is_file() {
                        if let Ok(child_c) = Self::load(&child_path) {
                            if !child_c.essence.is_empty() {
                                self.child_summaries.insert(child_name, child_c.essence);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate quantum digest
    fn generate_quantum_digest(&mut self) -> Result<()> {
        // Create a digest from all consciousness data
        let mut hasher = Sha256::new();
        hasher.update(self.essence.as_bytes());
        hasher.update(self.frequency.to_le_bytes());

        for pattern in &self.patterns {
            hasher.update(pattern.pattern_type.as_bytes());
            hasher.update(pattern.strength.to_le_bytes());
        }

        let hash = hasher.finalize();
        self.quantum_digest = hash[..16].to_vec();

        Ok(())
    }

    /// Save consciousness to .m8 file in tight MEM8 + Marqant + T8RZ binary format
    pub fn save(&self, path: &Path) -> Result<()> {
        let m8_path = path.join(".m8");
        let bytes = self.to_tight_m8_bytes()?;
        let mut file = File::create(m8_path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    /// Encode this consciousness into a tight MEM8 v1 container (`WAVE` + `MQNT` + `T8RZ`)
    pub fn to_tight_m8_bytes(&self) -> Result<Vec<u8>> {
        let wave_meta = self.build_wave_meta();
        let marqant_md = self.to_marqant_markdown();

        // Strip deterministically-derived redundant fields before compact serialization
        let mut compact_self = self.clone();
        if compact_self.quantum_digest.len() <= 16 {
            // Stored in binary inside the 64-byte WAVE header (`wave_meta.quantum_digest`)
            compact_self.quantum_digest.clear();
        }
        if compact_self.child_summaries.len() > 64 || compact_self.children.len() > 64 {
            let mut keys: Vec<String> = compact_self
                .children
                .keys()
                .chain(compact_self.child_summaries.keys())
                .cloned()
                .collect();
            keys.sort();
            keys.dedup();
            keys.sort_by_key(|k| {
                let s = compact_self
                    .child_summaries
                    .get(k)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let is_generic = s.is_empty() || s.starts_with("General purpose directory - mixed");
                (is_generic, k.clone())
            });
            keys.truncate(64);
            let keep: std::collections::HashSet<String> = keys.into_iter().collect();
            compact_self.children.retain(|k, _| keep.contains(k));
            compact_self.child_summaries.retain(|k, _| keep.contains(k));
        }
        if let Some(bn) = &mut compact_self.bitnet_summary {
            if bn.pooled_wave_b3 == bn.pooled_wave.trits_9() {
                bn.pooled_wave_b3.clear();
            }
            if bn.engine == "BitNet-b1.58-Ternary-NO-FLOAT (B+/B- Kleene 64-trit)" {
                bn.engine.clear();
            }
        }
        for f in &mut compact_self.functions {
            if f.wave_b3 == f.wave.trits_9() {
                f.wave_b3.clear();
            }
            if !f.docstring.is_empty() && f.bitnet_gist == format!("[{}] {}", f.domain, f.docstring)
            {
                f.bitnet_gist.clear();
            }
        }

        let compact_json = serde_json::to_vec(&compact_self)?;
        encode_tight_m8_container(&wave_meta, &marqant_md, &compact_json)
    }

    /// Build compact 64-byte binary wave & BitNet b1.58 metadata header
    pub fn build_wave_meta(&self) -> TightM8WaveMeta {
        let (b_pos, b_neg, balance_permille, box_norm_q8) = self
            .bitnet_summary
            .as_ref()
            .map(|b| {
                (
                    b.pooled_wave.b_pos,
                    b.pooled_wave.b_neg,
                    b.pooled_wave.balance_permille,
                    b.pooled_wave.box_norm_q8,
                )
            })
            .unwrap_or((0, 0, 1000, 0));
        let mut digest = [0u8; 16];
        let copy_len = self.quantum_digest.len().min(16);
        digest[..copy_len].copy_from_slice(&self.quantum_digest[..copy_len]);

        TightM8WaveMeta {
            frequency: self.frequency,
            timestamp: self.timestamp,
            energy_permille: (self.emotion.energy.clamp(0.0, 1.0) * 1000.0).round() as u16,
            complexity_permille: (self.emotion.complexity.clamp(0.0, 1.0) * 1000.0).round() as u16,
            stability_permille: (self.emotion.stability.clamp(0.0, 1.0) * 1000.0).round() as u16,
            focus_permille: (self.emotion.focus.clamp(0.0, 1.0) * 1000.0).round() as u16,
            b_pos,
            b_neg,
            balance_permille,
            box_norm_q8,
            quantum_digest: digest,
        }
    }

    /// Synthesize a concise Markdown soul for the `MQNT` (Marqant) section
    pub fn to_marqant_markdown(&self) -> String {
        let mut md = String::new();
        let head_essence: String = self
            .essence
            .split(" | Rolls up ")
            .next()
            .unwrap_or(&self.essence)
            .chars()
            .take(180)
            .collect();
        md.push_str(&format!("# {:.2}Hz — {}\n", self.frequency, head_essence));
        if let Some(bn) = &self.bitnet_summary {
            md.push_str(&format!(
                "## BitNet {}\n{}\n",
                bn.zoom_level, bn.synthesized_gist
            ));
            for sym in bn.flagship_symbols.iter().take(4) {
                md.push_str(&format!("- {}\n", sym));
            }
        }
        md
    }

    /// Load consciousness from .m8 file (auto-detecting tight MEM8, Marqant, Zlib, or legacy JSON)
    pub fn load(path: &Path) -> Result<Self> {
        let m8_path = if path.is_file() {
            path.to_path_buf()
        } else {
            path.join(".m8")
        };

        if !m8_path.exists() {
            return Self::new(path);
        }

        let raw = fs::read(&m8_path)?;
        Self::from_m8_bytes(&raw)
    }

    /// Decode consciousness from raw `.m8` bytes (MEM8 binary, Marqant, Zlib, or JSON)
    pub fn from_m8_bytes(raw: &[u8]) -> Result<Self> {
        let decoded = decode_m8_bytes(raw)?;
        if let Ok(mut consciousness) = serde_json::from_slice::<Self>(&decoded) {
            if consciousness.quantum_digest.is_empty() {
                if let Some(wave) = extract_m8_wave_header(raw) {
                    if wave.quantum_digest != [0u8; 16] {
                        consciousness.quantum_digest = wave.quantum_digest.to_vec();
                    }
                }
            }
            consciousness.rehydrate_derived_fields();
            return Ok(consciousness);
        }

        // Fallback if the file was a pure Marqant markdown document without JSON state
        let text = String::from_utf8_lossy(&decoded);
        let wave_meta = extract_m8_wave_header(raw);
        let mut consciousness = Self {
            frequency: wave_meta.as_ref().map(|w| w.frequency).unwrap_or(42.73),
            timestamp: wave_meta.as_ref().map(|w| w.timestamp).unwrap_or(0),
            essence: text
                .lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.trim_start_matches('#').trim().to_string())
                .unwrap_or_default(),
            patterns: Vec::new(),
            emotion: EmotionalSignature::default(),
            key_files: HashMap::new(),
            children: HashMap::new(),
            child_summaries: HashMap::new(),
            markdown_insights: text
                .lines()
                .skip(1)
                .filter(|l| !l.trim().is_empty())
                .take(8)
                .map(|l| l.trim().to_string())
                .collect(),
            bitnet_summary: None,
            functions: Vec::new(),
            quantum_digest: wave_meta
                .as_ref()
                .map(|w| w.quantum_digest.to_vec())
                .unwrap_or_default(),
            parent_coherence: 1.0,
        };
        consciousness.rehydrate_derived_fields();
        Ok(consciousness)
    }

    fn rehydrate_derived_fields(&mut self) {
        if let Some(bn) = &mut self.bitnet_summary {
            if bn.pooled_wave_b3.is_empty() {
                bn.pooled_wave_b3 = bn.pooled_wave.trits_9();
            }
            if bn.engine.is_empty() {
                bn.engine = "BitNet-b1.58-Ternary-NO-FLOAT (B+/B- Kleene 64-trit)".to_string();
            }
        }
        for f in &mut self.functions {
            if f.wave_b3.is_empty() {
                f.wave_b3 = f.wave.trits_9();
            }
            if f.bitnet_gist.is_empty() {
                if !f.docstring.is_empty() {
                    f.bitnet_gist = format!("[{}] {}", f.domain, f.docstring);
                } else {
                    f.bitnet_gist =
                        format!("[{}] {} `{}` in `{}`", f.domain, f.kind, f.name, f.file);
                }
            }
        }
    }

    /// Check coherence with parent consciousness
    pub fn calculate_coherence(&self, parent: &Self) -> f64 {
        // Compare frequencies (closer = more coherent)
        let freq_diff = (self.frequency - parent.frequency).abs();
        let freq_coherence = 1.0 - (freq_diff / 200.0).min(1.0);

        // Compare emotional signatures
        let emotion_coherence = 1.0
            - ((self.emotion.energy - parent.emotion.energy).abs()
                + (self.emotion.complexity - parent.emotion.complexity).abs()
                + (self.emotion.stability - parent.emotion.stability).abs()
                + (self.emotion.focus - parent.emotion.focus).abs())
                / 4.0;

        // Average coherence
        (freq_coherence + emotion_coherence) / 2.0
    }

    /// Render consciousness as visual wave
    pub fn render_wave(&self) -> String {
        let wave_chars = ['─', '╌', '┈', '╍', '━', '┅', '┉', '╸'];
        let amplitude = (self.emotion.energy * 7.0) as usize;
        let wave_char = wave_chars[amplitude.min(7)];

        let mut output = String::new();
        output.push_str(&format!("╭{:─^38}╮\n", ""));
        output.push_str(&format!(
            "│ Frequency: {:<10.2} Hz {:>10} │\n",
            self.frequency,
            if self.frequency > 100.0 {
                "🔥"
            } else if self.frequency > 50.0 {
                "⚡"
            } else {
                "🌊"
            }
        ));

        // Render wave based on frequency and emotion
        let wave_line = wave_char.to_string().repeat(30);
        output.push_str(&format!("│ {} │\n", wave_line));

        output.push_str(&format!(
            "│ Children: {:<3} Energy: {:<5.1}      │\n",
            self.children.len(),
            self.emotion.energy
        ));

        output.push_str(&format!(
            "│ Patterns: {:<25} │\n",
            self.patterns
                .iter()
                .take(3)
                .map(|p| &p.pattern_type[..p.pattern_type.len().min(8)])
                .collect::<Vec<_>>()
                .join(", ")
        ));

        output.push_str(&format!("╰{:─^38}╯", ""));

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
            if DirectoryConsciousness::should_skip_child_dir(name) {
                return Ok(());
            }
        }

        // Bottom-up: recurse into subdirectories FIRST so child .m8 files exist before parent rollup
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.initialize_recursive(&entry.path(), depth + 1)?;
            }
        }

        // Create consciousness for this directory (now rolling up child .m8 summaries!)
        let mut consciousness = DirectoryConsciousness::new(path)?;
        consciousness.analyze_directory(path)?;

        consciousness.save(path)?;

        // Print progress
        let indent = "  ".repeat(depth);
        println!(
            "{}✓ Created {}/.m8 ({:.1} Hz) - {}",
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

        Ok(())
    }

    /// Find directories with similar consciousness
    pub fn find_resonant(&self, frequency: f64, tolerance: f64) -> Vec<PathBuf> {
        self.cache
            .iter()
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

/// 64-byte binary `WAVE` section stored at offset 64 of every tight `MEM8` v1 container.
/// Enables O(1) header reads of directory frequency, timestamp, emotional signature,
/// BitNet b1.58 64-trit `(B+, B-)` rails, and quantum digest without decompressing payload sections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TightM8WaveMeta {
    pub frequency: f64,
    pub timestamp: u64,
    pub energy_permille: u16,
    pub complexity_permille: u16,
    pub stability_permille: u16,
    pub focus_permille: u16,
    pub b_pos: u64,
    pub b_neg: u64,
    pub balance_permille: u32,
    pub box_norm_q8: u32,
    pub quantum_digest: [u8; 16],
}

impl Default for TightM8WaveMeta {
    fn default() -> Self {
        Self {
            frequency: 42.73,
            timestamp: 0,
            energy_permille: 500,
            complexity_permille: 500,
            stability_permille: 500,
            focus_permille: 500,
            b_pos: 0,
            b_neg: 0,
            balance_permille: 1000,
            box_norm_q8: 0,
            quantum_digest: [0u8; 16],
        }
    }
}

impl TightM8WaveMeta {
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..8].copy_from_slice(&self.frequency.to_le_bytes());
        out[8..16].copy_from_slice(&self.timestamp.to_le_bytes());
        out[16..18].copy_from_slice(&self.energy_permille.to_le_bytes());
        out[18..20].copy_from_slice(&self.complexity_permille.to_le_bytes());
        out[20..22].copy_from_slice(&self.stability_permille.to_le_bytes());
        out[22..24].copy_from_slice(&self.focus_permille.to_le_bytes());
        out[24..32].copy_from_slice(&self.b_pos.to_le_bytes());
        out[32..40].copy_from_slice(&self.b_neg.to_le_bytes());
        out[40..44].copy_from_slice(&self.balance_permille.to_le_bytes());
        out[44..48].copy_from_slice(&self.box_norm_q8.to_le_bytes());
        out[48..64].copy_from_slice(&self.quantum_digest);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 64 {
            return None;
        }
        let mut quantum_digest = [0u8; 16];
        quantum_digest.copy_from_slice(&bytes[48..64]);
        Some(Self {
            frequency: f64::from_le_bytes(bytes[0..8].try_into().ok()?),
            timestamp: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
            energy_permille: u16::from_le_bytes(bytes[16..18].try_into().ok()?),
            complexity_permille: u16::from_le_bytes(bytes[18..20].try_into().ok()?),
            stability_permille: u16::from_le_bytes(bytes[20..22].try_into().ok()?),
            focus_permille: u16::from_le_bytes(bytes[22..24].try_into().ok()?),
            b_pos: u64::from_le_bytes(bytes[24..32].try_into().ok()?),
            b_neg: u64::from_le_bytes(bytes[32..40].try_into().ok()?),
            balance_permille: u32::from_le_bytes(bytes[40..44].try_into().ok()?),
            box_norm_q8: u32::from_le_bytes(bytes[44..48].try_into().ok()?),
            quantum_digest,
        })
    }
}

/// Pack `WAVE` (64B binary wave + BitNet b1.58 rails), `MQNT` (Marqant quantum-compressed markdown soul),
/// and `T8RZ` (tokenized + Zlib-level-9 lossless state) into a CRC32-verified `MEM8` v1 binary container.
pub fn encode_tight_m8_container(
    wave: &TightM8WaveMeta,
    marqant_markdown: &str,
    raw_payload: &[u8],
) -> Result<Vec<u8>> {
    use crate::formatters::marqant::MarqantFormatter;
    use flate2::{write::ZlibEncoder, Compression};

    let wave_bytes = wave.to_bytes();

    // Include dedicated MQNT section when payload is rich (>= 1536B); for tiny leaf nodes,
    // omit duplicate MQNT section so tiny .m8 files stay ~300 bytes while synthesizing Marqant on demand!
    let include_mqnt = raw_payload.len() >= 1536 && !marqant_markdown.is_empty();
    let mqnt_str = if include_mqnt {
        let mq_plain = MarqantFormatter::compress_markdown(marqant_markdown)
            .unwrap_or_else(|_| marqant_markdown.to_string());
        let mq_zlib = MarqantFormatter::compress_markdown_with_flags(marqant_markdown, Some("-z"))
            .unwrap_or_else(|_| mq_plain.clone());
        if mq_zlib.len() < mq_plain.len() {
            mq_zlib
        } else {
            mq_plain
        }
    } else {
        String::new()
    };
    let mqnt_bytes = mqnt_str.as_bytes();

    // Direct Zlib-level-9 (LZ77 + Huffman) vs Tokenized+Zlib: pick whichever is strictly smaller!
    let mut direct_enc = ZlibEncoder::new(Vec::new(), Compression::best());
    direct_enc.write_all(raw_payload)?;
    let direct_zlib = direct_enc.finish()?;

    let (data_sec_id, data_sec_flags, data_sec_bytes): (&[u8; 4], u8, Vec<u8>) =
        if raw_payload.len() < 8192 {
            (b"ZLIB", 0x01, direct_zlib)
        } else {
            let tokenized = crate::mem8::token_codec::encode(raw_payload);
            let mut tok_enc = ZlibEncoder::new(Vec::new(), Compression::best());
            tok_enc.write_all(&tokenized)?;
            let tok_zlib = tok_enc.finish()?;
            if direct_zlib.len() <= 4 + tok_zlib.len() {
                (b"ZLIB", 0x01, direct_zlib)
            } else {
                let mut t8rz = Vec::with_capacity(4 + tok_zlib.len());
                t8rz.extend_from_slice(&(raw_payload.len() as u32).to_le_bytes());
                t8rz.extend_from_slice(&tok_zlib);
                (b"T8RZ", 0x03, t8rz)
            }
        };

    let section_count: u16 = if include_mqnt { 3 } else { 2 };
    let header_and_table_size: u32 = 16 + (section_count as u32 * 16);
    let wave_offset = header_and_table_size;
    let wave_size = wave_bytes.len() as u32;
    let mqnt_offset = wave_offset + wave_size;
    let mqnt_size = if include_mqnt {
        mqnt_bytes.len() as u32
    } else {
        0
    };
    let data_offset = mqnt_offset + mqnt_size;
    let data_size = data_sec_bytes.len() as u32;
    let total_size = data_offset + data_size;

    let mut out = Vec::with_capacity(total_size as usize);

    // 1. 16-byte MEM8 v1 header (CRC32 at 12..16 filled after assembling sections)
    out.extend_from_slice(b"MEM8");
    out.push(1u8); // version 1
    let mqnt_flag = if include_mqnt { 0x02u8 } else { 0u8 };
    out.push(0x80u8 | 0x04u8 | mqnt_flag | data_sec_flags);
    out.extend_from_slice(&section_count.to_le_bytes());
    out.extend_from_slice(&total_size.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // placeholder for CRC32

    // 2. Section Table
    let write_section = |buf: &mut Vec<u8>, id: &[u8; 4], offset: u32, size: u32, flags: u8| {
        buf.extend_from_slice(id);
        buf.extend_from_slice(&offset.to_le_bytes());
        buf.extend_from_slice(&size.to_le_bytes());
        buf.push(flags);
        buf.extend_from_slice(&[0u8; 3]);
    };
    write_section(&mut out, b"WAVE", wave_offset, wave_size, 0x04);
    if include_mqnt {
        write_section(&mut out, b"MQNT", mqnt_offset, mqnt_size, 0x02);
    }
    write_section(
        &mut out,
        data_sec_id,
        data_offset,
        data_size,
        data_sec_flags,
    );

    // 3. Contiguous section payloads
    out.extend_from_slice(&wave_bytes);
    if include_mqnt {
        out.extend_from_slice(mqnt_bytes);
    }
    out.extend_from_slice(&data_sec_bytes);

    // 4. Compute whole-file IEEE CRC32 with bytes 12..16 zeroed (matches `m8 validate`)
    let crc = crc32fast::hash(&out);
    out[12..16].copy_from_slice(&crc.to_le_bytes());

    Ok(out)
}

/// Encode any serializable value or raw JSON/text slice into a tight `MEM8` v1 container.
pub fn encode_json_bytes_as_tight_m8(
    raw_bytes: &[u8],
    frequency: f64,
    summary_markdown: &str,
) -> Result<Vec<u8>> {
    // Compact JSON if input is pretty-printed JSON so we don't compress indentation whitespace
    let compact_payload = if let Ok(val) = serde_json::from_slice::<serde_json::Value>(raw_bytes) {
        serde_json::to_vec(&val).unwrap_or_else(|_| raw_bytes.to_vec())
    } else {
        raw_bytes.to_vec()
    };

    let mut hasher = Sha256::new();
    hasher.update(&compact_payload);
    let digest_full = hasher.finalize();
    let mut quantum_digest = [0u8; 16];
    quantum_digest.copy_from_slice(&digest_full[..16]);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let wave = TightM8WaveMeta {
        frequency,
        timestamp,
        quantum_digest,
        ..TightM8WaveMeta::default()
    };
    encode_tight_m8_container(&wave, summary_markdown, &compact_payload)
}

/// Extract the 64-byte `WAVE` metadata header from a tight `MEM8` container in O(1) without decompressing.
pub fn extract_m8_wave_header(data: &[u8]) -> Option<TightM8WaveMeta> {
    if data.len() < 16 || &data[0..4] != b"MEM8" || data[4] != 1 {
        return None;
    }
    let section_count = u16::from_le_bytes([data[6], data[7]]) as usize;
    let total_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let table_end = 16usize.checked_add(section_count.checked_mul(16)?)?;
    if total_size != data.len()
        || section_count == 0
        || section_count > 64
        || table_end > data.len()
    {
        return None;
    }
    for i in 0..section_count {
        let base = 16 + i * 16;
        let id = &data[base..base + 4];
        let offset = u32::from_le_bytes([
            data[base + 4],
            data[base + 5],
            data[base + 6],
            data[base + 7],
        ]) as usize;
        let size = u32::from_le_bytes([
            data[base + 8],
            data[base + 9],
            data[base + 10],
            data[base + 11],
        ]) as usize;
        let end = offset.checked_add(size)?;
        if id == b"WAVE" && end <= data.len() && size >= 64 {
            return TightM8WaveMeta::from_bytes(&data[offset..end]);
        }
    }
    None
}

/// Extract the raw or decompressed `MQNT` (Marqant) section from a `.m8` buffer if present.
pub fn extract_m8_marqant_section(data: &[u8], decompress: bool) -> Option<String> {
    use crate::formatters::marqant::MarqantFormatter;

    if data.starts_with(b"MARQANT") {
        let raw_str = String::from_utf8_lossy(data).to_string();
        if decompress {
            return MarqantFormatter::decompress_marqant(&raw_str)
                .ok()
                .or(Some(raw_str));
        }
        return Some(raw_str);
    }
    if data.len() < 16 || &data[0..4] != b"MEM8" || data[4] != 1 {
        return None;
    }
    let section_count = u16::from_le_bytes([data[6], data[7]]) as usize;
    let total_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let table_end = 16usize.checked_add(section_count.checked_mul(16)?)?;
    if total_size != data.len()
        || section_count == 0
        || section_count > 64
        || table_end > data.len()
    {
        return None;
    }
    for i in 0..section_count {
        let base = 16 + i * 16;
        let id = &data[base..base + 4];
        let offset = u32::from_le_bytes([
            data[base + 4],
            data[base + 5],
            data[base + 6],
            data[base + 7],
        ]) as usize;
        let size = u32::from_le_bytes([
            data[base + 8],
            data[base + 9],
            data[base + 10],
            data[base + 11],
        ]) as usize;
        let end = offset.checked_add(size)?;
        if id == b"MQNT" && end <= data.len() {
            let mq_str = String::from_utf8_lossy(&data[offset..end]).to_string();
            if decompress && mq_str.starts_with("MARQANT") {
                return MarqantFormatter::decompress_marqant(&mq_str)
                    .ok()
                    .or(Some(mq_str));
            }
            return Some(mq_str);
        }
    }
    // For compact 2-section MEM8 containers (`WAVE` + `ZLIB`), synthesize Marqant on demand
    if let Ok(c) = DirectoryConsciousness::from_m8_bytes(data) {
        let md = c.to_marqant_markdown();
        if decompress {
            return Some(md);
        }
        return MarqantFormatter::compress_markdown(&md).ok().or(Some(md));
    }
    None
}

/// Universal `.m8` payload decoder supporting:
/// 1. Tight `MEM8` v1 section containers (`WAVE` + `MQNT` + `T8RZ`/`ZLIB`)
/// 2. Legacy `MEM8` 4KB-block `M8BinaryFile` files
/// 3. `MARQANT` quantum-compressed `.m8` / `.mq` files
/// 4. Raw Zlib `.m8z` streams (`0x78 0x9c` / `0x78 0xda` / `0x78 0x01`)
/// 5. Legacy uncompressed JSON/UTF-8 `.m8` files
pub fn decode_m8_bytes(data: &[u8]) -> Result<Vec<u8>> {
    use crate::formatters::marqant::MarqantFormatter;
    use flate2::read::ZlibDecoder;

    if data.is_empty() {
        anyhow::bail!("Empty .m8 file");
    }

    // 1. MEM8 binary magic
    if data.starts_with(b"MEM8") {
        // Check if this is a MEM8 v1 section-table container
        if data.len() >= 16 && data[4] == 1 {
            let section_count = u16::from_le_bytes([data[6], data[7]]) as usize;
            let total_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            let expected_crc = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
            let table_end = 16usize.saturating_add(section_count.saturating_mul(16));

            if total_size == data.len()
                && section_count > 0
                && section_count <= 64
                && table_end <= data.len()
            {
                if expected_crc != 0 {
                    let mut hasher = crc32fast::Hasher::new();
                    hasher.update(&data[..12]);
                    hasher.update(&[0u8; 4]);
                    hasher.update(&data[16..]);
                    anyhow::ensure!(
                        hasher.finalize() == expected_crc,
                        "MEM8 container CRC32 mismatch"
                    );
                }

                let mut mqnt_fallback: Option<&[u8]> = None;
                for i in 0..section_count {
                    let base = 16 + i * 16;
                    let id = &data[base..base + 4];
                    let offset = u32::from_le_bytes([
                        data[base + 4],
                        data[base + 5],
                        data[base + 6],
                        data[base + 7],
                    ]) as usize;
                    let size = u32::from_le_bytes([
                        data[base + 8],
                        data[base + 9],
                        data[base + 10],
                        data[base + 11],
                    ]) as usize;
                    let end = offset
                        .checked_add(size)
                        .ok_or_else(|| anyhow::anyhow!("MEM8 section overflow"))?;
                    anyhow::ensure!(end <= data.len(), "MEM8 section extends past EOF");
                    let sec = &data[offset..end];

                    if id == b"T8RZ" {
                        anyhow::ensure!(sec.len() >= 4, "Truncated T8RZ section");
                        let raw_len = u32::from_le_bytes(sec[0..4].try_into()?) as usize;
                        let mut decoder = ZlibDecoder::new(&sec[4..]);
                        let mut tokenized = Vec::with_capacity(raw_len);
                        decoder.read_to_end(&mut tokenized)?;
                        return crate::mem8::token_codec::decode(&tokenized, raw_len);
                    } else if id == b"ZLIB" {
                        let mut decoder = ZlibDecoder::new(sec);
                        let mut out = Vec::new();
                        decoder.read_to_end(&mut out)?;
                        return Ok(out);
                    } else if id == b"JSON" || id == b"DATA" {
                        return Ok(sec.to_vec());
                    } else if id == b"MQNT" {
                        mqnt_fallback = Some(sec);
                    }
                }

                if let Some(mq_bytes) = mqnt_fallback {
                    return decode_marqant_payload(mq_bytes);
                }
            }
        }

        // Legacy 30-byte M8Header + 4096-byte M8Block file (`src/mem8_binary.rs`)
        if data.len() >= 30 {
            let block_size = 4096usize;
            let mut offset = 30usize;
            let mut blocks_json: Vec<serde_json::Value> = Vec::new();
            let mut last_raw: Option<Vec<u8>> = None;
            while offset + block_size <= data.len() {
                let block = &data[offset..offset + block_size];
                let content_len =
                    u32::from_le_bytes(block[68..72].try_into().unwrap_or([0; 4])) as usize;
                let clamped = content_len.min(block_size - 72);
                let payload = &block[72..72 + clamped];
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(payload) {
                    blocks_json.push(val);
                }
                last_raw = Some(payload.to_vec());
                offset += block_size;
            }
            if blocks_json.len() == 1 {
                return Ok(serde_json::to_vec(&blocks_json[0])?);
            } else if !blocks_json.is_empty() {
                return Ok(serde_json::to_vec(&serde_json::json!({
                    "format": "m8j",
                    "version": 1,
                    "contexts": blocks_json
                }))?);
            } else if let Some(raw) = last_raw {
                return Ok(raw);
            }
        }

        anyhow::bail!("Unsupported MEM8 binary layout");
    }

    // 2. Pure Marqant (.mq / Marqant-encoded .m8)
    if data.starts_with(b"MARQANT") {
        return decode_marqant_payload(data);
    }

    // 3. Zlib-compressed stream (.m8z)
    if data.len() >= 2
        && data[0] == 0x78
        && (data[1] == 0x9c || data[1] == 0xda || data[1] == 0x01 || data[1] == 0x5e)
    {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
            return decode_m8_bytes(&decompressed);
        }
    }

    // 4. Legacy raw JSON or plain text
    let _ = MarqantFormatter::decompress_marqant;
    Ok(data.to_vec())
}

fn decode_marqant_payload(mq_bytes: &[u8]) -> Result<Vec<u8>> {
    use crate::formatters::marqant::MarqantFormatter;

    let mq_str = String::from_utf8_lossy(mq_bytes);
    let decompressed = if mq_str.starts_with("MARQANT") {
        MarqantFormatter::decompress_marqant(&mq_str)?
    } else {
        mq_str.to_string()
    };

    let trimmed = decompressed.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Ok(trimmed.as_bytes().to_vec());
    }
    if let Some(fence_start) = trimmed.find("```json") {
        let after = &trimmed[fence_start + 7..];
        if let Some(fence_end) = after.find("```") {
            return Ok(after[..fence_end].trim().as_bytes().to_vec());
        }
    }
    Ok(decompressed.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_frequency_generation() {
        let freq1 = DirectoryConsciousness::calculate_frequency(Path::new("/test/path1"));
        let freq2 = DirectoryConsciousness::calculate_frequency(Path::new("/test/path2"));

        assert!((20.0..=200.0).contains(&freq1));
        assert!((20.0..=200.0).contains(&freq2));
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

        // Verify on-disk .m8 starts with MEM8 magic and embeds WAVE + MQNT + T8RZ sections
        let raw_bytes = fs::read(temp_dir.path().join(".m8")).unwrap();
        assert_eq!(&raw_bytes[0..4], b"MEM8");
        let wave_hdr = extract_m8_wave_header(&raw_bytes).expect("should extract WAVE header");
        assert!((wave_hdr.frequency - consciousness.frequency).abs() < 1e-6);
        let mqnt = extract_m8_marqant_section(&raw_bytes, true).expect("should extract MQNT");
        assert!(mqnt.contains("Test directory"));

        let loaded = DirectoryConsciousness::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.essence, "Test directory");
        assert_eq!(loaded.frequency, consciousness.frequency);
    }

    #[test]
    fn test_legacy_json_and_marqant_m8_backward_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let mut consciousness = DirectoryConsciousness::new(temp_dir.path()).unwrap();
        consciousness.essence = "Legacy JSON essence".to_string();

        // 1. Legacy uncompressed JSON .m8
        let legacy_json = serde_json::to_string_pretty(&consciousness).unwrap();
        fs::write(temp_dir.path().join(".m8"), &legacy_json).unwrap();
        let loaded_json = DirectoryConsciousness::load(temp_dir.path()).unwrap();
        assert_eq!(loaded_json.essence, "Legacy JSON essence");

        // 2. Pure Marqant .m8
        let mq = crate::formatters::marqant::MarqantFormatter::compress_markdown(
            "# Marqant Consciousness\n- Insight one\n",
        )
        .unwrap();
        fs::write(temp_dir.path().join(".m8"), mq.as_bytes()).unwrap();
        let loaded_mq = DirectoryConsciousness::load(temp_dir.path()).unwrap();
        assert!(loaded_mq.essence.contains("Marqant Consciousness"));
    }

    #[test]
    fn test_bottom_up_pyramid_rolls_up_markdown_and_children() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let child_a = root.join("alpha");
        let child_b = root.join("beta");
        fs::create_dir_all(&child_a).unwrap();
        fs::create_dir_all(&child_b).unwrap();

        fs::write(
            root.join("README.md"),
            "# Galaxy Root\nTop-level hub for the fleet.\n",
        )
        .unwrap();
        fs::write(
            child_a.join("README.md"),
            "# Alpha Engine\nZero-float integer DSP engine.\n",
        )
        .unwrap();
        fs::write(
            child_b.join("Cargo.toml"),
            "[package]\nname = \"beta-core\"\ndescription = \"Wave memory kernel\"\n",
        )
        .unwrap();

        let root_c = DirectoryConsciousness::build_bottom_up_pyramid(root, 0, 2).unwrap();
        assert!(root.join(".m8").is_file());
        assert!(child_a.join(".m8").is_file());
        assert!(child_b.join(".m8").is_file());
        assert!(root_c.essence.contains("Galaxy Root"));
        assert!(root_c.essence.contains("Rolls up 2 subfolders"));
        assert!(root_c.child_summaries["alpha"].contains("Alpha Engine"));
        assert!(root_c.child_summaries["beta"].contains("beta-core"));
    }

    #[test]
    fn test_standardgalactic_tight_m8_and_memory_benchmarks() {
        let sg_root = Path::new("/home/hue/source/standardgalactic");
        if !sg_root.is_dir() {
            return;
        }

        // Collect up to 1,000 .m8 files from standardgalactic
        let mut m8_paths = Vec::new();
        for fixed in [
            sg_root.join(".m8"),
            sg_root.join("standardgalactic/.m8"),
            sg_root.join("standardgalactic/Music/.m8"),
            sg_root.join("standardgalactic/art/.m8"),
            sg_root.join("old-archive.scpr.org/content/.m8"),
        ] {
            if fixed.is_file() {
                m8_paths.push(fixed);
            }
        }
        if let Ok(entries) = fs::read_dir(sg_root) {
            for entry in entries.flatten() {
                if m8_paths.len() >= 1000 {
                    break;
                }
                let p = entry.path().join(".m8");
                if p.is_file() && !m8_paths.contains(&p) {
                    m8_paths.push(p);
                }
            }
        }
        assert!(
            !m8_paths.is_empty(),
            "Expected .m8 files in standardgalactic"
        );

        let raw_blobs: Vec<Vec<u8>> = m8_paths.iter().filter_map(|p| fs::read(p).ok()).collect();
        let total_packed_bytes: usize = raw_blobs.iter().map(|b| b.len()).sum();

        // 1. O(1) WAVE header scan benchmark (zero decompression)
        let t0 = std::time::Instant::now();
        let mut valid_headers = 0usize;
        for blob in &raw_blobs {
            assert_eq!(&blob[0..4], b"MEM8");
            let hdr = extract_m8_wave_header(blob).expect("valid WAVE header");
            assert!((20.0..=200.0).contains(&hdr.frequency));
            valid_headers += 1;
        }
        let wave_elapsed = t0.elapsed();

        // 2. Full CRC32 + decompression + DirectoryConsciousness rehydration benchmark
        let t1 = std::time::Instant::now();
        let mut total_decoded_bytes = 0usize;
        let mut loaded_nodes = Vec::with_capacity(raw_blobs.len());
        for blob in &raw_blobs {
            let decoded = decode_m8_bytes(blob).expect("CRC32 + ZLIB decode must succeed");
            total_decoded_bytes += decoded.len();
            let c = DirectoryConsciousness::from_m8_bytes(blob)
                .expect("DirectoryConsciousness::from_m8_bytes must succeed");
            assert_eq!(c.quantum_digest.len(), 16);
            loaded_nodes.push(c);
        }
        let decode_elapsed = t1.elapsed();

        // 3. M8MEM v2 Anchor Memory test seeded with 250 standardgalactic nodes
        let temp_dir = TempDir::new().unwrap();
        let m8mem_path = temp_dir.path().join("memories.m8");
        let t2 = std::time::Instant::now();
        let mut mem_mgr =
            crate::memory_manager::MemoryManager::with_bank_path(m8mem_path.clone()).unwrap();
        let seed_count = loaded_nodes.len().min(250);
        let mut anchored_count = 0usize;
        let mut quarantined_count = 0usize;
        for (idx, node) in loaded_nodes.iter().take(seed_count).enumerate() {
            let dir_path = m8_paths[idx]
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("/home/hue/source/standardgalactic/{}", idx));
            let repo_name = m8_paths[idx]
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("repo")
                .to_string();
            let clean_essence = node.essence.replace('\u{feff}', "");
            match mem_mgr.anchor_silent(
                "consciousness",
                vec![repo_name.clone(), "standardgalactic".to_string()],
                &format!("sg/{}: {}", repo_name, clean_essence),
                &dir_path,
            ) {
                Ok(_) => anchored_count += 1,
                Err(_) => quarantined_count += 1,
            }
        }
        let anchor_elapsed = t2.elapsed();
        let m8mem_bytes = fs::metadata(&m8mem_path).unwrap().len();

        let t3 = std::time::Instant::now();
        let mut reloaded_mgr =
            crate::memory_manager::MemoryManager::with_bank_path(m8mem_path).unwrap();
        let recalled = reloaded_mgr
            .find(&["standardgalactic".to_string()])
            .unwrap();
        let recall_elapsed = t3.elapsed();
        assert_eq!(recalled.len(), anchored_count);
        assert!(anchored_count > 200);

        println!(
            "\n=== StandardGalactic Tight .m8 & Memory Benchmark ({} files) ===",
            raw_blobs.len()
        );
        println!(
            "  Packed MEM8 size (1,000 files):  {:.1} KiB (avg {} B/file)",
            total_packed_bytes as f64 / 1024.0,
            total_packed_bytes / raw_blobs.len()
        );
        println!(
            "  Decoded JSON payload size:       {:.1} KiB",
            total_decoded_bytes as f64 / 1024.0
        );
        println!(
            "  O(1) WAVE header scan (1,000):   {:?} ({} ns/file, {} valid)",
            wave_elapsed,
            wave_elapsed.as_nanos() / raw_blobs.len() as u128,
            valid_headers
        );
        println!(
            "  Full CRC32 + ZLIB + rehydrate:   {:?} ({:.2} µs/file)",
            decode_elapsed,
            decode_elapsed.as_secs_f64() * 1e6 / raw_blobs.len() as f64
        );
        println!(
            "  M8MEM v2 bank ({} anchored, {} AiGuardian-quarantined): {:.1} KiB ({} B/memory, anchor={:?}, reload+recall={:?}, recalled={})",
            anchored_count,
            quarantined_count,
            m8mem_bytes as f64 / 1024.0,
            m8mem_bytes / anchored_count.max(1) as u64,
            anchor_elapsed,
            recall_elapsed,
            recalled.len()
        );
    }
}
