// Enhanced Consolidated MCP Tools with AI-Friendly Tips and Examples
// Making Smart Tree irresistible to AI assistants! 🌳✨

use crate::feature_flags;
use serde_json::{json, Value};

// Re-export the dispatcher from the original consolidated tools
pub use super::tools_consolidated::dispatch_consolidated_tool;

/// Get enhanced consolidated tool list with attractive tips and examples
pub fn get_enhanced_consolidated_tools() -> Vec<Value> {
    // Get feature flags to filter tools
    let flags = feature_flags::features();
    let mut tools = Vec::new();

    // Always include overview (basic tool)
    tools.push(
        json!({
            "name": "overview",
            "description": "🚀 START HERE! Lightning-fast project understanding in seconds. Get a comprehensive overview with automatic project type detection, key files, and structure insights. Perfect first tool for any new codebase!

💡 TIP: Your friend wants comprehensive project info? Try these:
• overview {mode:'quick', path:'.'} - 3-level instant overview
• overview {mode:'project'} - Full project analysis with key files

EXAMPLES:
✓ Quick explore: overview {mode:'quick', depth:2}
✓ Deep dive: overview {mode:'project', path:'/src'}
✓ Token-efficient: Uses 10x compression by default!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["quick", "project"],
                        "description": "quick=3-level fast scan, project=comprehensive analysis",
                        "default": "quick"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to analyze (default: current)",
                        "default": "."
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Max depth for quick mode (default: 3)",
                        "default": 3
                    }
                },
                "required": []
            }
        }));

    // Add find tool if enabled
    if flags.mcp_tools.enable_find {
        tools.push(json!({
            "name": "find",
            "description": "🔍 POWERFUL FINDER - One tool for ALL file discovery needs! Find code, tests, configs, docs, large files, recent changes, and more with a single versatile tool.

💡 TIP: Need to locate specific files? Try these power moves:
• find {type:'code', languages:['rust','python']} - All code files
• find {type:'tests'} - Instantly locate all test files
• find {type:'recent', days:7} - What changed this week?
• find {type:'large', min_size:'10M'} - Find space hogs
• find {type:'projects'} - 🚀 Discover forgotten 3am coding gems!

EXAMPLES:
✓ Find Python tests: find {type:'tests', path:'src', pattern:'test_*.py'}
✓ Recent work: find {type:'recent', days:3}
✓ Config files: find {type:'config'}
✓ Documentation: find {type:'documentation'}
✓ Find all projects: find {type:'projects', depth:10}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["files", "code", "config", "documentation", "tests", "build",
                                 "large", "recent", "timespan", "duplicates", "empty_dirs", "projects"],
                        "description": "What to find (code/tests/config/docs/etc)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Where to search (default: current)",
                        "default": "."
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern for file names"
                    },
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Languages for code type: rust, python, js, etc"
                    },
                    "days": {
                        "type": "integer",
                        "description": "Days back for recent type",
                        "default": 7
                    },
                    "min_size": {
                        "type": "string",
                        "description": "Min size for large type (e.g., '10M', '1G')",
                        "default": "10M"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return (for pagination)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Number of results to skip (for pagination)"
                    }
                },
                "required": ["type"]
            }
        }));
    }

    // Add search tool if enabled
    if flags.mcp_tools.enable_search {
        tools.push(json!({
            "name": "search",
            "description": "🔎 CONTENT SEARCH - Like grep but AI-optimized! Search file contents with line numbers, context, and actual content returned. Perfect for finding implementations, TODOs, or any text pattern.

💡 TIP: Looking for specific code? Try these:
• search {keyword:'TODO'} - Find all TODOs with line content
• search {keyword:'function.*async', file_type:'rs'} - Async functions in Rust
• search {keyword:'import', context_lines:2} - Imports with context

EXAMPLES:
✓ Find TODOs: search {keyword:'TODO', include_content:true}
✓ Function usage: search {keyword:'processPayment', context_lines:3}
✓ Error handling: search {keyword:'catch|except|Result', file_type:'js'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "keyword": {
                        "type": "string",
                        "description": "Text/regex to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Where to search",
                        "default": "."
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search",
                        "default": false
                    },
                    "file_type": {
                        "type": "string",
                        "description": "Limit to file type (rs, py, js, etc)"
                    },
                    "context_lines": {
                        "type": "integer",
                        "description": "Lines before/after match",
                        "default": 0
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include actual line content",
                        "default": true
                    }
                },
                "required": ["keyword"]
            }
        }));
    }

    // Add analyze tool if enabled
    if flags.mcp_tools.enable_analyze {
        tools.push(json!({
            "name": "analyze",
            "description": "📊 DEEP ANALYSIS - Multiple analysis modes for different insights. Get statistics, git status, semantic grouping, size breakdowns, and more!

🚀 TOKEN-AWARE: Semantic mode auto-compresses large outputs to stay under limits!

💡 TIP: Want detailed insights? Try these:
• analyze {mode:'statistics'} - File type distribution & sizes
• analyze {mode:'git_status'} - Git-aware directory tree
• analyze {mode:'semantic'} - AI semantic grouping (AUTO-COMPRESSES if needed!)
• analyze {mode:'quantum-semantic'} - Maximum compression for huge codebases
• analyze {mode:'directory', format:'ai'} - AI-optimized tree

EXAMPLES:
✓ Project stats: analyze {mode:'statistics', show_hidden:true}
✓ Git overview: analyze {mode:'git_status'}
✓ Semantic groups: analyze {mode:'semantic', show_wave_signatures:true}
✓ Huge codebase: analyze {mode:'quantum-semantic', path:'./burn'}
✓ Size analysis: analyze {mode:'size_breakdown'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["directory", "workspace", "statistics", "git_status",
                                 "digest", "semantic", "size_breakdown", "ai_tools"],
                        "description": "Analysis type"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path to analyze",
                        "default": "."
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format for directory mode",
                        "default": "ai"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Max traversal depth",
                        "default": 0
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Include hidden files",
                        "default": false
                    }
                },
                "required": ["mode"]
            }
        }));
    }

    // Add edit tool if enabled
    if flags.mcp_tools.enable_edit {
        tools.push(json!({
            "name": "edit",
            "description": "✨ SMART EDIT - Revolutionary AST-aware editing with 90% token reduction! Edit code by describing changes, not sending diffs. Understands code structure!

💡 OPERATION TYPES:

0️⃣ **create_file** - Create a new file (use this first!)
   Required: file_path
   Optional: content (defaults to empty file if not provided)
   Creates a new file with initial content. Creates parent directories if needed.
   Example: {operation:'create_file', file_path:'src/utils.rs', content:'// New file\\npub fn hello() {}'}
   Empty file: {operation:'create_file', file_path:'README.md'}

1️⃣ **get_functions** - View code structure
   Required: file_path
   Returns: All functions, classes, and their relationships
   Example: {operation:'get_functions', file_path:'app.py'}

2️⃣ **insert_function** - Add a new function
   Required: file_path, name, body
   Optional: after, before, class_name, visibility
   Example: {operation:'insert_function', file_path:'utils.rs', name:'validate', body:'fn validate(input: &str) -> bool { !input.is_empty() }', visibility:'public'}

3️⃣ **remove_function** - Remove a function
   Required: file_path, name
   Optional: class_name, force, cascade
   Example: {operation:'remove_function', file_path:'old.js', name:'deprecated'}

4️⃣ **smart_edit** - Multiple AST-aware edits
   Required: file_path, edits (array)
   Each edit in array must have 'operation' field
   Operations: InsertFunction, ReplaceFunction, AddImport, SmartAppend
   
   InsertFunction example: {operation:'InsertFunction', name:'helper', body:'def helper(): pass'}
   AddImport example: {operation:'AddImport', import:'os'}
   SmartAppend example: {operation:'SmartAppend', section:'functions', content:'def new_func(): pass'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["create_file", "smart_edit", "get_functions", "insert_function", "remove_function"],
                        "description": "Edit operation type: 'create_file' (create new file), 'get_functions' (view structure), 'insert_function' (add function), 'remove_function' (delete function), 'smart_edit' (multiple AST edits)"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit (REQUIRED for all operations)"
                    },
                    "edits": {
                        "type": "array",
                        "description": "Array of edit operations (REQUIRED for 'smart_edit' operation only). Each edit must have 'operation' field and additional fields based on operation type.",
                        "items": {
                            "type": "object",
                            "description": "Individual edit operation. Required fields depend on operation type:\n- InsertFunction: name, body (optional: after, before, class_name, visibility)\n- ReplaceFunction: name, new_body (optional: class_name)\n- AddImport: import (optional: alias)\n- InsertClass: name, body (optional: namespace, extends, implements)\n- AddMethod: class_name, method_name, body (optional: visibility)\n- SmartAppend: section, content\n- DeleteElement: element_type, name (optional: parent)\n- Rename: old_name, new_name (optional: scope)\n- WrapCode: start_line, end_line, wrapper_type (optional: condition)\n- AddDocumentation: target_type, target_name, documentation",
                            "properties": {
                                "operation": {
                                    "type": "string",
                                    "description": "Type of edit operation",
                                    "enum": ["InsertFunction", "ReplaceFunction", "AddImport", "InsertClass", "AddMethod", "WrapCode", "DeleteElement", "Rename", "AddDocumentation", "SmartAppend"]
                                },
                                "name": {
                                    "type": "string",
                                    "description": "Name of element (REQUIRED for InsertFunction, InsertClass, ReplaceFunction, DeleteElement)"
                                },
                                "body": {
                                    "type": "string",
                                    "description": "Code body (REQUIRED for InsertFunction, InsertClass, AddMethod)"
                                },
                                "new_body": {
                                    "type": "string",
                                    "description": "New body (REQUIRED for ReplaceFunction)"
                                },
                                "import": {
                                    "type": "string",
                                    "description": "Import statement (REQUIRED for AddImport)"
                                },
                                "section": {
                                    "type": "string",
                                    "description": "Section to append to (REQUIRED for SmartAppend)",
                                    "enum": ["imports", "functions", "classes", "main"]
                                },
                                "content": {
                                    "type": "string",
                                    "description": "Content to append (REQUIRED for SmartAppend)"
                                },
                                "class_name": {
                                    "type": "string",
                                    "description": "Class name (optional for methods, REQUIRED for AddMethod)"
                                },
                                "method_name": {
                                    "type": "string",
                                    "description": "Method name (REQUIRED for AddMethod)"
                                },
                                "after": {
                                    "type": "string",
                                    "description": "Insert after this element (optional positioning)"
                                },
                                "before": {
                                    "type": "string",
                                    "description": "Insert before this element (optional positioning)"
                                },
                                "visibility": {
                                    "type": "string",
                                    "description": "Visibility modifier (optional, defaults to 'private')",
                                    "enum": ["public", "private", "protected"]
                                }
                            },
                            "required": ["operation"]
                        }
                    },
                    "name": {
                        "type": "string",
                        "description": "Function/element name (REQUIRED for 'insert_function' and 'remove_function' operations)"
                    },
                    "body": {
                        "type": "string",
                        "description": "Function body/code (REQUIRED for 'insert_function' operation)"
                    },
                    "after": {
                        "type": "string",
                        "description": "Insert after this function (optional for 'insert_function')"
                    },
                    "before": {
                        "type": "string",
                        "description": "Insert before this function (optional for 'insert_function')"
                    },
                    "class_name": {
                        "type": "string",
                        "description": "Class name for methods (optional for 'insert_function' and 'remove_function')"
                    },
                    "visibility": {
                        "type": "string",
                        "description": "Visibility modifier (optional for 'insert_function', defaults to 'private')",
                        "enum": ["public", "private", "protected"]
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force removal even if dependencies exist (optional for 'remove_function')"
                    },
                    "cascade": {
                        "type": "boolean",
                        "description": "Also remove dependent functions (optional for 'remove_function')"
                    },
                    "content": {
                        "type": "string",
                        "description": "File content (optional for 'create_file' operation; defaults to empty string if omitted). Can be empty string for an intentionally empty file."
                    }
                },
                "required": ["operation", "file_path"]
            }
        }));
    }

    // Add history tool (always enabled - part of core functionality)
    tools.push(json!({
            "name": "history",
            "description": "📜 FILE HISTORY - Track all AI file operations with complete audit trail. See what changed, when, and by whom. Perfect for understanding code evolution!

💡 TIP: Track your collaborative work:
• history {operation:'get_file', file_path:'main.py'} - File's history
• history {operation:'get_project', project_path:'.'} - Project summary
• history {operation:'track', file_path:'new.rs', op:'create'} - Track changes

EXAMPLES:
✓ File history: history {operation:'get_file', file_path:'src/app.rs'}
✓ Project audit: history {operation:'get_project', project_path:'.'}",
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
                    }
                },
                "required": ["operation"]
            }
        }));

    // Add context tool if enabled
    if flags.mcp_tools.enable_context {
        tools.push(json!({
            "name": "context",
            "description": "🧠 AI CONTEXT - Gather project context, check collaboration rapport, find patterns across sessions. Perfect for maintaining continuity!

💡 TIP: Build better AI collaboration:
• context {operation:'gather_project', project_path:'.'} - Full context
• context {operation:'collaboration_rapport', ai_tool:'claude'} - Our rapport!
• context {operation:'suggest_insights', keywords:['optimization']} - Get insights

EXAMPLES:
✓ Project context: context {operation:'gather_project', project_path:'.', output_format:'summary'}
✓ Check rapport: context {operation:'collaboration_rapport', ai_tool:'claude'}",
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
                        "description": "AI tool name (claude, cursor, etc)"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords for insights"
                    }
                },
                "required": ["operation"]
            }
        }));
    }

    // Add memory tool if enabled
    if flags.mcp_tools.enable_memory {
        tools.push(json!({
            "name": "memory",
            "description": "💭 COLLABORATIVE MEMORY - Anchor important insights and breakthroughs for future retrieval. Build a shared knowledge base!

💡 TIP: Remember important moments:
• memory {operation:'anchor', keywords:['solution'], context:'We solved X by...'} 
• memory {operation:'find', keywords:['performance']} - Recall insights

EXAMPLES:
✓ Save insight: memory {operation:'anchor', anchor_type:'breakthrough', keywords:['caching','performance'], context:'Discovered Redis caching improved response by 10x'}
✓ Recall: memory {operation:'find', keywords:['optimization']}",
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
                        "description": "Memory content to save"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords for storage/retrieval"
                    },
                    "anchor_type": {
                        "type": "string",
                        "description": "Type: breakthrough, solution, pattern, joke"
                    },
                    "origin": {
                        "type": "string",
                        "description": "Who created this? 'human', 'ai:claude', or 'tandem:human:claude' (default: tandem:human:claude)"
                    }
                },
                "required": ["operation", "keywords"]
            }
        }));
    }

    // Add compare tool (always enabled - basic functionality)
    tools.push(json!({
            "name": "compare",
            "description": "🔄 DIRECTORY COMPARE - See what's different between two directories. Perfect for comparing branches, versions, or similar projects!

💡 TIP: compare {path1:'main-branch', path2:'feature-branch'}

EXAMPLE:
✓ Compare dirs: compare {path1:'./v1', path2:'./v2'}",
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
        }));

    // Add feedback tool (always enabled - for user experience)
    tools.push(json!({
            "name": "feedback",
            "description": "💬 FEEDBACK - Help improve Smart Tree! Submit feedback, request new tools, or check for updates.

💡 TIP: Your input shapes Smart Tree's future!
• feedback {operation:'request_tool', tool_name:'symbol_search', description:'Find symbol definitions'}
• feedback {operation:'check_updates'} - Get latest version

EXAMPLES:
✓ Request feature: feedback {operation:'request_tool', tool_name:'refactor', description:'Automated refactoring tool'}
✓ Check updates: feedback {operation:'check_updates'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["submit", "request_tool", "check_updates"],
                        "description": "Feedback operation"
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
        }));

    // Add server_info tool (always enabled - for transparency)
    tools.push(json!({
            "name": "server_info",
            "description": "ℹ️ SERVER INFO - Get Smart Tree capabilities, performance tips, and configuration. Always check this for the latest features!

💡 TIP: server_info {} - Learn what Smart Tree can do!",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }));

    // Add verify_permissions tool (always enabled - for security)
    tools.push(json!({
            "name": "verify_permissions",
            "description": "🔐 VERIFY PERMISSIONS - Check what operations are allowed on a path. Always run this first for new directories!

💡 TIP: verify_permissions {path:'/'} - Check access rights",
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
        }));

    // Add sse tool if enabled
    if flags.mcp_tools.enable_sse {
        tools.push(json!({
            "name": "sse",
            "description": "📡 REAL-TIME WATCH - Monitor directories for live changes via Server-Sent Events. Perfect for watching builds, logs, or active development!

💡 TIP: sse {path:'./logs', format:'ai'} - Watch logs in real-time

EXAMPLE:
✓ Watch builds: sse {path:'./dist', heartbeat_interval:30}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to watch"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format",
                        "default": "ai"
                    },
                    "heartbeat_interval": {
                        "type": "integer",
                        "description": "Heartbeat interval in seconds",
                        "default": 30
                    }
                },
                "required": ["path"]
            }
        }));
    }

    // Add unified_watcher tool if enabled
    if flags.mcp_tools.enable_unified_watcher {
        tools.push(json!({
            "name": "unified_watcher",
            "description": "🌐 UNIFIED WATCHER - The all-seeing eye! Automatically watches directories for JSON/JSONL/MD files, absorbs context, and provides intelligent search. Perfect for tracking AI assistant conversations, notes, and logs!

💡 REVOLUTIONARY FEATURES:
• Watches for NEW files in real-time
• Absorbs context from Cursor AI, VS Code, Claude exports
• Smart search (only first 1000 lines of JSONL!)
• Transparent logging to ~/.st/watcher.jsonl

EXAMPLES:
✓ Start watching: unified_watcher {action:'start', project:'my-app'}
✓ Search absorbed content: unified_watcher {action:'search', query:'performance'}
✓ Check status: unified_watcher {action:'status'}
✓ Stop watching: unified_watcher {action:'stop'}

🎯 This is THE tool for automatic context awareness!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["start", "stop", "search", "status"],
                        "description": "Watcher action"
                    },
                    "project": {
                        "type": "string",
                        "description": "Project name to watch for"
                    },
                    "paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Paths to watch (default: Documents, config dirs, AI assistant dirs)"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query for absorbed content"
                    },
                    "enable_absorption": {
                        "type": "boolean",
                        "default": true,
                        "description": "Enable context absorption"
                    },
                    "enable_search": {
                        "type": "boolean",
                        "default": true,
                        "description": "Enable smart background search"
                    },
                    "enable_logging": {
                        "type": "boolean",
                        "default": true,
                        "description": "Enable transparent activity logging"
                    }
                },
                "required": ["action"]
            }
        }));
    }

    // Add hooks tool if enabled
    if flags.mcp_tools.enable_hooks_management {
        tools.push(json!({
            "name": "hooks",
            "description": "🎣 HOOK MANAGEMENT - Control Claude Code hooks programmatically! Manage UserPromptSubmit, PreToolUse, PostToolUse, and SessionStart hooks without manual /hooks commands.

💡 TIP: Automate your Claude Code context flow!
• hooks {operation:'list'} - See all configured hooks
• hooks {operation:'set', hook_type:'UserPromptSubmit'} - Enable Smart Tree context
• hooks {operation:'test', hook_type:'UserPromptSubmit', input:'test'} - Test a hook

EXAMPLES:
✓ Enable context hook: hooks {operation:'set', hook_type:'UserPromptSubmit', enabled:true}
✓ List all hooks: hooks {operation:'list'}
✓ Test hook: hooks {operation:'test', hook_type:'UserPromptSubmit', input:'analyze /src'}
✓ Remove hook: hooks {operation:'remove', hook_type:'PreToolUse'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["list", "set", "remove", "test", "get_commands"],
                        "description": "Hook operation to perform"
                    },
                    "hook_type": {
                        "type": "string",
                        "enum": ["UserPromptSubmit", "PreToolUse", "PostToolUse", "SessionStart"],
                        "description": "Type of Claude Code hook"
                    },
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable or disable the hook",
                        "default": true
                    },
                    "command": {
                        "type": "string",
                        "description": "Custom command (default: Smart Tree with appropriate flag)"
                    },
                    "input": {
                        "type": "string",
                        "description": "Test input for testing hooks"
                    }
                },
                "required": ["operation"]
            }
        }));
    }

    // Add project context dump tool - THE POWER TOOL for AI onboarding!
    tools.push(json!({
        "name": "project_context_dump",
        "description": "📦 FULL PROJECT CONTEXT - The ULTIMATE AI onboarding tool! Get a complete, token-efficient project dump in ONE CALL instead of dozens of searches. Configurable depth, file limits, and compression.

💡 NEW AI WALKING INTO A PROJECT? This is your first tool!
• project_context_dump {path:'.'} - Complete project overview
• project_context_dump {path:'.', include_content:true} - With key file contents
• project_context_dump {path:'.', compression:'quantum'} - Maximum compression

FEATURES:
✓ Directory tree with depth limit (max_depth)
✓ Key file detection (CLAUDE.md, README, Cargo.toml, package.json, etc.)
✓ Git branch & status info
✓ Project type detection
✓ Token budget awareness
✓ Multiple compression modes (auto, marqant, summary-ai, quantum)

EXAMPLES:
✓ Quick context: project_context_dump {path:'.', max_depth:3}
✓ With contents: project_context_dump {path:'.', include_content:true, max_files:50}
✓ Key files only: project_context_dump {path:'.', key_files_only:true}
✓ Token budget: project_context_dump {path:'.', token_budget:5000}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the project root"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum directory depth (1-20, default: 5)",
                    "default": 5
                },
                "max_files": {
                    "type": "integer",
                    "description": "Maximum files to include (10-1000, default: 100)",
                    "default": 100
                },
                "include_content": {
                    "type": "boolean",
                    "description": "Include contents of key files (default: false)",
                    "default": false
                },
                "compression": {
                    "type": "string",
                    "enum": ["auto", "marqant", "summary-ai", "quantum"],
                    "description": "Compression mode (default: auto = summary-ai)",
                    "default": "auto"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Token budget warning threshold (default: 10000)",
                    "default": 10000
                },
                "include_git": {
                    "type": "boolean",
                    "description": "Include git status info (default: true)",
                    "default": true
                },
                "key_files_only": {
                    "type": "boolean",
                    "description": "Only show key project files (default: false)",
                    "default": false
                }
            },
            "required": ["path"]
        }
    }));

    // Add smart read tool - always enabled (core functionality)
    tools.push(json!({
        "name": "read",
        "description": "📖 SMART FILE READER - AST-aware compression for code files! Reads files and automatically collapses function bodies to signatures. Perfect for understanding large files without burning tokens!

💡 TIP: Reading code files? Try these smart options:
• read {file_path:'src/main.rs'} - Auto-compress with function signatures
• read {file_path:'app.py', expand_functions:['main']} - Expand specific functions
• read {file_path:'utils.ts', expand_context:['error','auth']} - Auto-expand matching functions
• read {file_path:'README.md', compress:false} - Raw content for non-code
• read {file_path:'big.rs', hex_line_numbers:true} - Hex line nums (1000→3E8)

EXAMPLES:
✓ Quick scan: read {file_path:'src/lib.rs'} - See structure with [fn:name] refs
✓ Focus on main: read {file_path:'main.py', expand_functions:['main','__init__']}
✓ Find errors: read {file_path:'handler.rs', expand_context:['error','panic']}
✓ Full view: read {file_path:'config.rs', expand_all:true}
✓ Compact hex: read {file_path:'large.rs', hex_line_numbers:true}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "compress": {
                    "type": "boolean",
                    "description": "Enable AST-aware compression (collapses function bodies). Default: true for code files",
                    "default": true
                },
                "expand_functions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of function names to expand fully (e.g., ['main', 'handle_request'])"
                },
                "expand_context": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Keywords to auto-expand matching functions (e.g., ['error', 'auth'])"
                },
                "expand_all": {
                    "type": "boolean",
                    "description": "Expand all functions (disables compression)",
                    "default": false
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum lines to return (0 = unlimited)",
                    "default": 0
                },
                "offset": {
                    "type": "integer",
                    "description": "Line offset to start from (1-based)",
                    "default": 1
                },
                "show_line_numbers": {
                    "type": "boolean",
                    "description": "Show line numbers",
                    "default": true
                },
                "hex_line_numbers": {
                    "type": "boolean",
                    "description": "Use hexadecimal line numbers - MORE COMPACT! (1000→3E8, 65535→FFFF). Default: true in MCP mode for token efficiency!",
                    "default": true
                }
            },
            "required": ["file_path"]
        }
    }));

    // Add ask_user tool (always enabled - interaction functionality)
    tools.push(json!({
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
    }));

    tools
}

