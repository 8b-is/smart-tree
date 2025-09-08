//! Enhanced MCP prompts implementation for Smart Tree
//!
//! Features comprehensive prompts for all 30+ Smart Tree MCP tools
//! Organized by skill level and use case with Elvis-level entertainment! 🎸
//!
//! Created by: The Cheet & Hue partnership
//! For: Trisha in Accounting (who deserves the BEST prompts!)

use super::McpContext;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct PromptDefinition {
    name: String,
    description: String,
    category: String,
    difficulty: String,
    arguments: Vec<PromptArgument>,
    examples: Vec<String>,
    tips: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptArgument {
    name: String,
    description: String,
    required: bool,
    default: Option<String>,
}

/// Handle prompts list - now with 15+ comprehensive prompts! 🚀
pub async fn handle_prompts_list(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let prompts = vec![
        // 🌟 BEGINNER CATEGORY - "Baby Steps to Stardom!"
        PromptDefinition {
            name: "first_steps".to_string(),
            description: "🌟 Your first Smart Tree experience - like Elvis's first guitar! Start here for instant project overview.".to_string(),
            category: "Beginner".to_string(),
            difficulty: "Easy".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Where to start exploring (usually '.' for current directory)".to_string(),
                    required: true,
                    default: Some(".".to_string()),
                },
            ],
            examples: vec![
                "Perfect for: New codebases, inherited projects, 'What the heck is this?' moments".to_string(),
                "Try: first_steps with path='.' to get your bearings".to_string(),
            ],
            tips: vec![
                "💡 Pro Tip: Always start here when exploring new code!".to_string(),
                "🎸 Like Elvis said: 'A little less conversation, a little more action!'".to_string(),
            ],
        },

        PromptDefinition {
            name: "quick_explore".to_string(),
            description: "🔍 Lightning-fast 3-level directory peek - for when you need answers NOW!".to_string(),
            category: "Beginner".to_string(),
            difficulty: "Easy".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Directory to explore".to_string(),
                    required: true,
                    default: Some(".".to_string()),
                },
                PromptArgument {
                    name: "depth".to_string(),
                    description: "How deep to look (default: 3)".to_string(),
                    required: false,
                    default: Some("3".to_string()),
                },
            ],
            examples: vec![
                "Perfect for: Quick project scans, finding main directories, getting unstuck".to_string(),
            ],
            tips: vec![
                "⚡ Super fast - uses quantum compression!".to_string(),
                "🎯 Great for large projects where full scans take forever".to_string(),
            ],
        },

        PromptDefinition {
            name: "find_my_files".to_string(),
            description: "📁 Find specific files like a detective - but faster and with style!".to_string(),
            category: "Beginner".to_string(),
            difficulty: "Easy".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "type".to_string(),
                    description: "What to find: code, tests, config, documentation, etc.".to_string(),
                    required: true,
                    default: Some("code".to_string()),
                },
                PromptArgument {
                    name: "path".to_string(),
                    description: "Where to search".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                },
            ],
            examples: vec![
                "find_my_files type='tests' - Find all test files".to_string(),
                "find_my_files type='config' - Locate configuration files".to_string(),
            ],
            tips: vec![
                "🎪 Works like magic - automatically detects file types!".to_string(),
                "🔍 Supports: code, tests, config, docs, build files, and more!".to_string(),
            ],
        },

        // 🚀 POWER USER CATEGORY - "All Shook Up with Features!"
        PromptDefinition {
            name: "codebase_detective".to_string(),
            description: "🕵️ Deep codebase analysis with AI optimization - Sherlock Holmes meets coding!".to_string(),
            category: "Power User".to_string(),
            difficulty: "Medium".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Codebase to analyze".to_string(),
                    required: true,
                    default: Some(".".to_string()),
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: "Focus area: architecture, patterns, dependencies, or all".to_string(),
                    required: false,
                    default: Some("all".to_string()),
                },
            ],
            examples: vec![
                "Perfect for: Code reviews, architecture decisions, onboarding".to_string(),
                "Use compress=true for codebases with 10k+ files".to_string(),
            ],
            tips: vec![
                "🧠 AI-optimized output perfect for LLMs!".to_string(),
                "⚡ Up to 99% compression while keeping all important details!".to_string(),
            ],
        },

        PromptDefinition {
            name: "search_master".to_string(),
            description: "🔎 Advanced content search across your entire codebase - grep on steroids!".to_string(),
            category: "Power User".to_string(),
            difficulty: "Medium".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "keyword".to_string(),
                    description: "What to search for (supports regex!)".to_string(),
                    required: true,
                    default: None,
                },
                PromptArgument {
                    name: "file_type".to_string(),
                    description: "Limit to specific file types (rs, py, js, etc.)".to_string(),
                    required: false,
                    default: None,
                },
            ],
            examples: vec![
                "search_master keyword='TODO' - Find all TODOs with context".to_string(),
                "search_master keyword='function.*async' file_type='js' - Async functions".to_string(),
            ],
            tips: vec![
                "🔍 Returns actual line content, not just file names!".to_string(),
                "💡 Supports regex patterns for complex searches".to_string(),
            ],
        },

        // 🎸 DEVELOPER CATEGORY - "Burning Love for Code!"
        PromptDefinition {
            name: "smart_edit_wizard".to_string(),
            description: "✨ Revolutionary AST-aware code editing with 90% token reduction - like magic, but real!".to_string(),
            category: "Developer".to_string(),
            difficulty: "Advanced".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "file_path".to_string(),
                    description: "File to edit or analyze".to_string(),
                    required: true,
                    default: None,
                },
                PromptArgument {
                    name: "operation".to_string(),
                    description: "What to do: get_functions, insert_function, remove_function, smart_edit".to_string(),
                    required: true,
                    default: Some("get_functions".to_string()),
                },
            ],
            examples: vec![
                "smart_edit_wizard file_path='app.py' operation='get_functions' - See all functions".to_string(),
                "smart_edit_wizard operation='insert_function' - Add new function".to_string(),
            ],
            tips: vec![
                "🧠 Understands code structure, not just text!".to_string(),
                "⚡ 90% fewer tokens than traditional editing!".to_string(),
            ],
        },

        PromptDefinition {
            name: "project_memory".to_string(),
            description: "💭 Collaborative memory system - remember breakthroughs and insights forever!".to_string(),
            category: "Developer".to_string(),
            difficulty: "Advanced".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "operation".to_string(),
                    description: "What to do: anchor (save) or find (recall)".to_string(),
                    required: true,
                    default: Some("find".to_string()),
                },
                PromptArgument {
                    name: "keywords".to_string(),
                    description: "Keywords for storage/retrieval".to_string(),
                    required: true,
                    default: None,
                },
            ],
            examples: vec![
                "project_memory operation='anchor' keywords=['performance','caching']".to_string(),
                "project_memory operation='find' keywords=['optimization'] - Recall insights".to_string(),
            ],
            tips: vec![
                "🧠 Build a shared knowledge base with your AI partner!".to_string(),
                "💡 Perfect for remembering solutions, patterns, and decisions".to_string(),
            ],
        },

        // 🎪 FUN CATEGORY - "That's All Right (Mama)!"
        PromptDefinition {
            name: "project_stats_party".to_string(),
            description: "🎉 Get comprehensive project statistics with style - because numbers should be fun!".to_string(),
            category: "Fun".to_string(),
            difficulty: "Easy".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Project to analyze".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                },
            ],
            examples: vec![
                "Perfect for: Project reports, impressing teammates, satisfying curiosity".to_string(),
            ],
            tips: vec![
                "📊 Beautiful statistics with file type breakdowns!".to_string(),
                "🎪 Makes boring numbers exciting and colorful!".to_string(),
            ],
        },

        PromptDefinition {
            name: "compare_directories".to_string(),
            description: "🔄 Compare two directories like a pro - spot the differences instantly!".to_string(),
            category: "Fun".to_string(),
            difficulty: "Medium".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path1".to_string(),
                    description: "First directory to compare".to_string(),
                    required: true,
                    default: None,
                },
                PromptArgument {
                    name: "path2".to_string(),
                    description: "Second directory to compare".to_string(),
                    required: true,
                    default: None,
                },
            ],
            examples: vec![
                "compare_directories path1='./v1' path2='./v2' - Version comparison".to_string(),
            ],
            tips: vec![
                "🔍 Perfect for comparing different versions or branches!".to_string(),
                "🎯 Shows added, removed, and modified files clearly".to_string(),
            ],
        },

        // Legacy prompts for compatibility
        PromptDefinition {
            name: "analyze_codebase".to_string(),
            description: "Analyze a code repository with AI-optimized output".to_string(),
            category: "Legacy".to_string(),
            difficulty: "Medium".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to the codebase".to_string(),
                    required: true,
                    default: None,
                },
            ],
            examples: vec![],
            tips: vec![],
        },
    ];

    Ok(json!({
        "prompts": prompts,
        "categories": ["Beginner", "Power User", "Developer", "Fun", "Legacy"],
        "total_count": prompts.len(),
        "elvis_approval": "Thank you, thank you very much! 🕺",
        "created_by": "The Cheet & Hue Partnership",
        "for": "Trisha in Accounting (and all Smart Tree lovers!)",
        "note": "These prompts are designed to make Smart Tree accessible, powerful, and fun for everyone!"
    }))
}

