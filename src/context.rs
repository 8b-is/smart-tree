//! Project context detection module
//!
//! # Hot-path shape
//!
//! This module is **syscall- and allocation-bound, not ALU-bound**: the dominant costs are
//! `open`/`read` of up to six candidate files per call, plus TOML/JSON parsing of the one
//! file that matches. Inline assembly cannot shorten a syscall, so the wins here come from
//! removing redundant metadata probes and unbounded reads:
//!
//! 1. `path.exists()` + `read_to_string` became a single `read_to_string`/`open`
//!    (one syscall instead of two per candidate, and no check-to-use window).
//! 2. `read_go_mod` reads one line instead of the whole file.
//! 3. `read_readme` streams lines and stops at the first usable one instead of reading and
//!    UTF-8-validating an entire README (which may be megabytes).
//! 4. The only CPU-bound kernel, `truncate_string`, delegates to
//!    [`crate::felix_asm::felix_truncate_chars`], which classifies the prefix with the
//!    bounded SIMD ASCII scan instead of decoding the whole string twice.

use crate::felix_asm::felix_truncate_chars;
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Attempts to detect the project type and description
pub fn detect_project_context(root_path: &Path) -> Option<String> {
    // Candidate order is the documented preference order; each probe runs only when every
    // earlier probe returned `None`.
    read_cargo_toml(root_path)
        .or_else(|| read_package_json(root_path))
        .or_else(|| read_pyproject_toml(root_path))
        .or_else(|| read_go_mod(root_path))
        .or_else(|| read_git_description(root_path))
        .or_else(|| read_readme(root_path))
}

/// Manifest parsing needs the complete document, so this keeps a full read — but the
/// preceding `exists()` probe (an extra `stat`) is gone: a single `open` now answers both
/// "is it there?" and "what is in it?".
fn read_cargo_toml(root_path: &Path) -> Option<String> {
    let content = fs::read_to_string(root_path.join("Cargo.toml")).ok()?;
    let toml: toml::Value = toml::from_str(&content).ok()?;

    let package = toml.get("package")?;
    let name = package.get("name")?.as_str()?;
    let desc = package.get("description")?.as_str()?;

    Some(format!("Rust: {} - {}", name, truncate_string(desc, 80)))
}

fn read_package_json(root_path: &Path) -> Option<String> {
    let content = fs::read_to_string(root_path.join("package.json")).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    let name = json.get("name")?.as_str()?;
    // NOTE: preserved behavior. The `?` makes a package with no `description` key fall
    // through to the next detector; `unwrap_or` only covers a present-but-non-string
    // value. Changing that `?` would be a behavior fix, not an optimization.
    let desc = json.get("description")?.as_str().unwrap_or("No description");

    Some(format!("Node: {} - {}", name, truncate_string(desc, 80)))
}

fn read_pyproject_toml(root_path: &Path) -> Option<String> {
    let content = fs::read_to_string(root_path.join("pyproject.toml")).ok()?;
    let toml: toml::Value = toml::from_str(&content).ok()?;

    // Try both [project] and [tool.poetry] sections
    if let Some(project) = toml.get("project") {
        let name = project.get("name")?.as_str()?;
        let desc = project
            .get("description")?
            .as_str()
            .unwrap_or("No description");
        return Some(format!("Python: {} - {}", name, truncate_string(desc, 80)));
    } else if let Some(tool) = toml.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            let name = poetry.get("name")?.as_str()?;
            let desc = poetry
                .get("description")?
                .as_str()
                .unwrap_or("No description");
            return Some(format!("Python: {} - {}", name, truncate_string(desc, 80)));
        }
    }

    None
}

/// Only `module` on the first line is used, so exactly one buffered line is read instead of
/// reading and UTF-8-validating the whole file. A later invalid byte in `go.mod` no longer
/// suppresses detection; the first line's encoding is still strictly validated.
fn read_go_mod(root_path: &Path) -> Option<String> {
    let file = fs::File::open(root_path.join("go.mod")).ok()?;

    let mut first_line = String::new();
    BufReader::new(file).read_line(&mut first_line).ok()?;

    // Match `str::lines()`: drop the line terminator, including CRLF.
    let line = first_line.strip_suffix('\n').unwrap_or(&first_line);
    let line = line.strip_suffix('\r').unwrap_or(line);

    let module_name = line.strip_prefix("module ")?.trim();
    Some(format!("Go: {}", module_name))
}

