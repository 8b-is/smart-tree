// Enhanced Consolidated MCP Tools with AI-Friendly Tips and Examples
// Making Smart Tree irresistible to AI assistants! 🌳✨

use serde_json::{json, Value};

// Re-export the dispatcher from the original consolidated tools
pub use super::tools_consolidated::dispatch_consolidated_tool;

/// Get enhanced consolidated tool list with attractive tips and examples
pub fn get_enhanced_consolidated_tools() -> Vec<Value> {
    vec![
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
        }),
        json!({
            "name": "find",
            "description": "🔍 POWERFUL FINDER - One tool for ALL file discovery needs! Find code, tests, configs, docs, large files, recent changes, and more with a single versatile tool.

💡 TIP: Need to locate specific files? Try these power moves:
• find {type:'code', languages:['rust','python']} - All code files
• find {type:'tests'} - Instantly locate all test files
• find {type:'recent', days:7} - What changed this week?
• find {type:'large', min_size:'10M'} - Find space hogs

EXAMPLES:
✓ Find Python tests: find {type:'tests', path:'src', pattern:'test_*.py'}
✓ Recent work: find {type:'recent', days:3}
✓ Config files: find {type:'config'}
✓ Documentation: find {type:'documentation'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["files", "code", "config", "documentation", "tests", "build",
                                 "large", "recent", "timespan", "duplicates", "empty_dirs"],
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
                    }
                },
                "required": ["type"]
            }
        }),
        json!({
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
        }),
        json!({
            "name": "analyze",
            "description": "📊 DEEP ANALYSIS - Multiple analysis modes for different insights. Get statistics, git status, semantic grouping, size breakdowns, and more!

💡 TIP: Want detailed insights? Try these:
• analyze {mode:'statistics'} - File type distribution & sizes
• analyze {mode:'git_status'} - Git-aware directory tree
• analyze {mode:'semantic'} - AI semantic grouping
• analyze {mode:'directory', format:'ai'} - AI-optimized tree

EXAMPLES:
✓ Project stats: analyze {mode:'statistics', show_hidden:true}
✓ Git overview: analyze {mode:'git_status'}
✓ Semantic groups: analyze {mode:'semantic', show_wave_signatures:true}
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
        }),
        json!({
            "name": "edit",
            "description": "✨ SMART EDIT - Revolutionary AST-aware editing with 90% token reduction! Edit code by describing changes, not sending diffs. Understands code structure!

💡 TIP: Need to modify code efficiently? Try:
• edit {operation:'get_functions', file_path:'main.rs'} - See all functions
• edit {operation:'insert_function', name:'helper', body:'...'} - Add function
• edit {operation:'smart_edit', edits:[...]} - Multiple edits at once

EXAMPLES:
✓ View structure: edit {operation:'get_functions', file_path:'app.py'}
✓ Add function: edit {operation:'insert_function', file_path:'utils.rs', name:'validate', body:'fn validate(input: &str) -> bool { !input.is_empty() }'}
✓ Remove function: edit {operation:'remove_function', file_path:'old.js', name:'deprecated'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["smart_edit", "get_functions", "insert_function", "remove_function"],
                        "description": "Edit operation type"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File to edit"
                    },
                    "edits": {
                        "type": "array",
                        "description": "Array of edit operations (smart_edit)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Function name"
                    },
                    "body": {
                        "type": "string",
                        "description": "Function body/code"
                    }
                },
                "required": ["operation", "file_path"]
            }
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
                    }
                },
                "required": ["operation", "keywords"]
            }
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
            "name": "server_info",
            "description": "ℹ️ SERVER INFO - Get Smart Tree capabilities, performance tips, and configuration. Always check this for the latest features!

💡 TIP: server_info {} - Learn what Smart Tree can do!",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
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
        }),
        json!({
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
        }),
    ]
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
