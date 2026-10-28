//! One test in its own integration-test process: DiffStorage uses process cwd.
use serde_json::{json, Value};
use st::mcp::smart_edit::handle_smart_edit;
use std::{fs, path::PathBuf};

struct RestoreCwd(PathBuf);

impl Drop for RestoreCwd {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).expect("restore test cwd");
    }
}

fn replace(name: &str, body: &str) -> Value {
    json!({"operation": "replace_function", "name": name, "new_body": body})
}

#[tokio::test]
async fn batch_modes_preserve_files_on_rejection_and_commit_sequential_edits() {
    let root = tempfile::tempdir().unwrap();
    let _restore = RestoreCwd(std::env::current_dir().unwrap());
    std::env::set_current_dir(root.path()).unwrap();
    let file = root.path().join("example.rs");
    let original = "// café 猫\nfn alpha() -> i32 { 1 }\n";
    fs::write(&file, original).unwrap();

    // A later operation error must discard an earlier successful replacement,
    // stop before subsequent operations, and never initialize diff storage.
    for (edits, expected) in [
        (
            json!([
                replace("alpha", "2"),
                replace("missing", "3"),
                replace("alpha", "4")
            ]),
            "Atomic batch aborted at edit index 1",
        ),
        (
            json!([replace("alpha", "2"), {"operation": "replace_function"}]),
            "Invalid edit at index 1",
        ),
        (
            json!([{"operation": "unknown_operation"}]),
            "Invalid edit at index 0",
        ),
    ] {
        let error = handle_smart_edit(Some(json!({
            "file_path": file, "batch_mode": "atomic", "edits": edits,
        })))
        .await
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
        assert_eq!(fs::read(&file).unwrap(), original.as_bytes());
        assert!(!root.path().join(".st_bumpers").exists());
        assert!(!root.path().join(".gitignore").exists());
    }

    // Invalid mode values cannot silently fall back to partial persistence.
    for mode in [json!("typo"), json!(true), Value::Null] {
        let error = handle_smart_edit(Some(json!({
            "file_path": file, "batch_mode": mode, "edits": [replace("alpha", "2")],
        })))
        .await
        .unwrap_err();
        assert!(error.to_string().contains("Invalid batch_mode"));
        assert_eq!(fs::read_to_string(&file).unwrap(), original);
        assert!(!root.path().join(".st_bumpers").exists());
        assert!(!root.path().join(".gitignore").exists());
    }

    // Existing diff history and ignore rules must also remain byte-identical.
    fs::create_dir(root.path().join(".st_bumpers")).unwrap();
    let receipt = root.path().join(".st_bumpers/existing.diff");
    fs::write(&receipt, "existing receipt").unwrap();
    fs::write(root.path().join(".gitignore"), "target/\n").unwrap();
    assert!(handle_smart_edit(Some(json!({
        "file_path": file, "batch_mode": "atomic",
        "edits": [replace("alpha", "2"), replace("missing", "3")],
    })))
    .await
    .is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), original);
    assert_eq!(fs::read_to_string(&receipt).unwrap(), "existing receipt");
    assert_eq!(
        fs::read_to_string(root.path().join(".gitignore")).unwrap(),
        "target/\n"
    );
    assert_eq!(
        fs::read_dir(root.path().join(".st_bumpers"))
            .unwrap()
            .count(),
        1
    );

    // Sequential semantics: the second operation resolves a newly inserted name.
    let result = handle_smart_edit(Some(json!({
        "file_path": file, "batch_mode": "atomic", "edits": [
            {"operation": "insert_function", "name": "beta", "body": "() -> i32 { 2 }"},
            replace("beta", "3"),
        ],
    })))
    .await
    .unwrap();
    let payload: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["batch_mode"], "atomic");
    assert_eq!(payload["edits_applied"].as_array().unwrap().len(), 2);
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("café 猫"));
    assert!(content.contains("fn beta() -> i32"));
    assert!(content.contains('3'));
    // Compile and execute the resulting source, not only its reported structure.
    let test_source = root.path().join("verify.rs");
    fs::write(
        &test_source,
        format!("{content}\nfn main() {{ assert_eq!(alpha(), 1); assert_eq!(beta(), 3); }}"),
    )
    .unwrap();
    let executable = root.path().join("verify");
    let compiled = std::process::Command::new("rustc")
        .arg(&test_source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(std::process::Command::new(&executable)
        .status()
        .unwrap()
        .success());

    // Omitted mode and explicit best_effort retain the existing partial-edit API.
    for mode in [None, Some("best_effort")] {
        fs::write(&file, original).unwrap();
        let mut params = json!({
            "file_path": file, "edits": [replace("alpha", "2"), replace("missing", "3")],
        });
        if let Some(mode) = mode {
            params["batch_mode"] = json!(mode);
        }
        let result = handle_smart_edit(Some(params)).await.unwrap();
        let payload: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(payload["batch_mode"], "best_effort");
        assert_eq!(payload["edits_applied"][0]["status"], "success");
        assert_eq!(payload["edits_applied"][1]["status"], "error");
        assert!(fs::read_to_string(&file).unwrap().contains('2'));
    }
}