fn read_git_description(root_path: &Path) -> Option<String> {
    // `description` is required to be tiny, so the whole-file read stays; the `exists()`
    // probe does not.
    let content = fs::read_to_string(root_path.join(".git/description")).ok()?;
    let desc = content.trim();

    // Skip the default git description
    if desc.contains("Unnamed repository") {
        return None;
    }

    Some(format!("Git: {}", truncate_string(desc, 80)))
}

/// Streams each candidate until the first usable line, so a multi-megabyte README costs one
/// buffered read rather than a full read plus whole-file UTF-8 validation.
///
/// Preserved quirk: an unreadable candidate (permissions, a directory, invalid UTF-8 before
/// any usable line) still stops the search rather than trying later names, because the
/// original used `?` on the whole-file read. Only genuinely missing files (`NotFound`) fall
/// through to the next candidate name.
fn read_readme(root_path: &Path) -> Option<String> {
    // Try various README filenames
    const README_NAMES: [&str; 5] = [
        "README.md",
        "README.MD",
        "readme.md",
        "README",
        "README.txt",
    ];

    for name in README_NAMES {
        let file = match fs::File::open(root_path.join(name)) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return None,
        };

        for line in BufReader::new(file).lines() {
            let line = line.ok()?;
            let trimmed = line.trim();
            // Skip empty lines and markdown headers
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Some(truncate_string(trimmed, 100));
            }
        }
    }

    None
}

