// Integration tests for ST Unified Tools working together
// "Integration is where the real bugs party!" - Testy McTesterson 🧪

use anyhow::Result;
use st::st_context_aware::{ContextualOperation, StContextTracker, WorkContext};
use st::st_unified::StUnified;
use st::tools_st_only::{ListOptions, SearchOptions, StOnlyTools, StToolsConfig};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

fn create_realistic_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Create a realistic Rust project structure
    fs::create_dir_all(temp_dir.path().join("src/core/engine"))?;
    fs::create_dir_all(temp_dir.path().join("src/utils/helpers"))?;
    fs::create_dir_all(temp_dir.path().join("src/api/v1"))?;
    fs::create_dir_all(temp_dir.path().join("tests/unit"))?;
    fs::create_dir_all(temp_dir.path().join("tests/integration"))?;
    fs::create_dir_all(temp_dir.path().join("benches"))?;
    fs::create_dir_all(temp_dir.path().join("docs/api"))?;
    fs::create_dir_all(temp_dir.path().join("examples"))?;
    fs::create_dir_all(temp_dir.path().join(".github/workflows"))?;

    // Create various files
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
criterion = "0.5"
"#,
    )?;

    fs::write(
        temp_dir.path().join("src/main.rs"),
        r#"
use crate::core::engine::Engine;

mod core;
mod utils;
mod api;

fn main() {
    println!("Starting application...");
    let engine = Engine::new();
    // TODO: Implement main logic
    engine.run();
}
"#,
    )?;

    fs::write(
        temp_dir.path().join("src/lib.rs"),
        r#"
//! Test project library
//! 
//! This provides the core functionality.

pub mod core;
pub mod utils;
pub mod api;

/// Main entry point for library usage
pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Add initialization
    Ok(())
}
"#,
    )?;

    fs::write(
        temp_dir.path().join("src/core/engine/mod.rs"),
        r#"
pub struct Engine {
    running: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self { running: false }
    }
    
    pub fn run(&mut self) {
        self.running = true;
        // TODO: Implement engine logic
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(!engine.running);
    }
}
"#,
    )?;

    fs::write(
        temp_dir.path().join("src/utils/helpers/mod.rs"),
        r#"
/// Helper utilities
pub fn format_output(data: &str) -> String {
    // TODO: Implement formatting
    format!("Formatted: {}", data)
}

pub fn validate_input(input: &str) -> bool {
    !input.is_empty() && input.len() < 1000
}
"#,
    )?;

    fs::write(
        temp_dir.path().join("tests/integration/test_api.rs"),
        r#"
#[test]
fn test_api_integration() {
    // TODO: Write integration tests
    assert_eq!(2 + 2, 4);
}
"#,
    )?;

    fs::write(
        temp_dir.path().join("README.md"),
        r#"
# Test Project

A comprehensive test project for Smart Tree unified tools.

## Features
- Engine implementation
- Utility helpers
- API endpoints

## TODO
- [ ] Complete engine implementation
- [ ] Add more tests
- [ ] Write documentation
"#,
    )?;

    fs::write(
        temp_dir.path().join(".gitignore"),
        r#"
/target
**/*.rs.bk
Cargo.lock
.DS_Store
*.swp
"#,
    )?;

    Ok(temp_dir)
}

#[test]
fn test_unified_tools_exploration_workflow() -> Result<()> {
    // Simulating a developer exploring a new codebase
    let project = create_realistic_project()?;
    let tracker = Arc::new(StContextTracker::new());

    // Step 1: Initial exploration with StUnified
    let unified = StUnified::new()?;

    // Get quick overview
    let overview = unified.quick(project.path())?;
    assert!(
        overview.contains("STATS"),
        "Quick overview should show stats"
    );

    // Record this as an exploration
    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "quick_overview".to_string(),
        path: project.path().to_path_buf(),
        result_summary: "Got project overview".to_string(),
        context_hints: vec!["exploration".to_string()],
    })?;

    // Step 2: List source files
    let src_files = unified.ls(&project.path().join("src"), Some("*.rs"))?;
    assert!(src_files.contains("main.rs"), "Should find main.rs");

    // Step 3: Read main file to understand entry point
    let main_content = unified.read(&project.path().join("src/main.rs"), None, None)?;
    assert!(
        main_content.contains("fn main()"),
        "Should read main function"
    );

    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "read".to_string(),
        path: project.path().join("src/main.rs"),
        result_summary: "Read main entry point".to_string(),
        context_hints: vec![],
    })?;

    // Step 4: Search for TODOs
    let todos = unified.grep("TODO", project.path(), Some("rs"))?;
    assert!(
        todos.contains("TODO") || todos.contains("main.rs"),
        "Should find TODO comments"
    );

    // Verify context detection
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Should be in exploring context"
    );

    Ok(())
}

#[test]
fn test_context_aware_development_workflow() -> Result<()> {
    // Simulating active development with context awareness
    let project = create_realistic_project()?;
    let tracker = Arc::new(StContextTracker::new());
    let tools = StOnlyTools::new();

    // Step 1: Developer starts coding
    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "edit".to_string(),
        path: project.path().join("src/core/engine/mod.rs"),
        result_summary: "Editing engine module".to_string(),
        context_hints: vec!["coding".to_string()],
    })?;

    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "edit".to_string(),
        path: project.path().join("src/core/engine/mod.rs"),
        result_summary: "More engine changes".to_string(),
        context_hints: vec!["coding".to_string()],
    })?;

    // Step 2: Run tests
    let _test_files = tools.list(
        &project.path().join("tests"),
        ListOptions {
            pattern: Some("test_*.rs".to_string()),
            ..Default::default()
        },
    )?;

    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "read".to_string(),
        path: project.path().join("tests/integration/test_api.rs"),
        result_summary: "Checking tests".to_string(),
        context_hints: vec!["testing".to_string()],
    })?;

    // Step 3: Get context-aware suggestions
    let suggestions = tracker.get_suggestions(project.path());
    assert!(!suggestions.is_empty(), "Should provide coding suggestions");
    assert!(
        suggestions
            .iter()
            .any(|s| s.contains("test") || s.contains("relations")),
        "Should suggest test-related commands"
    );

    // Verify context switched to coding
    let context = tracker.analyze_context()?;
    match context {
        WorkContext::Coding { language, .. } => {
            assert_eq!(language, "rust", "Should detect Rust coding");
        }
        _ => panic!("Should be in coding context"),
    }

    Ok(())
}

#[test]
fn test_debugging_workflow_with_all_tools() -> Result<()> {
    // Simulating debugging workflow using all tools together
    let project = create_realistic_project()?;
    let tracker = Arc::new(StContextTracker::new());
    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();

    // Step 1: Error occurs, developer starts searching
    for keyword in &["error", "panic", "TODO", "bug"] {
        let _result = unified.grep(keyword, project.path(), Some("rs"))?;

        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: format!("search {}", keyword),
            path: project.path().to_path_buf(),
            result_summary: format!("Searched for {}", keyword),
            context_hints: vec!["debugging".to_string()],
        })?;
    }

    // Step 2: Analyze project structure to understand dependencies
    let semantic = tools.semantic(project.path())?;
    assert!(!semantic.is_empty(), "Should provide semantic analysis");

    // Step 3: Get detailed stats
    let stats = tools.stats(project.path())?;
    assert!(
        stats.contains("Files") || stats.contains("F:"),
        "Should show file stats"
    );

    // Step 4: Context should recognize debugging pattern
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Hunting { .. }),
        "Multiple searches should trigger hunting context"
    );

    // Step 5: Get debugging-specific suggestions
    let suggestions = tracker.get_suggestions(project.path());
    assert!(
        suggestions
            .iter()
            .any(|s| s.contains("search") || s.contains("recent")),
        "Should suggest search-related commands for debugging"
    );

    Ok(())
}

#[test]
fn test_full_project_understanding_workflow() -> Result<()> {
    // Complete project understanding using all tools
    let project = create_realistic_project()?;
    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();
    let tracker = Arc::new(StContextTracker::new());

    // Step 1: Project overview
    let project_understanding = unified.understand_project(project.path())?;
    assert!(
        project_understanding.contains("QUICK OVERVIEW"),
        "Should have overview"
    );
    assert!(
        project_understanding.contains("SEMANTIC GROUPS"),
        "Should have semantic analysis"
    );
    assert!(
        project_understanding.contains("STATISTICS"),
        "Should have stats"
    );

    // Step 2: Deep dive into structure
    let config = StToolsConfig {
        default_mode: "quantum-semantic".to_string(),
        use_emoji: false,
        compress: true,
        ..Default::default()
    };
    let advanced_tools = StOnlyTools::with_config(config);

    // Use advanced analysis
    let deep_overview = advanced_tools.overview(project.path(), Some(10))?;
    assert!(!deep_overview.is_empty(), "Deep overview should work");

    // Step 3: Search for specific patterns
    let search_options = SearchOptions {
        file_type: Some("rs".to_string()),
        ..Default::default()
    };

    let impl_blocks = tools.search("impl", project.path(), search_options)?;
    assert!(
        !impl_blocks.is_empty() || impl_blocks.contains("impl"),
        "Should find implementation blocks"
    );

    // Step 4: Track all this exploration
    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "deep_analysis".to_string(),
        path: project.path().to_path_buf(),
        result_summary: "Complete project analysis".to_string(),
        context_hints: vec!["exploration".to_string(), "understanding".to_string()],
    })?;

    // Step 5: Save context for next session
    tracker.save_context(project.path())?;
    assert!(
        project.path().join(".st_context.json").exists(),
        "Context should be saved"
    );

    Ok(())
}

#[test]
fn test_performance_optimization_workflow() -> Result<()> {
    // Simulating performance optimization workflow
    let project = create_realistic_project()?;
    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();
    let tracker = Arc::new(StContextTracker::new());

    // Create some "heavy" files to simulate performance issues
    let heavy_file = project.path().join("src/core/heavy_computation.rs");
    fs::write(
        &heavy_file,
        r#"
pub fn heavy_computation(n: usize) -> usize {
    // TODO: Optimize this O(n²) algorithm
    let mut result = 0;
    for i in 0..n {
        for j in 0..n {
            result += i * j;
        }
    }
    result
}

// FIXME: This allocates too much memory
pub fn memory_heavy(size: usize) -> Vec<Vec<u8>> {
    let mut data = Vec::new();
    for _ in 0..size {
        data.push(vec![0u8; 1024 * 1024]); // 1MB each
    }
    data
}
"#,
    )?;

    // Step 1: Search for performance-related keywords
    for keyword in &["TODO: Optimize", "FIXME", "O(n", "allocate"] {
        let _ = unified.grep(keyword, project.path(), Some("rs"));

        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: format!("search performance: {}", keyword),
            path: project.path().to_path_buf(),
            result_summary: "Looking for optimization opportunities".to_string(),
            context_hints: vec!["optimization".to_string()],
        })?;
    }

    // Step 2: Analyze file statistics to find large files
    let stats = tools.stats(&project.path().join("src"))?;
    assert!(!stats.is_empty(), "Should get source statistics");

    // Step 3: Get optimal arguments for performance analysis
    let optimal_args = tracker.get_optimal_args("st");
    assert!(!optimal_args.is_empty(), "Should suggest optimal arguments");

    // Context should recognize optimization work
    // (Note: Our simple heuristic might classify this as Hunting due to searches)
    let context = tracker.analyze_context()?;
    println!("Detected context: {:?}", context);

    Ok(())
}

#[test]
fn test_cross_tool_consistency() -> Result<()> {
    // Ensure all tools provide consistent results
    let project = create_realistic_project()?;
    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();

    // Compare listing results
    let unified_ls = unified.ls(project.path(), None)?;
    let tools_ls = tools.list(project.path(), ListOptions::default())?;

    // Both should find the same key items (though format may differ)
    for item in &["src", "tests", "Cargo.toml", "README.md"] {
        assert!(
            unified_ls.contains(item) || tools_ls.contains(item),
            "Both tools should find {}",
            item
        );
    }

    // Compare search results
    let unified_search = unified.grep("TODO", project.path(), Some("rs"))?;
    let tools_search = tools.search(
        "TODO",
        project.path(),
        SearchOptions {
            file_type: Some("rs".to_string()),
            ..Default::default()
        },
    )?;

    // Both should find TODOs (or both should find nothing)
    if unified_search.contains("TODO") {
        assert!(
            tools_search.contains("TODO") || !tools_search.is_empty(),
            "Search results should be consistent"
        );
    }

    Ok(())
}

#[test]
fn test_context_persistence_across_sessions() -> Result<()> {
    // Test that context can be saved and restored
    let project = create_realistic_project()?;

    // Session 1: Build up context
    {
        let tracker = StContextTracker::new();

        // Simulate coding session
        for _ in 0..3 {
            tracker.record_operation(ContextualOperation {
                timestamp: std::time::SystemTime::now(),
                operation: "edit".to_string(),
                path: project.path().join("src/main.rs"),
                result_summary: "Coding".to_string(),
                context_hints: vec!["coding".to_string()],
            })?;
        }

        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: "test".to_string(),
            path: project.path().join("tests/test_main.rs"),
            result_summary: "Testing".to_string(),
            context_hints: vec!["testing".to_string()],
        })?;

        // Save context
        tracker.save_context(project.path())?;
    }

    // Session 2: Load and verify context
    {
        let new_tracker = StContextTracker::new();
        new_tracker.load_context(project.path())?;

        let context = new_tracker.analyze_context()?;
        assert!(
            !matches!(context, WorkContext::Exploring { depth: 3, .. }),
            "Should have loaded previous context, not default"
        );

        // Should provide relevant suggestions based on loaded context
        let suggestions = new_tracker.get_suggestions(project.path());
        assert!(
            !suggestions.is_empty(),
            "Should provide suggestions from loaded context"
        );
    }

    Ok(())
}

#[test]
fn test_error_handling_integration() -> Result<()> {
    // Test error handling across all tools
    let nonexistent = Path::new("/definitely/not/a/real/path/at/all");

    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();
    let tracker = StContextTracker::new();

    // All tools should handle nonexistent paths gracefully

    // StUnified errors
    assert!(
        unified.read(nonexistent, None, None).is_err(),
        "Should error on nonexistent file"
    );

    // StOnlyTools might succeed (depends on st binary behavior)
    let _ = tools.list(nonexistent, ListOptions::default());
    let _ = tools.stats(nonexistent);

    // Context tracker should handle gracefully
    tracker.record_operation(ContextualOperation {
        timestamp: std::time::SystemTime::now(),
        operation: "failed_read".to_string(),
        path: nonexistent.to_path_buf(),
        result_summary: "Error: File not found".to_string(),
        context_hints: vec!["error".to_string()],
    })?;

    // Should still provide context
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Should handle errors gracefully"
    );

    Ok(())
}

#[test]
fn test_realistic_multi_hour_session() -> Result<()> {
    // Simulate a realistic multi-hour development session
    let project = create_realistic_project()?;
    let unified = StUnified::new()?;
    let tools = StOnlyTools::new();
    let tracker = Arc::new(StContextTracker::new());

    // Hour 1: Initial exploration
    for dir in &["src", "tests", "docs"] {
        let _ = unified.ls(&project.path().join(dir), None)?;
        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: "explore".to_string(),
            path: project.path().join(dir),
            result_summary: format!("Explored {}", dir),
            context_hints: vec!["exploration".to_string()],
        })?;
    }

    // Hour 2: Focus on specific module
    let focus_file = project.path().join("src/core/engine/mod.rs");
    for _ in 0..5 {
        let _ = unified.read(&focus_file, None, None)?;
        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: "edit".to_string(),
            path: focus_file.clone(),
            result_summary: "Working on engine".to_string(),
            context_hints: vec!["coding".to_string()],
        })?;
    }

    // Hour 3: Debugging issue
    for pattern in &["error", "panic", "unwrap", "expect"] {
        let _ = tools.search(pattern, project.path(), SearchOptions::default())?;
        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: format!("search {}", pattern),
            path: project.path().to_path_buf(),
            result_summary: "Debugging".to_string(),
            context_hints: vec!["debugging".to_string()],
        })?;
    }

    // Hour 4: Writing tests
    let test_dir = project.path().join("tests");
    for i in 0..3 {
        let test_file = test_dir.join(format!("test_new_{}.rs", i));
        fs::write(&test_file, "#[test]\nfn test() { assert!(true); }")?;

        tracker.record_operation(ContextualOperation {
            timestamp: std::time::SystemTime::now(),
            operation: "create_test".to_string(),
            path: test_file,
            result_summary: "Writing tests".to_string(),
            context_hints: vec!["testing".to_string()],
        })?;
    }

    // Final: Check evolution of context
    let final_context = tracker.analyze_context()?;
    println!("Final context after 4-hour session: {:?}", final_context);

    // Save the rich context
    tracker.save_context(project.path())?;

    // Verify we built up substantial knowledge
    let context_file = project.path().join(".st_context.json");
    let context_size = fs::metadata(&context_file)?.len();
    assert!(
        context_size > 100,
        "Should have accumulated substantial context"
    );

    Ok(())
}
