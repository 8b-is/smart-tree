// 🎸 The Cheet's Aggregate Tests - "Testing the unified markdown singularity!" 🌌

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mq_aggregate_basic() {
    // Create a temporary directory with test markdown files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test markdown files
    fs::write(
        temp_path.join("README.md"),
        "# Test Project\n\nThis is a test project.",
    )
    .unwrap();
    fs::write(
        temp_path.join("INSTALL.md"),
        "# Installation\n\nRun `cargo install`.",
    )
    .unwrap();
    fs::create_dir(temp_path.join("docs")).unwrap();
    fs::write(
        temp_path.join("docs/API.md"),
        "# API Reference\n\n## Functions\n\nAPI docs here.",
    )
    .unwrap();

    // Run mq aggregate
    let output_path = temp_path.join("test.mq");
    let mut cmd = Command::cargo_bin("mq").unwrap();
    let output = cmd
        .args([
            "aggregate",
            temp_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute mq aggregate");

    assert!(
        output.status.success(),
        "mq aggregate failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that the output file exists
    assert!(temp_dir.path().join("test.mq").exists());

    // Read and verify the aggregate file
    let aggregate_content = fs::read_to_string(temp_dir.path().join("test.mq")).unwrap();

    // Should have V2 header
    assert!(aggregate_content.starts_with("MARQANT_V2"));

    // Should have manifest
    assert!(aggregate_content.contains("::manifest::"));
    assert!(aggregate_content.contains("README.md:"));
    assert!(aggregate_content.contains("INSTALL.md:"));
    // Handle both Unix and Windows path separators
    assert!(
        aggregate_content.contains("docs/API.md:") || aggregate_content.contains("docs\\API.md:")
    );
    assert!(aggregate_content.contains("::end-manifest::"));

    // Should have file markers
    assert!(aggregate_content.contains("::file:README.md::"));
    assert!(aggregate_content.contains("::file:INSTALL.md::"));
    // Handle both Unix and Windows path separators
    assert!(
        aggregate_content.contains("::file:docs/API.md::")
            || aggregate_content.contains("::file:docs\\API.md::")
    );

    // When content is small, might not have tokens, but should have separator
    assert!(aggregate_content.contains("---"));
}

#[test]
fn test_mq_aggregate_with_compression() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a larger markdown file to test compression
    let mut large_content = String::from("# Large Document\n\n");
    for i in 0..100 {
        large_content.push_str(&format!(
            "## Section {}\n\nThis is section {}. It contains repeated content.\n\n",
            i, i
        ));
    }
    fs::write(temp_path.join("large.md"), &large_content).unwrap();

    // Run mq aggregate with zlib
    let aggregate_path = temp_path.join("compressed.mq");
    let mut cmd = Command::cargo_bin("mq").unwrap();
    let output = cmd
        .args([
            "aggregate",
            temp_path.to_str().unwrap(),
            "-o",
            aggregate_path.to_str().unwrap(),
            "--zlib",
        ])
        .output()
        .expect("Failed to execute mq aggregate");

    assert!(output.status.success());

    assert!(aggregate_path.exists());

    // Compressed version should be smaller
    let aggregate_size = fs::metadata(&aggregate_path).unwrap().len();
    let original_size = large_content.len() as u64;

    assert!(
        aggregate_size < original_size,
        "Compressed size {} should be less than original {}",
        aggregate_size,
        original_size
    );

    // Verify header has -zlib flag
    let content = fs::read_to_string(&aggregate_path).unwrap();
    assert!(content.contains("-aggregate -zlib"));
}

#[test]
fn test_mq_aggregate_with_exclusions() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create files that should be included
    fs::write(temp_path.join("README.md"), "# Main").unwrap();

    // Create files that should be excluded
    fs::create_dir(temp_path.join("vendor")).unwrap();
    fs::write(temp_path.join("vendor/external.md"), "# External").unwrap();

    fs::create_dir(temp_path.join("node_modules")).unwrap();
    fs::write(temp_path.join("node_modules/package.md"), "# Package").unwrap();

    // Run with exclusions
    let output_path = temp_path.join("filtered.mq");
    let mut cmd = Command::cargo_bin("mq").unwrap();
    let output = cmd
        .args([
            "aggregate",
            temp_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "--exclude",
            "vendor/*",
            "--exclude",
            "node_modules/*",
        ])
        .output()
        .expect("Failed to execute mq aggregate");

    assert!(output.status.success());

    let content = fs::read_to_string(temp_path.join("filtered.mq")).unwrap();

    // Should include README
    assert!(content.contains("::file:README.md::"));

    // Should NOT include excluded files
    assert!(!content.contains("vendor/external.md"));
    assert!(!content.contains("node_modules/package.md"));
}

// 🎸 "Testing aggregation like a supergroup - bringing all the hits together!" - The Cheet
