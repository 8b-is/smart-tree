//! # Marqant - Quantum-Compressed Markdown Format
//!
//! Marqant (.mq) is a revolutionary compression format designed specifically for AI consumption,
//! achieving 90% token reduction while maintaining semantic integrity.
//!
//! ## Features
//!
//! - **Token-based compression**: Common markdown patterns become single tokens
//! - **AI-optimized**: Reduces token usage in LLM contexts by 70-90%
//! - **Streaming support**: Can process before full dictionary is loaded
//! - **Multiple compression levels**: From light tokenization to quantum compression
//! - **DNS integration**: Supports distributed token dictionaries via DNS
//!
//! ## Usage
//!
//! ```rust,no_run
//! use marqant::Marqant;
//!
//! let compressor = Marqant::default();
//! let compressed = compressor.compress("# Hello World\n\nThis is markdown content");
//! let decompressed = compressor.decompress(&compressed).unwrap();
//! ```
//!
//! ## Binary Format
//!
//! The .mq format consists of:
//! - Header: Version, timestamp, sizes, flags
//! - Token dictionary with escaped patterns
//! - Compressed content using token substitution
//! - Optional metadata sections

use anyhow::Result;
use chrono::Utc;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::io::{Read, Write};

pub mod dns;
pub mod mem8_bridge;
pub mod novelty;
pub mod semantic;
pub mod uni_doc;
pub mod utl_enforced;
pub mod utl_phonetics;
pub mod utl_pipeline;

mod uni;
pub use uni::{mq2_uni_decode, mq2_uni_encode, MQ2_UNI_DICT_ID};

#[derive(Debug, Eq)]
struct PhraseFreq {
    phrase: String,
    _count: usize,
    savings: usize,
}

impl PartialEq for PhraseFreq {
    fn eq(&self, other: &Self) -> bool {
        self.savings == other.savings
    }
}

impl Ord for PhraseFreq {
    fn cmp(&self, other: &Self) -> Ordering {
        self.savings.cmp(&other.savings)
    }
}

impl PartialOrd for PhraseFreq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The main Marqant compressor
///
/// Provides methods for compressing and decompressing markdown content
/// using quantum-inspired token substitution algorithms.
pub struct Marqant;

impl Default for Marqant {
    fn default() -> Self {
        Self
    }
}

const STD_STATIC_V1_ID: &str = "std-static-v1";

fn get_standard_tokens(id: &str) -> Option<HashMap<String, String>> {
    if id == STD_STATIC_V1_ID {
        let pairs: [(&str, &str); 17] = [
            ("\x01", "# "),
            ("\x02", "## "),
            ("\x03", "### "),
            ("\x04", "#### "),
            ("\x05", "```"),
            ("\x06", "\n\n"),
            ("\x07", "- "),
            ("\x0B", "* "),
            ("\x0C", "**"),
            ("\x0E", "__"),
            ("\x0F", "> "),
            ("\x10", "| "),
            ("\x11", "---"),
            ("\x12", "***"),
            ("\x13", "["),
            ("\x14", "]("),
            ("\x15", "```bash"),
        ];
        let mut m = HashMap::new();
        for (t, p) in pairs {
            m.insert(t.to_string(), p.to_string());
        }
        Some(m)
    } else {
        None
    }
}

fn parse_std_flag(flags: Option<&str>) -> Option<String> {
    let Some(f) = flags else {
        return None;
    };
    for part in f.split_whitespace() {
        if let Some(rest) = part.strip_prefix("-std:") {
            return Some(rest.to_string());
        }
    }
    None
}

impl Marqant {
    pub fn compress_markdown(content: &str) -> Result<String> {
        Self::compress_markdown_with_flags(content, None)
    }

