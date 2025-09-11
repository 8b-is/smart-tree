//! Import Claude Desktop conversations into MEM|8 memory system
//!
//! "Every conversation is a wave in the ocean of consciousness" - Omni

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

// Import from Smart Tree's MEM|8 module
use st::mem8::ConversationMemory;

/// Claude Desktop message format
#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    uuid: Option<String>,
    timestamp: Option<String>,
    cwd: Option<String>,
    sessionId: Option<String>,
    message: Option<MessageContent>,
    summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    role: Option<String>,
    content: Value,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-jsonl-file> [source-name]", args[0]);
        eprintln!("\nExample:");
        eprintln!(
            "  {} ~/.claude/projects/mem8/conversation.jsonl mem8-project",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let source_name = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        input_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    });

    println!(
        "🧠 Importing Claude conversation from: {}",
        input_path.display()
    );
    println!("   Source: {}", source_name);

    // Read the JSONL file
    let file = fs::File::open(input_path).context("Failed to open input file")?;
    let reader = BufReader::new(file);

    let mut messages = Vec::new();
    let mut summaries = Vec::new();
    let mut project_path = None;
    let mut timestamp_range = (None, None);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let msg: ClaudeMessage =
            serde_json::from_str(&line).context("Failed to parse JSONL line")?;

        // Track timestamp range
        if let Some(ts) = &msg.timestamp {
            if timestamp_range.0.is_none() || timestamp_range.0.as_ref().unwrap() > ts {
                timestamp_range.0 = Some(ts.clone());
            }
            if timestamp_range.1.is_none() || timestamp_range.1.as_ref().unwrap() < ts {
                timestamp_range.1 = Some(ts.clone());
            }
        }

        // Extract project path from first message with cwd
        if project_path.is_none() && msg.cwd.is_some() {
            project_path = msg.cwd.clone();
        }

        match msg.msg_type.as_str() {
            "summary" => {
                if let Some(summary) = msg.summary {
                    summaries.push(summary);
                }
            }
            "user" | "assistant" => {
                if let Some(content) = msg.message {
                    let role = content.role.clone().unwrap_or(msg.msg_type.clone());
                    let text = extract_text_from_content(&content.content);

                    if !text.is_empty() {
                        messages.push(json!({
                            "role": role,
                            "content": text,
                            "timestamp": msg.timestamp,
                            "uuid": msg.uuid,
                        }));
                    }
                }
            }
            _ => {}
        }
    }

    println!(
        "📊 Parsed {} messages and {} summaries",
        messages.len(),
        summaries.len()
    );

    if let Some(path) = &project_path {
        println!("📁 Project: {}", path);
    }

    if let (Some(start), Some(end)) = &timestamp_range {
        println!("⏰ Time range: {} to {}", start, end);
    }

    // Create the conversation JSON structure
    let conversation = json!({
        "type": "claude_desktop",
        "source": source_name,
        "project_path": project_path,
        "summaries": summaries,
        "messages": messages,
        "metadata": {
            "import_time": chrono::Utc::now().to_rfc3339(),
            "message_count": messages.len(),
            "time_range": timestamp_range,
        }
    });

    // Initialize conversation memory
    let mut memory =
        ConversationMemory::new().context("Failed to initialize conversation memory")?;

    // Save to MEM|8
    let saved_path = memory
        .save_conversation(&conversation, Some(source_name))
        .context("Failed to save conversation to MEM|8")?;

    println!(
        "✅ Successfully imported conversation to: {}",
        saved_path.display()
    );

    // List all conversations to show it's there
    println!("\n📚 Current conversations in memory:");
    let conversations = memory.list_conversations()?;
    for conv in conversations.iter().take(5) {
        println!("   - {} ({} messages)", conv.file_name, conv.message_count);
    }

    if conversations.len() > 5 {
        println!("   ... and {} more", conversations.len() - 5);
    }

    Ok(())
}

/// Extract text from various Claude content formats
fn extract_text_from_content(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .filter_map(|item| {
                item.get("text")
                    .and_then(|t| t.as_str())
                    .map(|text| text.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(obj) => {
            if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                text.to_string()
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}
