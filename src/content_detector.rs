//! Content detection engine - "Understanding what's in your directories" - Omni
//! Analyzes directory contents to determine the type of project/collection

use std::collections::HashMap;
use std::path::Path;
use crate::scanner::FileNode;

/// Types of content that can be detected in a directory
#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryType {
    /// Software project with language and framework info
    CodeProject {
        language: Language,
        framework: Option<Framework>,
        has_tests: bool,
        has_docs: bool,
    },
    /// Photo/image collection
    PhotoCollection {
        image_count: usize,
        date_range: Option<(String, String)>,
        cameras: Vec<String>,
    },
    /// Document archive (PDFs, docs, etc.)
    DocumentArchive {
        categories: HashMap<String, usize>,
        total_docs: usize,
    },
    /// Media library (videos, audio)
    MediaLibrary {
        video_count: usize,
        audio_count: usize,
        total_duration: Option<String>,
    },
    /// Data science workspace
    DataScience {
        notebooks: usize,
        datasets: usize,
        languages: Vec<String>,
    },
    /// Mixed content or unknown
    MixedContent {
        dominant_type: Option<String>,
        file_types: HashMap<String, usize>,
        total_files: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Cpp,
    Ruby,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Framework {
    // Rust
    Actix,
    Rocket,
    Tokio,
    // Python
    Django,
    Flask,
    FastAPI,
    // JavaScript/TypeScript
    React,
    Vue,
    Angular,
    NextJs,
    Express,
    // Other
    Other(String),
}

/// Analyzes a directory and detects its content type
pub struct ContentDetector;

impl ContentDetector {
    /// Analyze nodes and detect directory type
    pub fn detect(nodes: &[FileNode], root_path: &Path) -> DirectoryType {
        // Count file extensions
        let mut ext_counts: HashMap<String, usize> = HashMap::new();
        let mut total_files = 0;
        
        for node in nodes {
            if !node.is_dir {
                total_files += 1;
                if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
                    *ext_counts.entry(ext.to_lowercase()).or_insert(0) += 1;
                }
            }
        }
        
        // Check for code project indicators
        if Self::is_code_project(&ext_counts, nodes, root_path) {
            return Self::analyze_code_project(nodes, root_path, &ext_counts);
        }
        
        // Check for photo collection
        if Self::is_photo_collection(&ext_counts) {
            return Self::analyze_photo_collection(nodes, &ext_counts);
        }
        
        // Check for document archive
        if Self::is_document_archive(&ext_counts) {
            return Self::analyze_document_archive(nodes);
        }
        
        // Check for media library
        if Self::is_media_library(&ext_counts) {
            return Self::analyze_media_library(nodes, &ext_counts);
        }
        
        // Check for data science
        if Self::is_data_science(&ext_counts) {
            return Self::analyze_data_science(&ext_counts);
        }
        
        // Default to mixed content
        DirectoryType::MixedContent {
            dominant_type: Self::get_dominant_type(&ext_counts),
            file_types: ext_counts,
            total_files,
        }
    }
    
    fn is_code_project(ext_counts: &HashMap<String, usize>, nodes: &[FileNode], _root_path: &Path) -> bool {
        // Check for common code file extensions
        let code_extensions = ["rs", "py", "js", "ts", "go", "java", "cpp", "c", "rb", "php"];
        let code_files: usize = code_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        // Check for project files
        let has_project_files = nodes.iter().any(|n| {
            let name = n.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            matches!(name, "Cargo.toml" | "package.json" | "requirements.txt" | "go.mod" | "pom.xml" | "Gemfile")
        });
        
        code_files > 5 || has_project_files
    }
    
    fn analyze_code_project(nodes: &[FileNode], _root_path: &Path, ext_counts: &HashMap<String, usize>) -> DirectoryType {
        // Detect primary language
        let language = if ext_counts.contains_key("rs") {
            Language::Rust
        } else if ext_counts.contains_key("py") {
            Language::Python
        } else if ext_counts.contains_key("ts") {
            Language::TypeScript
        } else if ext_counts.contains_key("js") {
            Language::JavaScript
        } else if ext_counts.contains_key("go") {
            Language::Go
        } else if ext_counts.contains_key("java") {
            Language::Java
        } else if ext_counts.contains_key("cpp") || ext_counts.contains_key("cc") {
            Language::Cpp
        } else if ext_counts.contains_key("rb") {
            Language::Ruby
        } else {
            Language::Other("Unknown".to_string())
        };
        
        // Detect framework
        let framework = Self::detect_framework(nodes, &language);
        
        // Check for tests and docs
        let has_tests = nodes.iter().any(|n| {
            let path_str = n.path.to_string_lossy();
            path_str.contains("test") || path_str.contains("spec")
        });
        
        let has_docs = nodes.iter().any(|n| {
            let name = n.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let path_str = n.path.to_string_lossy();
            name.ends_with(".md") || path_str.contains("docs/")
        });
        
        DirectoryType::CodeProject {
            language,
            framework,
            has_tests,
            has_docs,
        }
    }
    
    fn detect_framework(nodes: &[FileNode], language: &Language) -> Option<Framework> {
        for node in nodes {
            let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            match language {
                Language::Rust => {
                    // Check Cargo.toml for dependencies
                    if name == "Cargo.toml" {
                        // In real implementation, would read file and check deps
                        return None; // Placeholder
                    }
                }
                Language::JavaScript | Language::TypeScript => {
                    if name == "package.json" {
                        // Would check for React, Vue, etc. in dependencies
                        return None; // Placeholder
                    }
                }
                Language::Python => {
                    if name == "requirements.txt" || name == "pyproject.toml" {
                        // Would check for Django, Flask, etc.
                        return None; // Placeholder
                    }
                }
                _ => {}
            }
        }
        None
    }
    
    fn is_photo_collection(ext_counts: &HashMap<String, usize>) -> bool {
        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "raw", "dng", "heic"];
        let image_files: usize = image_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        image_files > 10
    }
    
    fn analyze_photo_collection(_nodes: &[FileNode], ext_counts: &HashMap<String, usize>) -> DirectoryType {
        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "raw", "dng", "heic"];
        let image_count: usize = image_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        DirectoryType::PhotoCollection {
            image_count,
            date_range: None, // Would need EXIF parsing
            cameras: vec![],  // Would need EXIF parsing
        }
    }
    
    fn is_document_archive(ext_counts: &HashMap<String, usize>) -> bool {
        let doc_extensions = ["pdf", "doc", "docx", "txt", "odt", "rtf"];
        let doc_files: usize = doc_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        doc_files > 10
    }
    
    fn analyze_document_archive(nodes: &[FileNode]) -> DirectoryType {
        let mut categories = HashMap::new();
        
        // Simple categorization based on filename patterns
        for node in nodes {
            if !node.is_dir {
                let name = node.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let category = if name.contains("invoice") || name.contains("receipt") || name.contains("bank") {
                    "Financial"
                } else if name.contains("homework") || name.contains("assignment") || name.contains("grade") {
                    "School"
                } else if name.contains("resume") || name.contains("cv") || name.contains("letter") {
                    "Personal"
                } else {
                    "Other"
                };
                
                *categories.entry(category.to_string()).or_insert(0) += 1;
            }
        }
        
        let total_docs = categories.values().sum();
        
        DirectoryType::DocumentArchive {
            categories,
            total_docs,
        }
    }
    
    fn is_media_library(ext_counts: &HashMap<String, usize>) -> bool {
        let video_extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv"];
        let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "m4a"];
        
        let video_files: usize = video_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        let audio_files: usize = audio_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        video_files + audio_files > 10
    }
    
    fn analyze_media_library(_nodes: &[FileNode], ext_counts: &HashMap<String, usize>) -> DirectoryType {
        let video_extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv"];
        let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "m4a"];
        
        let video_count: usize = video_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        let audio_count: usize = audio_extensions.iter()
            .filter_map(|ext| ext_counts.get(*ext))
            .sum();
        
        DirectoryType::MediaLibrary {
            video_count,
            audio_count,
            total_duration: None, // Would need media parsing
        }
    }
    
    fn is_data_science(ext_counts: &HashMap<String, usize>) -> bool {
        ext_counts.contains_key("ipynb") || 
        (ext_counts.contains_key("csv") && ext_counts["csv"] > 5) ||
        (ext_counts.contains_key("parquet") || ext_counts.contains_key("feather"))
    }
    
    fn analyze_data_science(ext_counts: &HashMap<String, usize>) -> DirectoryType {
        let notebooks = ext_counts.get("ipynb").copied().unwrap_or(0);
        let datasets = ext_counts.get("csv").copied().unwrap_or(0) +
                      ext_counts.get("parquet").copied().unwrap_or(0) +
                      ext_counts.get("feather").copied().unwrap_or(0);
        
        let mut languages = vec![];
        if ext_counts.contains_key("py") {
            languages.push("Python".to_string());
        }
        if ext_counts.contains_key("r") {
            languages.push("R".to_string());
        }
        if ext_counts.contains_key("jl") {
            languages.push("Julia".to_string());
        }
        
        DirectoryType::DataScience {
            notebooks,
            datasets,
            languages,
        }
    }
    
    fn get_dominant_type(ext_counts: &HashMap<String, usize>) -> Option<String> {
        ext_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ext, _)| ext.clone())
    }
}