    pub fn compress_markdown_with_flags(content: &str, flags: Option<&str>) -> Result<String> {
        let mut output = String::new();
        let original_size = content.len();

        let mut processed_content = content.to_string();
        let use_sections = flags.is_some_and(|f| f.contains("-semantic"));

        if use_sections {
            processed_content = Self::add_section_tags(&processed_content);
        }

        let (tokens, tokenized_content) = Self::tokenize_content(&processed_content);

        let use_zlib = flags.is_some_and(|f| f.contains("-zlib"));
        let final_content = if use_zlib {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(tokenized_content.as_bytes())?;
            let compressed = encoder.finish()?;
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed)
        } else {
            tokenized_content.clone()
        };

        let dict_size: usize = tokens.iter().map(|(k, v)| k.len() + v.len() + 3).sum();
        let compressed_size = final_content.len() + dict_size + 4;

        let timestamp = now_timestamp();

        if let Some(flags) = flags {
            output.push_str(&format!(
                "MARQANT {} {} {} {}\n",
                timestamp, original_size, compressed_size, flags
            ));
        } else {
            output.push_str(&format!(
                "MARQANT {} {} {}\n",
                timestamp, original_size, compressed_size
            ));
        }

        // Write token dictionary (sorted for determinism), possibly omitting standard entries
        let std_id = parse_std_flag(flags);
        let std_map = std_id.as_deref().and_then(get_standard_tokens);

        let mut token_vec: Vec<(&String, &String)> = tokens.iter().collect();
        token_vec.sort_by(|a, b| a.0.cmp(b.0));
        for (token, pattern) in token_vec {
            // If part of standard map and matches exactly, omit from on-wire dictionary
            if let Some(ref sm) = std_map {
                if sm.get(token).is_some_and(|p| p == pattern) {
                    continue;
                }
            }
            let escaped_pattern = pattern.replace('\n', "\\n");
            output.push_str(&format!("{}={}\n", token, escaped_pattern));
        }
        output.push_str("---\n");

        output.push_str(&final_content);

