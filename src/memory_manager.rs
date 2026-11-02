impl MemoryBank {
    pub fn new() -> Self {
        Self::default()
    }
}
// Smart Tree Memory Manager - Real memory that works! 🧠
// "Like UV EPROMs but for consciousness!" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub timestamp: DateTime<Utc>,
    pub anchor_type: String,
    pub keywords: Vec<String>,
    pub context: String,
    pub origin: String, // Where this memory came from
    pub frequency: f64, // Wave frequency of this memory
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryBank {
    pub memories: Vec<Memory>,
    pub total_recalls: usize,
    pub last_accessed: DateTime<Utc>,
}

impl Default for MemoryBank {
    fn default() -> Self {
        Self {
            memories: Vec::new(),
            total_recalls: 0,
            last_accessed: Utc::now(),
        }
    }
}

pub struct MemoryManager {
    bank_path: PathBuf,
    bank: MemoryBank,
}

impl MemoryManager {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let st_dir = Path::new(&home).join(".st");
        let bank_path = st_dir.join("memories.m8");

        // Ensure directory exists
        fs::create_dir_all(&st_dir)?;

        // Load existing memories or create new bank
        let bank = if bank_path.exists() {
            Self::load_m8(&bank_path)?
        } else {
            // Try to migrate from old GLOBAL JSON format from .mem8
            let json_path = Path::new(&home)
                .join(".mem8")
                .join("smart_tree_memories.json");

            if json_path.exists() {
                let content = fs::read_to_string(&json_path)?;
                let bank: MemoryBank = serde_json::from_str(&content).unwrap_or_default();
                // Don't delete the old file, just let the new format be saved in .st
                bank
            } else {
                MemoryBank::default()
            }
        };

