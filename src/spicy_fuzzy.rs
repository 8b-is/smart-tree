// Spicy Fuzzy Search - "Finding needles in haystacks at the speed of thought!" 🔍
// Fuzzy matching with MEM8 context caching for instant recall

use anyhow::Result;
use bincode;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::memory_manager::MemoryManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatch {
    pub path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub score: i64,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub match_positions: Vec<usize>, // Character positions of matches
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryContext {
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
    pub file_contents_hash: HashMap<PathBuf, u64>, // Quick content fingerprints
    pub last_scan: SystemTime,
    pub wave_signature: u32, // Quantum wave signature for this directory
}

pub struct SpicyFuzzySearch {
    matcher: Arc<SkimMatcherV2>,
    memory_manager: MemoryManager,
    context_cache: HashMap<PathBuf, DirectoryContext>,
}

impl SpicyFuzzySearch {
    pub fn new() -> Result<Self> {
        let matcher = Arc::new(SkimMatcherV2::default().smart_case().use_cache(true));
        let memory_manager = MemoryManager::new()?;

        // Load cached contexts from M8
        let context_cache = Self::load_contexts_from_m8()?;

        Ok(Self {
            matcher,
            memory_manager,
            context_cache,
        })
    }

    /// Fuzzy search across file contents with MEM8 caching
    pub fn search_content(
        &mut self,
        root_path: &Path,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<FileMatch>> {
        // Check if we have a cached context for this directory
        let context = self.get_or_create_context(root_path)?;

        // Parallel fuzzy search across all files
        let matcher = self.matcher.clone();
        let query = query.to_string();

        let mut all_matches: Vec<FileMatch> = context
            .files
            .par_iter()
            .filter_map(|file_path| Self::search_file(&matcher, file_path, &query).ok())
            .flatten()
            .collect();

        // Sort by score (highest first)
        all_matches.sort_by(|a, b| b.score.cmp(&a.score));
        all_matches.truncate(max_results);

        // Store search results in MEM8 for pattern learning
        self.store_search_pattern(&query, &all_matches)?;

        Ok(all_matches)
    }

    /// Search within a single file
    fn search_file(matcher: &SkimMatcherV2, path: &Path, query: &str) -> Result<Vec<FileMatch>> {
        // Skip binary files
        if Self::is_binary(path)? {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if let Some((score, indices)) = matcher.fuzzy_indices(line, query) {
                // Get context lines (2 before, 2 after)
                let context_before = lines
                    .get(line_idx.saturating_sub(2)..line_idx)
                    .map(|ls| ls.iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                let context_after = lines
                    .get(line_idx + 1..=(line_idx + 2).min(lines.len() - 1))
                    .map(|ls| ls.iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                matches.push(FileMatch {
                    path: path.to_path_buf(),
                    line_number: line_idx + 1,
                    line_content: line.to_string(),
                    score,
                    context_before,
                    context_after,
                    match_positions: indices,
                });
            }
        }

        Ok(matches)
    }

    /// Fuzzy search file names only
    pub fn search_filenames(
        &mut self,
        root_path: &Path,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<(PathBuf, i64)>> {
        let context = self.get_or_create_context(root_path)?;
        let matcher = self.matcher.clone();
        let query = query.to_string();

        let mut matches: Vec<(PathBuf, i64)> = context
            .files
            .par_iter()
            .filter_map(|path| {
                let filename = path.file_name()?.to_str()?;
                matcher
                    .fuzzy_match(filename, &query)
                    .map(|score| (path.clone(), score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.truncate(max_results);

        Ok(matches)
    }

    /// Get or create directory context with M8 caching
    fn get_or_create_context(&mut self, path: &Path) -> Result<DirectoryContext> {
        // Check cache first
        if let Some(context) = self.context_cache.get(path) {
            // Check if cache is still fresh (< 5 minutes old)
            if context.last_scan.elapsed()?.as_secs() < 300 {
                return Ok(context.clone());
            }
        }

        // Create new context
        let context = self.scan_directory(path)?;

        // Store in cache and M8
        self.context_cache
            .insert(path.to_path_buf(), context.clone());
        self.save_context_to_m8(&context)?;

        Ok(context)
    }

    /// Scan directory and create context
    fn scan_directory(&self, path: &Path) -> Result<DirectoryContext> {
        let mut files = Vec::new();
        let mut file_contents_hash = HashMap::new();

        // Walk directory recursively
        for entry in walkdir::WalkDir::new(path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let file_path = entry.path().to_path_buf();

                // Skip common ignored patterns
                if Self::should_ignore(&file_path) {
                    continue;
                }

                // Calculate content hash for change detection
                if let Ok(content) = fs::read(&file_path) {
                    let hash = crc32fast::hash(&content) as u64;
                    file_contents_hash.insert(file_path.clone(), hash);
                }

                files.push(file_path);
            }
        }

        // Generate quantum wave signature for this directory
        let wave_signature = self.generate_wave_signature(path);

        Ok(DirectoryContext {
            path: path.to_path_buf(),
            files,
            file_contents_hash,
            last_scan: SystemTime::now(),
            wave_signature,
        })
    }

    /// Generate a unique wave signature for a directory
    fn generate_wave_signature(&self, path: &Path) -> u32 {
        let path_str = path.display().to_string();
        let hash = crc32fast::hash(path_str.as_bytes());

        // Mix with timestamp for temporal component
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        hash.wrapping_add(now) ^ 0xDEADBEEF // Spicy constant!
    }

    /// Check if file should be ignored
    fn should_ignore(path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Common ignore patterns
            if name.starts_with('.') && name != ".env" {
                return true;
            }

            // Binary and build artifacts
            matches!(
                name,
                "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | "*.pyc"
                    | "*.pyo"
                    | "*.so"
                    | "*.dll"
                    | "*.exe"
            )
        } else {
            false
        }
    }

    /// Quick binary file detection
    fn is_binary(path: &Path) -> Result<bool> {
        let mut buffer = [0u8; 512];
        use std::io::Read;
        let mut file = fs::File::open(path)?;
        let bytes_read = file.read(&mut buffer)?;

        // Check for null bytes (common in binary files)
        Ok(buffer[..bytes_read].contains(&0))
    }

    /// Save directory context to M8 format
    fn save_context_to_m8(&self, context: &DirectoryContext) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let m8_path = cwd
            .join(".st")
            .join("contexts")
            .join(format!("{:08x}.m8", context.wave_signature));

        // Create directories if needed
        if let Some(parent) = m8_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize and compress
        let data = bincode::serialize(context)?;
        use std::io::Write;
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&data)?;
        let compressed = encoder.finish()?;

        fs::write(m8_path, compressed)?;
        Ok(())
    }

    /// Load cached contexts from M8 files
    fn load_contexts_from_m8() -> Result<HashMap<PathBuf, DirectoryContext>> {
        let mut contexts = HashMap::new();
        let cwd = std::env::current_dir()?;
        let contexts_dir = cwd
            .join(".st")
            .join("contexts");

        if !contexts_dir.exists() {
            return Ok(contexts);
        }

        for entry in fs::read_dir(contexts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("m8") {
                if let Ok(compressed) = fs::read(&path) {
                    // Decompress and deserialize
                    use std::io::Read;
                    let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
                    let mut data = Vec::new();

                    if decoder.read_to_end(&mut data).is_ok() {
                        if let Ok(context) = bincode::deserialize::<DirectoryContext>(&data) {
                            contexts.insert(context.path.clone(), context);
                        }
                    }
                }
            }
        }

        Ok(contexts)
    }

    /// Store search patterns for learning
    fn store_search_pattern(&mut self, query: &str, results: &[FileMatch]) -> Result<()> {
        // Create a memory anchor for this search pattern
        let anchor_type = "search_pattern";
        let keywords = vec![query.to_string()];

        // Create context from top results
        let context = results
            .iter()
            .take(3)
            .map(|m| format!("{}:{}", m.path.display(), m.line_number))
            .collect::<Vec<_>>()
            .join(", ");

        // Use MemoryManager to persist the pattern
        self.memory_manager
            .anchor(anchor_type, keywords, &context, "spicy_fuzzy")?;
        Ok(())
    }

    /// Get suggested searches based on past patterns
    pub fn get_suggestions(&mut self, partial_query: &str) -> Vec<String> {
        self.memory_manager
            .find(&[partial_query.to_string()])
            .map(|mems| {
                mems.into_iter()
                    .filter_map(|m| m.keywords.first().cloned())
                    .take(5)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Integration point for the spicy TUI
pub fn create_fuzzy_searcher() -> Result<SpicyFuzzySearch> {
    SpicyFuzzySearch::new()
}