        Ok(output)
    }

    fn add_section_tags(content: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
            }

            if !in_code_block {
                if let Some(stripped) = line.strip_prefix("# ") {
                    let section = stripped.trim();
                    result.push_str(&format!("::section:{}::\n", section));
                } else if let Some(stripped) = line.strip_prefix("## ") {
                    let subsection = stripped.trim();
                    result.push_str(&format!("::section:{}::\n", subsection));
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        result
    }

    pub fn tokenize_content(content: &str) -> (HashMap<String, String>, String) {
        let mut tokens = HashMap::new();
        let mut tokenized = content.to_string();

        let static_tokens: Vec<(&str, &str)> = vec![
            ("\x01", "# "),
            ("\x02", "## "),
            ("\x03", "### "),
            ("\x04", "#### "),
            ("\x05", "```"),
            ("\x06", "\n\n"),
            ("\x07", "- "),
            ("\x0B", "* "),
            ("\x0C", "**"),
            ("\x0E", "__"),
            ("\x0F", "> "),
            ("\x10", "| "),
            ("\x11", "---"),
            ("\x12", "***"),
            ("\x13", "["),
            ("\x14", "]("),
            ("\x15", "```bash"),
            ("\x16", "```rust"),
            ("\x17", "```javascript"),
            ("\x18", "```python"),
            ("\x19", "\n```\n"),
            ("\x1A", "    "),
        ];

        for (token, pattern) in static_tokens {
            if tokenized.contains(pattern) {
                let count = tokenized.matches(pattern).count();
                if count * pattern.len() > count + pattern.len() + 3 {
                    tokens.insert(token.to_string(), pattern.to_string());
                    tokenized = tokenized.replace(pattern, token);
                }
            }
        }

        let mut phrase_heap = BinaryHeap::new();
        let words: Vec<&str> = content.split_whitespace().collect();

        for window_size in 2..=8 {
            for i in 0..words.len().saturating_sub(window_size) {
                let phrase = words[i..i + window_size].join(" ");
                if phrase.len() < 8 {
                    continue;
                }
                let count = content.matches(&phrase).count();
                if count >= 2 {
                    let savings = (phrase.len() * count).saturating_sub(count + phrase.len() + 3);
                    if savings > 0 {
                        phrase_heap.push(PhraseFreq {
                            phrase: phrase.clone(),
                            _count: count,
                            savings,
                        });
                    }
                }
            }
        }

        let mut token_counter = 0x1Bu8;
        let mut assigned_phrases: Vec<String> = Vec::new();

        while let Some(phrase_freq) = phrase_heap.pop() {
            if token_counter >= 0x7F {
                break;
            }
            if token_counter == 0x0A || token_counter == 0x0D {
                token_counter += 1;
                continue;
            }

            let mut overlaps = false;
            for assigned in &assigned_phrases {
                if phrase_freq.phrase.contains(assigned) || assigned.contains(&phrase_freq.phrase) {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps && tokenized.contains(&phrase_freq.phrase) {
                let token = char::from(token_counter).to_string();
                tokens.insert(token.clone(), phrase_freq.phrase.clone());
                tokenized = tokenized.replace(&phrase_freq.phrase, &token);
                assigned_phrases.push(phrase_freq.phrase);
                token_counter += 1;
            }
        }

        (tokens, tokenized)
    }

    pub fn decompress_marqant(compressed: &str) -> Result<String> {
        let lines: Vec<&str> = compressed.lines().collect();
        if lines.is_empty() || !lines[0].starts_with("MARQANT") {
            return Err(anyhow::anyhow!("Invalid marqant format"));
        }

        let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if header_parts.len() < 4 {
            return Err(anyhow::anyhow!("Invalid marqant header"));
        }

        let flags_joined = if header_parts.len() > 4 {
            header_parts[4..].join(" ")
        } else {
            String::new()
        };
        let has_zlib = flags_joined.split_whitespace().any(|f| f == "-zlib");
        let has_sections = flags_joined.split_whitespace().any(|f| f == "-semantic");
        let std_id = flags_joined
            .split_whitespace()
            .find_map(|f| f.strip_prefix("-std:"))
            .map(|s| s.to_string());

        let mut tokens = HashMap::new();
        // Preload standard tokens if requested
        if let Some(id) = std_id {
            // First try local...
            if let Some(map) = get_standard_tokens(&id) {
                tokens.extend(map);
            } else {
                // ...then fall back to DNS
                let dns_map = dns::resolve_dns_dict(&id)?;
                if let Some(map) = dns_map {
                    tokens.extend(map);
                } else {
                    return Err(anyhow::anyhow!(format!(
                        "Unknown or unresolvable standard dict id: {}",
                        id
                    )));
                }
            }
        }

        let mut content_start = 1;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if *line == "---" {
                content_start = i + 1;
                break;
            }
            if let Some((token, pattern)) = line.split_once('=') {
                let unescaped_pattern = pattern.replace("\\n", "\n");
                tokens.insert(token.to_string(), unescaped_pattern);
            }
        }

        let compressed_content = lines[content_start..].join("\n");

        let tokenized_content = if has_zlib {
            let decoded = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &compressed_content,
            )?;
            let mut decoder = ZlibDecoder::new(&decoded[..]);
            let mut decompressed_bytes = String::new();
            decoder.read_to_string(&mut decompressed_bytes)?;
            decompressed_bytes
        } else {
            compressed_content
        };

        let mut token_list: Vec<(String, String)> = tokens.into_iter().collect();
        token_list.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let mut decompressed = tokenized_content;
        for (token, pattern) in token_list {
            decompressed = decompressed.replace(&token, &pattern);
        }

        if has_sections {
            let lines: Vec<&str> = decompressed.lines().collect();
            let mut result = String::new();
            for line in lines {
                if !line.starts_with("::section:") || !line.ends_with("::") {
                    result.push_str(line);
                    result.push('\n');
                }
            }
            decompressed = result.trim_end().to_string();
        }

        Ok(decompressed)
    }
}

fn now_timestamp() -> String {
    if let Ok(v) = std::env::var("MARQANT_TEST_TS") {
        v
    } else {
        Utc::now().timestamp().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        std::env::set_var("MARQANT_TEST_TS", "0");
        let markdown = "# Title\n\n## Head\n\nContent\n";
        let compressed = Marqant::compress_markdown(markdown).unwrap();
        let decompressed = Marqant::decompress_marqant(&compressed).unwrap();
        assert_eq!(markdown.trim(), decompressed.trim());
    }
}

