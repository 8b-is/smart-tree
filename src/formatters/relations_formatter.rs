//! Relations formatter that works with the standard formatter interface
//! "Making relations a first-class mode!" - Omni

use crate::formatters::Formatter;
use crate::relations::RelationAnalyzer;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

/// Main relations formatter - delegates to text formatter by default
pub struct RelationsFormatter {
    filter: Option<String>,
    focus: Option<std::path::PathBuf>,
}

impl RelationsFormatter {
    pub fn new(filter: Option<String>, focus: Option<std::path::PathBuf>) -> Self {
        Self { filter, focus }
    }
}

impl Formatter for RelationsFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        _nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Create relation analyzer
        let mut analyzer = RelationAnalyzer::new();

        // Analyze the directory
        eprintln!("🔍 Analyzing code relationships...");
        analyzer.analyze_directory(root_path)?;

        // Apply filters if specified
        if let Some(filter_type) = &self.filter {
            // In a real implementation, we'd filter the relations
            eprintln!("📋 Filtering by: {}", filter_type);
        }

        // Focus on specific file if requested
        if let Some(focus_file) = &self.focus {
            let relations = analyzer.get_file_relations(focus_file);
            eprintln!(
                "📄 Found {} relationships for {}",
                relations.len(),
                focus_file.display()
            );
        }

        // Write directly since we have a trait object
        writeln!(writer, "🔗 Code Relationship Analysis")?;
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer)?;

        // For now, just show a summary until we refactor the formatters
        let relations = analyzer.get_relations();
        writeln!(writer, "Total relationships found: {}", relations.len())?;
        writeln!(writer, "Files analyzed: {}", root_path.display())?;

        Ok(())
    }
}
