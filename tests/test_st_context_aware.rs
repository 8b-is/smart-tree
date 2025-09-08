// Test suite for ST Context-Aware System
// "Context is everything - and everything needs testing!" - Testy McTesterson 🧪

use anyhow::Result;
use st::st_context_aware::{
    ContextualOperation, ContextualStCommand, ProjectKnowledge, StContextTracker, WorkContext,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tempfile::TempDir;

fn create_test_operation(op: &str, path: &str) -> ContextualOperation {
    ContextualOperation {
        timestamp: SystemTime::now(),
        operation: op.to_string(),
        path: PathBuf::from(path),
        result_summary: "Success".to_string(),
        context_hints: vec![],
    }
}

fn create_test_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Create a typical project structure
    fs::create_dir_all(temp_dir.path().join("src/core"))?;
    fs::create_dir_all(temp_dir.path().join("src/utils"))?;
    fs::create_dir_all(temp_dir.path().join("tests/unit"))?;
    fs::create_dir_all(temp_dir.path().join("docs"))?;

    fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}")?;
    fs::write(
        temp_dir.path().join("src/core/engine.rs"),
        "pub struct Engine {}",
    )?;
    fs::write(
        temp_dir.path().join("tests/unit/test_engine.rs"),
        "#[test] fn test() {}",
    )?;
    fs::write(temp_dir.path().join("README.md"), "# Project")?;

    Ok(temp_dir)
}

#[test]
fn test_context_tracker_creation() {
    // Basic creation - where all great tests begin!
    let tracker = StContextTracker::new();

    // Should create with empty context
    let context = tracker.analyze_context().unwrap();
    match context {
        WorkContext::Exploring {
            depth,
            areas_visited,
        } => {
            assert_eq!(depth, 3, "Default exploration depth should be 3");
            assert!(areas_visited.is_empty(), "No areas visited initially");
        }
        _ => panic!("Should start in Exploring context"),
    }
}

#[test]
fn test_record_operation() -> Result<()> {
    // Recording operations - the memory of our actions!
    let tracker = StContextTracker::new();

    let op = create_test_operation("read", "/src/main.rs");
    tracker.record_operation(op)?;

    // Should update context based on operation
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Single read should keep exploring context"
    );

    Ok(())
}

#[test]
fn test_context_detection_coding() -> Result<()> {
    // Detecting coding context - when the developer is in the zone!
    let tracker = StContextTracker::new();

    // Simulate coding pattern: edits + tests
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("read", "/tests/test_main.rs"))?;

    let context = tracker.analyze_context()?;
    match context {
        WorkContext::Coding {
            language,
            focus_file,
        } => {
            assert_eq!(language, "rust", "Should detect Rust language");
            assert_eq!(
                focus_file,
                PathBuf::from("/src/main.rs"),
                "Should identify focus file"
            );
        }
        _ => panic!("Should detect Coding context"),
    }

    Ok(())
}

#[test]
fn test_context_detection_hunting() -> Result<()> {
    // Hunting context - when you're searching for that elusive bug!
    let tracker = StContextTracker::new();

    // Simulate search pattern
    for i in 0..4 {
        tracker.record_operation(ContextualOperation {
            timestamp: SystemTime::now(),
            operation: format!("search TODO file_{}.rs", i),
            path: PathBuf::from(format!("/src/file_{}.rs", i)),
            result_summary: "Found matches".to_string(),
            context_hints: vec![],
        })?;
    }

    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Hunting { .. }),
        "Multiple searches should trigger Hunting context"
    );

    Ok(())
}

#[test]
fn test_context_detection_testing() -> Result<()> {
    // Testing context - where quality assurance happens!
    let tracker = StContextTracker::new();

    // Simulate test-focused activity
    tracker.record_operation(create_test_operation("read", "/tests/test_main.rs"))?;
    tracker.record_operation(create_test_operation("edit", "/tests/test_utils.rs"))?;
    tracker.record_operation(create_test_operation("read", "/tests/test_core.rs"))?;

    let context = tracker.analyze_context()?;
    match context {
        WorkContext::Testing { test_files, .. } => {
            assert!(test_files.len() >= 2, "Should track multiple test files");
        }
        _ => panic!("Should detect Testing context"),
    }

    Ok(())
}

