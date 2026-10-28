use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::index::{Mem8Index, ProjectStatus};

#[derive(Parser)]
#[command(name = "mem8")]
#[command(about = "Universal AI memory index management")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize or update the memory index
    Init {
        /// Force reinitialize even if index exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show current user profile and preferences
    Profile {
        /// Section to show (preferences, communication, knowledge, personality)
        #[arg(short, long)]
        section: Option<String>,
    },
    
    /// List all projects and their status
    Projects {
        /// Filter by status (active, paused, completed)
        #[arg(short, long)]
        status: Option<String>,
    },
    
    /// Add or update a project
    Project {
        /// Project name
        name: String,
        
        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Set status (active, paused, completed)
        #[arg(short, long)]
        status: Option<String>,
        
        /// Add a note
        #[arg(short, long)]
        note: Option<String>,
        
        /// Set current focus
        #[arg(short, long)]
        focus: Option<String>,
    },
    
    /// Register a new memory block (.m8 file)
    Register {
        /// Path to .m8 file
        path: PathBuf,
        
        /// Tags to add
        #[arg(short, long)]
        tags: Vec<String>,
    },
    
    /// Search across all memories
    Search {
        /// Search query
        query: String,
        
        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Show memory statistics
    Stats,
    
    /// Export index as human-readable format
    Export {
        /// Output format (json, yaml, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { force } => init_index(force),
        Commands::Profile { section } => show_profile(section),
        Commands::Projects { status } => list_projects(status),
        Commands::Project { name, path, status, note, focus } => 
            update_project(name, path, status, note, focus),
        Commands::Register { path, tags } => register_memory(path, tags),
        Commands::Search { query, limit } => search_memories(query, limit),
        Commands::Stats => show_stats(),
        Commands::Export { format, output } => export_index(format, output),
    }
}

fn init_index(force: bool) -> Result<()> {
    let index_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mem8")
        .join("index.m8");
    
    if index_path.exists() && !force {
        println!("❓ Index already exists. Use --force to reinitialize.");
        return Ok(());
    }
    
    let index = Mem8Index::new();
    index.save()?;
    
    println!("✅ Initialized ~/.mem8/index.m8");
    println!("🧠 Your universal AI memory index is ready!");
    
    Ok(())
}

fn show_profile(section: Option<String>) -> Result<()> {
    let index = Mem8Index::load_or_create()?;
    let profile = &index.user_profile;
    
    match section.as_deref() {
        Some("preferences") => {
            println!("🛠️  Technology Preferences:");
            println!("\n📦 Package Managers:");
            for (pm, pref) in &profile.preferences.package_managers {
                let emoji = if pref.preference > 0 { "✅" } else { "❌" };
                println!("  {} {}: {} (score: {})", emoji, pm, 
                    pref.reasons.join(", "), pref.preference);
            }
            
            println!("\n💻 Operating Systems:");
            for (os, pref) in &profile.preferences.operating_systems {
                let emoji = if pref.base_preference > 0 { "✅" } else { "⚠️" };
                println!("  {} {}: {}", emoji, os, 
                    pref.reactions.join(", "));
                if let Some(strategy) = &pref.nudge_strategy {
                    println!("    💡 Strategy: {}", strategy);
                }
            }
        },
        Some("communication") => {
            println!("💬 Communication Style:");
            println!("  Detail preference: {:?}", profile.communication_style.detail_preference);
            println!("  Humor: Puns={}, Technical jokes={}", 
                profile.communication_style.humor_style.appreciates_puns,
                profile.communication_style.humor_style.technical_jokes);
            println!("  Learning: Examples={}, By doing={}", 
                profile.communication_style.learning_style.prefers_examples,
                profile.communication_style.learning_style.learns_by_doing);
        },
        Some("personality") => {
            println!("🧩 Personality Insights:");
            let insights = &profile.personality_insights;
            println!("  Work style: Perfectionism={:.1}, Experimentation={:.1}", 
                insights.work_style.perfectionist_score,
                insights.work_style.experimentation_willingness);
            println!("  Problem solving: Research first={}, Trial & error comfort={:.1}", 
                insights.problem_solving.research_first,
                insights.problem_solving.trial_and_error_comfort);
            println!("  Collaboration: Autonomy={}, Teaching enthusiasm={:.1}", 
                insights.collaboration.prefers_autonomy,
                insights.collaboration.teaching_enthusiasm);
        },
        _ => {
            // Show overview
            println!("👤 User Profile: {}", profile.name);
            println!("\n📊 Quick Stats:");
            println!("  Preferences tracked: {} package managers, {} languages, {} OS",
                profile.preferences.package_managers.len(),
                profile.preferences.languages.len(),
                profile.preferences.operating_systems.len());
            println!("  Knowledge areas: {} expertise, {} learning",
                profile.knowledge_map.expertise.len(),
                profile.knowledge_map.learning.len());
            println!("\nUse --section [preferences|communication|personality] for details");
        }
    }
    
    Ok(())
}

fn list_projects(status_filter: Option<String>) -> Result<()> {
    let index = Mem8Index::load_or_create()?;
    
    let status_match = status_filter.as_ref().and_then(|s| {
        match s.to_lowercase().as_str() {
            "active" => Some(ProjectStatus::Active),
            "paused" => Some(ProjectStatus::Paused),
            "completed" => Some(ProjectStatus::Completed),
            _ => None,
        }
    });
    
    println!("📁 Projects:");
    
    for (name, project) in &index.projects {
        if let Some(ref status) = status_match {
            match project.status {
                ref ps if std::mem::discriminant(ps) != std::mem::discriminant(status) => continue,
                _ => {}
            }
        }
        
        let status_emoji = match project.status {
            ProjectStatus::Active => "🟢",
            ProjectStatus::Paused => "🟡",
            ProjectStatus::Completed => "✅",
            ProjectStatus::Archived => "📦",
            ProjectStatus::Planning => "📝",
        };
        
        println!("\n{} {} ({:?})", status_emoji, name, project.status);
        println!("  📍 Path: {}", project.path.display());
        if let Some(focus) = &project.current_focus {
            println!("  🎯 Focus: {}", focus);
        }
        if !project.blockers.is_empty() {
            println!("  🚧 Blockers: {}", project.blockers.join(", "));
        }
        println!("  ⏰ Last worked: {}", project.last_worked.format("%Y-%m-%d %H:%M"));
    }
    
    Ok(())
}

fn update_project(
    name: String, 
    path: Option<PathBuf>, 
    status: Option<String>,
    note: Option<String>,
    focus: Option<String>
) -> Result<()> {
    let mut index = Mem8Index::load_or_create()?;
    
    let project = index.projects.entry(name.clone()).or_insert_with(|| {
        super::index::ProjectContext {
            name: name.clone(),
            path: path.clone().unwrap_or_else(|| PathBuf::from(&name)),
            status: ProjectStatus::Active,
            technologies: vec![],
            current_focus: None,
            blockers: vec![],
            last_worked: chrono::Utc::now(),
            related_memories: vec![],
            notes: vec![],
        }
    });
    
    if let Some(p) = path {
        project.path = p;
    }
    
    if let Some(s) = status {
        project.status = match s.to_lowercase().as_str() {
            "active" => ProjectStatus::Active,
            "paused" => ProjectStatus::Paused,
            "completed" => ProjectStatus::Completed,
            "archived" => ProjectStatus::Archived,
            "planning" => ProjectStatus::Planning,
            _ => project.status.clone(),
        };
    }
    
    if let Some(n) = note {
        project.notes.push(format!("{}: {}", chrono::Utc::now().format("%Y-%m-%d"), n));
    }
    
    if let Some(f) = focus {
        project.current_focus = Some(f);
    }
    
    project.last_worked = chrono::Utc::now();
    
    index.save()?;
    println!("✅ Updated project: {}", name);
    
    Ok(())
}

fn register_memory(path: PathBuf, tags: Vec<String>) -> Result<()> {
    let mut index = Mem8Index::load_or_create()?;
    
    // Read .m8 file metadata
    let file_size = std::fs::metadata(&path)?.len() as usize;
    
    let entry = super::index::MemoryBlockEntry {
        id: uuid::Uuid::new_v4(),
        file_path: path.clone(),
        source_type: "manual".to_string(),
        created_at: chrono::Utc::now(),
        message_count: 0, // Would be extracted from .m8 file
        compressed_size: file_size,
        tags,
        summary: format!("Memory block from {}", path.display()),
        key_concepts: vec![],
    };
    
    index.memory_blocks.insert(entry.id, entry);
    index.metadata.total_memories += 1;
    index.metadata.last_updated = chrono::Utc::now();
    
    index.save()?;
    println!("✅ Registered memory block: {}", path.display());
    
    Ok(())
}

fn search_memories(query: String, limit: usize) -> Result<()> {
    let index = Mem8Index::load_or_create()?;
    
    println!("🔍 Searching for: {}", query);
    println!("Found {} results (showing up to {}):\n", 
        index.memory_blocks.len().min(limit), limit);
    
    // Simple search - in real implementation would use proper search
    let query_lower = query.to_lowercase();
    let mut results = vec![];
    
    for (id, block) in &index.memory_blocks {
        if block.summary.to_lowercase().contains(&query_lower) ||
           block.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) ||
           block.key_concepts.iter().any(|c| c.to_lowercase().contains(&query_lower)) {
            results.push((id, block));
        }
    }
    
    for (i, (_id, block)) in results.iter().take(limit).enumerate() {
        println!("{}. {} ({})", i + 1, block.summary, block.file_path.display());
        println!("   Tags: {}", block.tags.join(", "));
        println!("   Size: {} bytes", block.compressed_size);
        println!();
    }
    
    Ok(())
}

