//! Security Vigilance Mode - Smart Tree as your security sentinel! 🕵️‍♂️
//! 
//! Always watching for anomalies, even in the "boring" places where bad actors hide.
//! Takes random samples, tracks recent modifications, and looks for suspicious patterns.

use crate::scanner::FileNode;
use chrono::{DateTime, Local, Duration};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Security alert levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    /// 🟢 Normal - Nothing suspicious
    Normal,
    /// 🟡 Interesting - Worth noting
    Interesting,
    /// 🟠 Suspicious - Potential issue
    Suspicious,
    /// 🔴 Critical - Definite security concern
    Critical,
}

impl AlertLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Normal => "🟢",
            Self::Interesting => "🟡",
            Self::Suspicious => "🟠",
            Self::Critical => "🔴",
        }
    }
}

/// Security finding
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub path: PathBuf,
    pub alert_level: AlertLevel,
    pub reason: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Local>,
}

/// Tracks recent file modifications
#[derive(Debug)]
pub struct RecentWriteTracker {
    /// Last N modified files per directory
    recent_writes: HashMap<PathBuf, VecDeque<(PathBuf, DateTime<Local>)>>,
    /// How many recent writes to track
    track_count: usize,
}

impl RecentWriteTracker {
    pub fn new(track_count: usize) -> Self {
        Self {
            recent_writes: HashMap::new(),
            track_count,
        }
    }
    
    pub fn add_file(&mut self, dir: &Path, file: &Path, modified: DateTime<Local>) {
        let writes = self.recent_writes
            .entry(dir.to_path_buf())
            .or_insert_with(|| VecDeque::with_capacity(self.track_count));
        
        writes.push_front((file.to_path_buf(), modified));
        if writes.len() > self.track_count {
            writes.pop_back();
        }
    }
    
