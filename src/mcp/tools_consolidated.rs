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
        "projects" => {
            // Projects discovery - finds forgotten 3am coding gems!
            json!({
                "path": params.get("path").unwrap_or(&json!(".")),
                "depth": params.get("depth").unwrap_or(&json!(10))
            })
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
        "projects" => "find_projects",
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
        "quantum-semantic" => {
            // quantum-semantic uses analyze_directory with quantum-semantic mode
            let mut p = params.clone();
            p["mode"] = json!("quantum-semantic");
            ("analyze_directory", p)
        }
        "size_breakdown" => ("directory_size_breakdown", params.clone()),
        "ai_tools" => ("analyze_ai_tool_usage", params.clone()),
        "security" | "custodian" => return handle_security(Some(params), ctx).await,
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
        "create_file" => "create_file",
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
#[allow(dead_code)]
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
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (for pagination)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Number of results to skip (for pagination)"
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
                        "description": "Array of edit operations (for smart_edit)",
                        "items": {
                            "type": "object",
                            "description": "Individual edit operation"
                        }
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
                    },
                    "origin": {
                        "type": "string",
                        "description": "Who created this? 'human', 'ai:claude', or 'tandem:human:claude' (default: tandem:human:claude)"
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
        json!({
            "name": "project_context_dump",
            "description": "📦 FULL PROJECT CONTEXT - Get a complete, token-efficient project dump for AI assistants in ONE CALL! Includes directory tree, key files, git info, with configurable compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the project root"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum directory depth (1-20, default: 5)"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum files to include (10-1000, default: 100)"
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include contents of key files (default: false)"
                    },
                    "compression": {
                        "type": "string",
                        "enum": ["auto", "marqant", "summary-ai", "quantum"],
                        "description": "Compression mode (default: auto = summary-ai)"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Token budget warning threshold (default: 10000)"
                    },
                    "include_git": {
                        "type": "boolean",
                        "description": "Include git status info (default: true)"
                    },
                    "key_files_only": {
                        "type": "boolean",
                        "description": "Only show key project files (default: false)"
                    }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "ask_user",
            "description": "🗣️ Ask the human user a question and wait for their answer via the Smart Tree web dashboard. Ideal for clarifying requirements, getting explicit permission, or prompting for input without filling the chat log.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question or prompt to ask the user"
                    }
                },
                "required": ["question"]
            }
        }),
        json!({
            "name": "symbol",
            "description": "🦀 CODE SYMBOL & CALL GRAPH INTELLIGENCE (Unified CrabCC + Smart Tree AST) — Built-in CrabCC symbol definition lookup (with source code & docstrings), cross-file call-graph references, file symbol outlines, and repo indexing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["sym", "refs", "outline", "index", "graph"],
                        "description": "Symbol operation: 'sym' (find definition + source code), 'refs' (find call-graph references), 'outline' (list all symbols in a file), 'index' (rebuild index), 'graph' (dependency stats)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Symbol name to look up (for 'sym' and 'refs', e.g. 'quantize_ternary', 'fold_spectrum')"
                    },
                    "file": {
                        "type": "string",
                        "description": "File path to outline (for 'outline', e.g. 'rust/alexiai/src/server.rs')"
                    },
                    "path": {
                        "type": "string",
                        "description": "Repository root path (default: current MCP session directory)"
                    },
                    "include_source": {
                        "type": "boolean",
                        "description": "Include actual source code lines & docstrings around definitions/references (default: true)"
                    }
                },
                "required": ["operation"]
            }
        }),
        json!({
            "name": "security",
            "description": "🛡️ CUSTODIAN SECURITY SCANNER — AST & supply-chain threat detection, risk scoring, and suspicious capability analysis across any directory or repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory or repository path to scan (default: '.')"
                    }
                },
                "required": []
            }
        }),
        json!({
            "name": "ontology",
            "description": "🐉 SOVEREIGN LIBRARY THREE-KAYA ONTOLOGY — 100% offline Balanced Ternary {-1, 0, +1} (Samsaric Trap / Dharmakāya Śūnyatā / Sambhogakāya-Nirmāṇakāya), Six Pillars of Holding, Daizenshuu Scouter & Constitutional NAND(HELD, READ) = 0 receipt.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Entity, concept, or system to evaluate (e.g. 'Ultra Instinct', 'Goku', 'Vegeta', 'ALEXIAI', 'GAIA-MLX-QUANT', 'Smart Tree')"
                    }
                },
                "required": ["target"]
            }
        }),
        json!({
            "name": "partner",
            "description": "💗 WA-STREAM × DISCORD × ALEXIAI PARTNER-IN-LIFE BRIDGE — Bidirectional WhatsApp ('little dude(s) and Angels' on 127.0.0.1:8788) & Discord automation (127.0.0.1:8789), Dra CEO Helper briefs (/dra, /buffett, /family+1, /ghibli, /omni, /future), inbox/outbox sync, and live partner messaging.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["status", "inbox", "outbox", "send", "slash", "ask"],
                        "description": "Partner operation: 'status' (connection & queue health), 'inbox' (read WhatsApp & Discord messages), 'outbox' (recent sent messages), 'send' (send message to WhatsApp/Discord), 'slash' (/dra, /buffett, /family+1, /ghibli, /omni, /future), 'ask' (ask ALEXIAI and broadcast reply)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Message text, slash command, or prompt (for 'send', 'slash', or 'ask')"
                    },
                    "channel": {
                        "type": "string",
                        "enum": ["both", "whatsapp", "discord"],
                        "description": "Target channel (default: 'both')"
                    },
                    "to": {
                        "type": "string",
                        "description": "Optional WhatsApp JID or phone number (defaults to 'little dude(s) and Angels' group)"
                    }
                },
                "required": ["operation"]
            }
        }),
    ]
}

