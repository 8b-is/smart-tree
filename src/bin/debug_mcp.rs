// Quick debug script to test MCP find_files
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    // Create test directory
    let test_path = PathBuf::from("./tmp");

    // Ensure test directory exists
    std::fs::create_dir_all(&test_path).ok();

    std::fs::create_dir(test_path.join("dir1")).ok();
    std::fs::create_dir(test_path.join("dir2")).ok();
    std::fs::write(test_path.join("file1.txt"), "test").unwrap();
    std::fs::write(test_path.join("file2.txt"), "test").unwrap();

    // Run MCP command
    let mut child = Command::new("./target/release/st")
        .arg("--mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.as_mut().unwrap();

    // Send initialize request first
    let init_request = r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"roots":{"listChanged":true},"sampling":{}}},"id":1}"#;
    stdin.write_all(init_request.as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();

    // Then send the tools/list request to verify
    let list_request = r#"{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}"#;
    stdin.write_all(list_request.as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();

    // Now send our actual request using the correct tool name "overview"
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"overview","arguments":{{"path":"{}","mode":"quick"}}}},"id":3}}"#,
        test_path.to_str().unwrap()
    );
    stdin.write_all(request.as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("=== MCP Response ===");

    // Parse each JSON response
    for (i, line) in stdout.lines().enumerate() {
        if line.starts_with(r#"{"jsonrpc""#) {
            println!("\n--- Response {} ---", i + 1);

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // Check if it's an error response
                if let Some(error) = json.get("error") {
                    println!("ERROR: {}", serde_json::to_string_pretty(&error).unwrap());
                }

                // Check for tool call result
                if json.get("id") == Some(&serde_json::json!(3)) {
                    if let Some(result) = json.get("result") {
                        if let Some(content) = result.get("content") {
                            if let Some(content_array) = content.as_array() {
                                for item in content_array {
                                    if let Some(text) = item["text"].as_str() {
                                        println!("\n=== Tool Output ===");
                                        println!("{}", text);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Just pretty print other responses
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json).unwrap_or(line.to_string())
                    );
                }
            }
        }
    }

    // Also print stderr if there were errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("\n=== STDERR ===");
        println!("{}", stderr);
    }
}
