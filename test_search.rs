// Quick test for the new search with line content feature
use serde_json::json;

fn main() {
    // Create a test search request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "search_in_files",
            "arguments": {
                "path": "src",
                "keyword": "TODO",
                "include_content": true,
                "max_matches_per_file": 5
            }
        }
    });
    
    println!("Test request for search_in_files with line content:");
    println!("{}", serde_json::to_string_pretty(&request).unwrap());
    println!("\nExpected result format:");
    println!("{}", serde_json::to_string_pretty(&json!({
        "keyword": "TODO",
        "files_with_matches": 2,
        "include_content": true,
        "max_matches_per_file": 5,
        "results": [
            {
                "path": "/src/main.rs",
                "matches": 3,
                "truncated": false,
                "lines": [
                    {
                        "line_number": 42,
                        "content": "// TODO: Add better error handling",
                        "column": 3
                    },
                    {
                        "line_number": 156,
                        "content": "fn process_todo_items() {",
                        "column": 14
                    }
                ]
            }
        ]
    })).unwrap());
}