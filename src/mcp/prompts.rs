//! MCP prompts implementation for Smart Tree

use super::McpContext;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct PromptDefinition {
    name: String,
    description: String,
    arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptArgument {
    name: String,
    description: String,
    required: bool,
}

pub async fn handle_prompts_list(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let prompts = vec![
        PromptDefinition {
            name: "analyze_codebase".to_string(),
            description: "Analyze a code repository with AI-optimized output".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to the codebase".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "include_hidden".to_string(),
                    description: "Include hidden files and directories".to_string(),
                    required: false,
                },
            ],
        },
        PromptDefinition {
            name: "find_large_files".to_string(),
            description: "Find the largest files in a directory tree".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to search".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "min_size".to_string(),
                    description: "Minimum file size (e.g., '10M')".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "limit".to_string(),
                    description: "Number of files to show".to_string(),
                    required: false,
                },
            ],
        },
        PromptDefinition {
            name: "recent_changes".to_string(),
            description: "Find recently modified files".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to search".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "days".to_string(),
                    description: "Number of days to look back".to_string(),
                    required: false,
                },
            ],
        },
        PromptDefinition {
            name: "project_structure".to_string(),
            description: "Get a clean project structure overview".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to the project".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "max_depth".to_string(),
                    description: "Maximum directory depth".to_string(),
                    required: false,
                },
            ],
        },
    ];

    Ok(json!({
        "prompts": prompts
    }))
}

pub async fn handle_prompts_get(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing prompt name"))?;
    let arguments = params["arguments"].clone();

    match name {
        "analyze_codebase" => get_analyze_codebase_prompt(arguments),
        "find_large_files" => get_find_large_files_prompt(arguments),
        "recent_changes" => get_recent_changes_prompt(arguments),
        "project_structure" => get_project_structure_prompt(arguments),
        _ => Err(anyhow::anyhow!("Unknown prompt: {}", name)),
    }
}

fn get_analyze_codebase_prompt(args: Value) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;
    let include_hidden = args["include_hidden"].as_bool().unwrap_or(false);

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "Please analyze the codebase at {} using Smart Tree. \
                First use quick_tree to get a 3-level overview, then use analyze_directory with mode='ai' (default) for details. \
                For large codebases (>10k files), switch to mode='claude' with compress=true for 100x compression! \
                {}",
                path,
                if include_hidden { "Include hidden files." } else { "" }
            )
        }
    })];

    Ok(json!({
        "description": "Analyzes a codebase with AI-optimized output",
        "messages": messages
    }))
}

fn get_find_large_files_prompt(args: Value) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;
    let min_size = args["min_size"].as_str().unwrap_or("10M");
    let limit = args["limit"].as_u64().unwrap_or(10);

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "Find the {} largest files in {} that are at least {} in size. \
                Use the find_files tool with appropriate size filters, then sort and limit the results.",
                limit, path, min_size
            )
        }
    })];

    Ok(json!({
        "description": "Finds large files in a directory tree",
        "messages": messages
    }))
}

fn get_recent_changes_prompt(args: Value) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;
    let days = args["days"].as_u64().unwrap_or(7);

    // Calculate date string
    use chrono::{Duration, Local};
    let date = Local::now() - Duration::days(days as i64);
    let date_str = date.format("%Y-%m-%d").to_string();

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "Find all files modified in the last {} days (since {}) in {}. \
                Use the find_files tool with newer_than='{}' parameter.",
                days, date_str, path, date_str
            )
        }
    })];

    Ok(json!({
        "description": "Finds recently modified files",
        "messages": messages
    }))
}

fn get_project_structure_prompt(args: Value) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;
    let max_depth = args["max_depth"].as_u64().unwrap_or(3);

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "Generate a clean project structure overview for {}. \
                Use the analyze_directory tool with mode='classic', max_depth={}, \
                and show_ignored=false to get a clear view of the project layout.",
                path, max_depth
            )
        }
    })];

    Ok(json!({
        "description": "Gets a clean project structure overview",
        "messages": messages
    }))
}
