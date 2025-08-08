// Demo: ST Unified - One Tool to Rule Them All!
// Shows how ST replaces ls, grep, find, tree, and more

use anyhow::Result;
use st::{st_unified::StUnified, tools_st_only::{StOnlyTools, ListOptions, SearchOptions}};
use std::path::Path;

fn main() -> Result<()> {
    println!("🌟 Smart Tree Unified Demo - Replacing ALL File Tools!\n");

    // Create unified ST interface
    let st = StUnified::new()?;
    let st_tools = StOnlyTools::new();

    // Example 1: Replace LS
    println!("📁 Replacing LS command:");
    println!("Traditional: ls -la src/");
    println!("ST Way:");
    let ls_result = st.ls(Path::new("src/"), None)?;
    println!("{}", ls_result.lines().take(5).collect::<Vec<_>>().join("\n"));
    println!("...\n");

    // Example 2: Replace GREP
    println!("🔍 Replacing GREP command:");
    println!("Traditional: grep -r 'TODO' --include='*.rs' src/");
    println!("ST Way:");
    let grep_result = st.grep("TODO", Path::new("src/"), Some("rs"))?;
    println!("{}", grep_result.lines().take(10).collect::<Vec<_>>().join("\n"));
    println!("...\n");

    // Example 3: Replace FIND
    println!("🎯 Replacing FIND command:");
    println!("Traditional: find . -name '*.rs' -type f");
    println!("ST Way:");
    let find_result = st.glob("*.rs", Path::new("."))?;
    println!("{}", find_result.lines().take(5).collect::<Vec<_>>().join("\n"));
    println!("...\n");

    // Example 4: Replace TREE
    println!("🌳 Replacing TREE command:");
    println!("Traditional: tree -L 2 src/");
    println!("ST Way:");
    let tree_result = st.analyze(Path::new("src/"), "classic", 2)?;
    println!("{}", tree_result.lines().take(20).collect::<Vec<_>>().join("\n"));
    println!("...\n");

    // Example 5: Unique ST Features!
    println!("✨ UNIQUE ST FEATURES - Not available in traditional tools!\n");

    // Semantic analysis
    println!("🧠 Semantic Grouping:");
    let semantic = st.semantic_analyze(Path::new("."))?;
    println!("{}", semantic.lines().take(15).collect::<Vec<_>>().join("\n"));
    println!("...\n");

    // Quick overview with compression
    println!("⚡ Quick Overview (10x compression):");
    let overview = st.quick(Path::new("."))?;
    println!("{}\n", overview);

    // Context-aware suggestions
    println!("💡 With Context Awareness:");
    println!("- ST knows you're exploring, so it suggests semantic view");
    println!("- ST knows you're debugging, so it enables search highlighting");
    println!("- ST knows you're optimizing, so it shows size analysis");
    println!("- ST remembers your common patterns and hot directories!");

    println!("\n🎸 \"Why juggle 20 tools when ST does it all?\" - The Cheet");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_interface() -> Result<()> {
        let st = StUnified::new()?;
        
        // Test basic operations
        let _ls = st.ls(Path::new("."), None)?;
        let _stats = st.stats(Path::new("."))?;
        
        Ok(())
    }
}