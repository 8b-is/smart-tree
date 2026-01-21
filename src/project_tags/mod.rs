use std::fs;
use std::path::Path;

pub fn add(project_path: &str, tag: &str) {
    let tags_file = Path::new(project_path).join(".project_tags");
    let mut tags = if tags_file.exists() {
        fs::read_to_string(&tags_file).unwrap_or_default()
    } else {
        String::new()
    };

    if !tags.split_whitespace().any(|t| t == tag) {
        tags.push_str(tag);
        tags.push(' ');
        fs::write(tags_file, tags).expect("Failed to write to .project_tags");
    }
}

pub fn remove(project_path: &str, tag: &str) {
    let tags_file = Path::new(project_path).join(".project_tags");
    if tags_file.exists() {
        let tags = fs::read_to_string(&tags_file).unwrap_or_default();
        let new_tags: Vec<&str> = tags.split_whitespace().filter(|t| *t != tag).collect();
        fs::write(tags_file, new_tags.join(" ")).expect("Failed to write to .project_tags");
    }
}
