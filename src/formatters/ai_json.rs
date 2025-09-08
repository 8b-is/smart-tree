use super::{ai::AiFormatter, Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;

pub struct AiJsonFormatter {
    ai_formatter: AiFormatter,
}

impl AiJsonFormatter {
    pub fn new(no_emoji: bool, _path_mode: PathDisplayMode) -> Self {
        Self {
            ai_formatter: AiFormatter::new(no_emoji, _path_mode),
        }
    }
}

impl Formatter for AiJsonFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // First get the AI format output as a string
        let mut ai_output = Vec::new();
        self.ai_formatter
            .format(&mut ai_output, nodes, stats, root_path)?;
        let ai_text = String::from_utf8_lossy(&ai_output);

        // Parse the AI output to extract structured data
        let lines = ai_text.lines();
        let mut hex_tree_lines = Vec::new();
        let mut context = None;
        let mut hash = None;
        let mut stats_section = false;
        let mut file_count = 0u64;
        let mut dir_count = 0u64;
        let mut total_size = 0u64;
        let mut file_types = Vec::new();
        let mut large_files = Vec::new();
        let mut date_range = None;

        for line in lines {
            if line == "TREE_HEX_V1:" {
                continue;
            } else if line.starts_with("CONTEXT: ") {
                context = Some(line.strip_prefix("CONTEXT: ").unwrap_or("").to_string());
            } else if line.starts_with("HASH: ") {
                hash = Some(line.strip_prefix("HASH: ").unwrap_or("").to_string());
            } else if line == "STATS:" || line.is_empty() {
                stats_section = true;
            } else if line.starts_with("F:") && stats_section {
                // Parse stats line: F:45 D:12 S:23fc00 (2.3MB)
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(f) = parts.first().and_then(|s| s.strip_prefix("F:")) {
                    file_count = u64::from_str_radix(f, 16).unwrap_or(0);
                }
                if let Some(d) = parts.get(1).and_then(|s| s.strip_prefix("D:")) {
                    dir_count = u64::from_str_radix(d, 16).unwrap_or(0);
                }
                if let Some(s) = parts.get(2).and_then(|s| s.strip_prefix("S:")) {
                    total_size = u64::from_str_radix(s, 16).unwrap_or(0);
                }
            } else if line.starts_with("TYPES: ") && stats_section {
                let types_str = line.strip_prefix("TYPES: ").unwrap_or("");
                for type_entry in types_str.split_whitespace() {
                    if let Some((ext, count_hex)) = type_entry.split_once(':') {
                        if let Ok(count) = u64::from_str_radix(count_hex, 16) {
                            file_types.push(json!({
                                "extension": ext,
                                "count": count
                            }));
                        }
                    }
                }
            } else if line.starts_with("LARGE: ") && stats_section {
                let large_str = line.strip_prefix("LARGE: ").unwrap_or("");
                for file_entry in large_str.split_whitespace() {
                    if let Some((name, size_hex)) = file_entry.split_once(':') {
                        if let Ok(size) = u64::from_str_radix(size_hex, 16) {
                            large_files.push(json!({
                                "name": name,
                                "size": size
                            }));
                        }
                    }
                }
            } else if line.starts_with("DATES: ") && stats_section {
                date_range = Some(line.strip_prefix("DATES: ").unwrap_or("").to_string());
            } else if line == "END_AI" {
                break;
            } else if !stats_section && !line.is_empty() {
                // This is a hex tree line
                hex_tree_lines.push(line.to_string());
            }
        }

        // Build the JSON structure
        let mut json_output = json!({
            "version": "AI_JSON_V1",
            "hash": hash.unwrap_or_else(|| "unknown".to_string()),
            "hex_tree": hex_tree_lines,
            "statistics": {
                "files": file_count,
                "directories": dir_count,
                "total_size": total_size,
                "total_size_mb": format!("{:.1}", total_size as f64 / (1024.0 * 1024.0))
            }
        });

        // Add optional fields
        if let Some(ctx) = context {
            json_output["context"] = Value::String(ctx);
        }

        if !file_types.is_empty() {
            json_output["statistics"]["file_types"] = Value::Array(file_types);
        }

        if !large_files.is_empty() {
            json_output["statistics"]["largest_files"] = Value::Array(large_files);
        }

        if let Some(dates) = date_range {
            json_output["statistics"]["date_range"] = Value::String(dates);
        }

        // Write the JSON output
        writeln!(writer, "{}", serde_json::to_string_pretty(&json_output)?)?;

        Ok(())
    }
}