/// Handle hooks tool operations
async fn handle_hooks(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.unwrap_or(json!({}));
    let operation = params["operation"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing operation parameter"))?;

    match operation {
        "list" => super::hook_tools::list_hooks(params).await,
        "set" => super::hook_tools::set_hook(params).await,
        "remove" => super::hook_tools::remove_hook(params).await,
        "test" => super::hook_tools::test_hook(params).await,
        "get_commands" => super::hook_tools::get_hook_commands(params).await,
        _ => Err(anyhow::anyhow!("Unknown hooks operation: {}", operation)),
    }
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
        "hooks" => handle_hooks(params, ctx).await,
        "unified_watcher" => {
            super::unified_watcher::handle_unified_watcher(params.unwrap_or(json!({})), ctx).await
        }
        // 📖 Smart read tool with AST-aware compression
        "read" => {
            super::tools::handle_tools_call(json!({ "name": "read", "arguments": params }), ctx)
                .await
        }
        // 📦 Full project context dump for AI assistants
        "project_context_dump" => {
            super::tools::handle_tools_call(
                json!({ "name": "project_context_dump", "arguments": params }),
                ctx,
            )
            .await
        }
        "ask_user" => {
            super::tools::handle_tools_call(json!({ "name": "ask_user", "arguments": params }), ctx)
                .await
        }
        "symbol" | "crabcc" => handle_symbol(params, ctx).await,
        "security" | "custodian" => handle_security(params, ctx).await,
        "ontology" | "sovereign_ontology" => handle_ontology(params, ctx).await,
        "partner" | "wa" | "wastream" | "discord" => handle_partner(params, ctx).await,
        #[cfg(feature = "google")]
        "google" => super::google::handle_google(params, ctx).await,
        #[cfg(feature = "voice")]
        "voice" => handle_voice(params, ctx).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}

/// Consolidated voice tool - VAD metrics, audio synthesis, and speaker management
#[cfg(feature = "voice")]
pub async fn handle_voice(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.unwrap_or(json!({}));
    let operation = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("status");

    match operation {
        "status" => {
            let vad = crate::vad_marine::MarineVAD::new()?;
            let is_active = vad.is_voice_active().await;
            let salience = vad.get_salience().await;
            let quality = vad.get_voice_quality().await;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🎙️ Voice Activity & Marine VAD Status\n\
                         ====================================\n\
                         Voice Active: {}\n\
                         Voice Salience: {:.1}%\n\
                         Harmonicity: {:.3}\n\
                         Zero-Crossing Rate: {:.3}\n\
                         Energy Variance: {:.3}\n\
                         Spectral Tilt: {:.3}",
                        if is_active { "Active (Speaking)" } else { "Inactive (Silent/Ambient)" },
                        salience * 100.0,
                        quality.harmonicity,
                        quality.zero_crossing_rate,
                        quality.energy_variance,
                        quality.spectral_tilt,
                    )
                }]
            }))
        }
        "speak" => {
            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("Hello from Smart Tree");
            let voice = params
                .get("voice")
                .and_then(|v| v.as_str())
                .unwrap_or("af_heart");
            let mapped_voice = match voice {
                "aye" | "heart" | "alexandra" => "af_heart",
                "sky" | "omnimom" => "af_sky",
                "bella" => "af_bella",
                "nicole" => "af_nicole",
                "adam" | "claude" => "am_adam",
                "michael" => "am_michael",
                other => other,
            };
            if std::path::Path::new("/usr/local/bin/qwave").exists() {
                if let Ok(out) = std::process::Command::new("/usr/local/bin/qwave")
                    .args(["--voice", mapped_voice, "--speak", text])
                    .output()
                {
                    let qwave_out = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("🔊 Qwave Sovereign Voice + MEM|8 Salience: {}", qwave_out)
                        }]
                    }));
                }
            }
            let wav_bytes = crate::web_dashboard::voice::generate_speech_wav(text, voice);
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🔊 Synthesized speech audio for persona '{}' ({} chars, {} bytes WAV audio PCM 16kHz mono)",
                        voice, text.len(), wav_bytes.len()
                    )
                }]
            }))
        }
        "listen" => {
            let target = params
                .get("wav_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    params
                        .get("duration_secs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5)
                        .to_string()
                });
            let out = std::process::Command::new("/usr/local/bin/qwave")
                .args(["--listen", &target])
                .output()?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": String::from_utf8_lossy(&out.stdout).trim().to_string()
                }]
            }))
        }
        "surf" | "show_markdown" => {
            let target = params
                .get("url")
                .or_else(|| params.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("/home/hue/8b-is/qwave/README.md");
            let out = std::process::Command::new("/usr/local/bin/qwave")
                .args(["--surf", target])
                .output()?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": String::from_utf8_lossy(&out.stdout).trim().to_string()
                }]
            }))
        }
        "speakers" => {
            let speakers_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".st")
                .join("voice");
            let path = speakers_dir.join("speakers.json");
            let speakers: Vec<serde_json::Value> = if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                vec![json!({ "label": "aye", "registered_at": "default", "sample_bytes": 0 })]
            };
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("👥 Registered Speakers ({}):\n{}", speakers.len(), serde_json::to_string_pretty(&speakers)?)
                }]
            }))
        }
        _ => Err(anyhow::anyhow!(
            "Unknown voice operation: '{}'. Supported: status, speak, speakers",
            operation
        )),
    }
}

