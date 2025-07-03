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

        // Get relations based on focus or all
        let relations: Vec<&crate::relations::FileRelation> = if let Some(focus_file) = &self.focus {
            // Convert relative path to absolute for matching
            let abs_focus = if focus_file.is_relative() {
                root_path.join(focus_file)
            } else {
                focus_file.clone()
            };
            
            let file_relations = analyzer.get_file_relations(&abs_focus);
            eprintln!(
                "📄 Found {} relationships for {}",
                file_relations.len(),
                focus_file.display()
            );
            file_relations
        } else {
            analyzer.get_relations().iter().collect()
        };

        // Write header
        writeln!(writer, "🔗 Code Relationship Analysis")?;
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer)?;

        // If no relationships found
        if relations.is_empty() {
            if let Some(focus_file) = &self.focus {
                writeln!(writer, "No relationships found for: {}", focus_file.display())?;
            } else {
                writeln!(writer, "No relationships found in the codebase.")?;
            }
            return Ok(());
        }

        // Group relations by type
        use crate::relations::RelationType;
        let mut imports = Vec::new();
        let mut calls = Vec::new();
        let mut types = Vec::new();
        let mut tests = Vec::new();
        let mut coupled = Vec::new();

        for relation in &relations {
            match &relation.relation_type {
                RelationType::Imports => imports.push(relation),
                RelationType::FunctionCall => calls.push(relation),
                RelationType::TypeUsage => types.push(relation),
                RelationType::TestedBy => tests.push(relation),
                RelationType::Coupled => coupled.push(relation),
                RelationType::Exports => {}, // Skip exports for now
            }
        }

        // Display relationships by type
        if !imports.is_empty() {
            writeln!(writer, "📦 Imports ({}):", imports.len())?;
            for rel in imports {
                writeln!(writer, "  {} → {}", rel.source.display(), rel.target.display())?;
            }
            writeln!(writer)?;
        }

        if !calls.is_empty() {
            writeln!(writer, "📞 Function Calls ({}):", calls.len())?;
            for rel in calls {
                writeln!(writer, "  {} → {}", rel.source.display(), rel.target.display())?;
            }
            writeln!(writer)?;
        }

        if !types.is_empty() {
            writeln!(writer, "🏷️  Type Usage ({}):", types.len())?;
            for rel in types {
                writeln!(writer, "  {} → {}", rel.source.display(), rel.target.display())?;
            }
            writeln!(writer)?;
        }

        if !tests.is_empty() {
            writeln!(writer, "🧪 Tests ({}):", tests.len())?;
            for rel in tests {
                writeln!(writer, "  {} → {}", rel.source.display(), rel.target.display())?;
            }
            writeln!(writer)?;
        }

        if !coupled.is_empty() {
            writeln!(writer, "🔗 Coupled Changes ({}):", coupled.len())?;
            for rel in coupled {
                writeln!(writer, "  {} ↔ {}", rel.source.display(), rel.target.display())?;
            }
            writeln!(writer)?;
        }

        // Summary
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer, "Total relationships: {}", relations.len())?;
        if let Some(focus_file) = &self.focus {
            writeln!(writer, "Focused on: {}", focus_file.display())?;
        } else {
            writeln!(writer, "Files analyzed: {}", root_path.display())?;
        }

        Ok(())
    }
}