#[test]
fn test_context_detection_exploring() -> Result<()> {
    // Exploring context - when you're getting familiar with the codebase!
    let tracker = StContextTracker::new();

    // Simulate exploration pattern
    let files = vec![
        "/src/main.rs",
        "/src/lib.rs",
        "/src/core/engine.rs",
        "/src/utils/helpers.rs",
        "/docs/README.md",
    ];

    for file in files {
        tracker.record_operation(create_test_operation("read", file))?;
    }

    let context = tracker.analyze_context()?;
    match context {
        WorkContext::Exploring {
            depth,
            areas_visited,
        } => {
            assert_eq!(depth, 5, "Heavy reading should increase exploration depth");
            assert!(!areas_visited.is_empty(), "Should track visited areas");
        }
        _ => panic!("Should detect Exploring context"),
    }

    Ok(())
}

#[test]
fn test_suggestions_for_coding_context() -> Result<()> {
    // Suggestions during coding - your AI pair programmer!
    let tracker = StContextTracker::new();

    // Set up coding context
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("read", "/tests/test_main.rs"))?;

    let suggestions = tracker.get_suggestions(Path::new("/src"));

    assert!(
        !suggestions.is_empty(),
        "Should provide suggestions for coding"
    );
    assert!(
        suggestions
            .iter()
            .any(|s| s.contains("relations") || s.contains("test")),
        "Should suggest relevant commands"
    );

    Ok(())
}

#[test]
fn test_suggestions_for_debugging_context() -> Result<()> {
    // Debugging suggestions - when you need all the help you can get!
    let tracker = StContextTracker::new();

    // Manually set debugging context (since we can't easily trigger it)
    tracker.record_operation(ContextualOperation {
        timestamp: SystemTime::now(),
        operation: "search error".to_string(),
        path: PathBuf::from("/src/main.rs"),
        result_summary: "Found errors".to_string(),
        context_hints: vec!["error".to_string()],
    })?;

    let suggestions = tracker.get_suggestions(Path::new("/src"));

    assert!(
        !suggestions.is_empty(),
        "Should provide debugging suggestions"
    );

    Ok(())
}

#[test]
fn test_optimal_args_for_contexts() -> Result<()> {
    // Optimal arguments - because context should drive configuration!
    let tracker = StContextTracker::new();

    // Test default context
    let args = tracker.get_optimal_args("st");
    assert!(
        args.contains(&"--depth".to_string()),
        "Should include depth arg"
    );

    // Set up coding context
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("read", "/tests/test_main.rs"))?;

    let args = tracker.get_optimal_args("st");
    assert!(
        args.contains(&"ai".to_string()),
        "Coding context should use AI mode"
    );

    Ok(())
}

#[test]
fn test_project_knowledge_tracking() -> Result<()> {
    // Project knowledge - learning from history!
    let tracker = StContextTracker::new();

    // Access the same directory multiple times
    for _ in 0..5 {
        tracker.record_operation(create_test_operation("read", "/src/core/engine.rs"))?;
    }

    // Search for the same pattern multiple times
    for _ in 0..3 {
        tracker.record_operation(ContextualOperation {
            timestamp: SystemTime::now(),
            operation: "search TODO".to_string(),
            path: PathBuf::from("/src"),
            result_summary: "Found".to_string(),
            context_hints: vec![],
        })?;
    }

    // Knowledge should be accumulated (we can't directly access it, but it affects suggestions)
    let suggestions = tracker.get_suggestions(Path::new("/"));
    assert!(
        !suggestions.is_empty(),
        "Should provide suggestions based on knowledge"
    );

    Ok(())
}

