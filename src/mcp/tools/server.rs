//! Server info and permission tools
//!
//! Contains server_info and verify_permissions handlers.

use super::definitions::VerifyPermissionsArgs;
use crate::mcp::permissions::get_available_tools;
use crate::mcp::{is_path_allowed, McpContext};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// Get server information and capabilities
pub async fn server_info(_args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let cache_stats = ctx.cache.stats().await;

    // Get current date/time for AI assistants
    use chrono::{Local, Utc};
    let now_local = Local::now();
    let now_utc = Utc::now();

    // Add a rotating Omni quote for a touch of joy
    let omni_quotes = [
        "Waves remember what structure forgets.",
        "Compression is rhythm; meaning is melody.",
        "Directories are forests; walk softly and listen.",
        "Entropy is just unexplained context.",
    ];
    let omni_quote = {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        omni_quotes
            .choose(&mut rng)
            .copied()
            .unwrap_or(omni_quotes[0])
    };

    let info = json!({
        "server": {
            "name": "Smart Tree MCP Server",
            "version": env!("CARGO_PKG_VERSION"),
            "current_time": {
                "local": now_local.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
                "utc": now_utc.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "date_format_hint": "Use YYYY-MM-DD format for date filters (e.g., 2025-07-11)"
            },
            "build": {
                "name": env!("CARGO_PKG_NAME"),
                "description": env!("CARGO_PKG_DESCRIPTION"),
                "authors": env!("CARGO_PKG_AUTHORS"),
                "repository": env!("CARGO_PKG_REPOSITORY"),
                "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            },
            "protocol": {
                "version": "1.0",
                "features": ["tools", "resources", "prompts", "notifications"],
            },
        },
        "omni": {
            "quote": omni_quote,
        },
        "why_pick_smart_tree_over_other_tools": {
            "overview_vs_ls_glob_read": "973x faster & 90-95% fewer tokens; 1 call replaces 20+ Read/Glob/ls operations and includes git status + .m8 consciousness.",
            "find_vs_glob": "Semantic file discovery (`tests`, `code`, `config`, `docs`, `recent`, `large`, `projects`) replaces 50+ glob patterns in <50ms.",
            "search_vs_grep": "Parallel Rust regex engine with in-memory caching (<1ms repeat queries) returning line numbers, content, and context in one call.",
            "read_vs_raw_cat": "AST-aware smart reader collapses function bodies to [fn:name] signatures (80-90% token reduction) with selective function expansion.",
            "edit_vs_text_diff": "AST-aware structural editing (`InsertFunction`, `ReplaceFunction`, `AddImport`, atomic batch_mode) saves 90% output tokens.",
            "symbol_vs_blind_grep": "Unified CrabCC + Smart Tree AST symbol definition lookup (`sym`), call-graph references (`refs`), and file outlines (`outline`).",
            "memory_vs_stateless": "Persistent MEM8 wave memory (`~/.st/memories.m8`), bottom-up `.m8` directory consciousness pyramids, and `~/.ssh/config` fleet host memory.",
            "g8t_vs_unverified_code": "Hardware-benchmarked, cycle-verified (`_rdtsc`/`cntvct_el0` exact rational `p/q` `cycles_per_op`) fastest Rust functions from g8t.is."
        },
        "capabilities": {
            "output_formats": [
                "classic", "hex", "json", "ai", "stats", "csv", "tsv", "digest",
                "quantum", "semantic", "quantum-semantic", "summary", "summary-ai"
            ],
            "compression": {
                "supported": true,
                "formats": ["zlib", "quantum", "base64"],
            },
            "streaming": {
                "supported": true,
                "formats": ["hex", "ai", "quantum", "quantum-semantic"],
            },
            "search": {
                "content_search": true,
                "pattern_matching": true,
                "regex_support": true,
            },
        },
        "configuration": {
            "cache": {
                "enabled": ctx.config.cache_enabled,
                "ttl_seconds": ctx.config.cache_ttl,
                "max_size_bytes": ctx.config.max_cache_size,
                "current_entries": cache_stats.entries,
                "current_size_bytes": cache_stats.size,
                "hit_rate": format!("{:.1}%", cache_stats.hit_rate * 100.0),
            },
            "security": {
                "allowed_paths": ctx.config.allowed_paths.len(),
                "blocked_paths": ctx.config.blocked_paths.len(),
                "default_blocks": ["/etc", "/sys", "/proc"],
            },
            "mcp_optimization": {
                "compression_enabled": !std::env::var("MCP_NO_COMPRESS")
                    .is_ok_and(|v| v == "1" || v.to_lowercase() == "true"),
                "emoji_disabled": true,
                "auto_ai_modes": true,
                "env_vars": {
                    "MCP_NO_COMPRESS": "Set to '1' or 'true' to disable compression for AIs that can't handle it",
                },
            },
        },
        "features": {
            "quantum_compression": {
                "description": "Ultra-compressed binary format with 90%+ compression",
                "status": "active",
                "notes": "Base64-encoded for JSON transport in MCP",
            },
            "mcp_optimization": {
                "description": "Automatic API optimization for any output mode",
                "status": "active",
                "features": ["compression (disable with MCP_NO_COMPRESS=1)", "no emoji", "AI mode selection"],
                "recommended_for": ["MCP servers", "LLM APIs", "AI assistants"],
            },
            "tokenization": {
                "description": "Semantic tokenization for common patterns",
                "tokens": {
                    "directories": ["node_modules=0x80", ".git=0x81", "src=0x82"],
                    "extensions": [".js=0x90", ".rs=0x91", ".py=0x92"],
                },
            },
        },
        "recommended_workflow": {
            "step_1": {
                "tool": "quick_tree",
                "why": "Always start here! Gets you a 3-level overview with 10x compression. Perfect for understanding the basic structure.",
                "example": "quick_tree(path='.')",
            },
            "step_2": {
                "tool": "project_overview or analyze_workspace",
                "why": "For deeper understanding of project type, dependencies, and structure. Use project_overview for single projects, analyze_workspace for complex/multi-language codebases.",
                "example": "project_overview(path='.')",
            },
            "step_3_options": {
                "for_specific_files": {
                    "tools": ["find_code_files", "find_config_files", "find_documentation", "find_tests"],
                    "why": "Use these targeted searches to locate specific file types quickly",
                },
                "for_code_analysis": {
                    "tool": "analyze_directory with mode='quantum-semantic'",
                    "why": "Best mode for understanding code structure with semantic compression and tokenization",
                },
                "for_search": {
                    "tool": "search_in_files",
                    "why": "Find specific functions, TODOs, or any text pattern across the codebase",
                },
                "for_statistics": {
                    "tool": "get_statistics",
                    "why": "Understand file distribution, sizes, and project composition",
                },
            },
            "pro_tips": [
                "Always use quick_tree first - it's optimized for initial exploration",
                "For large codebases, use mode='summary-ai' for 10x compression",
                "quantum-semantic mode is AMAZING for code understanding - try it!",
                "Cache is enabled by default - repeated calls are instant",
                "Use search_in_files to find specific implementations quickly",
                "semantic_analysis groups files by purpose - great for large projects",
            ],
        },
        "statistics": {
            "uptime_seconds": 0,
            "requests_handled": 0,
            "cache_hits": cache_stats.hits,
            "cache_misses": cache_stats.misses,
        },
        "tips": [
            "🌟 ALWAYS start with 'quick_tree' for any new directory!",
            "Use 'summary-ai' format for optimal LLM API transmission (10x compression!)",
            "Enable compression with compress=true for large directories",
            "Use 'quantum-semantic' mode for the best code analysis experience",
            "Stream mode available for very large directories",
            "Content search supported with 'search_in_files' tool",
            "The cache makes repeated queries instant - don't worry about calling tools multiple times!",
        ],
    });

    // Convert to pretty JSON string
    let json_string = serde_json::to_string_pretty(&info)?;

    // Return in MCP content format
    Ok(json!({
        "content": [{
            "type": "text",
            "text": json_string
        }]
    }))
}

/// Verify permissions for a path before using other tools
pub async fn verify_permissions(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: VerifyPermissionsArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.path);

    // Basic security check
    if !is_path_allowed(&path, &ctx.config) {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("🚫 Access Denied: Path '{}' is not in allowed paths list.\n\nAllowed paths:\n{}",
                    path.display(),
                    ctx.config.allowed_paths.iter()
                        .map(|p| format!("  - {}", p.display()))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }]
        }));
    }

    // Get permission cache
    let mut perm_cache = ctx.permissions.lock().await;

    // Verify permissions
    let perms = perm_cache.verify(&path)?;

    // Get available tools based on permissions
    let tools = get_available_tools(&perms);

    // Format output
    let mut output = format!("🔐 Permission Check for: {}\n\n", path.display());

    // Show permission status
    output.push_str("📊 Permission Status:\n");
    output.push_str(&format!(
        "  • Exists: {}\n",
        if perms.exists { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Readable: {}\n",
        if perms.readable { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Writable: {}\n",
        if perms.writable { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Type: {}\n",
        if perms.is_directory {
            "📁 Directory"
        } else if perms.is_file {
            "📄 File"
        } else {
            "❓ Unknown"
        }
    ));

    if let Some(error) = &perms.error {
        output.push_str(&format!("  • Error: {}\n", error));
    }

    output.push_str("\n🛠️ Available Tools:\n");

    // Group tools by availability
    let mut available = vec![];
    let mut unavailable = vec![];

    for tool in &tools {
        if tool.available {
            available.push(tool);
        } else {
            unavailable.push(tool);
        }
    }

    // Show available tools
    if !available.is_empty() {
        output.push_str("\n✅ Ready to Use:\n");
        for tool in available {
            output.push_str(&format!("  • {} - Call with this path\n", tool.name));
        }
    }

    // Show unavailable tools with reasons
    if !unavailable.is_empty() {
        output.push_str("\n❌ Not Available (with reasons):\n");
        for tool in unavailable {
            output.push_str(&format!(
                "  • {} - {}\n",
                tool.name,
                tool.reason
                    .as_ref()
                    .unwrap_or(&"Unknown reason".to_string())
            ));
        }
    }

    // Add helpful tips
    output.push_str("\n💡 Tips:\n");
    if !perms.exists {
        output
            .push_str("  • The path doesn't exist. Check your spelling or use a different path.\n");
    } else if !perms.readable {
        output.push_str("  • No read permission. You may need to run with elevated privileges.\n");
    } else if !perms.writable && perms.is_file {
        output.push_str("  • File is read-only. You can analyze but not edit.\n");
    }

    // Trisha says...
    output.push('\n');
    output.push_str("Trisha from Accounting says: \"It's like checking if you have the keys ");
    output.push_str("before bringing the whole toolbox! Smart thinking!\" 🔑\n");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": {
            "permissions": perms,
            "available_tools": tools.iter().filter(|t| t.available).map(|t| &t.name).collect::<Vec<_>>(),
            "unavailable_tools": tools.iter().filter(|t| !t.available).map(|t| &t.name).collect::<Vec<_>>(),
        }
    }))
}
