// Quick debug script to test MCP find_files
use std::process::{Command, Stdio};
use std::io::Write;

fn main() {
    // Create test directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let test_path = temp_dir.path();
    
    std::fs::create_dir(test_path.join("dir1")).unwrap();
    std::fs::create_dir(test_path.join("dir2")).unwrap();
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
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"find_files","arguments":{{"path":"{}","pattern":".*","entry_type":"d"}}}},"id":3}}"#,
        test_path.to_str().unwrap()
    );
    
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(request.as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();
    
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    println!("=== MCP Response ===");
    println!("{}", stdout);
    
    // Try to extract JSON response
    for line in stdout.lines() {
        if line.starts_with(r#"{"jsonrpc""#) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(text) = json["result"]["content"][0]["text"].as_str() {
                    println!("\n=== Parsed Content ===");
                    println!("{}", text);
                    
                    // Pretty print the JSON content
                    if let Ok(content_json) = serde_json::from_str::<serde_json::Value>(text) {
                        println!("\n=== Pretty JSON ===");
                        println!("{}", serde_json::to_string_pretty(&content_json).unwrap());
                    }
                }
            }
        }
    }
}