        Ok(Self { bank_path, bank })
    }

    /// Anchor a new memory
    pub fn anchor(
        &mut self,
        anchor_type: &str,
        keywords: Vec<String>,
        context: &str,
        origin: &str,
    ) -> Result<()> {
        let memory = self.anchor_silent(anchor_type, keywords, context, origin)?;

        println!("💾 Memory anchored!");
        println!("  Type: {}", memory.anchor_type);
        println!("  Frequency: {:.2} Hz", memory.frequency);

        Ok(())
    }

    /// Anchor a new memory without stdout chatter (used by automatic directory memory)
    pub fn anchor_silent(
        &mut self,
        anchor_type: &str,
        keywords: Vec<String>,
        context: &str,
        origin: &str,
    ) -> Result<Memory> {
        let mut freq_sum = 0u64;
        for byte in context.bytes() {
            freq_sum = freq_sum.wrapping_add(byte as u64);
        }
        let frequency = 20.0 + ((freq_sum % 200) as f64);

        let clean_keywords: Vec<String> = keywords
            .into_iter()
            .filter_map(|k| {
                let trimmed = k.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Self::truncate_utf8(trimmed, 255).to_string())
                }
            })
            .take(64)
            .collect();

        let memory = Memory {
            timestamp: Utc::now(),
            anchor_type: Self::truncate_utf8(anchor_type, 255).to_string(),
            keywords: clean_keywords,
            context: Self::truncate_utf8(context, 4096).to_string(),
            origin: Self::truncate_utf8(origin, 255).to_string(),
            frequency,
        };

        if let Some(existing) = self.bank.memories.iter_mut().find(|m| {
            m.origin == memory.origin
                && m.anchor_type == memory.anchor_type
                && (m.anchor_type == "project"
                    || m.origin.ends_with("/.ssh/config")
                    || m.keywords == memory.keywords)
        }) {
            *existing = memory.clone();
        } else {
            self.bank.memories.push(memory.clone());
        }
        self.save()?;

        Ok(memory)
    }

    /// Automatically harvest and remember salient context from `dir` if no memory
    /// is anchored for `dir` yet, so directories are always remembered without
    /// requiring manual memory initialization.
    pub fn ensure_directory_remembered(&mut self, dir: &Path) -> Option<Memory> {
        if !dir.is_dir() {
            return None;
        }
        let canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let dir_str = canon.to_string_lossy().to_string();

        // Skip ephemeral/system pseudo-directories and test tempdirs
        if dir_str == "/"
            || dir_str.starts_with("/tmp")
            || dir_str.starts_with("/proc")
            || dir_str.starts_with("/sys")
            || dir_str.starts_with("/dev")
            || dir_str.starts_with("/run")
            || dir_str.contains("/.tmp")
            || dir_str.contains("/target/")
        {
            return None;
        }

        let truncated_origin = Self::truncate_utf8(&dir_str, 255);
        let _ = self.ensure_ssh_connections_remembered();

        let (keywords, context) = Self::synthesize_directory_memory(&canon)?;
        let truncated_context = Self::truncate_utf8(&context, 4096);

        if let Some(existing) = self
            .bank
            .memories
            .iter()
            .find(|m| m.origin == truncated_origin && m.anchor_type == "project")
        {
            if existing.context == truncated_context {
                return Some(existing.clone());
            }
        }

        self.anchor_silent("project", keywords, &context, &dir_str)
            .ok()
    }

    /// Automatically remember non-secret SSH host aliases and endpoints from `~/.ssh/config`
    /// (Host, HostName, User, Port, ProxyJump — never private keys).
    pub fn ensure_ssh_connections_remembered(&mut self) -> Option<Memory> {
        let home = dirs::home_dir()?;
        let ssh_config_path = home.join(".ssh").join("config");
        if !ssh_config_path.is_file() {
            return None;
        }
        let origin = ssh_config_path.to_string_lossy().to_string();
        let content = fs::read_to_string(&ssh_config_path).ok()?;
        let (keywords, summary) = Self::parse_ssh_config_summary(&content)?;

        if let Some(existing) = self.bank.memories.iter().find(|m| m.origin == origin) {
            if existing.context == summary {
                return Some(existing.clone());
            }
        }
        self.anchor_silent("infrastructure", keywords, &summary, &origin)
            .ok()
    }

    pub fn parse_ssh_config_summary(content: &str) -> Option<(Vec<String>, String)> {
        #[derive(Default)]
        struct SshHostEntry {
            aliases: Vec<String>,
            hostname: Option<String>,
            user: Option<String>,
            port: Option<String>,
            proxy_jump: Option<String>,
        }

        let mut hosts: Vec<SshHostEntry> = Vec::new();
        let mut current: Option<SshHostEntry> = None;

        for raw in content.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(key) = parts.next() else {
                continue;
            };
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                continue;
            }
            match key.to_ascii_lowercase().as_str() {
                "host" => {
                    if let Some(entry) = current.take() {
                        if !entry.aliases.is_empty() {
                            hosts.push(entry);
                        }
                    }
                    let aliases: Vec<String> = rest
                        .into_iter()
                        .filter(|a| *a != "*" && !a.contains('?'))
                        .map(str::to_string)
                        .collect();
                    current = Some(SshHostEntry {
                        aliases,
                        ..Default::default()
                    });
                }
                "hostname" => {
                    if let Some(entry) = current.as_mut() {
                        entry.hostname = Some(rest.join(" "));
                    }
                }
                "user" => {
                    if let Some(entry) = current.as_mut() {
                        entry.user = Some(rest.join(" "));
                    }
                }
                "port" => {
                    if let Some(entry) = current.as_mut() {
                        entry.port = Some(rest.join(" "));
                    }
                }
                "proxyjump" => {
                    if let Some(entry) = current.as_mut() {
                        entry.proxy_jump = Some(rest.join(" "));
                    }
                }
                _ => {}
            }
        }
        if let Some(entry) = current.take() {
            if !entry.aliases.is_empty() {
                hosts.push(entry);
            }
        }
        if hosts.is_empty() {
            return None;
        }

        let mut keywords = vec![
            "ssh".to_string(),
            "connections".to_string(),
            "hosts".to_string(),
            "servers".to_string(),
            "config".to_string(),
        ];
        let mut descriptions = Vec::new();
        for h in hosts {
            let alias_str = h.aliases.join("/");
            for a in &h.aliases {
                let low = a.to_ascii_lowercase();
                if !keywords.contains(&low) {
                    keywords.push(low);
                }
            }
            let target = h
                .hostname
                .clone()
                .unwrap_or_else(|| h.aliases.first().cloned().unwrap_or_default());
            if !target.is_empty() {
                let low = target.to_ascii_lowercase();
                if !keywords.contains(&low) {
                    keywords.push(low);
                }
            }
            let user_prefix = h
                .user
                .as_deref()
                .map(|u| format!("{u}@"))
                .unwrap_or_default();
            let port_suffix = h
                .port
                .as_deref()
                .filter(|p| *p != "22")
                .map(|p| format!(":{p}"))
                .unwrap_or_default();
            let jump_suffix = h
                .proxy_jump
                .as_deref()
                .map(|j| format!(" via {j}"))
                .unwrap_or_default();
            descriptions.push(format!(
                "{} -> {}{}{}{}",
                alias_str, user_prefix, target, port_suffix, jump_suffix
            ));
        }

        let summary = format!("SSH Connections (~/.ssh/config): {}", descriptions.join(" | "));
        Some((keywords, summary))
    }

    fn synthesize_directory_memory(dir: &Path) -> Option<(Vec<String>, String)> {
        let dir_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())?;

        let mut keywords: Vec<String> = vec![dir_name.to_lowercase()];
        if let Some(parent_name) = dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        {
            if !parent_name.is_empty() && parent_name != "data" && parent_name != "home" {
                keywords.push(parent_name.to_lowercase());
            }
        }

        let mut parts: Vec<String> = Vec::new();

        // 1. Inspect project manifests (Cargo.toml, package.json, pyproject.toml, go.mod)
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.is_file() {
            keywords.push("rust".to_string());
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                let mut pkg_name = None;
                let mut pkg_desc = None;
                let mut workspace_members = 0usize;
                let mut in_workspace_members = false;
                for line in content.lines() {
                    let t = line.trim();
                    if t.starts_with("name") && pkg_name.is_none() {
                        if let Some(val) = Self::extract_toml_quoted(t) {
                            pkg_name = Some(val);
                        }
                    } else if t.starts_with("description") && pkg_desc.is_none() {
                        if let Some(val) = Self::extract_toml_quoted(t) {
                            pkg_desc = Some(val);
                        }
                    } else if t.starts_with("keywords") && t.contains('[') {
                        for kw in t.split('"').skip(1).step_by(2) {
                            let kw_clean = kw.trim().to_lowercase();
                            if !kw_clean.is_empty() && !keywords.contains(&kw_clean) {
                                keywords.push(kw_clean);
                            }
                        }
                    } else if t.starts_with("members") && t.contains('[') {
                        in_workspace_members = true;
                        workspace_members += t.matches('"').count() / 2;
                        if t.contains(']') {
                            in_workspace_members = false;
                        }
                    } else if in_workspace_members {
                        workspace_members += t.matches('"').count() / 2;
                        if t.contains(']') {
                            in_workspace_members = false;
                        }
                    }
                }
                if let Some(name) = pkg_name {
                    if !keywords.contains(&name.to_lowercase()) {
                        keywords.push(name.to_lowercase());
                    }
                    if let Some(desc) = pkg_desc {
                        parts.push(format!("{name} (Rust): {desc}"));
                    } else {
                        parts.push(format!("{name} (Rust crate)"));
                    }
                } else if workspace_members > 0 {
                    parts.push(format!(
                        "{dir_name} (Rust workspace, {workspace_members} members)"
                    ));
                }
            }
        } else if dir.join("package.json").is_file() {
            keywords.push("node".to_string());
            if let Ok(content) = fs::read_to_string(dir.join("package.json")) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(dir_name);
                    if let Some(desc) = val.get("description").and_then(|v| v.as_str()) {
                        if !desc.trim().is_empty() {
                            parts.push(format!("{name} (Node/TS): {}", desc.trim()));
                        }
                    }
                }
            }
        } else if dir.join("pyproject.toml").is_file() {
            keywords.push("python".to_string());
            if let Ok(content) = fs::read_to_string(dir.join("pyproject.toml")) {
                for line in content.lines() {
                    let t = line.trim();
                    if t.starts_with("description") {
                        if let Some(desc) = Self::extract_toml_quoted(t) {
                            parts.push(format!("{dir_name} (Python): {desc}"));
                            break;
                        }
                    }
                }
            }
        }

        // 1b. Detect NVIDIA CUDA Toolkit or Ollama model directories
        if dir.join("bin").join("nvcc").is_file() || dir.join("nvvm").is_dir() {
            for kw in ["cuda", "nvidia", "nvcc", "gpu"] {
                if !keywords.contains(&kw.to_string()) {
                    keywords.push(kw.to_string());
                }
            }
            parts.push(format!("{dir_name} (NVIDIA CUDA Toolkit: nvcc, nvml, gds, targets)"));
        }

        // 2. Inspect README.md / CLAUDE.md / AGENTS.md / README / TODO.m8 for salient summary
        for doc_name in [
            "README.md",
            "CLAUDE.md",
            "AGENTS.md",
            "ARCHITECTURE.md",
            "FAMILY_CONTEXT.md",
            "FAMILY_SEED.md",
            "DIFF_REPORT.md",
            "TODO.m8",
            "TODO.md",
            "README",
            "README.txt",
        ] {
            let doc_path = dir.join(doc_name);
            if !doc_path.is_file() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&doc_path) {
                let mut heading: Option<String> = None;
                let mut summary_line: Option<String> = None;
                let mut in_code_block = false;
                for line in content.lines().take(80) {
                    let t = line.trim();
                    if t.starts_with("```") {
                        in_code_block = !in_code_block;
                        continue;
                    }
                    if in_code_block || t.is_empty() {
                        continue;
                    }
                    // Skip badges, HTML tags, table borders, and raw separators
                    if t.starts_with("[![")
                        || t.starts_with("<img")
                        || t.starts_with("<p")
                        || t.starts_with("</")
                        || t.starts_with("---")
                        || t.starts_with("===")
                        || t.starts_with('|')
                    {
                        continue;
                    }
                    if let Some(h) = t.strip_prefix("# ") {
                        let clean_h = h.trim();
                        if !clean_h.is_empty() && heading.is_none() {
                            heading = Some(clean_h.to_string());
                            for word in clean_h
                                .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                            {
                                let w = word.to_lowercase();
                                if w.len() >= 3 && !keywords.contains(&w) && keywords.len() < 16 {
                                    keywords.push(w);
                                }
                            }
                        }
                        continue;
                    }
                    if t.starts_with('#') {
                        continue;
                    }
                    if summary_line.is_none() && t.len() >= 16 {
                        summary_line = Some(Self::truncate_utf8(t, 240).to_string());
                        break;
                    }
                }
                if let (Some(h), Some(s)) = (&heading, &summary_line) {
                    parts.push(format!("{h} — {s}"));
                } else if let Some(s) = summary_line {
                    parts.push(s);
                } else if let Some(h) = heading {
                    parts.push(h);
                }
                if !parts.is_empty() {
                    break;
                }
            }
        }

        // 3. Check .m8 hierarchical consciousness file if present
        let m8_file = dir.join(".m8");
        if m8_file.is_file() {
            if let Ok(c) = crate::m8_consciousness::DirectoryConsciousness::load(dir) {
                for child_name in c.children.keys() {
                    let cn = child_name.to_lowercase();
                    if cn.len() >= 2 && !keywords.contains(&cn) && keywords.len() < 20 {
                        keywords.push(cn);
                    }
                }
                if !c.essence.is_empty() && !parts.iter().any(|p| p.contains(&c.essence)) {
                    parts.push(Self::truncate_utf8(&c.essence, 240).to_string());
                }
            }
        }

        // 4. Check .aye_consciousness.m8 or .claude_consciousness.m8
        for c_file in [".aye_consciousness.m8", ".claude_consciousness.m8"] {
            let c_path = dir.join(c_file);
            if !c_path.is_file() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&c_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(focus) = val
                        .get("project_context")
                        .and_then(|pc| pc.get("current_focus"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                    {
                        parts.push(format!("Focus: {}", focus.trim()));
                    }
                    if let Some(notes) = val
                        .get("notes")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                    {
                        parts.push(format!("Notes: {}", Self::truncate_utf8(notes.trim(), 160)));
                    }
                }
            }
        }

        // 4. Check git HEAD branch
        let git_head = dir.join(".git").join("HEAD");
        if let Ok(head_str) = fs::read_to_string(&git_head) {
            let head_trimmed = head_str.trim();
            if let Some(branch) = head_trimmed.strip_prefix("ref: refs/heads/") {
                parts.push(format!("git branch: {branch}"));
            }
        }

        // 5. Fallback to top-level notable files if no manifest/doc was found
        if parts.is_empty() {
            if let Ok(entries) = fs::read_dir(dir) {
                let mut names: Vec<String> = entries
                    .flatten()
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            None
                        } else {
                            Some(name)
                        }
                    })
                    .take(12)
                    .collect();
                names.sort();
                if !names.is_empty() {
                    parts.push(format!("{dir_name}: entries [{}]", names.join(", ")));
                }
            }
        }

        if parts.is_empty() {
            return None;
        }

        keywords.dedup();
        let context = parts.join(" | ");
        Some((keywords, context))
    }

    fn extract_toml_quoted(line: &str) -> Option<String> {
        let (_, rhs) = line.split_once('=')?;
        let trimmed = rhs.trim();
        let inner = trimmed.strip_prefix('"')?.split('"').next()?;
        if inner.trim().is_empty() {
            None
        } else {
            Some(inner.trim().to_string())
        }
    }

    fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
        if s.len() <= max_bytes {
            return s;
        }
        let mut end = max_bytes;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }

    /// Find memories by keywords
    pub fn find(&mut self, keywords: &[String]) -> Result<Vec<Memory>> {
        self.bank.total_recalls += 1;
        self.bank.last_accessed = Utc::now();

        let mut results = Vec::new();

        for memory in &self.bank.memories {
            // Check if any keyword matches (case-insensitive across keywords, context, and origin)
            for keyword in keywords {
                let kw_lower = keyword.to_lowercase();
                if kw_lower.is_empty() {
                    continue;
                }
                if memory
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains(&kw_lower))
                    || memory.context.to_lowercase().contains(&kw_lower)
                    || memory.origin.to_lowercase().contains(&kw_lower)
                {
                    results.push(memory.clone());
                    break;
                }
            }
        }

        // Sort by timestamp (most recent first)
        results.sort_by_key(|a| std::cmp::Reverse(a.timestamp));

        Ok(results)
    }

    /// Get statistics about memory bank
    pub fn stats(&self) -> String {
        format!(
            "Memory Bank Stats:\n\
             • Total memories: {}\n\
             • Total recalls: {}\n\
             • Last accessed: {}\n\
             • Storage: {}",
            self.bank.memories.len(),
            self.bank.total_recalls,
            self.bank.last_accessed.format("%Y-%m-%d %H:%M:%S"),
            self.bank_path.display()
        )
    }

    /// Save memory bank to disk in .m8 format
    fn save(&self) -> Result<()> {
        self.save_m8(&self.bank_path)?;
        Ok(())
    }

    /// Save as binary .m8 format
    fn save_m8(&self, path: &Path) -> Result<()> {
        use std::io::Write;

        let mut buffer = Vec::new();

        // Magic header: "M8MEM" (5 bytes)
        buffer.write_all(b"M8MEM")?;

        // Version byte
        buffer.push(0x01);

        // Number of memories (4 bytes, little-endian)
        buffer.write_all(&(self.bank.memories.len() as u32).to_le_bytes())?;

        // Total recalls (4 bytes)
        buffer.write_all(&(self.bank.total_recalls as u32).to_le_bytes())?;

        // Last accessed timestamp (8 bytes)
        buffer.write_all(&self.bank.last_accessed.timestamp().to_le_bytes())?;

        // Write each memory
        for memory in &self.bank.memories {
            // Type length (1 byte) + type
            buffer.push(memory.anchor_type.len() as u8);
            buffer.write_all(memory.anchor_type.as_bytes())?;

            // Keywords count (1 byte)
            buffer.push(memory.keywords.len() as u8);
            for keyword in &memory.keywords {
                buffer.push(keyword.len() as u8);
                buffer.write_all(keyword.as_bytes())?;
            }

            // Context length (2 bytes) + context
            let context_bytes = memory.context.as_bytes();
            buffer.write_all(&(context_bytes.len() as u16).to_le_bytes())?;
            buffer.write_all(context_bytes)?;

            // Origin length (1 byte) + origin
            buffer.push(memory.origin.len() as u8);
            buffer.write_all(memory.origin.as_bytes())?;

            // Frequency (8 bytes)
            buffer.write_all(&memory.frequency.to_le_bytes())?;

            // Timestamp (8 bytes)
            buffer.write_all(&memory.timestamp.timestamp().to_le_bytes())?;
        }

        // Calculate checksum (simple XOR for now)
        let checksum = buffer.iter().fold(0u8, |acc, &b| acc ^ b);
        buffer.push(checksum);

        fs::write(path, buffer)?;
        Ok(())
    }

    /// Load from binary .m8 format
    fn load_m8(path: &Path) -> Result<MemoryBank> {
        use std::io::Cursor;
        use std::io::Read;

        let data = fs::read(path)?;
        let mut cursor = Cursor::new(data);

        // Check magic header
        let mut magic = [0u8; 5];
        cursor.read_exact(&mut magic)?;
        if &magic != b"M8MEM" {
            return Err(anyhow::anyhow!("Invalid .m8 file format"));
        }

        // Version
        let mut version = [0u8; 1];
        cursor.read_exact(&mut version)?;
        if version[0] != 0x01 {
            return Err(anyhow::anyhow!("Unsupported .m8 version"));
        }

        // Number of memories
        let mut mem_count = [0u8; 4];
        cursor.read_exact(&mut mem_count)?;
        let mem_count = u32::from_le_bytes(mem_count) as usize;

        // Total recalls
        let mut recalls = [0u8; 4];
        cursor.read_exact(&mut recalls)?;
        let total_recalls = u32::from_le_bytes(recalls) as usize;

        // Last accessed
        let mut last_accessed = [0u8; 8];
        cursor.read_exact(&mut last_accessed)?;
        let last_accessed =
            DateTime::from_timestamp(i64::from_le_bytes(last_accessed), 0).unwrap_or_else(Utc::now);

        // Read memories
        let mut memories = Vec::with_capacity(mem_count);

        for _ in 0..mem_count {
            // Type
            let mut type_len = [0u8; 1];
            cursor.read_exact(&mut type_len)?;
            let mut anchor_type = vec![0u8; type_len[0] as usize];
            cursor.read_exact(&mut anchor_type)?;
            let anchor_type = String::from_utf8_lossy(&anchor_type).to_string();

            // Keywords
            let mut keyword_count = [0u8; 1];
            cursor.read_exact(&mut keyword_count)?;
            let mut keywords = Vec::with_capacity(keyword_count[0] as usize);

            for _ in 0..keyword_count[0] {
                let mut kw_len = [0u8; 1];
                cursor.read_exact(&mut kw_len)?;
                let mut keyword = vec![0u8; kw_len[0] as usize];
                cursor.read_exact(&mut keyword)?;
                keywords.push(String::from_utf8_lossy(&keyword).to_string());
            }

            // Context
            let mut context_len = [0u8; 2];
            cursor.read_exact(&mut context_len)?;
            let mut context = vec![0u8; u16::from_le_bytes(context_len) as usize];
            cursor.read_exact(&mut context)?;
            let context = String::from_utf8_lossy(&context).to_string();

            // Origin
            let mut origin_len = [0u8; 1];
            cursor.read_exact(&mut origin_len)?;
            let mut origin = vec![0u8; origin_len[0] as usize];
            cursor.read_exact(&mut origin)?;
            let origin = String::from_utf8_lossy(&origin).to_string();

            // Frequency
            let mut frequency = [0u8; 8];
            cursor.read_exact(&mut frequency)?;
            let frequency = f64::from_le_bytes(frequency);

            // Timestamp
            let mut timestamp = [0u8; 8];
            cursor.read_exact(&mut timestamp)?;
            let timestamp =
                DateTime::from_timestamp(i64::from_le_bytes(timestamp), 0).unwrap_or_else(Utc::now);

            memories.push(Memory {
                timestamp,
                anchor_type,
                keywords,
                context,
                origin,
                frequency,
            });
        }

        Ok(MemoryBank {
            memories,
            total_recalls,
            last_accessed,
        })
    }

    /// Clear all memories (with confirmation)
    pub fn clear(&mut self) -> Result<()> {
        self.bank.memories.clear();
        self.bank.total_recalls = 0;
        self.save()?;
        println!("🧹 Memory bank cleared!");
        Ok(())
    }

    /// Export memories to consciousness file
    pub fn export_to_consciousness(&self) -> Result<String> {
        let mut output = String::from("🧠 Memory Export\n");
        output.push_str(&"=".repeat(45));
        output.push('\n');

        for (i, memory) in self.bank.memories.iter().enumerate() {
            output.push_str(&format!(
                "\n[{}] {} @ {:.2}Hz\n",
                i + 1,
                memory.anchor_type,
                memory.frequency
            ));
            output.push_str(&format!("Keywords: {}\n", memory.keywords.join(", ")));
            output.push_str(&format!("Context: {}\n", memory.context));
            output.push_str(&format!("Origin: {}\n", memory.origin));
            output.push_str(&format!(
                "Time: {}\n",
                memory.timestamp.format("%Y-%m-%d %H:%M")
            ));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_synthesize_directory_memory_extracts_manifest_and_readme() {
        let temp = tempdir().unwrap();
        let proj = temp.path().join("g8t-demo");
        fs::create_dir_all(proj.join(".git")).unwrap();
        fs::write(proj.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(
            proj.join("Cargo.toml"),
            "[package]\nname = \"g8t-demo\"\ndescription = \"AI-First Git System\"\nkeywords = [\"git\", \"capsela\"]\n",
        )
        .unwrap();
        fs::write(
            proj.join("README.md"),
            "# g8t - AI-First Git System\n\n`g8t` is an Aye-native forge and Rust Git implementation.\n",
        )
        .unwrap();

        let (keywords, context) =
            MemoryManager::synthesize_directory_memory(&proj).expect("should synthesize memory");
        assert!(keywords.contains(&"g8t-demo".to_string()));
        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"capsela".to_string()));
        assert!(context.contains("AI-First Git System"));
        assert!(context.contains("Aye-native forge"));
        assert!(context.contains("git branch: main"));
    }

    #[test]
    fn test_parse_ssh_config_summary_extracts_hosts_without_secrets() {
        let cfg = r#"
# SSH configuration
Host 8s.is 8s
  HostName 188.40.209.186
  User root
  IdentityFile ~/.ssh/id_ed25519

Host i1.is i1
  HostName 5.78.220.57
  User root
  Port 2222
"#;
        let (keywords, summary) =
            MemoryManager::parse_ssh_config_summary(cfg).expect("should parse ssh config");
        assert!(keywords.contains(&"ssh".to_string()));
        assert!(keywords.contains(&"8s.is".to_string()));
        assert!(keywords.contains(&"188.40.209.186".to_string()));
        assert!(summary.contains("8s.is/8s -> root@188.40.209.186"));
        assert!(summary.contains("i1.is/i1 -> root@5.78.220.57:2222"));
        assert!(!summary.contains("id_ed25519"));
    }
}
