//! Project context detection module — Zero-DOM NAND SWAR + Exact `p/q` Edition
//!
//! Replaces heavyweight `toml::from_str` / `serde_json::from_str` DOM allocation and
//! double-syscall (`exists()` + `read_to_string()`) probes with:
//! 1. **Single-syscall bounded 4 KiB stack reads** (`File::open` + `Read::read`).
//! 2. **Zero-allocation TOML (`[package]`, `[project]`, `[tool.poetry]`) & top-level JSON scanners.**
//! 3. **64-bit NAND SWAR + hardware `POPCNT` UTF-8 scalar boundary detection** via
//!    [`crate::felix_asm::felix_utf8_lead_mask_nand64`] and [`crate::felix_asm::felix_count_utf8_leads_nand64`].
//! 4. **Exact `p/q` rational truncation budgets (`PqBudget`)** reduced by [`crate::felix_asm::felix_binary_gcd_u64`].

use crate::felix_asm::{
    felix_binary_gcd_u64, felix_count_utf8_leads_nand64, felix_utf8_lead_mask_nand64,
};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Maximum bytes read from any manifest or README header (1 page = 4096 B, stack-allocated).
pub const MAX_PROBE_BYTES: usize = 4096;

/// Exact rational `p/q` truncation contract (`p` = retained scalar cap, `q` = stride/divisor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PqBudget {
    pub p: usize,
    pub q: usize,
}

impl PqBudget {
    /// 80-scalar manifest description budget (`80/1`, `77/80` payload-to-frame ratio).
    pub const MANIFEST_80: Self = Self { p: 80, q: 1 };
    /// 100-scalar README summary budget (`100/1`, `97/100` payload-to-frame ratio).
    pub const README_100: Self = Self { p: 100, q: 1 };
    /// 3-scalar ellipsis overhead (`3/1` for `"..."`).
    pub const ELLIPSIS_COST: Self = Self { p: 3, q: 1 };

    /// Exact integer threshold `⌊p / q⌋` without floating-point division.
    #[inline(always)]
    pub const fn max_scalars(self) -> usize {
        self.p / self.q
    }

    /// Exact reduced rational retention ratio `(kept_p, total_q)` after truncation.
    #[inline(always)]
    pub fn retention_pq(self, total_scalars: usize) -> (usize, usize) {
        if total_scalars == 0 {
            return (1, 1);
        }
        let max_len = self.max_scalars();
        let kept = if total_scalars <= max_len {
            total_scalars
        } else {
            max_len.saturating_sub(Self::ELLIPSIS_COST.p)
        };
        let g = felix_binary_gcd_u64(kept as u64, total_scalars as u64).max(1) as usize;
        (kept / g, total_scalars / g)
    }
}

/// Single-pass UTF-8 byte boundary finder using 64-bit NAND SWAR + hardware `POPCNT`.
///
/// Returns `(total_exceeds_max, byte_offset_for_cut)` where `byte_offset_for_cut`
/// is the exact byte index of the `(max_len - 3)`-th UTF-8 scalar.
#[inline]
pub fn utf8_truncate_plan_nand(s: &str, budget: PqBudget) -> (bool, usize) {
    let bytes = s.as_bytes();
    let max_len = budget.max_scalars();

    if bytes.len() <= max_len {
        return (false, bytes.len());
    }

    let keep_scalars = max_len.saturating_sub(PqBudget::ELLIPSIS_COST.p);
    let mut scalar_count: usize = 0;
    let mut cut_byte: usize = bytes.len();
    let mut cut_found = false;
    let mut i: usize = 0;

    while i + 8 <= bytes.len() {
        let mut chunk = [0u8; 8];
        chunk.copy_from_slice(&bytes[i..i + 8]);
        let word = u64::from_le_bytes(chunk);
        let mask = felix_utf8_lead_mask_nand64(word);
        let leads = felix_count_utf8_leads_nand64(word) as usize;

        if !cut_found && scalar_count + leads > keep_scalars {
            for lane in 0..8 {
                if ((mask >> (lane * 8)) & 0x80) != 0 {
                    if scalar_count == keep_scalars {
                        cut_byte = i + lane;
                        cut_found = true;
                        break;
                    }
                    scalar_count += 1;
                }
            }
            let remaining_start = (cut_byte - i) + 1;
            for lane in remaining_start..8 {
                if ((mask >> (lane * 8)) & 0x80) != 0 {
                    scalar_count += 1;
                }
            }
        } else {
            scalar_count += leads;
        }

        i += 8;
        if cut_found && scalar_count > max_len {
            return (true, cut_byte);
        }
    }

    while i < bytes.len() {
        let b = bytes[i] as u64;
        let is_lead = (felix_utf8_lead_mask_nand64(b) & 0x80) != 0;
        if is_lead {
            if !cut_found && scalar_count == keep_scalars {
                cut_byte = i;
                cut_found = true;
            }
            scalar_count += 1;
            if cut_found && scalar_count > max_len {
                return (true, cut_byte);
            }
        }
        i += 1;
    }

    (scalar_count > max_len, cut_byte)
}