    pub fn get_recent_writes(&self, dir: &Path) -> Vec<(PathBuf, DateTime<Local>)> {
        self.recent_writes
            .get(dir)
            .map(|deque| deque.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Security vigilance analyzer
pub struct SecurityVigilance {
    /// Track recent writes
    write_tracker: RecentWriteTracker,
    
    /// Security findings
    findings: Vec<SecurityFinding>,
    
    /// Suspicious patterns to look for
    suspicious_patterns: Vec<(regex::Regex, String, AlertLevel)>,
    
    /// Suspicious file names
    suspicious_names: HashMap<String, (String, AlertLevel)>,
    
    /// Allowed sample size for file inspection
    max_sample_size: usize,
    
    /// Directories that should rarely change
    protected_paths: Vec<String>,
}

impl SecurityVigilance {
    pub fn new() -> Self {
        let mut suspicious_patterns = vec![];
        let mut suspicious_names = HashMap::new();
        
        // Suspicious content patterns
        suspicious_patterns.push((
            regex::Regex::new(r"eval\s*\(|exec\s*\(").unwrap(),
            "Dynamic code execution detected".to_string(),
            AlertLevel::Suspicious,
        ));
        
        suspicious_patterns.push((
            regex::Regex::new(r"(?i)(password|passwd|pwd)\s*=\s*[\"'][^\"']+[\"']").unwrap(),
            "Hardcoded password detected".to_string(),
            AlertLevel::Critical,
        ));
        
        suspicious_patterns.push((
            regex::Regex::new(r"(?i)api[_-]?key\s*=\s*[\"'][^\"']+[\"']").unwrap(),
            "Hardcoded API key detected".to_string(),
            AlertLevel::Critical,
        ));
        
        suspicious_patterns.push((
            regex::Regex::new(r"0x[0-9a-fA-F]{40,}").unwrap(),
            "Possible crypto wallet address".to_string(),
            AlertLevel::Interesting,
        ));
        
        suspicious_patterns.push((
            regex::Regex::new(r"(?i)wget|curl.*http").unwrap(),
            "Network download command detected".to_string(),
            AlertLevel::Suspicious,
        ));
        
        suspicious_patterns.push((
            regex::Regex::new(r"/etc/passwd|/etc/shadow").unwrap(),
            "System file access detected".to_string(),
            AlertLevel::Critical,
        ));
        
        // Suspicious file names
        suspicious_names.insert(".env.prod".to_string(), 
            ("Production environment file".to_string(), AlertLevel::Suspicious));
        suspicious_names.insert("id_rsa".to_string(), 
            ("Private SSH key".to_string(), AlertLevel::Critical));
        suspicious_names.insert(".npmrc".to_string(), 
            ("NPM configuration with possible tokens".to_string(), AlertLevel::Interesting));
        suspicious_names.insert("wallet.dat".to_string(), 
            ("Cryptocurrency wallet file".to_string(), AlertLevel::Critical));
        
        // Backdoor-ish names
        suspicious_names.insert("backdoor.js".to_string(), 
            ("Suspicious filename: backdoor".to_string(), AlertLevel::Critical));
        suspicious_names.insert("shell.php".to_string(), 
            ("Web shell detected".to_string(), AlertLevel::Critical));
        suspicious_names.insert("c99.php".to_string(), 
            ("Known web shell name".to_string(), AlertLevel::Critical));
        
        let protected_paths = vec![
            "node_modules".to_string(),
            ".git".to_string(),
            "System32".to_string(),
            "/etc".to_string(),
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
        ];
        
        Self {
            write_tracker: RecentWriteTracker::new(5),
            findings: Vec::new(),
            suspicious_patterns,
            suspicious_names,
            max_sample_size: 1024, // 1KB sample
            protected_paths,
        }
    }
    
    /// Analyze a file node for security issues
    pub fn analyze_node(&mut self, node: &FileNode, parent_path: &Path) {
        let full_path = parent_path.join(&node.name);
        
        // Check if this is a recent write
        if let Ok(metadata) = fs::metadata(&full_path) {
            if let Ok(modified) = metadata.modified() {
                let modified_time: DateTime<Local> = modified.into();
                let now = Local::now();
                
                // Track if modified in last 24 hours
                if now.signed_duration_since(modified_time) < Duration::hours(24) {
                    self.write_tracker.add_file(parent_path, &full_path, modified_time);
                    
                    // Extra vigilance for recent writes in protected paths
                    if self.is_protected_path(&full_path) {
                        self.add_finding(SecurityFinding {
                            path: full_path.clone(),
                            alert_level: AlertLevel::Interesting,
                            reason: "Recent modification in protected directory".to_string(),
                            details: Some(format!("Modified: {}", modified_time.format("%Y-%m-%d %H:%M:%S"))),
                            timestamp: now,
                        });
                    }
                }
            }
        }
        
        // Check suspicious file names
        if let Some((reason, level)) = self.suspicious_names.get(&node.name.to_lowercase()) {
            self.add_finding(SecurityFinding {
                path: full_path.clone(),
                alert_level: *level,
                reason: reason.clone(),
                details: None,
                timestamp: Local::now(),
            });
        }
        
        // Sample file content for suspicious patterns
        if !node.is_directory && node.size > 0 {
            self.sample_file_content(&full_path);
        }
    }
    
    /// Take a random sample from a file and check for suspicious patterns
    fn sample_file_content(&mut self, path: &Path) {
        // Skip binary files (simple heuristic based on extension)
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if matches!(ext_str.as_str(), "exe" | "dll" | "so" | "dylib" | "bin" | "jpg" | "png" | "mp4") {
                return;
            }
        }
        
        // Try to read a sample
        if let Ok(mut file) = fs::File::open(path) {
            let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
            
            if file_size > 0 {
                let mut buffer = vec![0u8; self.max_sample_size.min(file_size as usize)];
                
                // Random offset for larger files
                if file_size > self.max_sample_size as u64 {
                    let max_offset = file_size - self.max_sample_size as u64;
                    let offset = thread_rng().gen_range(0..max_offset);
                    let _ = file.seek(std::io::SeekFrom::Start(offset));
                }
                
                if let Ok(bytes_read) = file.read(&mut buffer) {
                    buffer.truncate(bytes_read);
                    
                    // Convert to string (lossy for non-UTF8)
                    let content = String::from_utf8_lossy(&buffer);
                    
                    // Check patterns
                    for (pattern, reason, level) in &self.suspicious_patterns {
                        if pattern.is_match(&content) {
                            self.add_finding(SecurityFinding {
                                path: path.to_path_buf(),
                                alert_level: *level,
                                reason: reason.clone(),
                                details: Some("Found in random sample".to_string()),
                                timestamp: Local::now(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    /// Check if a path is in a protected directory
    fn is_protected_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        self.protected_paths.iter().any(|protected| {
            path_str.contains(&protected.to_lowercase())
        })
    }
    
    /// Add a security finding
    pub fn add_finding(&mut self, finding: SecurityFinding) {
        self.findings.push(finding);
    }
    
    /// Get all findings sorted by severity
    pub fn get_findings(&self) -> Vec<&SecurityFinding> {
        let mut findings: Vec<_> = self.findings.iter().collect();
        findings.sort_by_key(|f| std::cmp::Reverse(f.alert_level));
        findings
    }
    
    /// Get summary of findings
    pub fn summary(&self) -> String {
        let mut summary = String::from("🕵️ Security Vigilance Report\n\n");
        
        let mut counts = HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.alert_level).or_insert(0) += 1;
        }
        
        if counts.is_empty() {
            summary.push_str("✅ No security issues detected!\n");
        } else {
            summary.push_str("Findings by severity:\n");
            for level in [AlertLevel::Critical, AlertLevel::Suspicious, AlertLevel::Interesting, AlertLevel::Normal] {
                if let Some(count) = counts.get(&level) {
                    summary.push_str(&format!("  {} {} findings\n", level.emoji(), count));
                }
            }
            
            // Show critical findings
            summary.push_str("\n");
            for finding in self.findings.iter().filter(|f| f.alert_level == AlertLevel::Critical) {
                summary.push_str(&format!(
                    "{} {} - {}\n", 
                    finding.alert_level.emoji(),
                    finding.path.display(),
                    finding.reason
                ));
            }
        }
        
        summary
    }
}

// Vigilance patterns that could be in AI modes:
//
// 1. Always show last 5 modified files in each directory
// 2. Random sampling from files to detect:
//    - Hardcoded secrets
//    - Malicious code patterns
//    - Suspicious network calls
//    - Backdoors
// 3. Track modifications in "boring" directories like node_modules
// 4. Alert on files that shouldn't exist (like .env.prod in git)
// 5. Detect anomalies in system directories

// Trisha says: "It's like having a security guard who actually checks 
// the boring supply closets where people hide things! Smart!" 🔍