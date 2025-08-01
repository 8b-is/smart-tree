//! Tree Agent CLI - Orchestrate the living forest of development
//! 
//! Usage:
//!   tree init <project>         Initialize a new project
//!   tree assign <agent>         Assign an agent to a branch/pane
//!   tree observe               Observe all agents and update memory
//!   tree commit <agent>        Commit work for an agent
//!   tree suggest-merge         Suggest compatible merges
//!   tree mood-check           Check emotional state of all agents
//!   tree push                 Push to n8x.is nexus

use clap::{Parser, Subcommand};
use anyhow::Result;
use st::tree_agent::TreeAgent;

#[derive(Parser)]
#[command(name = "tree")]
#[command(about = "Orchestrate the living forest of AI-human development")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project orchestrator
    Init {
        /// Project name
        project: String,
    },
    
    /// Assign an agent to a branch and tmux pane
    Assign {
        /// Agent name (Claude, Omni, human name, etc.)
        agent: String,
        
        /// Git branch name
        #[arg(long)]
        branch: Option<String>,
        
        /// Tmux pane ID (creates new if not specified)
        #[arg(long)]
        pane: Option<String>,
    },
    
    /// Observe all agents and update memory
    Observe {
        /// Save state to file
        #[arg(long)]
        save_to: Option<String>,
    },
    
    /// Commit work for a specific agent
    Commit {
        /// Agent name
        agent: String,
        
        /// Commit message
        #[arg(short, long)]
        msg: String,
    },
    
    /// Suggest merges based on wave compatibility
    SuggestMerge {
        /// Automatically merge highly compatible branches
        #[arg(long)]
        auto: bool,
    },
    
    /// Check emotional state of all agents
    MoodCheck,
    
    /// Push to n8x.is nexus
    Push {
        /// Target nexus endpoint
        #[arg(long)]
        target: Option<String>,
        
        /// Project name override
        #[arg(long)]
        project: Option<String>,
    },
    
    /// Create a snapshot of current state
    Snapshot {
        /// Output file (.m8)
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { project } => {
            println!("🌱 Initializing living forest: {}", project);
            let _agent = TreeAgent::init(&project)?;
            
            // Save initial state
            std::fs::write(
                ".tree-agent.json",
                serde_json::to_string_pretty(&project)?
            )?;
            
            println!("✓ Forest initialized");
            println!("  Use 'tree assign <agent>' to add AI teammates");
        }
        
        Commands::Assign { agent, branch, pane } => {
            let mut tree_agent = load_agent()?;
            
            // Generate branch name if not provided
            let branch = branch.unwrap_or_else(|| {
                format!("{}-{}", agent.to_lowercase(), chrono::Utc::now().format("%Y%m%d-%H%M%S"))
            });
            
            tree_agent.assign_agent(&agent, pane.as_deref(), &branch)?;
        }
        
        Commands::Observe { save_to } => {
            let mut tree_agent = load_agent()?;
            
            let save_path = save_to.as_ref().map(|s| std::path::Path::new(s));
            tree_agent.observe(save_path)?;
        }
        
        Commands::Commit { agent, msg } => {
            let mut tree_agent = load_agent()?;
            tree_agent.commit_agent(&agent, &msg)?;
        }
        
        Commands::SuggestMerge { auto } => {
            let tree_agent = load_agent()?;
            tree_agent.suggest_merge(auto)?;
        }
        
        Commands::MoodCheck => {
            let tree_agent = load_agent()?;
            tree_agent.mood_check()?;
        }
        
        Commands::Push { target, project } => {
            let tree_agent = load_agent()?;
            
            if target.is_some() || project.is_some() {
                println!("Custom nexus configuration not yet implemented");
            }
            
            tree_agent.push_to_nexus()?;
        }
        
        Commands::Snapshot { output } => {
            let tree_agent = load_agent()?;
            
            let filename = output.unwrap_or_else(|| {
                format!("snapshot-{}.m8", chrono::Utc::now().format("%Y%m%d-%H%M%S"))
            });
            
            println!("📸 Creating snapshot: {}", filename);
            
            // Export MEM8 state
            let mut buffer = Vec::new();
            tree_agent.mem8.export_memories(&mut buffer)?;
            let buffer_len = buffer.len();
            std::fs::write(&filename, buffer)?;
            
            println!("✓ Snapshot saved ({} bytes)", buffer_len);
        }
    }
    
    Ok(())
}

fn load_agent() -> Result<TreeAgent> {
    // Read project name from saved state
    let project = if std::path::Path::new(".tree-agent.json").exists() {
        let content = std::fs::read_to_string(".tree-agent.json")?;
        serde_json::from_str::<String>(&content)?
    } else {
        return Err(anyhow::anyhow!(
            "No tree-agent project found. Run 'tree init <project>' first"
        ));
    };
    
    TreeAgent::init(&project)
}