/// Kept as the module-local name; the exact scalar semantics live in the FELIX hot-path
/// engine so the SIMD ASCII fast path is shared instead of duplicated.
#[inline]
fn truncate_string(s: &str, max_len: usize) -> String {
    felix_truncate_chars(s, max_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct Fixture {
        _dir: TempDir,
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("create temp dir");
            let root = dir.path().to_path_buf();
            Self { _dir: dir, root }
        }

        fn write(&self, name: &str, body: &str) -> &Self {
            let path = self.root.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::write(&path, body).expect("write fixture file");
            self
        }

        fn detect(&self) -> Option<String> {
            detect_project_context(&self.root)
        }
    }

    /// The original implementation, kept verbatim as the behavior oracle.
    mod legacy {
        use serde_json::Value;
        use std::fs;
        use std::path::Path;

        pub fn detect(root_path: &Path) -> Option<String> {
            if let Some(c) = read_cargo_toml(root_path) {
                return Some(c);
            }
            if let Some(c) = read_package_json(root_path) {
                return Some(c);
            }
            if let Some(c) = read_pyproject_toml(root_path) {
                return Some(c);
            }
            if let Some(c) = read_go_mod(root_path) {
                return Some(c);
            }
            if let Some(c) = read_git_description(root_path) {
                return Some(c);
            }
            read_readme(root_path)
        }

        fn read_cargo_toml(root_path: &Path) -> Option<String> {
            let path = root_path.join("Cargo.toml");
            if !path.exists() {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let toml: toml::Value = toml::from_str(&content).ok()?;
            let package = toml.get("package")?;
            let name = package.get("name")?.as_str()?;
            let desc = package.get("description")?.as_str()?;
            Some(format!("Rust: {} - {}", name, truncate(desc, 80)))
        }

        fn read_package_json(root_path: &Path) -> Option<String> {
            let path = root_path.join("package.json");
            if !path.exists() {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let json: Value = serde_json::from_str(&content).ok()?;
            let name = json.get("name")?.as_str()?;
            let desc = json.get("description")?.as_str().unwrap_or("No description");
            Some(format!("Node: {} - {}", name, truncate(desc, 80)))
        }

        fn read_pyproject_toml(root_path: &Path) -> Option<String> {
            let path = root_path.join("pyproject.toml");
            if !path.exists() {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let toml: toml::Value = toml::from_str(&content).ok()?;
            if let Some(project) = toml.get("project") {
                let name = project.get("name")?.as_str()?;
                let desc = project
                    .get("description")?
                    .as_str()
                    .unwrap_or("No description");
                return Some(format!("Python: {} - {}", name, truncate(desc, 80)));
            } else if let Some(tool) = toml.get("tool") {
                if let Some(poetry) = tool.get("poetry") {
                    let name = poetry.get("name")?.as_str()?;
                    let desc = poetry
                        .get("description")?
                        .as_str()
                        .unwrap_or("No description");
                    return Some(format!("Python: {} - {}", name, truncate(desc, 80)));
                }
            }
            None
        }

        fn read_go_mod(root_path: &Path) -> Option<String> {
            let path = root_path.join("go.mod");
            if !path.exists() {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let first_line = content.lines().next()?;
            if first_line.starts_with("module ") {
                let module_name = first_line.strip_prefix("module ")?.trim();
                return Some(format!("Go: {}", module_name));
            }
            None
        }

        fn read_git_description(root_path: &Path) -> Option<String> {
            let path = root_path.join(".git/description");
            if !path.exists() {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let desc = content.trim();
            if desc.contains("Unnamed repository") {
                return None;
            }
            Some(format!("Git: {}", truncate(desc, 80)))
        }

        fn read_readme(root_path: &Path) -> Option<String> {
            let names = [
                "README.md",
                "README.MD",
                "readme.md",
                "README",
                "README.txt",
            ];
            for name in &names {
                let path = root_path.join(name);
                if path.exists() {
                    let content = fs::read_to_string(&path).ok()?;
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with('#') {
                            return Some(truncate(trimmed, 100));
                        }
                    }
                }
            }
            None
        }

        fn truncate(s: &str, max_len: usize) -> String {
            if s.chars().count() <= max_len {
                s.to_string()
            } else {
                let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
                format!("{}...", truncated)
            }
        }
    }

    fn assert_parity(fixture: &Fixture) {
        assert_eq!(
            fixture.detect(),
            legacy::detect(&fixture.root),
            "new and legacy detection disagree for {:?}",
            fixture.root
        );
    }

    #[test]
    fn test_cargo_toml_detection_matches_legacy() {
        let fixture = Fixture::new().write(
            "Cargo.toml",
            "[package]\nname = \"demo\"\ndescription = \"A tiny demo crate\"\n",
        );
        assert_eq!(
            fixture.detect().as_deref(),
            Some("Rust: demo - A tiny demo crate")
        );
        assert_parity(&fixture);
    }

    #[test]
    fn test_cargo_toml_long_and_unicode_description_truncation() {
        let long = Fixture::new().write(
            "Cargo.toml",
            &format!(
                "[package]\nname = \"long\"\ndescription = \"{}\"\n",
                "d".repeat(200)
            ),
        );
        let detected = long.detect().expect("long description detected");
        assert_eq!(detected, format!("Rust: long - {}...", "d".repeat(77)));
        assert_parity(&long);

        let unicode = Fixture::new().write(
            "Cargo.toml",
            "[package]\nname = \"uni\"\ndescription = \"日本語の説明 😀 café\"\n",
        );
        assert_eq!(
            unicode.detect().as_deref(),
            Some("Rust: uni - 日本語の説明 😀 café")
        );
        assert_parity(&unicode);
    }

    #[test]
    fn test_invalid_or_incomplete_manifests_fall_through() {
        // No [package] table at all.
        let workspace_only = Fixture::new()
            .write("Cargo.toml", "[workspace]\nmembers = [\"a\"]\n")
            .write("package.json", "{\"name\":\"fallback\",\"description\":\"d\"}");
        assert_eq!(workspace_only.detect().as_deref(), Some("Node: fallback - d"));
        assert_parity(&workspace_only);

        // Malformed TOML.
        let broken = Fixture::new()
            .write("Cargo.toml", "this is not = = toml")
            .write("package.json", "{\"name\":\"n2\",\"description\":\"d2\"}");
        assert_eq!(broken.detect().as_deref(), Some("Node: n2 - d2"));
        assert_parity(&broken);
    }

    #[test]
    fn test_package_json_description_quirk_and_numeric_description() {
        // Preserved quirk: a missing description key falls through instead of reporting
        // "No description".
        let missing = Fixture::new()
            .write("package.json", "{\"name\":\"nodesc\"}")
            .write("README.md", "Plain readme fallback\n");
        assert_eq!(missing.detect().as_deref(), Some("Plain readme fallback"));
        assert_parity(&missing);

        // A present-but-non-string description does use the fallback text.
        let numeric = Fixture::new().write("package.json", "{\"name\":\"num\",\"description\":42}");
        assert_eq!(
            numeric.detect().as_deref(),
            Some("Node: num - No description")
        );
        assert_parity(&numeric);
    }

    #[test]
    fn test_pyproject_sections() {
        let project = Fixture::new().write(
            "pyproject.toml",
            "[project]\nname = \"pep621\"\ndescription = \"modern\"\n",
        );
        assert_eq!(project.detect().as_deref(), Some("Python: pep621 - modern"));
        assert_parity(&project);

        let poetry = Fixture::new().write(
            "pyproject.toml",
            "[tool.poetry]\nname = \"poetic\"\ndescription = \"old style\"\n",
        );
        assert_eq!(poetry.detect().as_deref(), Some("Python: poetic - old style"));
        assert_parity(&poetry);

        let neither = Fixture::new()
            .write("pyproject.toml", "[build-system]\nrequires = []\n")
            .write("go.mod", "module example.com/fallback\n");
        assert_eq!(neither.detect().as_deref(), Some("Go: example.com/fallback"));
        assert_parity(&neither);
    }

    #[test]
    fn test_go_mod_first_line_only() {
        let lf = Fixture::new().write("go.mod", "module example.com/lf\n\ngo 1.22\n");
        assert_eq!(lf.detect().as_deref(), Some("Go: example.com/lf"));
        assert_parity(&lf);

        let crlf = Fixture::new().write("go.mod", "module example.com/crlf\r\ngo 1.22\r\n");
        assert_eq!(crlf.detect().as_deref(), Some("Go: example.com/crlf"));
        assert_parity(&crlf);

        let no_terminator = Fixture::new().write("go.mod", "module example.com/bare");
        assert_eq!(
            no_terminator.detect().as_deref(),
            Some("Go: example.com/bare")
        );
        assert_parity(&no_terminator);

        let not_a_module = Fixture::new()
            .write("go.mod", "// comment\ngo 1.22\n")
            .write("README.md", "# Title\n\nActual text\n");
        assert_eq!(not_a_module.detect().as_deref(), Some("Actual text"));
        assert_parity(&not_a_module);

        let empty = Fixture::new().write("go.mod", "");
        assert_eq!(empty.detect(), None);
        assert_parity(&empty);
    }

    #[test]
    fn test_git_description_default_is_skipped() {
        let custom = Fixture::new().write(".git/description", "  A custom repo blurb  \n");
        assert_eq!(custom.detect().as_deref(), Some("Git: A custom repo blurb"));
        assert_parity(&custom);

        let default = Fixture::new()
            .write(
                ".git/description",
                "Unnamed repository; edit this file 'description' to name the repository.\n",
            )
            .write("README.md", "Real docs live here\n");
        assert_eq!(default.detect().as_deref(), Some("Real docs live here"));
        assert_parity(&default);
    }

    #[test]
    fn test_readme_selection_truncation_and_streaming() {
        let headers_then_text = Fixture::new().write(
            "README.md",
            "# Project\n\n## Section\n\nFirst real line\nSecond\n",
        );
        assert_eq!(
            headers_then_text.detect().as_deref(),
            Some("First real line")
        );
        assert_parity(&headers_then_text);

        let long_line = Fixture::new().write("README.md", &format!("{}\n", "w".repeat(400)));
        let expected = format!("{}...", "w".repeat(97));
        assert_eq!(long_line.detect().as_deref(), Some(expected.as_str()));
        assert_parity(&long_line);

        let unicode_line = Fixture::new().write(
            "README.md",
            "# t\n\n日本語 😀 line that is short\n",
        );
        assert_eq!(
            unicode_line.detect().as_deref(),
            Some("日本語 😀 line that is short")
        );
        assert_parity(&unicode_line);

        // Headers only in README.md must fall through to README.txt, as before.
        let multi = Fixture::new().write("README.md", "# Only headers\n\n## More\n");
        multi.write("README.txt", "text fallback\n");
        assert_eq!(multi.detect().as_deref(), Some("text fallback"));
        assert_parity(&multi);

        // A large README whose first usable line is near the top must still detect exactly.
        let mut big = String::from("Intro paragraph first\n");
        big.push_str(&"filler line\n".repeat(200_000));
        let huge = Fixture::new().write("README.md", &big);
        assert_eq!(huge.detect().as_deref(), Some("Intro paragraph first"));
        assert_parity(&huge);
    }

    #[test]
    fn test_empty_directory_and_preference_order() {
        let empty = Fixture::new();
        assert_eq!(empty.detect(), None);
        assert_parity(&empty);

        let rust_wins = Fixture::new()
            .write(
                "Cargo.toml",
                "[package]\nname = \"rust\"\ndescription = \"r\"\n",
            )
            .write("package.json", "{\"name\":\"node\",\"description\":\"n\"}")
            .write("go.mod", "module go\n")
            .write("README.md", "readme\n");
        assert_eq!(rust_wins.detect().as_deref(), Some("Rust: rust - r"));
        assert_parity(&rust_wins);

        let node_wins = Fixture::new()
            .write("package.json", "{\"name\":\"node\",\"description\":\"n\"}")
            .write("go.mod", "module go\n");
        assert_eq!(node_wins.detect().as_deref(), Some("Node: node - n"));
        assert_parity(&node_wins);
    }

    /// Exact A/B receipt against the original implementation.
    ///
    /// Run with: `cargo test --release -- --ignored --nocapture perf_receipt`
    /// Reports integer nanoseconds and a reduced `p/q` ratio; no floating point.
    #[test]
    #[ignore]
    fn perf_receipt_detect_project_context() {
        const ROUNDS: u64 = 7;
        const ITERS: u64 = 100;

        fn reduced(n: u64, d: u64) -> (u64, u64) {
            let (mut a, mut b) = (n, d);
            while b != 0 {
                let t = a % b;
                a = b;
                b = t;
            }
            let g = a.max(1);
            (n / g, d / g)
        }

        let rust = Fixture::new().write(
            "Cargo.toml",
            "[package]\nname = \"smart-tree\"\ndescription = \"An intelligent, AI-friendly directory visualization tool for humans and AIs\"\n",
        );

        let mut readme_body = String::from("Smart Tree\n\nAn intelligent directory visualizer.\n");
        readme_body.push_str(&"filler content line\n".repeat(120_000));
        let readme = Fixture::new().write("README.md", &readme_body);

        let sparse = Fixture::new().write("notes/0.txt", "one\ntwo\n");

        for (label, fixture) in [
            ("rust-manifest", &rust),
            ("readme-2.3MB", &readme),
            ("no-project-files", &sparse),
        ] {
            let mut legacy_best = u64::MAX / 2;
            let mut new_best = u64::MAX / 2;
            for _ in 0..ROUNDS {
                let start = std::time::Instant::now();
                for _ in 0..ITERS {
                    std::hint::black_box(legacy::detect(std::hint::black_box(&fixture.root)));
                }
                let legacy_ns = start.elapsed().as_nanos() as u64;

                let start = std::time::Instant::now();
                for _ in 0..ITERS {
                    std::hint::black_box(detect_project_context(std::hint::black_box(
                        &fixture.root,
                    )));
                }
                let new_ns = start.elapsed().as_nanos() as u64;

                legacy_best = legacy_best.min(legacy_ns);
                new_best = new_best.min(new_ns);
            }
            let (p, q) = reduced(legacy_best, new_best);
            println!(
                "{label}: legacy {legacy_best}ns/{ITERS} iters, optimized {new_best}ns/{ITERS} iters, ratio p/q = {p}/{q}"
            );
        }
    }
}
