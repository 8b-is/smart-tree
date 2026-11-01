//! Hooks change cwd by design. Exercise their real stdin/stdout contract in
//! child processes rather than invalidating the library test process's cwd.
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn hook(root: &Path, event: &str, payload: Value) -> Value {
    let parent_cwd = std::env::current_dir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_st"))
        .args(["--agy-hook", event])
        .current_dir(root)
        .env("SMART_TREE_NO_UPDATE_CHECK", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::env::current_dir().unwrap(), parent_cwd);
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn session_start_contract() {
    let dir = tempfile::tempdir().unwrap();
    let res = hook(
        dir.path(),
        "session-start",
        json!({
            "conversationId": "test-conv", "workspacePaths": [dir.path()],
        }),
    );
    let steps = res["injectSteps"].as_array().unwrap();
    assert!(!steps.is_empty());
    assert!(steps[0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap()
        .contains("Smart Tree Session Context"));
}

#[test]
fn pre_tool_use_contract() {
    let dir = tempfile::tempdir().unwrap();
    for (command, decision) in [
        ("cargo test", "allow"),
        ("npx claude-flow@latest init", "ask"),
    ] {
        let res = hook(
            dir.path(),
            "pre-tool-use",
            json!({
                "toolCall": {"name": "run_command", "args": {"CommandLine": command}},
            }),
        );
        assert_eq!(res["decision"], decision);
    }
}

#[test]
fn post_tool_use_and_stop_contract() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = tempfile::tempdir().unwrap();
    let post = hook(
        launcher.path(),
        "post-tool-use",
        json!({
            "workspacePaths": [dir.path()],
            "toolCall": {"name": "write_file", "args": {"Path": "src/lib.rs"}},
        }),
    );
    assert_eq!(post, json!({}));
    let stop = hook(
        launcher.path(),
        "stop",
        json!({
            "workspacePaths": [dir.path()], "terminationReason": "NO_TOOL_CALL",
            "lastUserInput": "Add smart-tree hooks to agy",
        }),
    );
    assert_eq!(stop["decision"], "allow");
    assert!(dir.path().join(".aye_consciousness.m8").exists());
    assert!(!launcher.path().join(".aye_consciousness.m8").exists());
}