fn show_stats() -> Result<()> {
    let index = Mem8Index::load_or_create()?;
    
    println!("📊 Memory Index Statistics:");
    println!("\n🧠 Index Metadata:");
    println!("  Version: {}", index.metadata.version);
    println!("  Created: {}", index.metadata.created_at.format("%Y-%m-%d"));
    println!("  Last updated: {}", index.metadata.last_updated.format("%Y-%m-%d %H:%M"));
    
    println!("\n💾 Memory Blocks:");
    println!("  Total blocks: {}", index.memory_blocks.len());
    println!("  Total conversations: {}", index.metadata.total_conversations);
    let total_size: usize = index.memory_blocks.values()
        .map(|b| b.compressed_size).sum();
    println!("  Total compressed size: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);
    
    println!("\n📁 Projects:");
    let active = index.projects.values()
        .filter(|p| matches!(p.status, ProjectStatus::Active)).count();
    println!("  Total: {} ({} active)", index.projects.len(), active);
    
    println!("\n🔗 Relationships:");
    println!("  Concept links: {}", index.relationships.concept_links.len());
    println!("  Project links: {}", index.relationships.project_links.len());
    
    Ok(())
}

fn export_index(format: String, output: Option<PathBuf>) -> Result<()> {
    let index = Mem8Index::load_or_create()?;
    
    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&index)?,
        "yaml" => serde_yaml::to_string(&index)?,
        "markdown" | _ => generate_markdown_export(&index),
    };
    
    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("✅ Exported to file");
    } else {
        println!("{}", content);
    }
    
    Ok(())
}

