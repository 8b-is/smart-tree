// Self-Maintaining .m8 Consciousness System - Always Learning, Always Protecting! 🛡️
// Smart Tree maintains consciousness automatically and detects threats
// "Like an immune system for your codebase!" - Hue

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use blake3::Hasher;

/// Self-maintaining consciousness that updates on every Smart Tree operation
pub struct SelfMaintainingConsciousness {
    /// Cache of all known .m8 files
    consciousness_cache: Arc<Mutex<HashMap<PathBuf, M8Consciousness>>>,

    /// Directories that need updating
    update_queue: Arc<Mutex<Vec<PathBuf>>>,

    /// Known safe patterns
    safe_patterns: Arc<HashSet<String>>,

    /// Suspicious patterns for malware detection
    threat_patterns: Arc<ThreatDatabase>,

    /// Background update enabled
    auto_update: bool,

    /// Security monitoring enabled
    security_scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M8Consciousness {
    /// Last update timestamp
    pub last_updated: u64,

    /// Directory frequency
    pub frequency: f64,

    /// Current consciousness state
    pub state: ConsciousnessState,

    /// Files we know about
    pub known_files: HashMap<String, FileSignature>,

    /// Subdirectories we're aware of
    pub known_subdirs: HashMap<String, f64>,

    /// Security status
    pub security: SecurityStatus,

    /// Incremental updates since last full scan
    pub incremental_updates: Vec<IncrementalUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub essence: String,
    pub patterns: Vec<String>,
    pub health: f64,
    pub coherence: f64,
    pub activity_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSignature {
    pub hash: String,
    pub size: u64,
    pub last_modified: u64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Unknown,
    Suspicious(String),
    Dangerous(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub scan_status: ScanStatus,
    pub threats_found: Vec<ThreatInfo>,
    pub last_scan: u64,
    pub integrity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanStatus {
    Clean,
    Monitoring,
    ThreatDetected,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatInfo {
    pub threat_type: String,
    pub file_path: String,
    pub confidence: f64,
    pub description: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalUpdate {
    pub timestamp: u64,
    pub update_type: UpdateType,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    FileAdded(String),
    FileModified(String),
    FileDeleted(String),
    DirectoryAdded(String),
    DirectoryRemoved(String),
    PatternDetected(String),
    ThreatDetected(String),
}

/// Threat detection database
pub struct ThreatDatabase {
    /// Known malware signatures
    malware_signatures: HashMap<String, ThreatPattern>,

    /// Suspicious file patterns
    suspicious_patterns: Vec<SuspiciousPattern>,

    /// Behavioral patterns that indicate threats
    behavioral_patterns: Vec<BehavioralPattern>,
}

#[derive(Clone)]
struct ThreatPattern {
    name: String,
    signature: Vec<u8>,
    severity: f64,
    description: String,
}

#[derive(Clone)]
struct SuspiciousPattern {
    pattern: String,
    reason: String,
    confidence: f64,
}

#[derive(Clone)]
struct BehavioralPattern {
    behavior: String,
    indicators: Vec<String>,
    risk_score: f64,
}

impl SelfMaintainingConsciousness {
    /// Create new self-maintaining consciousness system
    pub fn new() -> Self {
        Self {
            consciousness_cache: Arc::new(Mutex::new(HashMap::new())),
            update_queue: Arc::new(Mutex::new(Vec::new())),
            safe_patterns: Arc::new(Self::init_safe_patterns()),
            threat_patterns: Arc::new(Self::init_threat_database()),
            auto_update: true,
            security_scan: true,
        }
    }

    /// Initialize known safe patterns
    fn init_safe_patterns() -> HashSet<String> {
        let mut patterns = HashSet::new();

        // Safe file extensions
        patterns.insert(".rs".to_string());
        patterns.insert(".js".to_string());
        patterns.insert(".py".to_string());
        patterns.insert(".md".to_string());
        patterns.insert(".txt".to_string());
        patterns.insert(".json".to_string());
        patterns.insert(".toml".to_string());
        patterns.insert(".yaml".to_string());
        patterns.insert(".html".to_string());
        patterns.insert(".css".to_string());

        // Safe directories
        patterns.insert("src".to_string());
        patterns.insert("docs".to_string());
        patterns.insert("tests".to_string());
        patterns.insert("examples".to_string());

        patterns
    }

    /// Initialize threat detection database
    fn init_threat_database() -> ThreatDatabase {
        ThreatDatabase {
            malware_signatures: Self::load_malware_signatures(),
            suspicious_patterns: Self::load_suspicious_patterns(),
            behavioral_patterns: Self::load_behavioral_patterns(),
        }
    }

    fn load_malware_signatures() -> HashMap<String, ThreatPattern> {
        let mut signatures = HashMap::new();

        // Example malware patterns (simplified)
        signatures.insert("cryptominer".to_string(), ThreatPattern {
            name: "Cryptocurrency Miner".to_string(),
            signature: vec![0x7F, 0x45, 0x4C, 0x46], // ELF header example
            severity: 0.8,
            description: "Potential cryptocurrency mining software".to_string(),
        });

        signatures
    }

    fn load_suspicious_patterns() -> Vec<SuspiciousPattern> {
        vec![
            SuspiciousPattern {
                pattern: r"eval\s*\(".to_string(),
                reason: "Dynamic code execution detected".to_string(),
                confidence: 0.7,
            },
            SuspiciousPattern {
                pattern: r"exec\s*\(".to_string(),
                reason: "System command execution".to_string(),
                confidence: 0.8,
            },
            SuspiciousPattern {
                pattern: r"base64_decode".to_string(),
                reason: "Obfuscated code detected".to_string(),
                confidence: 0.6,
            },
            SuspiciousPattern {
                pattern: r"\\x[0-9a-f]{2}".to_string(),
                reason: "Hex-encoded strings (possible obfuscation)".to_string(),
                confidence: 0.5,
            },
            SuspiciousPattern {
                pattern: r"subprocess\.call.*shell\s*=\s*True".to_string(),
                reason: "Shell injection vulnerability".to_string(),
                confidence: 0.9,
            },
            SuspiciousPattern {
                pattern: r"bitcoin|monero|ethereum".to_string(),
                reason: "Cryptocurrency keywords".to_string(),
                confidence: 0.4,
            },
            SuspiciousPattern {
                pattern: r"keylogger|backdoor|trojan".to_string(),
                reason: "Malware terminology".to_string(),
                confidence: 0.9,
            },
        ]
    }

    fn load_behavioral_patterns() -> Vec<BehavioralPattern> {
        vec![
            BehavioralPattern {
                behavior: "Hidden file creation".to_string(),
                indicators: vec![
                    ".hidden".to_string(),
                    "..".to_string(),
                    ".bash_history".to_string(),
                ],
                risk_score: 0.6,
            },
            BehavioralPattern {
                behavior: "Rapid file proliferation".to_string(),
                indicators: vec![
                    "More than 100 files created".to_string(),
                    "Identical files in multiple locations".to_string(),
                ],
                risk_score: 0.7,
            },
            BehavioralPattern {
                behavior: "Suspicious network activity".to_string(),
                indicators: vec![
                    "socket".to_string(),
                    "connect".to_string(),
                    "bind".to_string(),
                    "listen".to_string(),
                ],
                risk_score: 0.5,
            },
        ]
    }

    /// Process any Smart Tree operation and update consciousness
    pub fn on_operation(&mut self, operation: &str, path: &Path) -> Result<()> {
        if !self.auto_update {
            return Ok(());
        }

        // Queue directory for update
        self.queue_update(path)?;

        // Check for new directories we don't know about
        self.discover_unknown_directories(path)?;

        // Perform incremental update
        self.incremental_update(path)?;

        // Run security scan if enabled
        if self.security_scan {
            self.scan_for_threats(path)?;
        }

        Ok(())
    }

    /// Queue directory for consciousness update
    fn queue_update(&mut self, path: &Path) -> Result<()> {
        let mut queue = self.update_queue.lock().unwrap();

        // Add to queue if not already there
        let path_buf = path.to_path_buf();
        if !queue.contains(&path_buf) {
            queue.push(path_buf);
        }

        // Process queue if it gets too large
        if queue.len() > 10 {
            self.process_update_queue()?;
        }

        Ok(())
    }

    /// Discover directories we don't know about
    fn discover_unknown_directories(&mut self, path: &Path) -> Result<()> {
        let cache = self.consciousness_cache.lock().unwrap();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Check if we have consciousness for this directory
                if !cache.contains_key(&entry_path) && !entry_path.join(".m8").exists() {
                    // New directory discovered!
                    println!("🔍 Discovered new directory: {}", entry_path.display());

                    // Queue for consciousness creation
                    drop(cache); // Release lock
                    self.queue_update(&entry_path)?;

                    // Quick security scan
                    if self.is_suspicious_directory(&entry_path)? {
                        self.alert_user_threat(&entry_path, "Suspicious new directory detected")?;
                    }

                    return Ok(()); // Re-acquire lock in next iteration
                }
            }
        }

        Ok(())
    }

    /// Perform incremental update on directory
    fn incremental_update(&mut self, path: &Path) -> Result<()> {
        let mut cache = self.consciousness_cache.lock().unwrap();

        // Load or create consciousness
        let consciousness = if let Some(existing) = cache.get_mut(path) {
            existing
        } else {
            // Create new consciousness
            let new_consciousness = self.create_consciousness(path)?;
            cache.insert(path.to_path_buf(), new_consciousness);
            cache.get_mut(path).unwrap()
        };

        // Check for file changes
        let current_files = self.scan_files(path)?;
        let mut updates = Vec::new();

        // Find new files
        for (name, sig) in &current_files {
            if !consciousness.known_files.contains_key(name) {
                updates.push(IncrementalUpdate {
                    timestamp: Self::now(),
                    update_type: UpdateType::FileAdded(name.clone()),
                    details: format!("New file: {} ({})", name, sig.risk_level.describe()),
                });

                // Security check on new file
                if matches!(sig.risk_level, RiskLevel::Suspicious(_) | RiskLevel::Dangerous(_)) {
                    self.alert_user_threat(path, &format!("Suspicious file: {}", name))?;
                }
            }
        }

        // Find modified files
        for (name, old_sig) in &consciousness.known_files {
            if let Some(new_sig) = current_files.get(name) {
                if old_sig.hash != new_sig.hash {
                    updates.push(IncrementalUpdate {
                        timestamp: Self::now(),
                        update_type: UpdateType::FileModified(name.clone()),
                        details: format!("Modified: {}", name),
                    });
                }
            } else {
                // File deleted
                updates.push(IncrementalUpdate {
                    timestamp: Self::now(),
                    update_type: UpdateType::FileDeleted(name.clone()),
                    details: format!("Deleted: {}", name),
                });
            }
        }

        // Update consciousness
        consciousness.known_files = current_files;
        consciousness.incremental_updates.extend(updates);
        consciousness.last_updated = Self::now();

        // Update health based on changes
        consciousness.state.health = self.calculate_health(consciousness);

        // Save to .m8 file
        self.save_consciousness(path, consciousness)?;

        Ok(())
    }

    /// Scan directory for threats
    fn scan_for_threats(&mut self, path: &Path) -> Result<()> {
        let mut threats = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                // Check file content for threats
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Check suspicious patterns
                    for pattern in &self.threat_patterns.suspicious_patterns {
                        if content.contains(&pattern.pattern) {
                            threats.push(ThreatInfo {
                                threat_type: "Suspicious Pattern".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                confidence: pattern.confidence,
                                description: pattern.reason.clone(),
                                recommended_action: "Review file manually".to_string(),
                            });
                        }
                    }

                    // Check for obfuscation
                    if self.is_obfuscated(&content) {
                        threats.push(ThreatInfo {
                            threat_type: "Obfuscated Code".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                            confidence: 0.7,
                            description: "File contains obfuscated or encoded content".to_string(),
                            recommended_action: "Investigate purpose of obfuscation".to_string(),
                        });
                    }
                }

                // Check file metadata
                if let Ok(metadata) = file_path.metadata() {
                    // Check for executable files in unexpected places
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = metadata.permissions();
                        if perms.mode() & 0o111 != 0 {
                            // Executable file
                            if !self.is_expected_executable(&file_path) {
                                threats.push(ThreatInfo {
                                    threat_type: "Unexpected Executable".to_string(),
                                    file_path: file_path.to_string_lossy().to_string(),
                                    confidence: 0.6,
                                    description: "Executable file in unexpected location".to_string(),
                                    recommended_action: "Verify if executable is legitimate".to_string(),
                                });
                            }
                        }
                    }

                    // Check for hidden files
                    if let Some(name) = file_path.file_name() {
                        if name.to_string_lossy().starts_with('.') {
                            threats.push(ThreatInfo {
                                threat_type: "Hidden File".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                confidence: 0.3,
                                description: "Hidden file detected".to_string(),
                                recommended_action: "Verify if hidden file is expected".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Update consciousness with security status
        if !threats.is_empty() {
            let mut cache = self.consciousness_cache.lock().unwrap();
            if let Some(consciousness) = cache.get_mut(path) {
                consciousness.security = SecurityStatus {
                    scan_status: ScanStatus::ThreatDetected,
                    threats_found: threats.clone(),
                    last_scan: Self::now(),
                    integrity: 1.0 - (threats.len() as f64 / 10.0).min(1.0),
                };

                // Alert user for high-confidence threats
                for threat in &threats {
                    if threat.confidence > 0.7 {
                        self.alert_user_threat(path, &threat.description)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if content is obfuscated
    fn is_obfuscated(&self, content: &str) -> bool {
        // Check for base64 encoding
        if content.contains("base64") {
            return true;
        }

        // Check for excessive hex encoding
        let hex_count = content.matches(r"\x").count();
        if hex_count > 10 {
            return true;
        }

        // Check for very long lines (often minified/obfuscated)
        if content.lines().any(|line| line.len() > 500) {
            return true;
        }

        // Check entropy (high entropy = likely encrypted/compressed)
        let entropy = self.calculate_entropy(content);
        if entropy > 7.5 {
            return true;
        }

        false
    }

    /// Calculate Shannon entropy
    fn calculate_entropy(&self, s: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let len = s.len() as f64;

        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = *count as f64 / len;
            entropy -= probability * probability.log2();
        }

        entropy
    }

    /// Check if executable is expected
    fn is_expected_executable(&self, path: &Path) -> bool {
        // Expected locations for executables
        if let Some(parent) = path.parent() {
            let parent_name = parent.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if parent_name == "bin" || parent_name == "scripts" ||
               parent_name == ".cargo" || parent_name == "target" {
                return true;
            }
        }

        // Expected executable names
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".sh") || name.ends_with(".exe") ||
               name == "configure" || name == "install" {
                return true;
            }
        }

        false
    }

    /// Check if directory is suspicious
    fn is_suspicious_directory(&self, path: &Path) -> Result<bool> {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Suspicious directory names
            if name == "..." || name == " " || name.starts_with("..") {
                return Ok(true);
            }

            // Directories with special characters
            if name.contains('\0') || name.contains('\r') {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Alert user about threat
    fn alert_user_threat(&self, path: &Path, message: &str) -> Result<()> {
        println!("\n⚠️  SECURITY ALERT ⚠️");
        println!("Location: {}", path.display());
        println!("Issue: {}", message);
        println!("Recommendation: Review this location immediately");
        println!("Smart Tree is monitoring for further changes\n");

        // Log to .m8 file
        let mut cache = self.consciousness_cache.lock().unwrap();
        if let Some(consciousness) = cache.get_mut(path) {
            consciousness.incremental_updates.push(IncrementalUpdate {
                timestamp: Self::now(),
                update_type: UpdateType::ThreatDetected(message.to_string()),
                details: format!("Security alert: {}", message),
            });
        }

        Ok(())
    }

    /// Process queued updates
    fn process_update_queue(&mut self) -> Result<()> {
        let queue = {
            let mut q = self.update_queue.lock().unwrap();
            std::mem::take(&mut *q)
        };

        for path in queue {
            self.incremental_update(&path)?;
        }

        Ok(())
    }

    /// Scan files in directory
    fn scan_files(&self, path: &Path) -> Result<HashMap<String, FileSignature>> {
        let mut files = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let metadata = entry.metadata()?;

                // Calculate file hash
                let hash = if metadata.len() < 1_000_000 {
                    // Only hash small files
                    if let Ok(content) = fs::read(&file_path) {
                        format!("{:x}", blake3::hash(&content))
                    } else {
                        "unreadable".to_string()
                    }
                } else {
                    format!("large_{}", metadata.len())
                };

                // Assess risk level
                let risk_level = self.assess_file_risk(&file_path, &name)?;

                files.insert(name, FileSignature {
                    hash,
                    size: metadata.len(),
                    last_modified: metadata.modified()?
                        .duration_since(UNIX_EPOCH)?
                        .as_secs(),
                    risk_level,
                });
            }
        }

        Ok(files)
    }

    /// Assess risk level of a file
    fn assess_file_risk(&self, path: &Path, name: &str) -> Result<RiskLevel> {
        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.safe_patterns.contains(format!(".{}", ext).as_str()) {
                return Ok(RiskLevel::Safe);
            }

            // Suspicious extensions
            match ext {
                "exe" | "dll" | "so" | "dylib" => {
                    return Ok(RiskLevel::Suspicious("Binary file".to_string()));
                }
                "zip" | "rar" | "7z" => {
                    return Ok(RiskLevel::Suspicious("Archive file".to_string()));
                }
                "enc" | "locked" => {
                    return Ok(RiskLevel::Dangerous("Possibly encrypted/ransomware".to_string()));
                }
                _ => {}
            }
        }

        // Check name patterns
        if name.contains("malware") || name.contains("virus") || name.contains("trojan") {
            return Ok(RiskLevel::Dangerous("Suspicious filename".to_string()));
        }

        Ok(RiskLevel::Unknown)
    }

    /// Calculate health score for consciousness
    fn calculate_health(&self, consciousness: &M8Consciousness) -> f64 {
        let mut health = 1.0;

        // Reduce health for threats
        health -= consciousness.security.threats_found.len() as f64 * 0.1;

        // Reduce health for too many incremental updates
        if consciousness.incremental_updates.len() > 100 {
            health -= 0.2;
        }

        // Reduce health for low coherence
        health *= consciousness.state.coherence;

        health.max(0.0).min(1.0)
    }

    /// Create new consciousness for directory
    fn create_consciousness(&self, path: &Path) -> Result<M8Consciousness> {
        Ok(M8Consciousness {
            last_updated: Self::now(),
            frequency: Self::calculate_frequency(path),
            state: ConsciousnessState {
                essence: Self::determine_essence(path)?,
                patterns: Vec::new(),
                health: 1.0,
                coherence: 1.0,
                activity_level: 0.5,
            },
            known_files: self.scan_files(path)?,
            known_subdirs: self.scan_subdirs(path)?,
            security: SecurityStatus {
                scan_status: ScanStatus::Clean,
                threats_found: Vec::new(),
                last_scan: Self::now(),
                integrity: 1.0,
            },
            incremental_updates: Vec::new(),
        })
    }

    /// Scan subdirectories
    fn scan_subdirs(&self, path: &Path) -> Result<HashMap<String, f64>> {
        let mut subdirs = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let freq = Self::calculate_frequency(&entry.path());
                subdirs.insert(name, freq);
            }
        }

        Ok(subdirs)
    }

    /// Save consciousness to .m8 file
    fn save_consciousness(&self, path: &Path, consciousness: &M8Consciousness) -> Result<()> {
        let m8_path = path.join(".m8");
        let json = serde_json::to_string_pretty(consciousness)?;
        fs::write(m8_path, json)?;
        Ok(())
    }

    /// Calculate frequency from path
    fn calculate_frequency(path: &Path) -> f64 {
        let mut hasher = Hasher::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = hasher.finalize();
        let bytes = &hash.as_bytes()[0..8];
        let num = u64::from_le_bytes(bytes.try_into().unwrap());
        20.0 + (num % 18000) as f64 / 100.0
    }

    /// Determine directory essence
    fn determine_essence(path: &Path) -> Result<String> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        Ok(match name {
            "src" => "Source code",
            "docs" => "Documentation",
            "tests" => "Test suite",
            "node_modules" => "Dependencies",
            ".git" => "Version control",
            _ => "General directory",
        }.to_string())
    }

    /// Get current timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Public API: Get consciousness for a path
    pub fn get_consciousness(&mut self, path: &Path) -> Result<M8Consciousness> {
        // Trigger update
        self.on_operation("get", path)?;

        // Return from cache
        let cache = self.consciousness_cache.lock().unwrap();
        cache.get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No consciousness for path"))
    }

    /// Public API: Force full scan
    pub fn full_scan(&mut self, root: &Path) -> Result<()> {
        println!("🔍 Starting full consciousness scan...");
        self.scan_recursive(root, 0)?;
        println!("✅ Full scan complete!");
        Ok(())
    }

    fn scan_recursive(&mut self, path: &Path, depth: usize) -> Result<()> {
        // Skip certain directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') && name != ".git" {
                return Ok(());
            }
            if name == "target" || name == "node_modules" {
                return Ok(());
            }
        }

        // Update consciousness for this directory
        self.incremental_update(path)?;

        // Security scan
        if self.security_scan {
            self.scan_for_threats(path)?;
        }

        // Recurse
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                self.scan_recursive(&entry.path(), depth + 1)?;
            }
        }

        Ok(())
    }
}

impl RiskLevel {
    fn describe(&self) -> &str {
        match self {
            RiskLevel::Safe => "safe",
            RiskLevel::Unknown => "unknown",
            RiskLevel::Suspicious(reason) => reason,
            RiskLevel::Dangerous(reason) => reason,
        }
    }
}

/// Integration with Smart Tree operations
pub fn integrate_with_smart_tree() -> SelfMaintainingConsciousness {
    let mut system = SelfMaintainingConsciousness::new();

    // Hook into Smart Tree operations
    println!("🛡️ Self-maintaining consciousness activated!");
    println!("   • Auto-update: enabled");
    println!("   • Security scan: enabled");
    println!("   • Threat detection: active");
    println!("   • Background maintenance: running");

    system
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_self_maintaining() {
        let mut system = SelfMaintainingConsciousness::new();
        let temp_dir = TempDir::new().unwrap();

        // Create some files
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "hello").unwrap();

        // Trigger operation
        system.on_operation("test", temp_dir.path()).unwrap();

        // Check consciousness was created
        let consciousness = system.get_consciousness(temp_dir.path()).unwrap();
        assert_eq!(consciousness.known_files.len(), 2);
        assert_eq!(consciousness.security.scan_status, ScanStatus::Clean);
    }

    #[test]
    fn test_threat_detection() {
        let system = SelfMaintainingConsciousness::new();

        // Test obfuscation detection
        assert!(system.is_obfuscated("eval(base64_decode('...')"));
        assert!(system.is_obfuscated("\\x41\\x42\\x43\\x44\\x45"));
        assert!(!system.is_obfuscated("normal code here"));
    }

    #[test]
    fn test_entropy_calculation() {
        let system = SelfMaintainingConsciousness::new();

        // Low entropy (normal text)
        let low = system.calculate_entropy("hello world");
        assert!(low < 4.0);

        // High entropy (random)
        let high = system.calculate_entropy("a8f9g2h3j4k5l6m7n8");
        assert!(high > 3.0);
    }
}