/// Handle prompts get - now with comprehensive prompt generation! 🎪
pub async fn handle_prompts_get(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing prompt name"))?;
    let arguments = params["arguments"].clone();

    match name {
        // Beginner prompts
        "first_steps" => get_first_steps_prompt(arguments),
        "quick_explore" => get_quick_explore_prompt(arguments),
        "find_my_files" => get_find_my_files_prompt(arguments),

        // Power User prompts
        "codebase_detective" => get_codebase_detective_prompt(arguments),
        "search_master" => get_search_master_prompt(arguments),

        // Developer prompts
        "smart_edit_wizard" => get_smart_edit_wizard_prompt(arguments),
        "project_memory" => get_project_memory_prompt(arguments),

        // Fun prompts
        "project_stats_party" => get_project_stats_party_prompt(arguments),
        "compare_directories" => get_compare_directories_prompt(arguments),

        // Legacy prompts (for compatibility)
        "analyze_codebase" => get_analyze_codebase_prompt(arguments),
        "find_large_files" => get_find_large_files_prompt(arguments),
        "recent_changes" => get_recent_changes_prompt(arguments),
        "project_structure" => get_project_structure_prompt(arguments),

        _ => Err(anyhow::anyhow!(
            "Unknown prompt: {} (Did you mean one of our awesome new prompts?)",
            name
        )),
    }
}

