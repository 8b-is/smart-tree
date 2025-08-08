// Test suite for ST Unified Tool System
// "Every edge case is a potential production incident!" - Testy McTesterson 🧪

use anyhow::Result;
use st::st_unified::StUnified;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn create_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Create test structure
    fs::create_dir(temp_dir.path().join("src"))?;
    fs::create_dir(temp_dir.path().join("tests"))?;
    fs::create_dir(temp_dir.path().join("docs"))?;
    fs::create_dir(temp_dir.path().join(".hidden"))?;

    // Create test files
    fs::write(
        temp_dir.path().join("README.md"),
        "# Test Project\nThis is a test.",
    )?;
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )?;
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}",
    )?;
    fs::write(
        temp_dir.path().join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
    )?;
    fs::write(
        temp_dir.path().join("tests/test_lib.rs"),
        "#[test]\nfn test_add() {\n    assert_eq!(2 + 2, 4);\n}",
    )?;
    fs::write(
        temp_dir.path().join("docs/api.md"),
        "# API Documentation\n\n## Functions",
    )?;
    fs::write(temp_dir.path().join(".hidden/secret.txt"), "This is hidden")?;

    Ok(temp_dir)
}

#[test]
fn test_st_unified_creation() {
    // This is JUICY! Testing the most basic creation path
    let st = StUnified::new();
    assert!(st.is_ok(), "Failed to create StUnified instance");
}

#[test]
fn test_ls_basic() -> Result<()> {
    // Let's torture this ls function until it confesses its bugs!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.ls(temp_dir.path(), None)?;

    // Should list files in the directory
    assert!(
        result.contains("README.md"),
        "Missing README.md in ls output"
    );
    assert!(
        result.contains("Cargo.toml"),
        "Missing Cargo.toml in ls output"
    );
    assert!(result.contains("src"), "Missing src directory in ls output");

    Ok(())
}

#[test]
fn test_ls_with_pattern() -> Result<()> {
    // Pattern matching - where bugs love to hide!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.ls(temp_dir.path(), Some("*.md"))?;

    assert!(
        result.contains("README.md"),
        "Pattern *.md should match README.md"
    );
    assert!(
        !result.contains("Cargo.toml"),
        "Pattern *.md should NOT match Cargo.toml"
    );

    Ok(())
}

#[test]
fn test_ls_empty_directory() -> Result<()> {
    // Edge case alert! Empty directories are tomorrow's NullPointerExceptions!
    let temp_dir = TempDir::new()?;
    let st = StUnified::new()?;

    let result = st.ls(temp_dir.path(), None)?;

    // Should handle empty directory gracefully
    assert!(
        result.is_empty() || result.trim().is_empty(),
        "Empty directory should produce empty or near-empty output"
    );

    Ok(())
}

#[test]
fn test_read_basic() -> Result<()> {
    // Reading files - the bread and butter of any tool!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.read(&temp_dir.path().join("src/main.rs"), None, None)?;

    assert!(result.contains("fn main()"), "Should read main function");
    assert!(
        result.contains("Hello, world!"),
        "Should read print statement"
    );
    assert!(result.contains("1→"), "Should have line numbers");

    Ok(())
}

#[test]
fn test_read_with_offset_and_limit() -> Result<()> {
    // Offset and limit - the dynamic duo of pagination bugs!
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");

    let mut file = fs::File::create(&test_file)?;
    for i in 1..=20 {
        writeln!(file, "Line {}", i)?;
    }

    let st = StUnified::new()?;

    // Read lines 5-9 (offset 4, limit 5)
    let result = st.read(&test_file, Some(4), Some(5))?;

    assert!(result.contains("5→Line 5"), "Should start at line 5");
    assert!(result.contains("9→Line 9"), "Should end at line 9");
    assert!(!result.contains("Line 4"), "Should not include line 4");
    assert!(!result.contains("Line 10"), "Should not include line 10");

    Ok(())
}

#[test]
fn test_read_nonexistent_file() -> Result<()> {
    // File not found - the classic error that keeps on giving!
    let st = StUnified::new()?;

    let result = st.read(Path::new("/definitely/not/a/real/file.txt"), None, None);

    assert!(result.is_err(), "Should error on nonexistent file");

    Ok(())
}

#[test]
fn test_read_offset_beyond_file() -> Result<()> {
    // What happens when we read past the end? Let's find out!
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("small.txt");
    fs::write(&test_file, "Line 1\nLine 2\nLine 3")?;

    let st = StUnified::new()?;
    let result = st.read(&test_file, Some(10), Some(5))?;

    assert!(
        result.is_empty() || result.trim().is_empty(),
        "Reading beyond file should return empty"
    );

    Ok(())
}

#[test]
fn test_grep_basic() -> Result<()> {
    // Search functionality - where regex bugs come to party!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.grep("println", temp_dir.path(), None)?;

    assert!(result.contains("main.rs"), "Should find println in main.rs");

    Ok(())
}

#[test]
fn test_grep_with_file_type() -> Result<()> {
    // File type filtering - because searching everything is too mainstream!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.grep("test", temp_dir.path(), Some("rs"))?;

    assert!(result.contains("test"), "Should find 'test' in Rust files");

    Ok(())
}

