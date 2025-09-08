//! Integration tests for Claude-specific features
//! Tests claude_init, context mode, and session negotiation

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test Claude project initialization
#[test]
fn test_claude_init_rust_project() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create Rust project markers
    fs::write(
        project_path.join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::write(project_path.join("main.rs"), "fn main() {}").unwrap();

    // Initialize Claude integration
    let initializer = st::claude_init::ClaudeInit::new(project_path.clone()).unwrap();
    initializer.setup().unwrap();

    // Verify .claude directory was created
    assert!(project_path.join(".claude").exists());
    assert!(project_path.join(".claude/settings.json").exists());
    assert!(project_path.join(".claude/CLAUDE.md").exists());

    // Verify settings content
    let settings = fs::read_to_string(project_path.join(".claude/settings.json")).unwrap();
    assert!(settings.contains("\"project_type\": \"Rust\""));
    assert!(settings.contains("\"auto_configured\": true"));
    assert!(settings.contains("st -m")); // Should have Smart Tree hooks
}

/// Test Claude init for Python project
#[test]
fn test_claude_init_python_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create Python project markers
    fs::write(project_path.join("requirements.txt"), "pytest\nnumpy").unwrap();
    fs::write(
        project_path.join("pyproject.toml"),
        "[project]\nname = \"test\"",
    )
    .unwrap();
    fs::write(project_path.join("main.py"), "def main(): pass").unwrap();

    // Initialize
    let initializer = st::claude_init::ClaudeInit::new(project_path.clone()).unwrap();
    initializer.setup().unwrap();

    // Verify Python-specific configuration
    let settings = fs::read_to_string(project_path.join(".claude/settings.json")).unwrap();
    assert!(settings.contains("\"project_type\": \"Python\""));

    let claude_md = fs::read_to_string(project_path.join(".claude/CLAUDE.md")).unwrap();
    assert!(claude_md.contains("uv sync")); // Python-specific commands
    assert!(claude_md.contains("pytest"));
}

/// Test update behavior on existing .claude directory
#[test]
fn test_claude_update_existing() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create initial .claude directory
    fs::create_dir_all(project_path.join(".claude")).unwrap();
    fs::write(
        project_path.join(".claude/settings.json"),
        r#"{"smart_tree": {"auto_configured": true}}"#,
    )
    .unwrap();

    // Run setup (should update, not error)
    let initializer = st::claude_init::ClaudeInit::new(project_path.clone()).unwrap();
    initializer.setup().unwrap();

    // Should have updated files
    assert!(project_path.join(".claude/CLAUDE.md").exists());
}

/// Test manual configuration protection
#[test]
fn test_claude_preserves_manual_config() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create manual configuration (no auto_configured flag)
    fs::create_dir_all(project_path.join(".claude")).unwrap();
    fs::write(
        project_path.join(".claude/settings.json"),
        r#"{"custom": "settings", "hooks": {}}"#,
    )
    .unwrap();

    // Run setup
    let initializer = st::claude_init::ClaudeInit::new(project_path.clone()).unwrap();
    initializer.setup().unwrap();

    // Should NOT overwrite manual settings
    let settings = fs::read_to_string(project_path.join(".claude/settings.json")).unwrap();
    assert!(settings.contains("\"custom\": \"settings\""));
}

/// Test compression mode selection based on project size
#[test]
fn test_compression_mode_auto_selection() {
    use st::mcp::session::CompressionMode;

    // Test auto-selection based on file count
    assert_eq!(CompressionMode::auto_select(10), CompressionMode::None);
    assert_eq!(CompressionMode::auto_select(100), CompressionMode::Light);
    assert_eq!(CompressionMode::auto_select(300), CompressionMode::Standard);
    assert_eq!(CompressionMode::auto_select(700), CompressionMode::Quantum);
    assert_eq!(
        CompressionMode::auto_select(2000),
        CompressionMode::QuantumSemantic
    );
}