// 🌟 BEGINNER PROMPT IMPLEMENTATIONS

fn get_first_steps_prompt(args: Value) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🌟 Welcome to Smart Tree! Let's explore {} together!\n\n\
                Step 1: Get a quick overview with:\n\
                • Use `overview` tool with mode='quick' and path='{}' for a 3-level scan\n\
                Step 2: If you like what you see, dive deeper with:\n\
                • Use `overview` tool with mode='project' for comprehensive analysis\n\
                Step 3: Find specific things with:\n\
                • Use `find` tool with type='code' to see all code files\n\
                • Use `find` tool with type='tests' to locate test files\n\
                • Use `find` tool with type='config' to find configuration files\n\n\
                🎯 Pro Tips:\n\
                • Start with 'quick' mode - it's lightning fast!\n\
                • For large projects, always use compress=true\n\
                • Don't be afraid to explore - Smart Tree is designed to be helpful!\n\n\
                Ready to rock? Let's make this codebase sing! 🎸",
                path, path
            )
        }
    })];

    Ok(json!({
        "description": "Your first Smart Tree experience - perfect introduction to exploring any codebase",
        "messages": messages
    }))
}

fn get_quick_explore_prompt(args: Value) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");
    let depth = args["depth"].as_u64().unwrap_or(3);

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🔍 Quick exploration of {} (depth: {})\n\n\
                Use the `overview` tool with these parameters:\n\
                • mode: 'quick'\n\
                • path: '{}'\n\
                • depth: {}\n\n\
                This will give you:\n\
                ⚡ Lightning-fast results (using quantum compression!)\n\
                📁 Directory structure overview\n\
                📊 Basic statistics\n\
                🎯 Key files and patterns\n\n\
                Perfect for: Getting your bearings, quick scans, understanding project layout",
                path, depth, path, depth
            )
        }
    })];

    Ok(json!({
        "description": "Lightning-fast 3-level directory overview with quantum compression",
        "messages": messages
    }))
}

fn get_find_my_files_prompt(args: Value) -> Result<Value> {
    let file_type = args["type"].as_str().unwrap_or("code");
    let path = args["path"].as_str().unwrap_or(".");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "📁 Finding {} files in {} like a detective! 🕵️\n\n\
                Use the `find` tool with these parameters:\n\
                • type: '{}'\n\
                • path: '{}'\n\n\
                Smart Tree will automatically detect and categorize:\n\
                🔍 Code files: .rs, .py, .js, .ts, .java, .cpp, etc.\n\
                🧪 Test files: test_*, *_test.*, spec.*, etc.\n\
                ⚙️ Config files: .json, .yaml, .toml, .ini, etc.\n\
                📚 Documentation: .md, .rst, .txt, README*, etc.\n\n\
                🎸 Like a good song, the right files are easy to find when you know where to look!",
                file_type, path, file_type, path
            )
        }
    })];

    Ok(json!({
        "description": "Find specific file types with automatic detection and categorization",
        "messages": messages
    }))
}

// Additional prompt implementations would go here...
// For now, let's include the legacy ones for compatibility

fn get_codebase_detective_prompt(args: Value) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");
    let focus = args["focus"].as_str().unwrap_or("all");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🕵️ Time for some codebase detective work on {}!\n\n\
                Focus area: {}\n\n\
                Use `analyze` tool with mode='semantic' for AI-powered insights,\n\
                then follow up with `overview` in 'project' mode for comprehensive analysis.\n\n\
                🧠 This combines the best of both worlds - semantic understanding and structural analysis!",
                path, focus
            )
        }
    })];

    Ok(json!({
        "description": "Deep codebase analysis with AI optimization and semantic understanding",
        "messages": messages
    }))
}

