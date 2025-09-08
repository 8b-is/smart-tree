// Test suite for Smart Tree Only Tools
// "One tool to test them all!" - Testy McTesterson 🧪

use anyhow::Result;
use st::tools_st_only::{ListOptions, SearchOptions, StOnlyTools, StToolsConfig};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn create_complex_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Create a more complex structure for thorough testing
    fs::create_dir_all(temp_dir.path().join("src/modules/core"))?;
    fs::create_dir_all(temp_dir.path().join("src/modules/utils"))?;
    fs::create_dir_all(temp_dir.path().join("tests/unit"))?;
    fs::create_dir_all(temp_dir.path().join("tests/integration"))?;
    fs::create_dir_all(temp_dir.path().join("docs/api"))?;
    fs::create_dir_all(temp_dir.path().join("examples"))?;
    fs::create_dir_all(temp_dir.path().join(".git/hooks"))?;

    // Create various file types
    fs::write(
        temp_dir.path().join("README.md"),
        "# Complex Project\n\nWith multiple sections",
    )?;
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"complex\"\nversion = \"0.1.0\"",
    )?;
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() {\n    // TODO: Implement\n    println!(\"Complex app\");\n}",
    )?;
    fs::write(
        temp_dir.path().join("src/lib.rs"),
        "//! Library root\n\npub mod modules;",
    )?;
    fs::write(
        temp_dir.path().join("src/modules/core/engine.rs"),
        "pub struct Engine {\n    // TODO: Add fields\n}",
    )?;
    fs::write(
        temp_dir.path().join("src/modules/utils/helpers.rs"),
        "pub fn helper() -> bool {\n    true\n}",
    )?;
    fs::write(
        temp_dir.path().join("tests/unit/test_engine.rs"),
        "#[test]\nfn test_engine() {\n    // TODO: Write test\n}",
    )?;
    fs::write(
        temp_dir.path().join("docs/api/engine.md"),
        "# Engine API\n\nTODO: Document",
    )?;
    fs::write(
        temp_dir.path().join(".gitignore"),
        "target/\n*.tmp\n.DS_Store",
    )?;

    Ok(temp_dir)
}

#[test]
fn test_config_default() {
    // Testing default configuration - where assumptions go to die!
    let config = StToolsConfig::default();

    assert_eq!(config.default_mode, "ai", "Default mode should be 'ai'");
    assert!(!config.use_emoji, "Emoji should be disabled by default");
    assert!(
        !config.compress,
        "Compression should be disabled by default"
    );
    assert!(
        config.st_binary.to_string_lossy().contains("st"),
        "Binary path should contain 'st'"
    );
}

#[test]
fn test_config_custom() {
    // Custom config - because one size never fits all!
    let config = StToolsConfig {
        st_binary: PathBuf::from("/custom/path/st"),
        default_mode: "quantum".to_string(),
        use_emoji: true,
        compress: true,
    };

    let _tools = StOnlyTools::with_config(config.clone());

    // The config should be stored internally
    // We can't directly access it, but we can test behavior
    assert_eq!(config.st_binary.to_str().unwrap(), "/custom/path/st");
}

#[test]
fn test_list_basic() -> Result<()> {
    // Basic listing - the foundation of file exploration!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = ListOptions::default();
    let result = tools.list(temp_dir.path(), options)?;

    // Should list top-level items
    assert!(
        result.contains("README.md") || result.contains("readme"),
        "Should list README.md"
    );
    assert!(
        result.contains("src") || result.contains("SRC"),
        "Should list src directory"
    );

    Ok(())
}

#[test]
fn test_list_with_pattern() -> Result<()> {
    // Pattern filtering - where wildcards meet their match!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = ListOptions {
        pattern: Some("*.md".to_string()),
        ..Default::default()
    };

    let result = tools.list(temp_dir.path(), options)?;

    assert!(
        result.contains("README.md") || result.contains("readme"),
        "Should find README.md with *.md pattern"
    );

    Ok(())
}

#[test]
fn test_list_with_file_type() -> Result<()> {
    // File type filtering - segregation for better organization!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = ListOptions {
        file_type: Some("rs".to_string()),
        ..Default::default()
    };

    let result = tools.list(temp_dir.path(), options)?;

    // Should only show Rust files
    assert!(
        result.contains(".rs") || result.contains("main") || result.contains("lib"),
        "Should filter for Rust files"
    );

    Ok(())
}