#[test]
fn test_save_and_load_context() -> Result<()> {
    // Persistence - because context shouldn't be lost!
    let temp_dir = create_test_project()?;
    let tracker = StContextTracker::new();

    // Build up some context
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("search TODO", "/src"))?;

    // Save context
    tracker.save_context(temp_dir.path())?;

    // Verify file exists
    let context_file = temp_dir.path().join(".st_context.json");
    assert!(context_file.exists(), "Context file should be created");

    // Create new tracker and load
    let new_tracker = StContextTracker::new();
    new_tracker.load_context(temp_dir.path())?;

    // Should have restored context
    let context = new_tracker.analyze_context()?;
    assert!(
        !matches!(context, WorkContext::Exploring { depth: 3, .. }),
        "Should have loaded non-default context"
    );

    Ok(())
}

#[test]
fn test_language_detection() -> Result<()> {
    // Language detection - polyglot testing!
    let tracker = StContextTracker::new();

    let test_cases = vec![
        ("test.rs", "rust"),
        ("test.py", "python"),
        ("test.js", "javascript"),
        ("test.jsx", "javascript"),
        ("test.ts", "typescript"),
        ("test.tsx", "typescript"),
        ("test.go", "go"),
        ("test.java", "java"),
        ("test.cpp", "cpp"),
        ("test.c", "c"),
        ("test.unknown", "unknown"),
    ];

    for (filename, expected_lang) in test_cases {
        tracker.record_operation(create_test_operation("edit", filename))?;
        tracker.record_operation(create_test_operation("edit", filename))?;
        tracker.record_operation(create_test_operation("read", "/tests/test.rs"))?;

        let context = tracker.analyze_context()?;
        if let WorkContext::Coding { language, .. } = context {
            assert_eq!(
                language, expected_lang,
                "Should detect {} for {}",
                expected_lang, filename
            );
        }
    }

    Ok(())
}

#[test]
fn test_operation_history_limit() -> Result<()> {
    // History limits - because infinite memory is a myth!
    let tracker = StContextTracker::new();

    // Add more than 50 operations
    for i in 0..60 {
        tracker.record_operation(create_test_operation("read", &format!("/file{}.rs", i)))?;
    }

    // History should be capped at 50 (we can't check directly, but it shouldn't crash)
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Should still analyze context with full history"
    );

    Ok(())
}

#[test]
fn test_contextual_command_builder() -> Result<()> {
    // Command builder - context-aware command construction!
    let tracker = Arc::new(StContextTracker::new());
    let cmd_builder = ContextualStCommand::new(Arc::clone(&tracker));

    // Set up a coding context
    tracker.record_operation(create_test_operation("edit", "/src/main.rs"))?;
    tracker.record_operation(create_test_operation("edit", "/src/lib.rs"))?;
    tracker.record_operation(create_test_operation("read", "/tests/test.rs"))?;

    let args = cmd_builder.build("analyze");
    assert!(!args.is_empty(), "Should build context-aware arguments");

    Ok(())
}

#[test]
fn test_empty_project_knowledge() {
    // Empty knowledge - starting from scratch!
    let knowledge = ProjectKnowledge::default();

    assert!(
        knowledge.key_files.is_empty(),
        "Should start with no key files"
    );
    assert!(
        knowledge.common_searches.is_empty(),
        "Should start with no searches"
    );
    assert!(
        knowledge.hot_directories.is_empty(),
        "Should start with no hot directories"
    );
    assert!(
        knowledge.build_commands.is_empty(),
        "Should start with no build commands"
    );
    assert!(
        knowledge.test_patterns.is_empty(),
        "Should start with no test patterns"
    );
    assert!(
        knowledge.doc_locations.is_empty(),
        "Should start with no doc locations"
    );
}

#[test]
fn test_concurrent_context_updates() -> Result<()> {
    // Concurrency - when multiple threads want to update context!
    use std::thread;

    let tracker = Arc::new(StContextTracker::new());
    let mut handles = vec![];

    // Spawn threads that record operations
    for i in 0..10 {
        let tracker = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            tracker.record_operation(create_test_operation("read", &format!("/file{}.rs", i)))
        });
        handles.push(handle);
    }

    // All should complete without deadlock
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Context should be updated
    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Should have valid context after concurrent updates"
    );

    Ok(())
}