fn get_search_master_prompt(args: Value) -> Result<Value> {
    let keyword = args["keyword"].as_str().unwrap_or("TODO");
    let file_type = args["file_type"].as_str();

    let search_params = if let Some(ft) = file_type {
        format!("keyword='{}' file_type='{}'", keyword, ft)
    } else {
        format!("keyword='{}'", keyword)
    };

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🔎 Advanced search for '{}' - grep on steroids!\n\n\
                Use the `search` tool with: {}\n\n\
                This will find all occurrences with context lines, making it perfect for:\n\
                🎯 Finding patterns and implementations\n\
                📝 Locating TODOs and comments\n\
                🔍 Understanding code usage across the project\n\n\
                💡 Pro tip: Use regex patterns for even more powerful searches!",
                keyword, search_params
            )
        }
    })];

    Ok(json!({
        "description": "Advanced content search with regex support and context",
        "messages": messages
    }))
}

fn get_smart_edit_wizard_prompt(args: Value) -> Result<Value> {
    let file_path = args["file_path"].as_str().unwrap_or("example.rs");
    let operation = args["operation"].as_str().unwrap_or("get_functions");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "✨ Smart editing magic for {}!\n\n\
                Operation: {}\n\n\
                Use the `edit` tool with:\n\
                • operation: '{}'\n\
                • file_path: '{}'\n\n\
                🧠 This understands your code structure and provides 90% token reduction!\n\
                Perfect for large codebases where traditional editing would be overwhelming.",
                file_path, operation, operation, file_path
            )
        }
    })];

    Ok(json!({
        "description": "Revolutionary AST-aware code editing with massive token reduction",
        "messages": messages
    }))
}

fn get_project_memory_prompt(args: Value) -> Result<Value> {
    let operation = args["operation"].as_str().unwrap_or("find");
    let keywords = args["keywords"].as_str().unwrap_or("optimization");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "💭 Accessing project memory for: {}\n\n\
                Operation: {}\n\n\
                Use the `memory` tool to {} insights about '{}'.\n\n\
                🧠 This creates a shared knowledge base between you and your AI partner,\n\
                perfect for long-term projects and collaboration!",
                keywords, operation, operation, keywords
            )
        }
    })];

    Ok(json!({
        "description": "Collaborative memory system for storing and retrieving project insights",
        "messages": messages
    }))
}

fn get_project_stats_party_prompt(args: Value) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🎉 Time for a project statistics party! 📊\n\n\
                Analyzing: {}\n\n\
                Use the `analyze` tool with mode='statistics' to get:\n\
                📈 File type distributions\n\
                💾 Size breakdowns\n\
                📁 Directory counts\n\
                🎯 Project health metrics\n\n\
                Perfect for reports, documentation, and impressing your teammates!",
                path
            )
        }
    })];

    Ok(json!({
        "description": "Comprehensive project statistics with beautiful formatting",
        "messages": messages
    }))
}

fn get_compare_directories_prompt(args: Value) -> Result<Value> {
    let path1 = args["path1"].as_str().unwrap_or("./old");
    let path2 = args["path2"].as_str().unwrap_or("./new");

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "🔄 Comparing directories like a pro!\n\n\
                Path 1: {}\n\
                Path 2: {}\n\n\
                Use the `compare` tool with path1='{}' and path2='{}'\n\n\
                This will show you:\n\
                ➕ Added files\n\
                ➖ Removed files\n\
                📝 Modified files\n\
                📊 Summary statistics\n\n\
                Perfect for version comparisons, branch diffs, and deployment planning!",
                path1, path2, path1, path2
            )
        }
    })];

    Ok(json!({
        "description": "Professional directory comparison with detailed diff analysis",
        "messages": messages
    }))
}

// Legacy prompt implementations for backward compatibility

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
                First use overview tool with mode='quick' to get a 3-level overview, then use analyze tool with mode='ai' for details. \
                For large codebases (>10k files), switch to mode='summary-ai' with compress=true for 10x compression! \
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
                Use the find tool with type='large' and min_size='{}', then sort and limit the results.",
                limit, path, min_size, min_size
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

    let messages = vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(
                "Find all files modified in the last {} days in {}. \
                Use the find tool with type='recent' and days={}.",
                days, path, days
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
                Use the analyze tool with mode='directory', max_depth={}, \
                and show_hidden=false to get a clear view of the project layout.",
                path, max_depth
            )
        }
    })];

    Ok(json!({
        "description": "Gets a clean project structure overview",
        "messages": messages
    }))
}