#[test]
fn test_list_with_sort() -> Result<()> {
    // Sorting - because chaos is not a feature!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let sort_options = vec!["name", "size", "date"];

    for sort in sort_options {
        let options = ListOptions {
            sort: Some(sort.to_string()),
            ..Default::default()
        };

        let result = tools.list(temp_dir.path(), options)?;
        assert!(!result.is_empty(), "Sort by {} should produce output", sort);
    }

    Ok(())
}

#[test]
fn test_list_with_limit() -> Result<()> {
    // Limiting results - pagination's best friend!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = ListOptions {
        limit: Some(3),
        ..Default::default()
    };

    let result = tools.list(temp_dir.path(), options)?;

    // Can't easily count lines in output, but should have content
    assert!(!result.is_empty(), "Limited list should still have output");

    Ok(())
}

#[test]
fn test_search_basic() -> Result<()> {
    // Search functionality - finding needles in haystacks!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = SearchOptions::default();
    let result = tools.search("TODO", temp_dir.path(), options)?;

    assert!(
        result.contains("TODO") || result.contains("main.rs") || result.contains("engine"),
        "Should find TODO comments"
    );

    Ok(())
}

#[test]
fn test_search_with_file_type() -> Result<()> {
    // Type-specific search - because context matters!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = SearchOptions {
        file_type: Some("md".to_string()),
        ..Default::default()
    };

    let result = tools.search("TODO", temp_dir.path(), options)?;

    // Should only search in markdown files
    assert!(
        result.contains("TODO") || result.contains("api") || result.contains("Document"),
        "Should find TODO in markdown files"
    );

    Ok(())
}

#[test]
fn test_search_case_sensitivity() -> Result<()> {
    // Case sensitivity - the eternal debate!
    let temp_dir = TempDir::new()?;
    fs::write(temp_dir.path().join("test.txt"), "TODO\ntodo\nToDo")?;

    let tools = StOnlyTools::new();

    let options = SearchOptions {
        case_sensitive: true,
        ..Default::default()
    };

    let result = tools.search("TODO", temp_dir.path(), options)?;

    // Should handle case-sensitive search
    assert!(!result.is_empty(), "Should find case-sensitive matches");

    Ok(())
}

#[test]
fn test_overview() -> Result<()> {
    // Project overview - the bird's eye view!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let result = tools.overview(temp_dir.path(), None)?;

    assert!(
        result.contains("STATS") || result.contains("F") || result.contains("D"),
        "Overview should contain statistics"
    );

    Ok(())
}

#[test]
fn test_overview_with_depth() -> Result<()> {
    // Depth control - how deep is your directory tree?
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let depths = vec![1, 3, 5, 10];

    for depth in depths {
        let result = tools.overview(temp_dir.path(), Some(depth))?;
        assert!(
            !result.is_empty(),
            "Overview with depth {} should produce output",
            depth
        );
    }

    Ok(())
}

#[test]
fn test_stats() -> Result<()> {
    // Statistics - numbers that tell stories!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let result = tools.stats(temp_dir.path())?;

    assert!(
        result.contains("Files") || result.contains("files") || result.contains("F:"),
        "Stats should show file count"
    );
    assert!(
        result.contains("Directories") || result.contains("directories") || result.contains("D:"),
        "Stats should show directory count"
    );

    Ok(())
}

#[test]
fn test_semantic() -> Result<()> {
    // Semantic analysis - understanding beyond syntax!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let result = tools.semantic(temp_dir.path())?;

    assert!(
        !result.is_empty(),
        "Semantic analysis should produce output"
    );

    Ok(())
}

#[test]
fn test_run_st_error_handling() -> Result<()> {
    // Error handling - where resilience meets reality!
    let config = StToolsConfig {
        st_binary: PathBuf::from("/definitely/not/a/real/binary"),
        ..Default::default()
    };

    let tools = StOnlyTools::with_config(config);
    let options = ListOptions::default();

    let result = tools.list(Path::new("."), options);

    assert!(result.is_err(), "Should error with invalid binary path");

    Ok(())
}

