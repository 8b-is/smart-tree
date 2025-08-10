// Consolidated MCP Tools - Reducing from 50+ to ~15 tools
// Each tool now has a 'mode' or 'type' parameter to specify the operation

use crate::mcp::McpContext;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// Consolidated find tool - combines all find_* operations
pub async fn handle_find(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let find_type = params["type"].as_str().context("type parameter required")?;

    // Transform parameters to match original tool format and dispatch
    let transformed_params = match find_type {
        "files" => params.clone(),
        "code" => {
            // Transform to find_code_files format
            params.clone()
        }
        "config" | "documentation" | "tests" | "build" => {
            // These just need path
            json!({ "path": params.get("path").unwrap_or(&json!(".")) })
        }
        "large" => {
            json!({
                "path": params.get("path").unwrap_or(&json!(".")),
                "min_size": params.get("min_size").unwrap_or(&json!("10M"))
            })
        }
        "recent" => {
            json!({
                "path": params.get("path").unwrap_or(&json!(".")),
                "days": params.get("days").unwrap_or(&json!(7))
            })
        }
        "timespan" => params.clone(),
        "duplicates" | "empty_dirs" => {
            json!({ "path": params.get("path").unwrap_or(&json!(".")) })
        }
        _ => return Err(anyhow::anyhow!("Unknown find type: {}", find_type)),
    };

    // Call the appropriate original tool
    let tool_name = match find_type {
        "files" => "find_files",
        "code" => "find_code_files",
        "config" => "find_config_files",
        "documentation" => "find_documentation",
        "tests" => "find_tests",
        "build" => "find_build_files",
        "large" => "find_large_files",
        "recent" => "find_recent_changes",
        "timespan" => "find_in_timespan",
        "duplicates" => "find_duplicates",
        "empty_dirs" => "find_empty_directories",
        _ => return Err(anyhow::anyhow!("Unknown find type: {}", find_type)),
    };

    // Call through the regular tools handler
    super::tools::handle_tools_call(
        json!({ "name": tool_name, "arguments": transformed_params }),
        ctx,
    )
    .await
}

/// Consolidated analyze tool - combines analysis operations
pub async fn handle_analyze(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let mode = params["mode"].as_str().context("mode parameter required")?;

    // Transform parameters based on the tool
    let (tool_name, transformed_params) = match mode {
        "directory" => {
            // analyze_directory expects 'mode' not 'format', but we rename 'format' to 'mode'
            let mut p = params.clone();
            if let Some(format) = p.get("format") {
                p["mode"] = format.clone();
                p.as_object_mut().unwrap().remove("format");
            }
            // Remove the consolidated 'mode' field as analyze_directory doesn't expect it
            p.as_object_mut().unwrap().remove("mode");
            ("analyze_directory", p)
        }
        "workspace" => ("analyze_workspace", params.clone()),
        "statistics" => ("get_statistics", params.clone()),
        "git_status" => ("get_git_status", params.clone()),
        "digest" => ("get_digest", params.clone()),
        "semantic" => ("semantic_analysis", params.clone()),
        "size_breakdown" => ("directory_size_breakdown", params.clone()),
        "ai_tools" => ("analyze_ai_tool_usage", params.clone()),
        _ => return Err(anyhow::anyhow!("Unknown analyze mode: {}", mode)),
    };

    super::tools::handle_tools_call(
        json!({ "name": tool_name, "arguments": transformed_params }),
        ctx,
    )
    .await
}

/// Consolidated search tool
pub async fn handle_search(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    super::tools::handle_tools_call(
        json!({ "name": "search_in_files", "arguments": params }),
        ctx,
    )
    .await
}

