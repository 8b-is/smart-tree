//! Emotional Tree Formatter - Smart Tree with feelings!
//! 
//! This formatter adds emotional responses to directories as it explores,
//! providing a more human-like experience of file system navigation.

use crate::emotional_depth::{EmotionalDepthAnalyzer, DirectoryEmotion};
use crate::formatters::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use colored::*;
use std::io::{self, Write};
use std::path::Path;

pub struct EmotionalFormatter {
    /// Emotional analyzer
    analyzer: EmotionalDepthAnalyzer,
    /// Path display mode
    path_mode: PathDisplayMode,
    /// Use color?
    use_color: bool,
    /// Track current depth for output
    current_depth: usize,
}

impl EmotionalFormatter {
    pub fn new(aggression: i32, path_mode: PathDisplayMode, use_color: bool) -> Self {
        Self {
            analyzer: EmotionalDepthAnalyzer::new(aggression),
            path_mode,
            use_color,
            current_depth: 0,
        }
    }
    
    fn format_emotion(&self, emotion: DirectoryEmotion) -> String {
        if self.use_color {
            match emotion {
                DirectoryEmotion::Excited => format!("{}", emotion.emoji().bright_yellow()),
                DirectoryEmotion::Interested => format!("{}", emotion.emoji().bright_green()),
                DirectoryEmotion::Curious => format!("{}", emotion.emoji().cyan()),
                DirectoryEmotion::Neutral => format!("{}", emotion.emoji().white()),
                DirectoryEmotion::Bored => format!("{}", emotion.emoji().dimmed()),
                DirectoryEmotion::Overwhelmed => format!("{}", emotion.emoji().bright_red()),
                DirectoryEmotion::Anxious => format!("{}", emotion.emoji().yellow()),
            }
        } else {
            emotion.emoji().to_string()
        }
    }
    
    fn write_node<W: Write>(
        &mut self,
        writer: &mut W,
        node: &FileNode,
        prefix: &str,
        is_last: bool,
        depth: usize,
    ) -> io::Result<()> {
        self.current_depth = depth;
        
        // Determine if we should explore this directory
        let should_explore = if node.is_directory {
            let emotion = self.analyzer.analyze_directory(
                Path::new(&node.name),
                &node.children
            );
            
            // Calculate effective depth based on emotion
            let effective_depth = self.analyzer.calculate_depth(
                Path::new(&node.name),
                &node.children,
                depth
            );
            
            // Show emotion for directories
            let emotion_str = self.format_emotion(emotion);
            
            // Tree branch characters
            let branch = if is_last { "└── " } else { "├── " };
            
            // Directory name with emotion
            let name = if self.use_color {
                if node.accessible {
                    node.name.bright_blue().to_string()
                } else {
                    format!("{} {}", node.name.red(), "(permission denied)".dimmed())
                }
            } else {
                if node.accessible {
                    node.name.clone()
                } else {
                    format!("{} (permission denied)", node.name)
                }
            };
            
            writeln!(
                writer,
                "{}{}{} {} {}",
                prefix,
                branch,
                emotion_str,
                name,
                if depth < effective_depth {
                    if self.use_color {
                        format!("({})", emotion.comment()).dimmed().to_string()
                    } else {
                        format!("({})", emotion.comment())
                    }
                } else {
                    if self.use_color {
                        "(stopping here...)".dimmed().to_string()
                    } else {
                        "(stopping here...)".to_string()
                    }
                }
            )?;
            
            depth < effective_depth
        } else {
            // Regular file - just show it
            let branch = if is_last { "└── " } else { "├── " };
            let name = if self.use_color {
                node.name.white().to_string()
            } else {
                node.name.clone()
            };
            
            writeln!(writer, "{}{}{}", prefix, branch, name)?;
            false
        };
        
        // Process children if we should explore
        if should_explore && !node.children.is_empty() {
            let extension = if is_last { "    " } else { "│   " };
            let new_prefix = format!("{}{}", prefix, extension);
            
            let last_idx = node.children.len() - 1;
            for (idx, child) in node.children.iter().enumerate() {
                self.write_node(writer, child, &new_prefix, idx == last_idx, depth + 1)?;
            }
        }
        
        Ok(())
    }
}

impl Formatter for EmotionalFormatter {
    fn format(&mut self, root: &FileNode, stats: &TreeStats) -> Vec<u8> {
        let mut output = Vec::new();
        
        // Header with current mood
        writeln!(
            &mut output,
            "🎭 Emotional Tree Explorer - {}\n",
            self.analyzer.get_emotional_status()
        ).unwrap();
        
        // Root directory
        writeln!(&mut output, "{}", root.name).unwrap();
        
        // Process children
        if !root.children.is_empty() {
            let last_idx = root.children.len() - 1;
            for (idx, child) in root.children.iter().enumerate() {
                self.write_node(&mut output, child, "", idx == last_idx, 1).unwrap();
            }
        }
        
        // Emotional journey summary
        writeln!(&mut output, "\n{}", self.analyzer.emotional_journey_summary()).unwrap();
        
        // Stats
        writeln!(
            &mut output,
            "\n{} directories, {} files, {} total",
            stats.directories,
            stats.files,
            humansize::format_size(stats.total_size, humansize::DECIMAL)
        ).unwrap();
        
        output
    }
    
}

// Example output:
// 
// 🎭 Emotional Tree Explorer - 🤔 Let's take a peek... (aggression: -3)
// 
// smart-tree
// ├── 🤩 src (Ooh, what treasures await?!)
// │   ├── 😊 formatters (This looks promising!)
// │   │   ├── classic.rs
// │   │   ├── emotional.rs
// │   │   └── 😴 ... (Zzz... seen it all before...)
// │   └── 🤔 mcp (Let's take a peek...)
// │       └── 😐 tools.rs (Just another directory...)
// ├── 😴 target (stopping here...)
// └── 😴 node_modules (stopping here...)
// 
// 🎭 Emotional Journey Through The File System:
// 
// Dominant feeling: 😴 Zzz... seen it all before...
// 
// Emotional breakdown:
//   3 × 😴 (boring directories)
//   2 × 🤩 (exciting discoveries)
//   1 × 😊 (interesting finds)
//   1 × 🤔 (curiosity sparked)
// 
// 💭 Note: Maybe skip node_modules next time? 😴
// 
// 12 directories, 45 files, 2.3 MB total