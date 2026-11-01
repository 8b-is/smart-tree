//! Regression coverage through the public consolidated discovery handlers.
use super::tools_consolidated::{
    dispatch_consolidated_tool, handle_analyze, handle_find, handle_overview,
};
use super::{McpConfig, McpContext, McpServer};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

fn context(root: &Path) -> Arc<McpContext> {
    let server = McpServer::new(McpConfig {
        hex_numbers: false,
        allowed_paths: vec![root.to_path_buf()],
        ..McpConfig::default()
    });
    *server.context.cwd.lock().unwrap() = root.to_path_buf();
    server.context
}

fn payload(result: Value) -> Value {
    serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn missing_overview_does_not_poison_follow_up_paths() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path());
    for mode in ["quick", "project"] {
        for _ in 0..2 {
            let result = handle_overview(
                Some(json!({"path": "default-ctx:", "mode": mode})),
                ctx.clone(),
            )
            .await;
            assert!(
                result.is_err(),
                "missing path returned a successful {mode} overview"
            );
            assert_eq!(*ctx.cwd.lock().unwrap(), temp.path());
        }
        assert!(handle_overview(Some(json!({"mode": mode})), ctx.clone())
            .await
            .is_ok());
    }
    assert!(!temp.path().join("default-ctx:").exists());
}

#[tokio::test]
async fn removed_directory_cannot_be_served_from_cache() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    std::fs::create_dir(&root).unwrap();
    let ctx = context(temp.path());
    let args = json!({"path": root, "mode": "quick"});
    handle_overview(Some(args.clone()), ctx.clone())
        .await
        .unwrap();
    std::fs::remove_dir(&root).unwrap();
    assert!(handle_overview(Some(args), ctx.clone()).await.is_err());
    let file = temp.path().join("file.txt");
    std::fs::write(&file, "not a directory").unwrap();
    assert!(handle_overview(Some(json!({"path": file})), ctx)
        .await
        .is_err());
}

#[test]
fn path_normalization_keeps_creation_targets_without_remembering_missing_parents() {
    let temp = tempfile::tempdir().unwrap();
    let ctx = context(temp.path());
    let mut args = json!({"file_path": "new/child.rs"});
    super::helpers::normalize_path_args(&mut args, &ctx);
    assert_eq!(args["file_path"], json!(temp.path().join("new/child.rs")));
    assert_eq!(*ctx.cwd.lock().unwrap(), temp.path());

    let outside = tempfile::tempdir().unwrap();
    let mut args = json!({"path": outside.path()});
    super::helpers::normalize_path_args(&mut args, &ctx);
    assert_eq!(*ctx.cwd.lock().unwrap(), temp.path());

    let child = temp.path().join("existing");
    std::fs::create_dir(&child).unwrap();
    let mut args = json!({"path": "existing"});
    super::helpers::normalize_path_args(&mut args, &ctx);
    assert_eq!(*ctx.cwd.lock().unwrap(), child);
}

#[tokio::test]
async fn specialized_find_pagination_survives_both_dispatch_layers() {
    let temp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    for index in 0..8 {
        let dir = temp.path().join(format!("project{index}"));
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = 'fixture'\n").unwrap();
        std::fs::write(dir.join("test_example.rs"), "fn example() {}\n").unwrap();
        std::fs::write(dir.join("README.md"), "Fixture\n").unwrap();
    }
    let ctx = context(temp.path());
    for kind in [
        "files",
        "code",
        "config",
        "documentation",
        "tests",
        "build",
        "large",
        "recent",
        "timespan",
    ] {
        let args = json!({
            "path": temp.path(), "type": kind, "min_size": "1",
            "start_date": "2000-01-01"
        });
        let all = payload(handle_find(Some(args.clone()), ctx.clone()).await.unwrap());
        let files = all["files"].as_array().unwrap();
        assert!(
            files.len() > 2,
            "fixture must exercise pagination for {kind}"
        );
        for offset in [0, 2, files.len(), files.len() + 1] {
            let mut page_args = args.clone();
            page_args["limit"] = json!(2);
            page_args["offset"] = json!(offset);
            let page = payload(handle_find(Some(page_args), ctx.clone()).await.unwrap());
            let expected: Vec<_> = files.iter().skip(offset).take(2).cloned().collect();
            assert_eq!(
                page["files"],
                json!(expected),
                "wrong {kind} page at {offset}"
            );
            assert_eq!(page["total_count"], all["total_count"]);
            assert_eq!(page["offset"], json!(offset.to_string()));
            assert_eq!(page["limit_applied"], true);
        }
        let mut offset_only = args.clone();
        offset_only["offset"] = json!(2);
        let page = payload(handle_find(Some(offset_only), ctx.clone()).await.unwrap());
        assert_eq!(page["files"], json!(&files[2..]));
        let mut zero = args.clone();
        zero["limit"] = json!(0);
        let page = payload(handle_find(Some(zero), ctx.clone()).await.unwrap());
        assert_eq!(page["files"], json!([]));
        let mut invalid = args;
        invalid["limit"] = json!(-1);
        assert!(handle_find(Some(invalid), ctx.clone()).await.is_err());
    }
}