/// Consolidated overview tool - quick_tree and project_overview
pub async fn handle_overview(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let mode = params
        .get("mode")
        .and_then(|m| m.as_str())
        .unwrap_or("quick");

    let tool_name = match mode {
        "quick" => "quick_tree",
        "project" => "project_overview",
        _ => return Err(anyhow::anyhow!("Unknown overview mode: {}", mode)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Consolidated edit tool - combines all Smart Edit operations
pub async fn handle_edit(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let operation = params["operation"].as_str().context("operation required")?;

    let tool_name = match operation {
        "smart_edit" => "smart_edit",
        "get_functions" => "get_function_tree",
        "insert_function" => "insert_function",
        "remove_function" => "remove_function",
        _ => return Err(anyhow::anyhow!("Unknown edit operation: {}", operation)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Consolidated history tool - file tracking and history
pub async fn handle_history(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let operation = params["operation"].as_str().context("operation required")?;

    let tool_name = match operation {
        "track" => "track_file_operation",
        "get_file" => "get_file_history",
        "get_project" => "get_project_history_summary",
        _ => return Err(anyhow::anyhow!("Unknown history operation: {}", operation)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Consolidated context tool - project context and collaboration
pub async fn handle_context(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let operation = params["operation"].as_str().context("operation required")?;

    let tool_name = match operation {
        "gather_project" => "gather_project_context",
        "collaboration_rapport" => "get_collaboration_rapport",
        "engagement_heatmap" => "get_co_engagement_heatmap",
        "cross_domain_patterns" => "get_cross_domain_patterns",
        "suggest_insights" => "suggest_cross_session_insights",
        _ => return Err(anyhow::anyhow!("Unknown context operation: {}", operation)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Consolidated memory tool - collaborative memories
pub async fn handle_memory(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let operation = params["operation"].as_str().context("operation required")?;

    let tool_name = match operation {
        "anchor" => "anchor_collaborative_memory",
        "find" => "find_collaborative_memories",
        _ => return Err(anyhow::anyhow!("Unknown memory operation: {}", operation)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Consolidated compare tool
pub async fn handle_compare(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    super::tools::handle_tools_call(
        json!({ "name": "compare_directories", "arguments": params }),
        ctx,
    )
    .await
}

/// Consolidated SSE tool - Server-Sent Events
pub async fn handle_sse(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    super::tools::handle_tools_call(
        json!({ "name": "watch_directory_sse", "arguments": params }),
        ctx,
    )
    .await
}

/// Consolidated feedback tool
pub async fn handle_feedback(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let operation = params["operation"].as_str().context("operation required")?;

    let tool_name = match operation {
        "submit" => "submit_feedback",
        "request_tool" => "request_tool",
        "check_updates" => "check_for_updates",
        _ => return Err(anyhow::anyhow!("Unknown feedback operation: {}", operation)),
    };

    super::tools::handle_tools_call(json!({ "name": tool_name, "arguments": params }), ctx).await
}

/// Get consolidated tool list
pub fn get_consolidated_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "find",
            "description": "Find files, code, config, tests, documentation, duplicates, etc.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["files", "code", "config", "documentation", "tests", "build",
                                 "large", "recent", "timespan", "duplicates", "empty_dirs"],
                        "description": "Type of items to find"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Pattern to match (for files type)"
                    },
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Programming languages (for code type)"
                    },
                    "days": {
                        "type": "integer",
                        "description": "Number of days (for recent type)"
                    },
                    "min_size": {
                        "type": "string",
                        "description": "Minimum size (for large type)"
                    },
                    "start_date": {
                        "type": "string",
                        "description": "Start date YYYY-MM-DD (for timespan type)"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date YYYY-MM-DD (for timespan type)"
                    }
                },
                "required": ["type"]
            }
        }),
        json!({
            "name": "analyze",
            "description": "Analyze directories, workspaces, statistics, git status, etc.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["directory", "workspace", "statistics", "git_status",
                                 "digest", "semantic", "size_breakdown", "ai_tools"],
                        "description": "Analysis mode"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to analyze"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format (for directory mode)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth"
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Show hidden files"
                    }
                },
                "required": ["mode"]
            }
        }),
        json!({
            "name": "search",
            "description": "Search for content within files",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "keyword": {
                        "type": "string",
                        "description": "Keyword or pattern to search"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "File type to search"
                    },
                    "context_lines": {
                        "type": "integer",
                        "description": "Number of context lines"
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include file content in results"
                    }
                },
                "required": ["keyword"]
            }
        }),
        json!({
            "name": "overview",
            "description": "Get quick tree overview or comprehensive project overview",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["quick", "project"],
                        "description": "Overview mode"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to overview"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum depth (for quick mode)"
                    }
                },
                "required": []
            }
        }),
        json!({
            "name": "edit",
            "description": "Smart code editing operations - AST-aware with 90% token reduction",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["smart_edit", "get_functions", "insert_function", "remove_function"],
                        "description": "Edit operation"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File to edit"
                    },
                    "edits": {
                        "type": "array",
                        "description": "Array of edit operations (for smart_edit)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Function name"
                    },
                    "body": {
                        "type": "string",
                        "description": "Function body"
                    }
                },
                "required": ["operation", "file_path"]
            }
        }),
        json!({
            "name": "history",
            "description": "Track and retrieve file operation history",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["track", "get_file", "get_project"],
                        "description": "History operation"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path"
                    },
                    "op": {
                        "type": "string",
                        "description": "File operation type"
                    },
                    "old_content": {
                        "type": "string",
                        "description": "Previous content"
                    },
                    "new_content": {
                        "type": "string",
                        "description": "New content"
                    }
                },
                "required": ["operation"]
            }
        }),
        json!({
            "name": "context",
            "description": "Project context, collaboration rapport, and cross-domain patterns",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["gather_project", "collaboration_rapport", "engagement_heatmap",
                                 "cross_domain_patterns", "suggest_insights"],
                        "description": "Context operation"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path"
                    },
                    "ai_tool": {
                        "type": "string",
                        "description": "AI tool name"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords for search"
                    }
                },
                "required": ["operation"]
            }
        }),
        json!({
            "name": "memory",
            "description": "Anchor and find collaborative memories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["anchor", "find"],
                        "description": "Memory operation"
                    },
                    "context": {
                        "type": "string",
                        "description": "Memory context"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords"
                    },
                    "anchor_type": {
                        "type": "string",
                        "description": "Type of anchor"
                    }
                },
                "required": ["operation", "keywords"]
            }
        }),
        json!({
            "name": "compare",
            "description": "Compare two directories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path1": {
                        "type": "string",
                        "description": "First directory"
                    },
                    "path2": {
                        "type": "string",
                        "description": "Second directory"
                    }
                },
                "required": ["path1", "path2"]
            }
        }),
        json!({
            "name": "sse",
            "description": "Watch directory with Server-Sent Events",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to watch"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format"
                    },
                    "heartbeat_interval": {
                        "type": "integer",
                        "description": "Heartbeat interval in seconds"
                    }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "feedback",
            "description": "Submit feedback, request tools, or check updates",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["submit", "request_tool", "check_updates"],
                        "description": "Feedback operation"
                    },
                    "category": {
                        "type": "string",
                        "description": "Feedback category"
                    },
                    "title": {
                        "type": "string",
                        "description": "Feedback title"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description"
                    }
                },
                "required": ["operation"]
            }
        }),
        json!({
            "name": "server_info",
            "description": "Get Smart Tree server information and capabilities",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "verify_permissions",
            "description": "Verify permissions for a path",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to verify"
                    }
                },
                "required": ["path"]
            }
        }),
    ]
}

/// Tool dispatcher for consolidated tools
pub async fn dispatch_consolidated_tool(
    name: &str,
    params: Option<Value>,
    ctx: Arc<McpContext>,
) -> Result<Value> {
    match name {
        "find" => handle_find(params, ctx).await,
        "analyze" => handle_analyze(params, ctx).await,
        "search" => handle_search(params, ctx).await,
        "overview" => handle_overview(params, ctx).await,
        "edit" => handle_edit(params, ctx).await,
        "history" => handle_history(params, ctx).await,
        "context" => handle_context(params, ctx).await,
        "memory" => handle_memory(params, ctx).await,
        "compare" => handle_compare(params, ctx).await,
        "sse" => handle_sse(params, ctx).await,
        "feedback" => handle_feedback(params, ctx).await,
        "server_info" => {
            super::tools::handle_tools_call(
                json!({ "name": "server_info", "arguments": params }),
                ctx,
            )
            .await
        }
        "verify_permissions" => {
            super::tools::handle_tools_call(
                json!({ "name": "verify_permissions", "arguments": params }),
                ctx,
            )
            .await
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}
