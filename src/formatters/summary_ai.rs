//! AI Summary formatter - "Compressed intelligence for machines!" - Omni
//! Provides compressed, structured summaries optimized for AI consumption

use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use crate::content_detector::{ContentDetector, DirectoryType};
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;

pub struct SummaryAiFormatter {
    compress: bool,
}

impl SummaryAiFormatter {
    pub fn new(compress: bool) -> Self {
        Self { compress }
    }
}

impl Formatter for SummaryAiFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Detect directory type
        let dir_type = ContentDetector::detect(nodes, root_path);
        
        // Header
        writeln!(writer, "SUMMARY_AI_V1:")?;
        writeln!(writer, "PATH:{}", root_path.display())?;
        writeln!(writer, "STATS:F{:x}D{:x}S{:x}", 
            stats.total_files, stats.total_dirs, stats.total_size)?;
        
        // Directory type analysis
        match &dir_type {
            DirectoryType::CodeProject { language, framework, has_tests, has_docs } => {
                write!(writer, "TYPE:CODE[{:?}", language)?;
                if let Some(fw) = framework {
                    write!(writer, ",{:?}", fw)?;
                }
                writeln!(writer, "]T{}D{}", 
                    if *has_tests { "1" } else { "0" },
                    if *has_docs { "1" } else { "0" }
                )?;
                
                // Key files in compressed format
                write!(writer, "KEY:")?;
                let important = find_key_files(nodes, language);
                for (i, file) in important.iter().enumerate() {
                    if i > 0 { write!(writer, ",")?; }
                    write!(writer, "{}", file)?;
                }
                writeln!(writer)?;
                
                // File type distribution
                let ext_counts = get_extension_counts(nodes);
                write!(writer, "EXT:")?;
                for (i, (ext, count)) in ext_counts.iter().enumerate() {
                    if i > 0 { write!(writer, ",")?; }
                    write!(writer, "{}:{}", ext, count)?;
                }
                writeln!(writer)?;
            }
            
            DirectoryType::PhotoCollection { image_count, date_range, cameras } => {
                write!(writer, "TYPE:PHOTO[{}]", image_count)?;
                if let Some((start, end)) = date_range {
                    write!(writer, "DATE[{},{}]", start, end)?;
                }
                if !cameras.is_empty() {
                    write!(writer, "CAM[{}]", cameras.join(","))?;
                }
                writeln!(writer)?;
            }
            
            DirectoryType::DocumentArchive { categories, total_docs } => {
                write!(writer, "TYPE:DOCS[{}]", total_docs)?;
                if !categories.is_empty() {
                    write!(writer, "CAT[")?;
                    for (i, (cat, count)) in categories.iter().enumerate() {
                        if i > 0 { write!(writer, ",")?; }
                        write!(writer, "{}:{}", cat, count)?;
                    }
                    write!(writer, "]")?;
                }
                writeln!(writer)?;
            }
            
            DirectoryType::MediaLibrary { video_count, audio_count, total_duration, quality } => {
                write!(writer, "TYPE:MEDIA[V{},A{}]", video_count, audio_count)?;
                if let Some(duration) = total_duration {
                    write!(writer, "DUR[{}]", duration)?;
                }
                if !quality.is_empty() {
                    write!(writer, "Q[{}]", quality.join(","))?;
                }
                writeln!(writer)?;
            }
            
            DirectoryType::DataScience { notebooks, datasets, languages } => {
                write!(writer, "TYPE:DATA[N{},D{}]", notebooks, datasets)?;
                if !languages.is_empty() {
                    write!(writer, "LANG[{}]", languages.join(","))?;
                }
                writeln!(writer)?;
            }
            
            DirectoryType::MixedContent { dominant_type, file_types, total_files } => {
                write!(writer, "TYPE:MIXED[{}]", total_files)?;
                if let Some(dominant) = dominant_type {
                    write!(writer, "DOM[{}]", dominant)?;
                }
                writeln!(writer)?;
                
                // Top 5 file types
                let mut types: Vec<_> = file_types.iter().collect();
                types.sort_by(|a, b| b.1.cmp(a.1));
                write!(writer, "TOP:")?;
                for (i, (ext, count)) in types.iter().take(5).enumerate() {
                    if i > 0 { write!(writer, ",")?; }
                    write!(writer, "{}:{}", ext, count)?;
                }
                writeln!(writer)?;
            }
        }
        
        // Structure summary - top-level directories
        let mut dir_sizes: HashMap<String, (usize, u64)> = HashMap::new();
        for node in nodes {
            if let Ok(relative) = node.path.strip_prefix(root_path) {
                if let Some(first_component) = relative.components().next() {
                    if let Some(name) = first_component.as_os_str().to_str() {
                        let entry = dir_sizes.entry(name.to_string()).or_insert((0, 0));
                        entry.0 += 1;
                        if !node.is_dir {
                            entry.1 += node.size;
                        }
                    }
                }
            }
        }
        
        write!(writer, "DIRS:")?;
        let mut dirs: Vec<_> = dir_sizes.iter().collect();
        dirs.sort_by(|a, b| b.1.1.cmp(&a.1.1)); // Sort by size
        for (i, (name, (count, size))) in dirs.iter().take(10).enumerate() {
            if i > 0 { write!(writer, ",")?; }
            write!(writer, "{}[{},{:x}]", name, count, size)?;
        }
        writeln!(writer)?;
        
        // Largest files
        let mut files: Vec<_> = nodes.iter()
            .filter(|n| !n.is_dir)
            .collect();
        files.sort_by(|a, b| b.size.cmp(&a.size));
        
        write!(writer, "LARGE:")?;
        for (i, file) in files.iter().take(5).enumerate() {
            if i > 0 { write!(writer, ",")?; }
            let name = file.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            write!(writer, "{}:{:x}", name, file.size)?;
        }
        writeln!(writer)?;
        
        // Footer
        writeln!(writer, "END_SUMMARY_AI")?;
        
        Ok(())
    }
}