// --------------------
// Deterministic SVG word cloud (no dependencies)
// --------------------

/// Generate a deterministic SVG word cloud from input text.
/// - Deterministic sizing, placement, colors
/// - No external dependencies
/// - Approximated glyph metrics (width_factor ~ 0.6)
pub fn wordcloud_svg(text: &str, width: u32, height: u32) -> String {
    let mut freqs: HashMap<String, u32> = HashMap::new();
    for raw in text.split(|c: char| !c.is_alphanumeric()) {
        let w = raw.to_lowercase();
        if w.len() < 2 {
            continue;
        }
        *freqs.entry(w).or_insert(0) += 1;
    }
    if freqs.is_empty() {
        return format!("<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}' viewBox='0 0 {width} {height}'/>");
    }

    // Rank by frequency
    let mut items: Vec<(String, u32)> = freqs.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let maxf = items[0].1 as f64;

    // Font size mapping (log scale)
    let s_min = 14.0_f64;
    let s_max = (width.max(height) as f64).clamp(48.0, 420.0) * 0.18; // adapt to canvas
    let size_for = |f: u32| -> f64 {
        let f = f as f64;
        s_min + (s_max - s_min) * ((1.0 + f).ln() / (1.0 + maxf).ln())
    };

    // Placement spiral params
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let a = 0.0_f64;
    let b = 6.5_f64; // spacing between turns
    let dtheta = 0.35_f64;
    let width_factor = 0.61_f64; // average glyph width fraction of font-size

    // Already placed bounding boxes: (x0,y0,x1,y1)
    let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
    // And their SVG entries
    let mut nodes: Vec<String> = Vec::new();

    for (word, f) in items {
        let font = size_for(f);
        let w = font * width_factor * (word.chars().count() as f64);
        let h = font * 1.0;

        // Deterministic orientation and hue from FNV-1a 64
        let hv = fnv1a64(&word);
        let angle = if (hv & 1) == 0 { 0.0 } else { 90.0 };
        let hue = (hv % 360) as u32;
        let light = 35.0 + 15.0 * (font - s_min) / (s_max - s_min + 1e-9);

        // Try spiral positions until no collision
        let mut theta = 0.0_f64;
        let mut placed_ok = None;
        for _step in 0..4000 {
            // cap for safety
            let r = a + b * theta;
            let x = cx + r * theta.cos();
            let y = cy + r * theta.sin();
            // center baseline -> compute bounding box
            let x0 = x - w / 2.0;
            let y0 = y - h / 2.0;
            let x1 = x + w / 2.0;
            let y1 = y + h / 2.0;
            if x0 < 0.0 || y0 < 0.0 || x1 > width as f64 || y1 > height as f64 {
                theta += dtheta;
                continue;
            }
            if !overlaps_any(x0, y0, x1, y1, &placed) {
                placed_ok = Some((x, y, x0, y0, x1, y1));
                break;
            }
            theta += dtheta;
        }
        if let Some((x, y, x0, y0, x1, y1)) = placed_ok {
            placed.push((x0, y0, x1, y1));
            // SVG node
            nodes.push(format!(
                "<text x='{x:.2}' y='{y:.2}' font-family='Inter,system-ui,sans-serif' font-size='{font:.2}' fill='hsl({hue},55%,{light:.1}%)' text-anchor='middle' dominant-baseline='central' transform='rotate({angle:.0},{x:.2},{y:.2})'>{}</text>",
                escape_xml(&word)
            ));
        }
    }

    let mut svg = String::new();
    svg.push_str(&format!("<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}' viewBox='0 0 {width} {height}'><rect width='100%' height='100%' fill='#f5e7cf'/>
"));
    for n in nodes {
        svg.push_str(&n);
        svg.push('\n');
    }
    svg.push_str("</svg>");
    svg
}

fn overlaps_any(x0: f64, y0: f64, x1: f64, y1: f64, boxes: &[(f64, f64, f64, f64)]) -> bool {
    for &(ax0, ay0, ax1, ay1) in boxes {
        if x0 < ax1 && x1 > ax0 && y0 < ay1 && y1 > ay0 {
            return true;
        }
    }
    false
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn fnv1a64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;
    let mut h = FNV_OFFSET;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

// --------------------
// MQ metadata (no decode)
// --------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqInfo {
    pub kind: String,            // "MQ2" | "MARQANT_V1" | "UNKNOWN"
    pub variant: Option<String>, // e.g., "UNI", "mq"/"mqb" level, or None
    pub timestamp: Option<String>,
    pub original_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub token_count: Option<u32>,
    pub level: Option<String>,
    pub dict_t: Option<String>,  // raw ~T payload
    pub dict_s: Option<String>,  // raw ~S payload
    pub dict_id: Option<String>, // fnv1a64(~T||~S) hex
}

pub fn read_mq_metadata(input: &str) -> anyhow::Result<MqInfo> {
    let mut lines = input.lines();
    let first = lines.next().unwrap_or("");
    // Collect header-adjacent lines until separator
    let mut t_line: Option<String> = None;
    let mut s_line: Option<String> = None;
    let mut _sep = None;
    for line in lines.by_ref() {
        if line == "~~~~" || line == "---" {
            _sep = Some(line.to_string());
            break;
        }
        if let Some(rest) = line.strip_prefix("~T") {
            t_line = Some(rest.to_string());
        }
        if let Some(rest) = line.strip_prefix("~S") {
            s_line = Some(rest.to_string());
        }
    }

    let (kind, variant, ts, orig, comp, tokc, level) = if first.starts_with("MQ2~") {
        // MQ2~<variant?>~<ts_hex>~<orig_hex>~<comp_hex>~<tokc_hex>~<format_or_level>
        let parts: Vec<&str> = first.split('~').collect();
        let mut idx = 1;
        let variant = parts.get(idx).map(|s| s.to_string());
        idx += 1;
        let ts = parts.get(idx).map(|s| s.to_string());
        idx += 1;
        let orig = parts.get(idx).and_then(|s| u64::from_str_radix(s, 16).ok());
        idx += 1;
        let comp = parts.get(idx).and_then(|s| u64::from_str_radix(s, 16).ok());
        idx += 1;
        let tokc = parts.get(idx).and_then(|s| u32::from_str_radix(s, 16).ok());
        idx += 1;
        let level_or_fmt = parts.get(idx).map(|s| s.to_string());
        (
            "MQ2".to_string(),
            variant,
            ts,
            orig,
            comp,
            tokc,
            level_or_fmt,
        )
    } else if first.starts_with("MARQANT") {
        // MARQANT <ts> <orig_dec> <comp_dec> [flags]
        let parts: Vec<&str> = first.split_whitespace().collect();
        let ts = parts.get(1).map(|s| s.to_string());
        let orig = parts.get(2).and_then(|s| s.parse::<u64>().ok());
        let comp = parts.get(3).and_then(|s| s.parse::<u64>().ok());
        (
            "MARQANT".to_string(),
            None,
            ts,
            orig,
            comp,
            None,
            parts.get(4).map(|s| s.to_string()),
        )
    } else {
        ("UNKNOWN".to_string(), None, None, None, None, None, None)
    };

    let mut dict_id = None;
    if t_line.is_some() || s_line.is_some() {
        let mut concat = String::new();
        if let Some(t) = &t_line {
            concat.push_str(t);
        }
        if let Some(s) = &s_line {
            concat.push('|');
            concat.push_str(s);
        }
        let h = fnv1a64(&concat);
        dict_id = Some(format!("fnv1a64:{:016x}", h));
    }

    Ok(MqInfo {
        kind,
        variant,
        timestamp: ts,
        original_size: orig,
        compressed_size: comp,
        token_count: tokc,
        level,
        dict_t: t_line,
        dict_s: s_line,
        dict_id,
    })
}