/// Test context mode formatter
#[test]
fn test_context_mode_output() {
    use st::formatters::{context::ContextFormatter, Formatter};
    use st::{FileNode, TreeStats};
    use std::path::Path;

    // Create test data
    let nodes = vec![
        FileNode {
            path: PathBuf::from("src/main.rs"),
            metadata: st::FileMetadata {
                size: 1024,
                modified: None,
                created: None,
                accessed: None,
                permissions: None,
                file_type: st::FileType::RegularFile,
                is_hidden: false,
                is_symlink: false,
                target: None,
                depth: 1,
                filesystem_type: None,
            },
            content_type: None,
            line_content: None,
        },
        FileNode {
            path: PathBuf::from("Cargo.toml"),
            metadata: st::FileMetadata {
                size: 256,
                modified: None,
                created: None,
                accessed: None,
                permissions: None,
                file_type: st::FileType::RegularFile,
                is_hidden: false,
                is_symlink: false,
                target: None,
                depth: 0,
                filesystem_type: None,
            },
            content_type: None,
            line_content: None,
        },
    ];

    let stats = TreeStats {
        total_files: 2,
        total_dirs: 1,
        total_size: 1280,
        max_depth: 1,
        hidden_files: 0,
        hidden_dirs: 0,
        total_lines: None,
    };

    // Format with context formatter
    let formatter = ContextFormatter::new();
    let mut output = Vec::new();
    formatter
        .format(&mut output, &nodes, &stats, Path::new("."))
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();

    // Verify output contains expected sections
    assert!(output_str.contains("=== Smart Tree Context ==="));
    assert!(output_str.contains("📁 Project:"));
    assert!(output_str.contains("🌳 Structure:"));
    assert!(output_str.contains("STATS:F2D1S")); // 2 files, 1 dir, size encoded
}

/// Test MCP session negotiation
#[test]
fn test_mcp_session_negotiation() {
    use st::mcp::session::{
        CompressionMode, DepthMode, McpSession, SessionPreferences, ToolAdvertisement,
    };

    // Create new session
    let mut session = McpSession::new();
    assert!(!session.negotiated);

    // Negotiate with preferences
    let prefs = SessionPreferences {
        format: CompressionMode::Quantum,
        depth: DepthMode::Adaptive,
        tools: ToolAdvertisement::Lazy,
        project_path: Some(PathBuf::from("/test/project")),
    };

    let response = session.negotiate(Some(prefs));

    // Verify negotiation succeeded
    assert!(response.accepted);
    assert!(session.negotiated);
    assert_eq!(response.format, CompressionMode::Quantum);
    assert_eq!(response.tools_available.len(), 3); // Lazy mode: minimal tools
}

/// Test session context application
#[test]
fn test_session_context_application() {
    use serde_json::json;
    use st::mcp::session::{CompressionMode, McpSession, SessionPreferences};

    let session = McpSession::from_context(Some(PathBuf::from("/test/project")));

    // Test applying context to tool params
    let mut params = json!({});
    session.apply_context("overview", &mut params);

    // Should have injected project path
    assert_eq!(
        params.get("path").unwrap().as_str().unwrap(),
        "/test/project"
    );
}

/// Test environment variable compression mode
#[test]
fn test_compression_from_env() {
    use st::mcp::session::CompressionMode;

    // Test with environment variable set
    std::env::set_var("ST_COMPRESSION", "quantum");
    assert_eq!(CompressionMode::from_env(), CompressionMode::Quantum);

    std::env::set_var("ST_COMPRESSION", "quantum-semantic");
    assert_eq!(
        CompressionMode::from_env(),
        CompressionMode::QuantumSemantic
    );

    // Clean up
    std::env::remove_var("ST_COMPRESSION");
    assert_eq!(CompressionMode::from_env(), CompressionMode::Auto);
}

/// Test depth mode adaptive behavior
#[test]
fn test_depth_mode_adaptive() {
    use st::mcp::session::DepthMode;

    let depth = DepthMode::Adaptive;

    // Small directory: deep traversal
    assert_eq!(depth.to_depth(5), 10);

    // Medium directory: moderate depth
    assert_eq!(depth.to_depth(30), 5);

    // Large directory: shallow to avoid overwhelm
    assert_eq!(depth.to_depth(200), 3);
}

/// Test tool advertisement strategies
#[test]
fn test_tool_advertisement() {
    use st::mcp::session::{McpSession, SessionPreferences, ToolAdvertisement};
    use std::fs;

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create git repo marker
    fs::create_dir_all(project_path.join(".git")).unwrap();

    let mut session = McpSession::from_context(Some(project_path.clone()));
    session.preferences.tools = ToolAdvertisement::ContextAware;

    let tools = session.get_available_tools();

    // Should include git-aware tools
    assert!(tools.contains(&"history".to_string()));
}

/// Integration test for full Claude setup flow
#[test]
fn test_full_claude_integration_flow() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create a mixed project
    fs::write(
        project_path.join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();
    fs::write(project_path.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(project_path.join("main.rs"), "fn main() {}").unwrap();
    fs::write(project_path.join("index.js"), "console.log('test')").unwrap();

    // Initialize Claude
    let initializer = st::claude_init::ClaudeInit::new(project_path.clone()).unwrap();
    initializer.setup().unwrap();

    // Verify setup completed
    assert!(project_path.join(".claude").exists());

    // Run setup again (should update, not fail)
    initializer.setup().unwrap();

    // Files should still exist
    assert!(project_path.join(".claude/settings.json").exists());
    assert!(project_path.join(".claude/CLAUDE.md").exists());
}