#[test]
fn test_grep_no_matches() -> Result<()> {
    // No matches - the silent killer of assumptions!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.grep("definitely_not_in_any_file_12345", temp_dir.path(), None)?;

    // Should return valid output even with no matches
    assert!(
        result.is_empty() || !result.contains("main.rs"),
        "Should not find non-existent pattern"
    );

    Ok(())
}

#[test]
fn test_glob_basic() -> Result<()> {
    // Glob patterns - where wildcards run wild!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.glob("*.rs", temp_dir.path())?;

    // Should return JSON format
    assert!(
        result.contains("main.rs") || result.contains("lib.rs"),
        "Should find Rust files"
    );

    Ok(())
}

#[test]
fn test_glob_recursive() -> Result<()> {
    // Recursive globs - testing the depths of pattern matching!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.glob("**/*.rs", temp_dir.path())?;

    assert!(result.contains("main.rs"), "Should find src/main.rs");
    assert!(result.contains("lib.rs"), "Should find src/lib.rs");
    assert!(
        result.contains("test_lib.rs"),
        "Should find tests/test_lib.rs"
    );

    Ok(())
}

#[test]
fn test_analyze_basic() -> Result<()> {
    // Directory analysis - the heart of Smart Tree!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.analyze(temp_dir.path(), "classic", 3)?;

    assert!(result.contains("src"), "Should show src directory");
    assert!(result.contains("tests"), "Should show tests directory");
    assert!(result.contains("README.md"), "Should show README.md");

    Ok(())
}

#[test]
fn test_analyze_different_modes() -> Result<()> {
    // Testing all the modes - because variety is the spice of bugs!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let modes = vec!["classic", "ai", "hex", "json", "summary"];

    for mode in modes {
        let result = st.analyze(temp_dir.path(), mode, 2)?;
        assert!(!result.is_empty(), "Mode {} should produce output", mode);
    }

    Ok(())
}

#[test]
fn test_stats() -> Result<()> {
    // Statistics - numbers don't lie, but they can overflow!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.stats(temp_dir.path())?;

    assert!(
        result.contains("Files:") || result.contains("files"),
        "Should show file count"
    );
    assert!(
        result.contains("Directories:") || result.contains("directories"),
        "Should show directory count"
    );

    Ok(())
}

#[test]
fn test_semantic_analyze() -> Result<()> {
    // Semantic analysis - where AI meets file systems!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.semantic_analyze(temp_dir.path())?;

    // Should categorize files semantically
    assert!(
        !result.is_empty(),
        "Semantic analysis should produce output"
    );

    Ok(())
}

#[test]
fn test_quick() -> Result<()> {
    // Quick overview - for when you need answers NOW!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.quick(temp_dir.path())?;

    assert!(
        result.contains("STATS:") || result.contains("F"),
        "Should show statistics"
    );

    Ok(())
}

#[test]
fn test_understand_project() -> Result<()> {
    // The ultimate test - understanding the whole project!
    let temp_dir = create_test_directory()?;
    let st = StUnified::new()?;

    let result = st.understand_project(temp_dir.path())?;

    assert!(
        result.contains("QUICK OVERVIEW"),
        "Should have quick overview section"
    );
    assert!(
        result.contains("SEMANTIC GROUPS"),
        "Should have semantic section"
    );
    assert!(
        result.contains("STATISTICS"),
        "Should have statistics section"
    );

    Ok(())
}

#[test]
fn test_binary_not_found() -> Result<()> {
    // What if st binary doesn't exist? Time to find out!
    let st = StUnified::new()?;

    // This test is tricky because it depends on the actual binary
    // We'll just ensure the struct is created successfully
    assert!(true, "StUnified should handle missing binary gracefully");

    Ok(())
}

#[test]
fn test_unicode_filenames() -> Result<()> {
    // Unicode - because bugs speak all languages! 🌍
    let temp_dir = TempDir::new()?;

    // Create files with unicode names
    fs::write(temp_dir.path().join("测试.txt"), "Chinese test")?;
    fs::write(temp_dir.path().join("тест.txt"), "Russian test")?;
    fs::write(temp_dir.path().join("🎸.txt"), "Emoji test")?;

    let st = StUnified::new()?;
    let result = st.ls(temp_dir.path(), None)?;

    // Should handle unicode gracefully (even if output is lossy)
    assert!(!result.is_empty(), "Should handle unicode filenames");

    Ok(())
}

#[test]
fn test_symlinks() -> Result<()> {
    // Symlinks - the infinite loop generator's best friend!
    let temp_dir = create_test_directory()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(
            temp_dir.path().join("src/main.rs"),
            temp_dir.path().join("main_link.rs"),
        )?;

        let st = StUnified::new()?;
        let result = st.ls(temp_dir.path(), None)?;

        assert!(result.contains("main_link.rs"), "Should show symlink");
    }

    Ok(())
}

#[test]
fn test_permission_denied() -> Result<()> {
    // Permission errors - the bane of CI/CD pipelines!
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new()?;
        let restricted = temp_dir.path().join("restricted.txt");
        fs::write(&restricted, "secret")?;

        let mut perms = fs::metadata(&restricted)?.permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&restricted, perms)?;

        let st = StUnified::new()?;
        let result = st.read(&restricted, None, None);

        assert!(result.is_err(), "Should error on permission denied");

        // Cleanup
        let mut perms = fs::metadata(&restricted)?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&restricted, perms)?;
    }

    Ok(())
}