/// Truncates `s` to `max_len` UTF-8 scalars (with `"..."` suffix when truncated)
/// in a single pass and a single heap allocation.
#[inline]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    let (needs_ellipsis, cut_idx) = utf8_truncate_plan_nand(s, PqBudget { p: max_len, q: 1 });
    if !needs_ellipsis {
        s.to_string()
    } else {
        let mut out = String::with_capacity(cut_idx + 3);
        out.push_str(&s[..cut_idx]);
        out.push_str("...");
        out
    }
}

fn format_labeled_truncated(
    prefix: &str,
    name: Option<&str>,
    desc: &str,
    budget: PqBudget,
) -> String {
    let (needs_ellipsis, cut_idx) = utf8_truncate_plan_nand(desc, budget);
    let slice = &desc[..if needs_ellipsis { cut_idx } else { desc.len() }];
    let extra = if needs_ellipsis { 3 } else { 0 };

    match name {
        Some(n) => {
            let mut out =
                String::with_capacity(prefix.len() + 2 + n.len() + 3 + slice.len() + extra);
            out.push_str(prefix);
            out.push_str(": ");
            out.push_str(n);
            out.push_str(" - ");
            out.push_str(slice);
            if needs_ellipsis {
                out.push_str("...");
            }
            out
        }
        None => {
            let mut out = String::with_capacity(prefix.len() + 2 + slice.len() + extra);
            out.push_str(prefix);
            out.push_str(": ");
            out.push_str(slice);
            if needs_ellipsis {
                out.push_str("...");
            }
            out
        }
    }
}

#[inline]
fn read_head_stack<'a>(
    root_path: &Path,
    rel: &str,
    buf: &'a mut [u8; MAX_PROBE_BYTES],
) -> Option<&'a str> {
    let mut file = File::open(root_path.join(rel)).ok()?;
    let mut total = 0usize;
    while total < MAX_PROBE_BYTES {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return None,
        }
    }
    match std::str::from_utf8(&buf[..total]) {
        Ok(s) => Some(s),
        Err(e) => std::str::from_utf8(&buf[..e.valid_up_to()]).ok(),
    }
}

fn scan_toml_section<'a>(
    content: &'a str,
    target_section: &str,
) -> (Option<&'a str>, Option<&'a str>) {
    let mut in_section = false;
    let mut name: Option<&'a str> = None;
    let mut desc: Option<&'a str> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let sec_part = rest.split('#').next().unwrap_or(rest).trim_end();
            if let Some(sec) = sec_part.strip_suffix(']') {
                if in_section {
                    break;
                }
                in_section = sec.trim() == target_section;
            }
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = unquote_toml_or_json(v.trim());
            if key == "name" && name.is_none() {
                name = val;
            } else if key == "description" && desc.is_none() {
                desc = val;
            }
            if name.is_some() && desc.is_some() {
                break;
            }
        }
    }
    (name, desc)
}