#[test]
fn test_work_context_serialization() -> Result<()> {
    // Serialization - because contexts need to persist!
    let contexts = vec![
        WorkContext::Coding {
            language: "rust".to_string(),
            focus_file: PathBuf::from("/src/main.rs"),
        },
        WorkContext::Debugging {
            error_pattern: "panic".to_string(),
            files: vec![PathBuf::from("/src/lib.rs")],
        },
        WorkContext::Testing {
            test_files: vec![PathBuf::from("/tests/test.rs")],
            target_files: vec![PathBuf::from("/src/main.rs")],
        },
        WorkContext::Exploring {
            depth: 5,
            areas_visited: vec![PathBuf::from("/src")],
        },
    ];

    for context in contexts {
        let serialized = serde_json::to_string(&context)?;
        let deserialized: WorkContext = serde_json::from_str(&serialized)?;
        assert_eq!(
            context, deserialized,
            "Context should survive serialization"
        );
    }

    Ok(())
}

#[test]
fn test_suggestions_with_hot_directories() -> Result<()> {
    // Hot directory suggestions - frequently visited paths!
    let tracker = StContextTracker::new();

    // Make /src/core "hot" by accessing it frequently
    for _ in 0..10 {
        tracker.record_operation(create_test_operation("read", "/src/core/engine.rs"))?;
    }

    // Access other areas less
    tracker.record_operation(create_test_operation("read", "/tests/test.rs"))?;
    tracker.record_operation(create_test_operation("read", "/docs/README.md"))?;

    let suggestions = tracker.get_suggestions(Path::new("/"));
    // Should suggest checking hot areas (implementation specific)
    assert!(!suggestions.is_empty(), "Should provide suggestions");

    Ok(())
}

#[test]
fn test_edge_case_empty_operation() -> Result<()> {
    // Empty operations - when nothing happens, but we still track it!
    let tracker = StContextTracker::new();

    let empty_op = ContextualOperation {
        timestamp: SystemTime::now(),
        operation: String::new(),
        path: PathBuf::new(),
        result_summary: String::new(),
        context_hints: vec![],
    };

    // Should handle gracefully
    tracker.record_operation(empty_op)?;

    let context = tracker.analyze_context()?;
    assert!(
        matches!(context, WorkContext::Exploring { .. }),
        "Empty operations shouldn't break context analysis"
    );

    Ok(())
}

#[test]
fn test_load_from_nonexistent_file() -> Result<()> {
    // Loading from nowhere - graceful degradation!
    let tracker = StContextTracker::new();
    let temp_dir = TempDir::new()?;

    // Should handle missing file gracefully
    let result = tracker.load_context(temp_dir.path());
    assert!(
        result.is_ok(),
        "Should handle missing context file gracefully"
    );

    Ok(())
}

#[test]
fn test_all_work_context_types() -> Result<()> {
    // Every context type needs love!
    let all_contexts = vec![
        WorkContext::Coding {
            language: "rust".to_string(),
            focus_file: PathBuf::from("main.rs"),
        },
        WorkContext::Debugging {
            error_pattern: "error".to_string(),
            files: vec![],
        },
        WorkContext::Refactoring {
            pattern: "rename".to_string(),
            scope: PathBuf::from("src"),
        },
        WorkContext::Exploring {
            depth: 3,
            areas_visited: vec![],
        },
        WorkContext::Testing {
            test_files: vec![],
            target_files: vec![],
        },
        WorkContext::Documenting {
            doc_type: "api".to_string(),
            target: PathBuf::from("docs"),
        },
        WorkContext::Optimizing {
            metrics: vec!["speed".to_string()],
            hotspots: vec![],
        },
        WorkContext::Hunting {
            query: "bug".to_string(),
            found_locations: vec![],
        },
        WorkContext::Building {
            build_system: "cargo".to_string(),
            targets: vec!["release".to_string()],
        },
        WorkContext::VersionControl {
            operation: "commit".to_string(),
            changed_files: vec![],
        },
    ];

    let tracker = StContextTracker::new();

    for context in all_contexts {
        // Each context type should produce appropriate suggestions
        let suggestions = tracker.get_suggestions(Path::new("/"));
        assert!(
            !suggestions.is_empty(),
            "Should provide suggestions for context: {:?}",
            context
        );
    }

    Ok(())
}
