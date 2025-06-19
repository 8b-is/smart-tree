//! Project context detection module

use std::fs;
use std::path::Path;
use serde_json::Value;

/// Attempts to detect the project type and description
pub fn detect_project_context(root_path: &Path) -> Option<String> {
    // Try various project files in order of preference
    
    // Rust projects
    if let Some(context) = read_cargo_toml(root_path) {
        return Some(context);
    }
    
    // Node.js projects
    if let Some(context) = read_package_json(root_path) {
        return Some(context);
    }
    
    // Python projects
    if let Some(context) = read_pyproject_toml(root_path) {
        return Some(context);
    }
    
    // Go projects
    if let Some(context) = read_go_mod(root_path) {
        return Some(context);
    }
    
    // Git repositories
    if let Some(context) = read_git_description(root_path) {
        return Some(context);
    }
    
    // README files
    if let Some(context) = read_readme(root_path) {
        return Some(context);
    }
    
    None
}

fn read_cargo_toml(root_path: &Path) -> Option<String> {
    let cargo_path = root_path.join("Cargo.toml");
    if !cargo_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&cargo_path).ok()?;
    let toml: toml::Value = toml::from_str(&content).ok()?;
    
    let package = toml.get("package")?;
    let name = package.get("name")?.as_str()?;
    let desc = package.get("description")?.as_str()?;
    
    Some(format!("Rust: {} - {}", name, truncate_string(desc, 80)))
}

fn read_package_json(root_path: &Path) -> Option<String> {
    let package_path = root_path.join("package.json");
    if !package_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&package_path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;
    
    let name = json.get("name")?.as_str()?;
    let desc = json.get("description")?.as_str().unwrap_or("No description");
    
    Some(format!("Node: {} - {}", name, truncate_string(desc, 80)))
}

fn read_pyproject_toml(root_path: &Path) -> Option<String> {
    let pyproject_path = root_path.join("pyproject.toml");
    if !pyproject_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&pyproject_path).ok()?;
    let toml: toml::Value = toml::from_str(&content).ok()?;
    
    // Try both [project] and [tool.poetry] sections
    if let Some(project) = toml.get("project") {
        let name = project.get("name")?.as_str()?;
        let desc = project.get("description")?.as_str().unwrap_or("No description");
        return Some(format!("Python: {} - {}", name, truncate_string(desc, 80)));
    } else if let Some(tool) = toml.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            let name = poetry.get("name")?.as_str()?;
            let desc = poetry.get("description")?.as_str().unwrap_or("No description");
            return Some(format!("Python: {} - {}", name, truncate_string(desc, 80)));
        }
    }
    
    None
}

fn read_go_mod(root_path: &Path) -> Option<String> {
    let go_mod_path = root_path.join("go.mod");
    if !go_mod_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&go_mod_path).ok()?;
    let first_line = content.lines().next()?;
    
    if first_line.starts_with("module ") {
        let module_name = first_line.strip_prefix("module ")?.trim();
        return Some(format!("Go: {}", module_name));
    }
    
    None
}

fn read_git_description(root_path: &Path) -> Option<String> {
    let git_desc_path = root_path.join(".git/description");
    if !git_desc_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&git_desc_path).ok()?;
    let desc = content.trim();
    
    // Skip the default git description
    if desc.contains("Unnamed repository") {
        return None;
    }
    
    Some(format!("Git: {}", truncate_string(desc, 80)))
}

fn read_readme(root_path: &Path) -> Option<String> {
    // Try various README filenames
    let readme_names = ["README.md", "README.MD", "readme.md", "README", "README.txt"];
    
    for name in &readme_names {
        let readme_path = root_path.join(name);
        if readme_path.exists() {
            let content = fs::read_to_string(&readme_path).ok()?;
            
            // Extract first non-empty line after any headers
            for line in content.lines() {
                let trimmed = line.trim();
                // Skip empty lines and markdown headers
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    return Some(truncate_string(trimmed, 100));
                }
            }
        }
    }
    
    None
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}