#[tokio::test]
async fn consolidated_dispatch_preserves_format_pattern_and_mcp_envelope() {
    let temp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    std::fs::write(temp.path().join("test_alpha.py"), "def test_a(): pass\n").unwrap();
    std::fs::write(temp.path().join("test_beta.rs"), "fn test_b() {}\n").unwrap();
    let ctx = context(temp.path());

    // 1. `find` with `type: "tests"` and custom `pattern` preserves `pattern`
    let filtered = payload(
        handle_find(
            Some(json!({
                "path": temp.path(),
                "type": "tests",
                "pattern": r"test_alpha\.py$"
            })),
            ctx.clone(),
        )
        .await
        .unwrap(),
    );
    let files = filtered["files"].as_array().unwrap();
    assert!(
        files.iter().any(|f| f["name"] == "test_alpha.py"),
        "expected test_alpha.py in filtered results"
    );
    assert!(
        !files.iter().any(|f| f["name"] == "test_beta.rs"),
        "expected test_beta.rs to be excluded by custom pattern"
    );

    // 2. `analyze` with `mode: "directory"` and `format: "json"` preserves `format` -> `mode`
    let dir_json = handle_analyze(
        Some(json!({
            "path": temp.path(),
            "mode": "directory",
            "format": "json"
        })),
        ctx.clone(),
    )
    .await
    .unwrap();
    let dir_text = dir_json["content"][0]["text"].as_str().unwrap();
    assert!(
        serde_json::from_str::<Value>(dir_text).is_ok(),
        "directory analyze with format='json' should produce JSON output, got: {dir_text}"
    );

    // 3. `dispatch_consolidated_tool` guarantees MCP `content` envelope on `unified_watcher` and `hooks`
    for (tool, args) in [
        ("unified_watcher", json!({"action": "status"})),
        ("hooks", json!({"operation": "get_commands"})),
    ] {
        let resp = dispatch_consolidated_tool(tool, Some(args), ctx.clone())
            .await
            .unwrap();
        let text = resp["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("{tool} response missing MCP content[0].text"));
        assert!(!text.is_empty(), "{tool} MCP text should not be empty");
    }

    // 4. `find` works without `type` (defaults to "files") and accepts `type: "all"` alias
    for args in [
        json!({ "path": temp.path() }),
        json!({ "path": temp.path(), "type": "all" }),
    ] {
        let found = payload(handle_find(Some(args), ctx.clone()).await.unwrap());
        let found_files = found["files"].as_array().unwrap();
        assert!(
            found_files.iter().any(|f| f["name"] == "test_alpha.py"),
            "expected default/all find to return test_alpha.py"
        );
    }

    // 5. `analyze` works without `mode` (defaults to "directory")
    let default_analyze = handle_analyze(Some(json!({ "path": temp.path() })), ctx)
        .await
        .unwrap();
    assert!(
        default_analyze["content"][0]["text"].as_str().is_some(),
        "default analyze without mode should succeed"
    );
}