fn find_key_files(nodes: &[FileNode], language: &crate::content_detector::Language) -> Vec<String> {
    use crate::content_detector::Language;
    
    let mut key_files = Vec::new();
    let important_names = match language {
        Language::Rust => vec!["Cargo.toml", "main.rs", "lib.rs"],
        Language::Python => vec!["requirements.txt", "setup.py", "main.py", "__init__.py"],
        Language::JavaScript | Language::TypeScript => vec!["package.json", "index.js", "index.ts"],
        Language::Go => vec!["go.mod", "main.go"],
        Language::Java => vec!["pom.xml", "build.gradle", "Main.java"],
        _ => vec![],
    };
    
    for node in nodes {
        if node.is_dir {
            continue;
        }
        
        let name = node.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if important_names.contains(&name) {
            key_files.push(name.to_string());
        }
    }
    
    key_files
}

fn get_extension_counts(nodes: &[FileNode]) -> Vec<(String, usize)> {
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    
    for node in nodes {
        if !node.is_dir {
            if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
                *ext_counts.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    let mut counts: Vec<_> = ext_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.truncate(10); // Top 10 extensions
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::FileNode;
    use std::path::PathBuf;
    use std::collections::HashMap;
    
    #[test]
    fn test_ai_summary_formatter() {
        use crate::scanner::{FileType, FileCategory, FilesystemType};
        let formatter = SummaryAiFormatter::new(false);
        let nodes = vec![
            FileNode {
                path: PathBuf::from("/test/src/main.rs"),
                is_dir: false,
                size: 1000,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 2,
                file_type: FileType::RegularFile,
                category: FileCategory::Rust,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
            },
            FileNode {
                path: PathBuf::from("/test/Cargo.toml"),
                is_dir: false,
                size: 500,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::RegularFile,
                category: FileCategory::Toml,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
            },
        ];
        
        let stats = TreeStats {
            total_files: 2,
            total_dirs: 1,
            total_size: 1500,
            file_types: HashMap::new(),
            largest_files: vec![],
            newest_files: vec![],
            oldest_files: vec![],
        };
        
        let mut output = Vec::new();
        let result = formatter.format(&mut output, &nodes, &stats, &PathBuf::from("/test"));
        
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        
        // Check format markers
        assert!(output_str.starts_with("SUMMARY_AI_V1:"));
        assert!(output_str.contains("TYPE:CODE[Rust]"));
        assert!(output_str.contains("KEY:"));
        assert!(output_str.contains("END_SUMMARY_AI"));
    }
}