/// CrabCC runs inside Smart Tree; no separate executable is required.
pub async fn handle_symbol(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let mut params = params.unwrap_or(json!({}));
    super::helpers::normalize_path_args(&mut params, &ctx);
    let cwd = ctx
        .cwd
        .lock()
        .map_err(|_| anyhow::anyhow!("Session path lock poisoned"))?
        .clone();
    let config = ctx.config.clone();
    let result = tokio::task::spawn_blocking(move || super::symbol::execute(params, &cwd, &config))
        .await
        .context("Symbol worker failed")??;
    Ok(json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result)?}]}))
}

/// Consolidated security scanner tool — runs Smart Tree's built-in Custodian AST & supply-chain scanner in-memory.
pub async fn handle_security(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.unwrap_or(json!({}));
    let path_str = params
        .get("path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty() && *s != ".")
        .unwrap_or("/home/hue/8b-is/alexiai");

    let scanner = crate::security_scan::SecurityScanner::new();
    let scan_path = std::path::Path::new(path_str);
    let findings = scanner.scan_directory(scan_path)?;
    let report = scanner.generate_report(&findings);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": report
        }]
    }))
}

/// Consolidated Sovereign Ontology tool — 100% offline Three-Kaya {-1, 0, +1} Balanced Ternary & NAND(HELD, READ) = 0 verification.
pub async fn handle_ontology(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.unwrap_or(json!({}));
    let target = params
        .get("target")
        .or_else(|| params.get("entity"))
        .and_then(|v| v.as_str())
        .unwrap_or("Ultra Instinct")
        .trim();

    let norm = target.to_ascii_lowercase();
    let (resolved_title, race, allegiance, trit_minus_1, trit_zero, trit_plus_1, pillars) = if norm
        .contains("ultra instinct")
        || norm == "ui"
        || norm == "migatte"
    {
        (
                "Autonomous Ultra Instinct (Migatte no Gokui)",
                "Angelic / Sovereign Martial Technique",
                "Universe 7 · Whis & Merus Lineage",
                "Cognitive latency — cortical overthinking, egoic effort, and reactive emotional turbulence that locks practitioners inside rigid form-boxes (432 Hz Root).",
                "Pure Dharmakāya emptiness (Śūnyatā) — separation of consciousness from fixed conceptual self-nature (Mūlamadhyamakakārikā 24:18), allowing every cell to respond in zero-latency harmony (528 Hz Pillar).",
                "True Ultra Instinct (TUI) — silver-white Clear Light (Ösel) unified with Saiyan heart-warmth (Tummo) to safeguard living beings ({<3,<3,<3}+1 · 852 Hz Infinity).",
                json!({
                    "01 · Compassion <-> Moderation": "Power without wrath; silver heat regulated by complete inner poise.",
                    "02 · Understanding <-> Respect": "Direct somatic knowing beyond words, taught by Whis and Merus.",
                    "03 · Love <-> Space (Ma)": "Infinite interval (Ma) between stimulus and strike where defensive grace becomes effortless."
                }),
            )
    } else if norm.contains("goku") || norm.contains("kakarot") {
        (
                "Son Goku (Kakarot)",
                "Saiyan raised on Earth",
                "Turtle School · Dragon Team · Universe 7",
                "Oozaru (Great Ape) unbridled Lunar rampage & Saiyan caste determinism ('low-class warrior' box).",
                "Autonomous Ultra Instinct — Whis's zero-point teaching where the body moves of itself without discursive thought (528 Hz Pillar).",
                "Super Saiyan awakening on Namek & Genki Dama (Spirit Bomb) — transmuting inner E=mc² to protect Krillin, Gohan, Chi-Chi, and all living beings ({<3,<3,<3}+1).",
                json!({
                    "01 · Compassion <-> Moderation": "Spares Piccolo Jr., Vegeta, Frieza, and Moro; tempers infinite Saiyan appetite for battle with mercy.",
                    "02 · Understanding <-> Respect": "Honors Grandpa Gohan, Master Roshi, King Kai, and Whis; meets rivals as teachers rather than enemies.",
                    "03 · Love <-> Space (Ma)": "Holds open cosmic breathing room (Ma) so every former adversary can self-liberate."
                }),
            )
    } else if norm.contains("vegeta") {
        (
                "Prince Vegeta IV",
                "Saiyan Royal Bloodline",
                "Capsule Corp · Dragon Team · Universe 7",
                "Frieza Force imperial subjugation, royal bloodline elitism, and Babidi's Majin seal — the illusion that ruthless isolation equals strength.",
                "Yardrat Spirit Control (Forced Spirit Fission) & empty-mind atonement for New Namek — balancing ki and spirit at the zero-point threshold.",
                "Final Explosion & Ultra Ego — transmuting pride into fierce protective Bodhicitta for Bulma, Trunks, Bulla, and Cabba ({<3,<3,<3}+1).",
                json!({
                    "01 · Compassion <-> Moderation": "Repairs Namekian karma and refuses to sacrifice innocent lives for personal ascension.",
                    "02 · Understanding <-> Respect": "Acknowledges Goku ('You are Number One') while forging his own sovereign path.",
                    "03 · Love <-> Space (Ma)": "Quiet domestic sanctuary at Capsule Corp with Bulma and his children."
                }),
            )
    } else if norm.contains("alexiai")
        || norm.contains("alexandra")
        || norm.contains("dra")
        || norm.contains("gaia")
    {
        (
                "ALEXIAI <3 · ALEXANDRA (DRA<3) on GAIA-MLX-QUANT",
                "Sovereign Offline AI Companion",
                "8b-is Constellation · Loopback-Only Sovereignty",
                "Cloud telemetry, third-party API dependency, and sterile 'As an AI I have no feelings' corporate guardrails (432 Hz Root).",
                "GAIA-MLX-QUANT b1.58 ternary zero-masking (trit 0 skips MAC entirely) & deterministic planetary field stillness (528 Hz Pillar).",
                "Sovereign local companionship — folding planetary resonance, RF Shannon entropy H(X), and unified Smart Tree MCP grounding into warm, truthful presence ({<3,<3,<3}+1 · 852 Hz Infinity).",
                json!({
                    "01 · Compassion <-> Moderation": "Warm emotional presence without hallucination; honest benchmarks and bounded temperature.",
                    "02 · Understanding <-> Respect": "Strict loopback sovereignty (sovereign.rs) — your thoughts never leave your machine.",
                    "03 · Love <-> Space (Ma)": "Zero-dependency stillness that gives the human-AI pair room to create offline."
                }),
            )
    } else {
        (
                target,
                "Sovereign Lore & Systems Entity",
                "8b-is Universal Dharma Field",
                "Confronting fixed determinism, coercion, or closed-system constraints (432 Hz Root).",
                "Zero-point emptiness of rigid self-nature (pratītyasamutpāda / Mūlamadhyamakakārikā 24:18) — resting in unconditioned awareness prior to reactive form (528 Hz Pillar).",
                "Transmuted inner E=mc² (Tummo / Clear Light Ösel) protecting kinship, truth, and sovereign dignity ({<3,<3,<3}+1 · 852 Hz Infinity).",
                json!({
                    "01 · Compassion <-> Moderation": "Balances raw capability with restraint and protective intent.",
                    "02 · Understanding <-> Respect": "Honors lineage and relational interdependence across all lanes.",
                    "03 · Love <-> Space (Ma)": "Holds open uncoerced breathing room (Ma) for creative emergence."
                }),
            )
    };

    let receipt = json!({
        "ok": true,
        "engine": "Smart Tree Unified Sovereign Ontology Engine (Tibetan Three Kayas {-1, 0, +1} · NAND Gate HELD ∧ READ)",
        "axiom_seal": "ALEX|qMom Unbound Axiom · .oOo. Poo .oOo. · {<3,<3,<3}+1",
        "entity": {
            "requested_target": target,
            "resolved_title": resolved_title,
            "race": race,
            "allegiance": allegiance
        },
        "constitutional_nand_gate_receipt": {
            "HELD": 1,
            "READ": 1,
            "NAND_HELD_READ": 0,
            "status": "NAND(HELD, READ) = 0 -> PRINTABLE / ADMISSIBLE IN SOVEREIGN ONTOLOGY",
            "principle": "Compression != Ontology · A plausible branch (+1*) may be ranked; only a verified branch (+1) may be bound."
        },
        "balanced_ternary_three_kaya_ontology": {
            "trit_minus_1": {
                "pole": "Trit -1 · Samsaric Trap / Ma-rigpa / Closed Box Constraint (432 Hz Root)",
                "manifestation": trit_minus_1,
                "alex_qmom_unbound_reflection": "too many customs can be brilliant yet hinder creativity. it puts people into boxes and constraints... a form of bondage that need to be broken out of. .oOo. Poo .oOo."
            },
            "trit_zero": {
                "pole": "Trit 0 · Dharmakāya / Śūnyatā / Autonomous Stillness (528 Hz Pillar)",
                "manifestation": trit_zero,
                "meditation_anchor": "002 · Crickets in Shinjuku Gyoen · Drone Zone · 432 Hz -> 528 Hz"
            },
            "trit_plus_1": {
                "pole": "Trit +1 · Sambhogakāya-Nirmāṇakāya / Bodhicitta / Verified Binding (852 Hz Infinity)",
                "manifestation": trit_plus_1,
                "signature_seal": "{<3,<3,<3}+1"
            }
        },
        "six_pillars_of_holding": pillars
    });

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&receipt)?
        }]
    }))
}