#[inline]
fn unquote_toml_or_json(v: &str) -> Option<&str> {
    let bytes = v.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let mut i = 1usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return Some(&v[1..i]);
        }
        i += 1;
    }
    None
}

fn scan_package_json_top<'a>(content: &'a str) -> (Option<&'a str>, bool, Option<&'a str>) {
    let bytes = content.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0usize;
    let mut name: Option<&'a str> = None;
    let mut has_desc_key = false;
    let mut desc: Option<&'a str> = None;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'"' if depth == 1 => {
                let Some((key, next_i)) = parse_json_str(content, i) else {
                    break;
                };
                i = skip_ws(bytes, next_i);
                if i < bytes.len() && bytes[i] == b':' {
                    i = skip_ws(bytes, i + 1);
                    if key == "name" || key == "description" {
                        if key == "description" {
                            has_desc_key = true;
                        }
                        if i < bytes.len() && bytes[i] == b'"' {
                            if let Some((val, after_val)) = parse_json_str(content, i) {
                                if key == "name" && name.is_none() {
                                    name = Some(val);
                                } else if key == "description" && desc.is_none() {
                                    desc = Some(val);
                                }
                                i = after_val;
                                if name.is_some() && has_desc_key {
                                    break;
                                }
                                continue;
                            }
                        }
                    }
                }
            }
            b'"' => {
                if let Some((_, next_i)) = parse_json_str(content, i) {
                    i = next_i;
                } else {
                    break;
                }
            }
            _ => i += 1,
        }
    }
    (name, has_desc_key, desc)
}

#[inline]
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    i
}

#[inline]
fn parse_json_str(s: &str, start_quote: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if bytes.get(start_quote).copied() != Some(b'"') {
        return None;
    }
    let mut i = start_quote + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some((&s[start_quote + 1..i], i + 1));
        }
        i += 1;
    }
    None
}

/// Attempts to detect the project type and description using a single stack buffer.
pub fn detect_project_context(root_path: &Path) -> Option<String> {
    let mut scratch = [0u8; MAX_PROBE_BYTES];

    read_cargo_toml(root_path, &mut scratch)
        .or_else(|| read_package_json(root_path, &mut scratch))
        .or_else(|| read_pyproject_toml(root_path, &mut scratch))
        .or_else(|| read_go_mod(root_path, &mut scratch))
        .or_else(|| read_git_description(root_path, &mut scratch))
        .or_else(|| read_readme(root_path, &mut scratch))
}

fn read_cargo_toml(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    let content = read_head_stack(root_path, "Cargo.toml", buf)?;
    let (name, desc) = scan_toml_section(content, "package");
    Some(format_labeled_truncated(
        "Rust",
        Some(name?),
        desc?,
        PqBudget::MANIFEST_80,
    ))
}

fn read_package_json(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    let content = read_head_stack(root_path, "package.json", buf)?;
    let (name, has_desc_key, desc) = scan_package_json_top(content);
    if !has_desc_key {
        return None;
    }
    let desc_str = desc.unwrap_or("No description");
    Some(format_labeled_truncated(
        "Node",
        Some(name?),
        desc_str,
        PqBudget::MANIFEST_80,
    ))
}

fn read_pyproject_toml(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    let content = read_head_stack(root_path, "pyproject.toml", buf)?;

    let (proj_name, proj_desc) = scan_toml_section(content, "project");
    if let (Some(name), Some(desc)) = (proj_name, proj_desc) {
        return Some(format_labeled_truncated(
            "Python",
            Some(name),
            desc,
            PqBudget::MANIFEST_80,
        ));
    }

    let (poetry_name, poetry_desc) = scan_toml_section(content, "tool.poetry");
    if let (Some(name), Some(desc)) = (poetry_name, poetry_desc) {
        return Some(format_labeled_truncated(
            "Python",
            Some(name),
            desc,
            PqBudget::MANIFEST_80,
        ));
    }

    None
}

