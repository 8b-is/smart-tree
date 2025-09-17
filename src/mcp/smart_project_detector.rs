// Smart Project Detector - Context-aware project name detection
// "Bob is a name, but 'Project Bob' is a project!" - Aye

use regex::Regex;
use serde_json::Value;

/// Detect if content contains meaningful project references
pub fn contains_project_reference(content: &str, project_name: &str) -> bool {
    // Short names like "bob" need more context
    let needs_context = project_name.len() <= 3 ||
                       project_name.chars().all(char::is_alphanumeric);

    if needs_context {
        // For short/common names, require contextual markers
        contains_contextual_reference(content, project_name)
    } else {
        // For longer/unique names, simple contains is OK
        // But still check for word boundaries
        contains_word_boundary_match(content, project_name)
    }
}

/// Check for contextual references (for short/common project names)
fn contains_contextual_reference(content: &str, project_name: &str) -> bool {
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    // Patterns that indicate this is a PROJECT reference, not just the word
    let context_patterns = [
        // Markdown headers
        format!("# {}", lower_project),
        format!("## {}", lower_project),
        format!("# {} project", lower_project),
        format!("# project {}", lower_project),

        // Code/config contexts
        format!("\"project\": \"{}\"", lower_project),
        format!("'project': '{}'", lower_project),
        format!("project_name: {}", lower_project),
        format!("name: \"{}\"", lower_project),

        // Documentation patterns
        format!("{} documentation", lower_project),
        format!("{} readme", lower_project),
        format!("{} notes", lower_project),
        format!("{} todo", lower_project),

        // Development contexts
        format!("{} repository", lower_project),
        format!("{} codebase", lower_project),
        format!("{} development", lower_project),
        format!("{} implementation", lower_project),

        // File paths (strong indicator)
        format!("/{}/", lower_project),
        format!("/{}/src", lower_project),
        format!("~/{}/", lower_project),
        format!("documents/{}", lower_project),
        format!("projects/{}", lower_project),

        // Git contexts
        format!("git clone {}", lower_project),
        format!("cd {}", lower_project),
        format!("working on {}", lower_project),

        // AI assistant contexts
        format!("help with {}", lower_project),
        format!("question about {}", lower_project),
        format!("{} issue", lower_project),
        format!("{} bug", lower_project),
        format!("{} feature", lower_project),
    ];

    // Check if any context pattern matches
    context_patterns.iter().any(|pattern| lower_content.contains(pattern))
}

/// Check for word boundary matches (for longer project names)
fn contains_word_boundary_match(content: &str, project_name: &str) -> bool {
    // Build regex for word boundary matching
    let pattern = format!(r"\b{}\b", regex::escape(project_name));

    if let Ok(re) = Regex::new(&pattern) {
        re.is_match(content)
    } else {
        // Fallback to simple contains if regex fails
        content.contains(project_name)
    }
}

/// Extract project references from JSON content
pub fn extract_json_project_references(json: &Value, project_name: &str) -> Vec<String> {
    let mut references = Vec::new();

    // Check specific JSON fields that often contain project info
    let project_fields = [
        "project", "projectName", "project_name",
        "name", "title", "repository", "repo",
        "package", "module", "app", "application"
    ];

    for field in &project_fields {
        if let Some(value) = json.get(field) {
            if let Some(str_val) = value.as_str() {
                if str_val.to_lowercase().contains(&project_name.to_lowercase()) {
                    references.push(format!("{}: {}", field, str_val));
                }
            }
        }
    }

    // Check in messages/content fields for AI conversations
    let content_fields = ["message", "content", "text", "body", "prompt", "response"];

    for field in &content_fields {
        if let Some(value) = json.get(field) {
            if let Some(str_val) = value.as_str() {
                if contains_project_reference(str_val, project_name) {
                    // Extract a snippet around the reference
                    let snippet = extract_context_snippet(str_val, project_name, 100);
                    references.push(snippet);
                }
            }
        }
    }

    // Recursively check arrays and objects
    match json {
        Value::Array(arr) => {
            for item in arr {
                references.extend(extract_json_project_references(item, project_name));
            }
        }
        Value::Object(map) => {
            for (_key, value) in map {
                if value.is_object() || value.is_array() {
                    references.extend(extract_json_project_references(value, project_name));
                }
            }
        }
        _ => {}
    }

    references
}

/// Extract a context snippet around a project mention
fn extract_context_snippet(content: &str, project_name: &str, context_chars: usize) -> String {
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_project) {
        let start = pos.saturating_sub(context_chars / 2);
        let end = (pos + lower_project.len() + context_chars / 2).min(content.len());

        let snippet = &content[start..end];

        // Add ellipsis if truncated
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };

        format!("{}{}{}", prefix, snippet, suffix)
    } else {
        content.chars().take(context_chars * 2).collect()
    }
}

/// Score the relevance of a project reference
pub fn score_reference_relevance(content: &str, project_name: &str) -> f64 {
    let mut score = 0.0;
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    // Count exact matches
    let exact_matches = lower_content.matches(&lower_project).count();
    score += exact_matches as f64 * 1.0;

    // Bonus for being in a header
    if content.starts_with('#') && lower_content.contains(&lower_project) {
        score += 5.0;
    }

    // Bonus for being in quotes (likely a config value)
    if content.contains(&format!("\"{}\"", project_name)) ||
       content.contains(&format!("'{}'", project_name)) {
        score += 3.0;
    }

    // Bonus for path-like references
    if content.contains(&format!("/{}/", project_name)) {
        score += 4.0;
    }

    // Bonus for development-related keywords nearby
    let dev_keywords = ["project", "develop", "implement", "feature", "bug", "issue",
                       "repository", "code", "build", "test", "deploy"];

    for keyword in &dev_keywords {
        if lower_content.contains(keyword) && lower_content.contains(&lower_project) {
            score += 0.5;
        }
    }

    // Normalize score to 0-100
    (score * 10.0).min(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_name_detection() {
        // "bob" alone shouldn't match without context
        assert!(!contains_project_reference("I talked to bob yesterday", "bob"));

        // But with context it should
        assert!(contains_project_reference("# Bob Project Notes", "bob"));
        assert!(contains_project_reference("Working on bob development", "bob"));
        assert!(contains_project_reference("\"project\": \"bob\"", "bob"));
    }

    #[test]
    fn test_longer_name_detection() {
        // Longer names can match with word boundaries
        assert!(contains_project_reference("The smart-tree project is great", "smart-tree"));
        assert!(contains_project_reference("smart-tree development", "smart-tree"));

        // But not partial matches
        assert!(!contains_word_boundary_match("smart-trees", "smart-tree"));
    }

    #[test]
    fn test_json_extraction() {
        let json = serde_json::json!({
            "project": "bob",
            "message": "Working on bob feature implementation",
            "unrelated": "alice and bob went to the store"
        });

        let refs = extract_json_project_references(&json, "bob");
        assert_eq!(refs.len(), 2); // Should find project field and message field
    }
}