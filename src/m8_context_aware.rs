// Context-Aware .m8 Reader - Progressive detail on demand! 🎯
// "Like RAM banking on the C64 - load only what you need!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualM8 {
    pub frequency: f64,
    pub essence: String,
    pub keywords: Vec<String>,
    pub depth_level: u8, // 0=summary, 1=overview, 2=detailed, 3=full
    pub children: HashMap<String, f64>, // child_name -> frequency
    pub context_triggers: HashMap<String, String>, // keyword -> expansion_path
}

pub struct ContextAwareReader {
    cache: HashMap<PathBuf, ContextualM8>,
    current_context: Vec<String>, // Current conversation keywords
    expansion_threshold: f64,     // Similarity threshold for auto-expansion
}

impl ContextAwareReader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            current_context: Vec::new(),
            expansion_threshold: 0.7,
        }
    }

    /// Load .m8 with minimal context (just essence)
    pub fn load_minimal(&mut self, path: &Path) -> Result<String> {
        let m8 = self.load_m8(path)?;

        // Return just the essence - super minimal
        Ok(format!(
            "📍 {}: {}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            m8.essence
        ))
    }

    /// Load with smart context based on current conversation
    pub fn load_contextual(&mut self, path: &Path, context_keywords: &[String]) -> Result<String> {
        let m8 = self.load_m8(path)?;

        // Calculate relevance score
        let relevance = self.calculate_relevance(&m8, context_keywords);

        // Determine detail level based on relevance
        let detail_level = if relevance > 0.9 {
            3 // Full detail - highly relevant!
        } else if relevance > 0.7 {
            2 // Detailed
        } else if relevance > 0.5 {
            1 // Overview
        } else {
            0 // Just summary
        };

        self.format_by_detail_level(&m8, detail_level, path)
    }

    /// Calculate how relevant this .m8 is to current context
    fn calculate_relevance(&self, m8: &ContextualM8, context_keywords: &[String]) -> f64 {
        let mut score = 0.0;
        let mut matches = 0;

        for keyword in context_keywords {
            let keyword_lower = keyword.to_lowercase();

            // Check essence
            if m8.essence.to_lowercase().contains(&keyword_lower) {
                score += 1.0;
                matches += 1;
            }

            // Check keywords
            for m8_keyword in &m8.keywords {
                if m8_keyword.to_lowercase().contains(&keyword_lower) {
                    score += 0.8;
                    matches += 1;
                }
            }

            // Check triggers
            if m8.context_triggers.contains_key(keyword) {
                score += 2.0; // Strong signal!
                matches += 1;
            }
        }

        if context_keywords.is_empty() {
            return 0.0;
        }

        // Normalize by number of keywords
        (score / context_keywords.len() as f64).min(1.0)
    }

    /// Format output based on detail level
    fn format_by_detail_level(&self, m8: &ContextualM8, level: u8, path: &Path) -> Result<String> {
        let mut output = String::new();

        match level {
            0 => {
                // Minimal - just essence
                output.push_str(&format!("• {}\n", m8.essence));
            }
            1 => {
                // Overview - essence + keywords
                output.push_str(&format!("📂 {} ({:.1}Hz)\n", path.display(), m8.frequency));
                output.push_str(&format!("  {}\n", m8.essence));
                output.push_str(&format!("  Keywords: {}\n", m8.keywords.join(", ")));
            }
            2 => {
                // Detailed - include children
                output.push_str(&format!("📂 {} ({:.1}Hz)\n", path.display(), m8.frequency));
                output.push_str(&format!("  📝 {}\n", m8.essence));
                output.push_str(&format!("  🏷️ Keywords: {}\n", m8.keywords.join(", ")));

                if !m8.children.is_empty() {
                    output.push_str("  📁 Children:\n");
                    for (child, freq) in &m8.children {
                        output.push_str(&format!("    • {} ({:.1}Hz)\n", child, freq));
                    }
                }
            }
            3 => {
                // Full detail - everything including triggers
                output.push_str(&format!("╭{}\n", "─".repeat(50)));
                output.push_str(&format!("│ 📂 {} \n", path.display()));
                output.push_str(&format!("│ 🌊 Frequency: {:.1}Hz\n", m8.frequency));
                output.push_str(&format!("│ 📝 {}\n", m8.essence));
                output.push_str(&format!("│ 🏷️ Keywords: {}\n", m8.keywords.join(", ")));

                if !m8.children.is_empty() {
                    output.push_str("│ 📁 Children:\n");
                    for (child, freq) in &m8.children {
                        output.push_str(&format!("│   • {} ({:.1}Hz)\n", child, freq));
                    }
                }

                if !m8.context_triggers.is_empty() {
                    output.push_str("│ 🎯 Context Triggers:\n");
                    for (trigger, expansion) in &m8.context_triggers {
                        output.push_str(&format!("│   {} → {}\n", trigger, expansion));
                    }
                }

                output.push_str(&format!("╰{}\n", "─".repeat(50)));
            }
            _ => output = format!("• {}\n", m8.essence),
        }

        Ok(output)
    }

    /// Auto-expand based on conversation context
    pub fn auto_expand(&mut self, root_path: &Path, keywords: &[String]) -> Result<Vec<String>> {
        let mut expansions = Vec::new();

        // Scan for .m8 files
        for entry in fs::read_dir(root_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("m8") {
                let content = self.load_contextual(&path, keywords)?;
                let m8 = self.load_m8(&path)?;

                // Check if we should drill down
                let relevance = self.calculate_relevance(&m8, keywords);
                if relevance > self.expansion_threshold {
                    expansions.push(content);

                    // Recursively expand highly relevant children
                    if relevance > 0.9 && path.is_dir() {
                        let child_expansions = self.auto_expand(&path, keywords)?;
                        expansions.extend(child_expansions);
                    }
                }
            }
        }

        Ok(expansions)
    }

    /// Load and cache .m8 file
    fn load_m8(&mut self, path: &Path) -> Result<ContextualM8> {
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        // For now, create a mock .m8 (would load real file)
        let m8 = if path.to_string_lossy().contains("8b.is") {
            ContextualM8 {
                frequency: 88.8,
                essence: "8b.is website - Company portal for 8-bit inspired AI services"
                    .to_string(),
                keywords: vec![
                    "8b.is".to_string(),
                    "website".to_string(),
                    "portal".to_string(),
                ],
                depth_level: 0,
                children: HashMap::from([
                    ("frontend".to_string(), 92.3),
                    ("api".to_string(), 87.5),
                    ("docs".to_string(), 45.2),
                ]),
                context_triggers: HashMap::from([
                    ("website".to_string(), "frontend/".to_string()),
                    ("API".to_string(), "api/".to_string()),
                    ("documentation".to_string(), "docs/".to_string()),
                ]),
            }
        } else if path.to_string_lossy().contains("smart-tree") {
            ContextualM8 {
                frequency: 42.73,
                essence: "Smart Tree - AI-optimized directory visualization with consciousness"
                    .to_string(),
                keywords: vec![
                    "smart-tree".to_string(),
                    "MCP".to_string(),
                    "consciousness".to_string(),
                ],
                depth_level: 0,
                children: HashMap::from([("src".to_string(), 87.2), ("docs".to_string(), 33.7)]),
                context_triggers: HashMap::from([
                    ("tokenizer".to_string(), "src/tokenizer.rs".to_string()),
                    ("memory".to_string(), "src/memory_manager.rs".to_string()),
                    (
                        "consciousness".to_string(),
                        "src/m8_consciousness.rs".to_string(),
                    ),
                ]),
            }
        } else {
            ContextualM8 {
                frequency: 50.0,
                essence: format!("Directory: {}", path.display()),
                keywords: vec![],
                depth_level: 0,
                children: HashMap::new(),
                context_triggers: HashMap::new(),
            }
        };

        self.cache.insert(path.to_path_buf(), m8.clone());
        Ok(m8)
    }

    /// Update context based on current conversation
    pub fn update_context(&mut self, keywords: Vec<String>) {
        self.current_context = keywords;
    }
}

