//! Tests for MCP session-aware compression negotiation

use st::mcp::session::*;
use std::path::PathBuf;
use std::time::Duration;
use tokio;

/// Test session manager creation and retrieval
#[tokio::test]
async fn test_session_manager() {
    let manager = SessionManager::new();

    // Create new session
    let session1 = manager.get_or_create(None).await;
    assert!(!session1.id.is_empty());
    assert!(!session1.negotiated);

    // Retrieve existing session
    let session2 = manager.get_or_create(Some(session1.id.clone())).await;
    assert_eq!(session1.id, session2.id);

    // Create another new session
    let session3 = manager.get_or_create(None).await;
    assert_ne!(session1.id, session3.id);
}

/// Test session cleanup
#[tokio::test]
async fn test_session_cleanup() {
    let manager = SessionManager::new();

    // Create session
    let session = manager.get_or_create(None).await;
    let session_id = session.id.clone();

    // Update to add to manager
    manager.update(session).await;

    // Session should exist
    let retrieved = manager.get_or_create(Some(session_id.clone())).await;
    assert_eq!(retrieved.id, session_id);

    // Run cleanup (sessions < 1 hour old should remain)
    manager.cleanup().await;

    // Session should still exist
    let still_there = manager.get_or_create(Some(session_id.clone())).await;
    assert_eq!(still_there.id, session_id);
}

/// Test compression mode conversion
#[test]
fn test_compression_mode_conversion() {
    assert_eq!(CompressionMode::None.to_output_mode(), "classic");
    assert_eq!(CompressionMode::Light.to_output_mode(), "ai");
    assert_eq!(CompressionMode::Standard.to_output_mode(), "summary-ai");
    assert_eq!(CompressionMode::Quantum.to_output_mode(), "quantum");
    assert_eq!(
        CompressionMode::QuantumSemantic.to_output_mode(),
        "quantum-semantic"
    );
    assert_eq!(CompressionMode::Auto.to_output_mode(), "auto");
}

/// Test project path inference
#[test]
fn test_project_path_inference() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create project marker
    fs::write(project_path.join("Cargo.toml"), "[package]").unwrap();

    // Change to project directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_path).unwrap();

    // Create session - should infer project path
    let session = McpSession::from_context(None);
    assert_eq!(
        session.project_path.file_name().unwrap(),
        project_path.file_name().unwrap()
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

/// Test negotiation without preferences
#[test]
fn test_negotiation_without_preferences() {
    let mut session = McpSession::new();
    let response = session.negotiate(None);

    assert!(!response.accepted); // Should not accept without preferences
    assert!(!session.negotiated);
    assert_eq!(response.tools_available.len(), 2); // Minimal tools
}

/// Test negotiation with preferences
#[test]
fn test_negotiation_with_preferences() {
    let mut session = McpSession::new();

    let prefs = SessionPreferences {
        format: CompressionMode::QuantumSemantic,
        depth: DepthMode::Deep,
        tools: ToolAdvertisement::All,
        project_path: Some(PathBuf::from("/custom/path")),
    };

    let response = session.negotiate(Some(prefs));

    assert!(response.accepted);
    assert!(session.negotiated);
    assert_eq!(response.format, CompressionMode::QuantumSemantic);
    assert!(response.tools_available.len() > 10); // All tools
}

/// Test tool context application
#[test]
fn test_tool_context_application() {
    use serde_json::json;

    let mut session = McpSession::from_context(Some(PathBuf::from("/test/project")));
    session.preferences.format = CompressionMode::Quantum;

    // Test path injection
    let mut params = json!({
        "other_param": "value"
    });

    session.apply_context("find", &mut params);

    assert_eq!(params["path"].as_str().unwrap(), "/test/project");
    assert_eq!(params["other_param"].as_str().unwrap(), "value");

    // Test mode injection for overview
    let mut overview_params = json!({});
    session.apply_context("overview", &mut overview_params);

    assert_eq!(overview_params["mode"].as_str().unwrap(), "quantum");
}

/// Test all tool advertisement modes
#[test]
fn test_all_tool_advertisement_modes() {
    let session = McpSession::new();

    // Test All mode
    let mut test_session = session.clone();
    test_session.preferences.tools = ToolAdvertisement::All;
    let all_tools = test_session.get_available_tools();
    assert!(all_tools.len() >= 10);

    // Test Lazy mode
    test_session.preferences.tools = ToolAdvertisement::Lazy;
    let lazy_tools = test_session.get_available_tools();
    assert_eq!(lazy_tools.len(), 3);
    assert!(lazy_tools.contains(&"overview".to_string()));

    // Test Minimal mode
    test_session.preferences.tools = ToolAdvertisement::Minimal;
    let minimal_tools = test_session.get_available_tools();
    assert_eq!(minimal_tools.len(), 1);
    assert_eq!(minimal_tools[0], "overview");
}

/// Test environment variable parsing
#[test]
fn test_env_var_parsing() {
    // Test valid values
    std::env::set_var("ST_COMPRESSION", "none");
    assert_eq!(CompressionMode::from_env(), CompressionMode::None);

    std::env::set_var("ST_COMPRESSION", "QUANTUM"); // Case insensitive
    assert_eq!(CompressionMode::from_env(), CompressionMode::Quantum);

    std::env::set_var("ST_COMPRESSION", "max"); // Alias for quantum-semantic
    assert_eq!(
        CompressionMode::from_env(),
        CompressionMode::QuantumSemantic
    );

    // Test invalid value defaults to Auto
    std::env::set_var("ST_COMPRESSION", "invalid");
    assert_eq!(CompressionMode::from_env(), CompressionMode::Auto);

    // Clean up
    std::env::remove_var("ST_COMPRESSION");
}

/// Test session ID generation
#[test]
fn test_session_id_generation() {
    let session1 = McpSession::new();
    let session2 = McpSession::new();

    // IDs should be unique
    assert_ne!(session1.id, session2.id);

    // IDs should follow format
    assert!(session1.id.starts_with("STX-"));
    assert!(session2.id.starts_with("STX-"));
}

/// Test depth mode calculations
#[test]
fn test_depth_calculations() {
    // Shallow mode
    assert_eq!(DepthMode::Shallow.to_depth(100), 2);

    // Standard mode
    assert_eq!(DepthMode::Standard.to_depth(100), 4);

    // Deep mode
    assert_eq!(DepthMode::Deep.to_depth(100), 10);

    // Adaptive mode varies by directory count
    assert_eq!(DepthMode::Adaptive.to_depth(5), 10); // Small: deep
    assert_eq!(DepthMode::Adaptive.to_depth(25), 5); // Medium: moderate
    assert_eq!(DepthMode::Adaptive.to_depth(75), 4); // Large: shallow
    assert_eq!(DepthMode::Adaptive.to_depth(150), 3); // Huge: very shallow
}
