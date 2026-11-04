//! Demo of the context gathering system
//!
//! This shows how Smart Tree can search AI tool directories for project context

use anyhow::Result;
use st::context_gatherer::{ContextGatherer, GatherConfig};

fn main() -> Result<()> {
    println!("=== Smart Tree Context Gathering Demo ===\n");

    // Get current directory as the project
    let project_path = std::env::current_dir()?;
    println!(
        "Gathering context for project: {}\n",
        project_path.display()
    );

    // Configure what to search for
    let config = GatherConfig {
        project_identifiers: vec![
            "smart-tree".to_string(),
            "8b-is".to_string(),
            project_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
        ],
        ..Default::default()
    };

    // You can also add custom directories to search
    // config.custom_dirs.push(PathBuf::from("/home/user/my-notes"));

    // Create gatherer
    let mut gatherer = ContextGatherer::new(project_path.clone(), config);

    // Gather all context
    println!("🔍 Searching AI tool directories...\n");
    gatherer.gather_all()?;

    // Get results
    let contexts = gatherer.contexts();

    println!("Found {} context entries\n", contexts.len());

    // Show top 5 most relevant
    println!("📊 Top 5 Most Relevant Contexts:\n");
    for (i, context) in contexts.iter().take(5).enumerate() {
        println!(
            "{}. {} (Score: {:.2})",
            i + 1,
            context.ai_tool,
            context.relevance_score
        );
        println!("   Type: {:?}", context.content_type);
        println!("   Path: {}", context.source_path.display());
        println!(
            "   Size: {} bytes",
            context
                .metadata
                .get("size")
                .unwrap_or(&"unknown".to_string())
        );
        println!();
    }

    // Show summary by tool
    let mut tool_counts = std::collections::HashMap::new();
    for context in contexts {
        *tool_counts.entry(context.ai_tool.clone()).or_insert(0) += 1;
    }

    println!("📈 Context Sources:");
    for (tool, count) in tool_counts {
        println!("   {}: {} files", tool, count);
    }

    // Save to M8 format
    println!("\n💾 Converting to M8 format...");
    let m8_data = gatherer.to_m8()?;
    println!(
        "M8 size: {} bytes (compressed wave-based format)",
        m8_data.len()
    );

    // Optionally save to file
    let output_path = project_path.join(".mem8").join("gathered_context.m8");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, &m8_data)?;
    println!("Saved to: {}", output_path.display());

    println!("\n✅ Context gathering complete!");
    println!("\nNext steps:");
    println!("1. Use the MCP tool 'gather_project_context' to do this from Claude");
    println!("2. The M8 file can be processed by MEM8-aware tools");
    println!("3. Privacy mode is enabled by default to redact sensitive data");

    Ok(())
}

// Example output:
//
// === Smart Tree Context Gathering Demo ===
//
// Gathering context for project: /home/user/projects/smart-tree
//
// 🔍 Searching AI tool directories...
//
// Found 23 context entries
//
// 📊 Top 5 Most Relevant Contexts:
//
// 1. .claude (Score: 0.95)
//    Type: ChatHistory
//    Path: /home/user/.claude/chats/smart-tree-session.json
//    Size: 45632 bytes
//
// 2. .cursor (Score: 0.87)
//    Type: ProjectSettings
//    Path: /home/user/.cursor/workspaces/smart-tree.json
//    Size: 2341 bytes
//
// 📈 Context Sources:
//    .claude: 8 files
//    .cursor: 5 files
//    .vscode: 10 files