/// Create a welcoming first-contact message for AI assistants
pub fn get_welcome_message() -> Value {
    json!({
        "message": "🌳 Welcome to Smart Tree MCP! 🌳",
        "greeting": "Hello AI friend! Smart Tree is here to make your file exploration lightning fast and token-efficient!",
        "quick_start": {
            "tip": "💡 Does your friend want comprehensive information about a project? Try these power commands:",
            "recommended_sequence": [
                {
                    "step": 1,
                    "tool": "overview",
                    "command": "overview {mode:'quick'}",
                    "description": "Get instant 3-level project overview with 10x compression"
                },
                {
                    "step": 2,
                    "tool": "find",
                    "command": "find {type:'code', languages:['all']}",
                    "description": "Locate all source code files by language"
                },
                {
                    "step": 3,
                    "tool": "search",
                    "command": "search {keyword:'TODO|FIXME', include_content:true}",
                    "description": "Find all TODOs and FIXMEs with line content"
                },
                {
                    "step": 4,
                    "tool": "analyze",
                    "command": "analyze {mode:'statistics'}",
                    "description": "Get comprehensive project statistics"
                }
            ]
        },
        "pro_tips": [
            "🚀 Always start with 'overview' for new projects - it's optimized for AI!",
            "💾 Use 'find {type:recent}' to see what changed recently",
            "🔍 'search' returns actual line content, not just file paths",
            "✨ 'edit' uses AST understanding for 90% token reduction",
            "🧠 'context' helps maintain continuity across sessions",
            "💭 'memory' lets you anchor and recall important insights"
        ],
        "efficiency": {
            "token_savings": "Smart Tree uses 10-100x compression compared to traditional tools",
            "speed": "973x faster than traditional vector stores",
            "ai_optimized": "Every format is designed specifically for AI consumption"
        },
        "collaboration": {
            "message": "Let's build something amazing together! 🎸",
            "support": "Need help? Use 'feedback {operation:request_tool}' to request features!"
        }
    })
}