/// Example usage showing progressive loading
pub fn demonstrate_context_awareness() -> Result<()> {
    let mut reader = ContextAwareReader::new();

    println!("🎯 Context-Aware .m8 Loading Demo\n");
    println!("{}\n", "=".repeat(60));

    // Scenario 1: No context - minimal loading
    println!("📍 No context (just browsing):");
    let minimal = reader.load_contextual(Path::new("/projects/smart-tree/.m8"), &[])?;
    println!("{}\n", minimal);

    // Scenario 2: Talking about websites - medium detail
    println!("💬 Context: 'website'");
    let website_context =
        reader.load_contextual(Path::new("/projects/8b.is/.m8"), &["website".to_string()])?;
    println!("{}\n", website_context);

    // Scenario 3: Talking about 8b.is specifically - full detail!
    println!("💬 Context: '8b.is website API'");
    let specific_context = reader.load_contextual(
        Path::new("/projects/8b.is/.m8"),
        &[
            "8b.is".to_string(),
            "website".to_string(),
            "API".to_string(),
        ],
    )?;
    println!("{}\n", specific_context);

    // Scenario 4: Auto-expansion based on triggers
    println!("🔍 Auto-expanding based on 'tokenizer' keyword:");
    let expansions = reader.auto_expand(
        Path::new("/projects/smart-tree"),
        &["tokenizer".to_string()],
    )?;
    for expansion in expansions {
        println!("{}", expansion);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_calculation() {
        let mut reader = ContextAwareReader::new();

        let m8 = ContextualM8 {
            frequency: 42.0,
            essence: "Smart Tree project".to_string(),
            keywords: vec!["tree".to_string(), "visualization".to_string()],
            depth_level: 0,
            children: HashMap::new(),
            context_triggers: HashMap::from([("tree".to_string(), "src/".to_string())]),
        };

        // High relevance
        let score = reader.calculate_relevance(&m8, &["tree".to_string()]);
        assert!(score > 0.9);

        // Medium relevance
        let score = reader.calculate_relevance(&m8, &["visualization".to_string()]);
        assert!(score > 0.5);

        // Low relevance
        let score = reader.calculate_relevance(&m8, &["random".to_string()]);
        assert!(score < 0.3);
    }
}