fn generate_markdown_export(index: &Mem8Index) -> String {
    let mut md = String::new();
    
    md.push_str("# Mem8 Universal AI Memory Index\n\n");
    
    md.push_str("## User Profile\n\n");
    md.push_str(&format!("**Name**: {}\n\n", index.user_profile.name));
    
    md.push_str("### Technology Preferences\n\n");
    for (pm, pref) in &index.user_profile.preferences.package_managers {
        let emoji = if pref.preference > 0 { "👍" } else { "👎" };
        md.push_str(&format!("- {} **{}** (score: {})\n", emoji, pm, pref.preference));
    }
    
    md.push_str("\n## Active Projects\n\n");
    for (name, project) in &index.projects {
        if matches!(project.status, ProjectStatus::Active) {
            md.push_str(&format!("### {}\n", name));
            md.push_str(&format!("- Path: `{}`\n", project.path.display()));
            if let Some(focus) = &project.current_focus {
                md.push_str(&format!("- Current focus: {}\n", focus));
            }
            md.push_str("\n");
        }
    }
    
    md.push_str("\n## Memory Statistics\n\n");
    md.push_str(&format!("- Total memory blocks: {}\n", index.memory_blocks.len()));
    md.push_str(&format!("- Total conversations: {}\n", index.metadata.total_conversations));
    
    md
}