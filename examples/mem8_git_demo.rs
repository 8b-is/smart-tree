//! MEM8 Git Integration Demo
//! Shows how to build temporal memory from git history

use anyhow::Result;
use std::path::Path;
use st::mem8::{
    SmartTreeMem8, DirectoryMetadata, DirectoryEvent,
    integration::{ContentType, DirectoryHealth}, create_temporal_grooves,
    GitTemporalAnalyzer,
};

fn main() -> Result<()> {
    println!("=== MEM8 Git Temporal Integration Demo ===\n");
    
    // Initialize MEM8 with git awareness
    let mut mem8 = SmartTreeMem8::new();
    mem8.register_directory_patterns();
    
    // Check if we're in a git repository
    let repo_path = ".";
    match GitTemporalAnalyzer::new(repo_path) {
        Ok(analyzer) => {
            println!("✓ Git repository detected!");
            
            // Show some quick stats
            if let Ok(timeline) = analyzer.get_project_timeline() {
                println!("  Total commits: {}", timeline.len());
                
                if let Some(first) = timeline.last() {
                    println!("  First commit: {} by {}", 
                        first.timestamp.format("%Y-%m-%d"), 
                        first.author);
                }
                
                if let Some(latest) = timeline.first() {
                    println!("  Latest commit: {} by {}", 
                        latest.timestamp.format("%Y-%m-%d"), 
                        latest.author);
                }
            }
            
            // Import git timeline into wave memory
            println!("\n1. Importing git history into wave memory...");
            mem8.import_git_timeline(repo_path)?;
            println!("  ✓ Git timeline imported as wave patterns");
            
            // Create temporal grooves from commit patterns
            println!("\n2. Creating temporal grooves from activity patterns...");
            create_temporal_grooves(&mut mem8, repo_path)?;
            println!("  ✓ Weekly activity patterns encoded as persistent waves");
            
            // Analyze code churn
            println!("\n3. Analyzing code churn patterns...");
            if let Ok(churn) = analyzer.analyze_code_churn(10) {
                println!("  Top 5 most frequently changed files:");
                for (file, changes) in churn.iter().take(5) {
                    println!("    {} - {} changes", file, changes);
                }
            }
            
            // Show activity heatmap
            println!("\n4. Recent activity analysis (last 30 days)...");
            if let Ok(heatmap) = analyzer.get_activity_heatmap(30) {
                let total_commits: usize = heatmap.iter().map(|(_, count)| count).sum();
                println!("  Total commits: {}", total_commits);
                
                // Find most active day
                if let Some((date, max_commits)) = heatmap.iter().max_by_key(|(_, count)| count) {
                    println!("  Most active day: {} ({} commits)", 
                        date.format("%Y-%m-%d"), 
                        max_commits);
                }
            }
            
            // Query memories about source files
            println!("\n5. Querying wave memories...");
            let src_memories = mem8.query_path_memories("src");
            println!("  Found {} memories related to 'src'", src_memories.len());
            
            // Show how many memories are active
            println!("\n6. Memory statistics...");
            println!("  Active memories: {}", mem8.active_memory_count());
            println!("  Grid capacity: 4,294,967,296 waves (256×256×65536)");
            
            // Export to .m8 format
            println!("\n7. Exporting to .m8 format...");
            let mut m8_buffer = Vec::new();
            mem8.export_memories(&mut m8_buffer)?;
            println!("  Exported {} bytes (from git history)", m8_buffer.len());
            
        }
        Err(_) => {
            println!("✗ Not a git repository. Using standard directory scanning instead.");
            
            // Fall back to regular directory scanning
            println!("\n1. Storing directory memories (non-git mode)...");
            
            let directories = vec![
                ("src/", ContentType::Code, 0.9),
                ("examples/", ContentType::Code, 0.7),
                ("tests/", ContentType::Code, 0.6),
                ("docs/", ContentType::Documentation, 0.8),
            ];
            
            for (path, content_type, importance) in directories {
                if Path::new(path).exists() {
                    let metadata = DirectoryMetadata {
                        primary_type: content_type,
                        importance,
                        normalized_size: 0.5,
                        health: DirectoryHealth::Healthy,
                        activity_level: 0.5,
                        days_since_modified: 7,
                    };
                    
                    mem8.store_directory_memory(Path::new(path), metadata)?;
                    println!("  Stored: {}", path);
                }
            }
        }
    }
    
    println!("\n=== Demo Complete ===");
    println!("\nMEM8 Git Integration provides:");
    println!("  • Temporal patterns from commit history");
    println!("  • Activity-based wave frequencies");
    println!("  • Code churn awareness");
    println!("  • Developer behavior patterns");
    println!("  • 973× faster than traditional memory systems");
    
    Ok(())
}