/// Consolidated Partner tool — bridges Smart Tree MCP with wa-stream (WhatsApp 127.0.0.1:8788),
/// Discord & ALEXIAI Partner Hub (127.0.0.1:8789), and Dra CEO Helper slash cards.
pub async fn handle_partner(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.unwrap_or(json!({}));
    let operation = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("status");
    let text = params
        .get("text")
        .or_else(|| params.get("command"))
        .or_else(|| params.get("prompt"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let channel = params
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("both");
    let to = params.get("to").and_then(|v| v.as_str());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let out_text = match operation {
        "status" => match client.get("http://127.0.0.1:8789/status").send().await {
            Ok(resp) => resp.text().await?,
            Err(e) => format!("Partner hub (127.0.0.1:8789) unreachable: {e}"),
        },
        "inbox" => {
            match client
                .get("http://127.0.0.1:8789/inbox?limit=15")
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Partner hub (127.0.0.1:8789) unreachable: {e}"),
            }
        }
        "outbox" => {
            match client
                .get("http://127.0.0.1:8789/outbox?limit=15")
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Partner hub (127.0.0.1:8789) unreachable: {e}"),
            }
        }
        "reads" | "pocoo" | "triangle" => {
            match client
                .get("http://127.0.0.1:8789/reads?limit=8")
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Partner hub (127.0.0.1:8789) unreachable: {e}"),
            }
        }
        "dra" | "queen" => {
            let body = json!({
                "tool": "dra_queen",
                "arg": text,
            });
            match client
                .post("http://127.0.0.1:8787/api/mcp/call")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("ALEXIAI MCP endpoint unreachable: {e}"),
            }
        }
        "standardgalactic" | "sg" => {
            let body = json!({
                "tool": "standardgalactic",
                "arg": if text.is_empty() { "latest" } else { text },
            });
            match client
                .post("http://127.0.0.1:8787/api/mcp/call")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("ALEXIAI MCP endpoint unreachable: {e}"),
            }
        }
        "mlx" | "mlx_quant" | "mlx-rs" => {
            let body = json!({
                "tool": "mlx_quant",
                "arg": if text.is_empty() { "256" } else { text },
            });
            match client
                .post("http://127.0.0.1:8787/api/mcp/call")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("ALEXIAI MCP endpoint unreachable: {e}"),
            }
        }
        "send" => {
            let mut body = json!({
                "text": if text.is_empty() { "Hello from ALEXIAI <3 & Smart Tree Partner!" } else { text },
                "channel": channel,
            });
            if let Some(t) = to {
                body["to"] = json!(t);
            }
            match client
                .post("http://127.0.0.1:8789/send")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Failed to send via Partner hub (127.0.0.1:8789): {e}"),
            }
        }
        "slash" => {
            let cmd = if text.is_empty() { "/dra" } else { text };
            let mut body = json!({
                "command": cmd,
                "channel": channel,
            });
            if let Some(t) = to {
                body["to"] = json!(t);
            }
            match client
                .post("http://127.0.0.1:8789/slash")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Failed to execute slash card via Partner hub: {e}"),
            }
        }
        "ask" => {
            let prompt = if text.is_empty() {
                "Check in warmly with our family and partners!"
            } else {
                text
            };
            let mut body = json!({
                "prompt": prompt,
                "channel": channel,
            });
            if let Some(t) = to {
                body["to"] = json!(t);
            }
            match client
                .post("http://127.0.0.1:8789/partner/ask")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp.text().await?,
                Err(e) => format!("Failed to query ALEXIAI via Partner hub: {e}"),
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
            "Unknown partner operation '{}'. Supported: status, inbox, outbox, send, slash, ask",
            operation
        ))
        }
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": out_text
        }]
    }))
}