#[test]
fn test_empty_directory_handling() -> Result<()> {
    // Empty directories - the void stares back!
    let temp_dir = TempDir::new()?;
    let tools = StOnlyTools::new();

    let result = tools.list(temp_dir.path(), ListOptions::default())?;
    assert!(
        result.is_empty() || result.trim().is_empty(),
        "Empty directory should produce minimal output"
    );

    let result = tools.stats(temp_dir.path())?;
    assert!(
        result.contains("0") || result.contains("empty"),
        "Stats should show zero files"
    );

    Ok(())
}

#[test]
fn test_nested_directory_operations() -> Result<()> {
    // Deep nesting - testing the depths of recursion!
    let temp_dir = TempDir::new()?;
    let mut path = temp_dir.path().to_path_buf();

    // Create deeply nested structure
    for i in 0..10 {
        path = path.join(format!("level{}", i));
        fs::create_dir(&path)?;
        fs::write(
            path.join(format!("file{}.txt", i)),
            format!("Content at level {}", i),
        )?;
    }

    let tools = StOnlyTools::new();
    let result = tools.overview(temp_dir.path(), Some(15))?;

    assert!(
        result.contains("level9") || result.contains("9") || result.contains("file"),
        "Should handle deeply nested directories"
    );

    Ok(())
}

#[test]
fn test_all_list_options_combined() -> Result<()> {
    // The ultimate combination test - when all options collide!
    let temp_dir = create_complex_test_directory()?;
    let tools = StOnlyTools::new();

    let options = ListOptions {
        pattern: Some("*.rs".to_string()),
        file_type: Some("rs".to_string()),
        sort: Some("size".to_string()),
        limit: Some(5),
    };

    let result = tools.list(temp_dir.path(), options)?;

    assert!(
        !result.is_empty(),
        "Combined options should still produce output"
    );

    Ok(())
}

#[test]
fn test_special_characters_in_filenames() -> Result<()> {
    // Special characters - the parser's nightmare!
    let temp_dir = TempDir::new()?;

    // Create files with special characters
    let special_names = vec![
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.multiple.dots.txt",
        "file(with)parens.txt",
        "file[with]brackets.txt",
        "file{with}braces.txt",
        "file'with'quotes.txt",
    ];

    for name in &special_names {
        fs::write(temp_dir.path().join(name), "content")?;
    }

    let tools = StOnlyTools::new();
    let result = tools.list(temp_dir.path(), ListOptions::default())?;

    // Should handle all special characters gracefully
    assert!(
        !result.is_empty(),
        "Should list files with special characters"
    );

    Ok(())
}

#[test]
fn test_large_file_handling() -> Result<()> {
    // Large files - testing memory limits!
    let temp_dir = TempDir::new()?;
    let large_file = temp_dir.path().join("large.txt");

    // Create a 1MB file
    let content = "x".repeat(1024 * 1024);
    fs::write(&large_file, content)?;

    let tools = StOnlyTools::new();
    let result = tools.stats(temp_dir.path())?;

    assert!(
        result.contains("1") || result.contains("MB") || result.contains("MiB"),
        "Stats should show large file size"
    );

    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<()> {
    // Concurrency - where race conditions come to play!
    use std::sync::Arc;
    use std::thread;

    let temp_dir = Arc::new(create_complex_test_directory()?);
    let mut handles = vec![];

    // Spawn multiple threads doing different operations
    for i in 0..5 {
        let temp_dir = Arc::clone(&temp_dir);
        let handle = thread::spawn(move || {
            let tools = StOnlyTools::new();

            match i % 3 {
                0 => tools.list(temp_dir.path(), ListOptions::default()),
                1 => tools.stats(temp_dir.path()),
                _ => tools.overview(temp_dir.path(), None),
            }
        });

        handles.push(handle);
    }

    // All operations should complete without panic
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "Concurrent operation should succeed");
    }

    Ok(())
}

#[test]
fn test_invalid_path_handling() -> Result<()> {
    // Invalid paths - the universal constant of file systems!
    let tools = StOnlyTools::new();

    let invalid_paths = vec![
        Path::new("/definitely/not/a/real/path"),
        Path::new(""),
        Path::new("\0"), // Null byte
    ];

    for path in invalid_paths {
        // Should handle gracefully without panic
        let _ = tools.list(path, ListOptions::default());
        let _ = tools.stats(path);
        let _ = tools.overview(path, None);
    }

    Ok(())
}