fn read_go_mod(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    let content = read_head_stack(root_path, "go.mod", buf)?;
    let first_line = content.lines().next()?;
    let module_name = first_line.strip_prefix("module ")?.trim();
    let mut out = String::with_capacity(4 + module_name.len());
    out.push_str("Go: ");
    out.push_str(module_name);
    Some(out)
}

fn read_git_description(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    let content = read_head_stack(root_path, ".git/description", buf)?;
    let desc = content.trim();
    if desc.is_empty() || desc.contains("Unnamed repository") {
        return None;
    }
    Some(format_labeled_truncated(
        "Git",
        None,
        desc,
        PqBudget::MANIFEST_80,
    ))
}

fn read_readme(root_path: &Path, buf: &mut [u8; MAX_PROBE_BYTES]) -> Option<String> {
    const README_NAMES: [&str; 5] = [
        "README.md",
        "README.MD",
        "readme.md",
        "README",
        "README.txt",
    ];

    for &name in &README_NAMES {
        let Some(content) = read_head_stack(root_path, name, buf) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Some(truncate_string(trimmed, PqBudget::README_100.max_scalars()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_nand_utf8_lead_counter_matches_char_count() {
        let long_ascii = "a".repeat(250);
        let long_crabs = "🦀".repeat(95);
        let samples = [
            "ASCII only 1234567890",
            "全 · 8b-is Sovereign Mesh ⊕◯⊖ Balanced Ternary 🦀🌳",
            long_ascii.as_str(),
            long_crabs.as_str(),
        ];

        for s in samples {
            let bytes = s.as_bytes();
            let mut count = 0usize;
            let mut i = 0usize;
            while i + 8 <= bytes.len() {
                let mut w = [0u8; 8];
                w.copy_from_slice(&bytes[i..i + 8]);
                count += felix_count_utf8_leads_nand64(u64::from_le_bytes(w)) as usize;
                i += 8;
            }
            while i < bytes.len() {
                if (felix_utf8_lead_mask_nand64(bytes[i] as u64) & 0x80) != 0 {
                    count += 1;
                }
                i += 1;
            }
            assert_eq!(count, s.chars().count());
        }
    }

    #[test]
    fn test_pq_budget_retention_and_truncation_multibyte() {
        assert_eq!(PqBudget::MANIFEST_80.retention_pq(100), (77, 100));
        assert_eq!(PqBudget::MANIFEST_80.retention_pq(40), (1, 1));

        let crabs = "🦀".repeat(85);
        let t = truncate_string(&crabs, 80);
        assert_eq!(t.chars().count(), 80);
        assert!(t.ends_with("..."));
    }

    #[test]
    fn test_detect_project_context_priority_order() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("st_ctx_test_{nonce}"));
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("README.md"), "# Header\n\nSmart Tree AI context.\n").unwrap();
        assert_eq!(
            detect_project_context(&dir).as_deref(),
            Some("Smart Tree AI context.")
        );

        fs::write(dir.join("go.mod"), "module github.com/8b-is/st-go\n").unwrap();
        assert_eq!(
            detect_project_context(&dir).as_deref(),
            Some("Go: github.com/8b-is/st-go")
        );

        fs::write(
            dir.join("pyproject.toml"),
            "[project]\nname = \"engram\"\ndescription = \"Lemire p/q lookup\"\n",
        )
        .unwrap();
        assert_eq!(
            detect_project_context(&dir).as_deref(),
            Some("Python: engram - Lemire p/q lookup")
        );

        fs::write(
            dir.join("package.json"),
            r#"{"name": "st-web", "description": "Tree viewer"}"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_context(&dir).as_deref(),
            Some("Node: st-web - Tree viewer")
        );

        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"smart-tree\"\ndescription = \"AI directory visualization\"\n",
        )
        .unwrap();
        assert_eq!(
            detect_project_context(&dir).as_deref(),
            Some("Rust: smart-tree - AI directory visualization")
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
