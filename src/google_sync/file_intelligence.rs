//! File Intelligence - Smart file organization suggestions
//!
//! Detects misplaced files and suggests where they should go.
//! Integrates with Liquid-rust for local AI personality.
//!
//! "Hey! Found your Q4 report hanging out in Audio Downloads.
//!  Want me to file it where it belongs?
//!  Don't worry, your data is safe with me!" - Liquid

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::models::{EmailMetadata, EmailTriage, FileSuggestion};

/// File type to expected directory mapping
fn file_type_rules() -> HashMap<&'static str, &'static [&'static str]> {
    let mut rules = HashMap::new();
    rules.insert("pdf", &["Documents", "Papers", "Reports"][..]);
    rules.insert("doc", &["Documents"][..]);
    rules.insert("docx", &["Documents"][..]);
    rules.insert("xls", &["Documents", "Spreadsheets"][..]);
    rules.insert("xlsx", &["Documents", "Spreadsheets"][..]);
    rules.insert("ppt", &["Documents", "Presentations"][..]);
    rules.insert("pptx", &["Documents", "Presentations"][..]);
    rules.insert("jpg", &["Pictures", "Photos", "Images"][..]);
    rules.insert("jpeg", &["Pictures", "Photos", "Images"][..]);
    rules.insert("png", &["Pictures", "Photos", "Images", "Screenshots"][..]);
    rules.insert("gif", &["Pictures", "Images"][..]);
    rules.insert("svg", &["Pictures", "Images", "Design"][..]);
    rules.insert("mp3", &["Music", "Audio"][..]);
    rules.insert("flac", &["Music", "Audio"][..]);
    rules.insert("wav", &["Music", "Audio"][..]);
    rules.insert("ogg", &["Music", "Audio"][..]);
    rules.insert("mp4", &["Videos"][..]);
    rules.insert("mkv", &["Videos"][..]);
    rules.insert("avi", &["Videos"][..]);
    rules.insert("mov", &["Videos"][..]);
    rules.insert("zip", &["Downloads", "Archives"][..]);
    rules.insert("tar", &["Downloads", "Archives"][..]);
    rules.insert("gz", &["Downloads", "Archives"][..]);
    rules.insert("rs", &["Development", "Code", "Projects"][..]);
    rules.insert("py", &["Development", "Code", "Projects"][..]);
    rules.insert("js", &["Development", "Code", "Projects"][..]);
    rules.insert("ts", &["Development", "Code", "Projects"][..]);
    rules
}

/// Directories that commonly accumulate misplaced files
const CLUTTER_DIRS: &[&str] = &[
    "Downloads",
    "Desktop",
    "Audio Downloads",
    "tmp",
    "Temp",
];

/// Liquid personality messages for file suggestions
const PERSONALITY_MESSAGES: &[&str] = &[
    "Hey! Found this hanging out in the wrong neighborhood. Want me to move it home?",
    "This looks like it took a wrong turn. I know just where it belongs!",
    "Don't worry, I've got this. Your files are safe with me!",
    "Looks like someone's been downloading to the wrong folder again... No judgment!",
    "I found a stray file! Let me help it find its way home.",
    "Your data is safe with me. I live here with you. Let me tidy up!",
];

/// Analyze a directory for misplaced files
pub fn suggest_filing(scan_path: &Path) -> Vec<FileSuggestion> {
    let mut suggestions = Vec::new();
    let rules = file_type_rules();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));

    // Walk the directory
    if let Ok(entries) = std::fs::read_dir(scan_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check if this file type has expected directories
            if let Some(expected_dirs) = rules.get(ext.as_str()) {
                // Check if current directory is NOT one of the expected ones
                let current_dir = scan_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                let in_clutter = CLUTTER_DIRS
                    .iter()
                    .any(|d| current_dir.eq_ignore_ascii_case(d));

                let in_expected = expected_dirs
                    .iter()
                    .any(|d| current_dir.eq_ignore_ascii_case(d));

                if in_clutter && !in_expected {
                    // Suggest moving to the best matching directory
                    let suggested_dir = find_best_directory(&home, expected_dirs, file_name);
                    let suggested_path = suggested_dir.join(file_name);

                    let confidence = compute_confidence(&ext, file_name, current_dir);
                    let reason = format!(
                        "{} file '{}' found in {}/, likely belongs in {}/",
                        ext.to_uppercase(),
                        file_name,
                        current_dir,
                        suggested_path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                    );

                    let msg_idx = suggestions.len() % PERSONALITY_MESSAGES.len();

                    suggestions.push(FileSuggestion {
                        file_path: path.to_string_lossy().to_string(),
                        suggested_path: suggested_path.to_string_lossy().to_string(),
                        reason,
                        confidence,
                        personality_message: PERSONALITY_MESSAGES[msg_idx].to_string(),
                    });
                }
            }
        }
    }

    // Sort by confidence descending
    suggestions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    suggestions
}

/// Find the best existing directory from the expected list
fn find_best_directory(home: &Path, expected: &[&str], file_name: &str) -> PathBuf {
    // Try to find an existing matching directory under home
    for dir_name in expected {
        let candidate = home.join(dir_name);
        if candidate.is_dir() {
            // Check for subdirectories that might be more specific
            if let Some(subdir) = find_specific_subdir(&candidate, file_name) {
                return subdir;
            }
            return candidate;
        }
    }

    // Fallback: use first expected directory under home
    home.join(expected.first().unwrap_or(&"Documents"))
}

/// Try to find a more specific subdirectory based on filename hints
fn find_specific_subdir(parent: &Path, file_name: &str) -> Option<PathBuf> {
    let name_lower = file_name.to_lowercase();

    // Common subdirectory patterns
    let hints: &[(&str, &[&str])] = &[
        ("report", &["Reports", "Work"]),
        ("invoice", &["Finance", "Invoices", "Work"]),
        ("receipt", &["Finance", "Receipts"]),
        ("resume", &["Career", "Work"]),
        ("contract", &["Legal", "Contracts", "Work"]),
        ("screenshot", &["Screenshots"]),
        ("wallpaper", &["Wallpapers"]),
        ("backup", &["Backups"]),
    ];

    for (hint, subdirs) in hints {
        if name_lower.contains(hint) {
            for subdir in *subdirs {
                let candidate = parent.join(subdir);
                if candidate.is_dir() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Compute confidence score for a filing suggestion
fn compute_confidence(ext: &str, file_name: &str, current_dir: &str) -> f64 {
    let mut score: f64 = 0.5; // Base confidence

    // Higher confidence for clearly misplaced types
    match ext {
        "pdf" | "doc" | "docx" | "xls" | "xlsx" => score += 0.2,
        "jpg" | "png" | "gif" => score += 0.15,
        "mp3" | "flac" | "wav" => score += 0.2,
        "mp4" | "mkv" | "mov" => score += 0.15,
        _ => {}
    }

    // Higher confidence if in Downloads/Desktop (common dumping grounds)
    let dir_lower = current_dir.to_lowercase();
    if dir_lower == "downloads" || dir_lower == "desktop" {
        score += 0.1;
    }

    // Higher confidence if filename has organizational hints
    let name_lower = file_name.to_lowercase();
    if name_lower.contains("report")
        || name_lower.contains("invoice")
        || name_lower.contains("receipt")
    {
        score += 0.1;
    }

    score.min(0.95) // Never fully certain
}

/// Triage emails by importance (works with warm analyzer results)
pub fn triage_emails(emails: &[EmailMetadata]) -> Vec<EmailTriage> {
    let mut triages = Vec::new();

    for email in emails {
        let (importance, reason, action) = score_email_importance(email);

        if importance >= 0.5 {
            let personality = if importance >= 0.8 {
                format!(
                    "Hey! This email from {} looks important. You should take a look!",
                    email.from
                )
            } else if importance >= 0.6 {
                format!(
                    "You might want to check this one from {}. Just saying!",
                    email.from
                )
            } else {
                "Found something that might need your attention.".to_string()
            };

            triages.push(EmailTriage {
                message_id: email.message_id.clone(),
                subject: email.subject.clone(),
                from: email.from.clone(),
                importance_score: importance,
                reason,
                action_suggestion: action,
                personality_message: personality,
            });
        }
    }

    triages.sort_by(|a, b| {
        b.importance_score
            .partial_cmp(&a.importance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    triages
}

/// Score email importance based on content signals
fn score_email_importance(email: &EmailMetadata) -> (f64, String, String) {
    let mut score: f64 = 0.3; // Base
    let mut reasons = Vec::new();

    // Unread emails are more important
    if !email.is_read {
        score += 0.2;
        reasons.push("unread");
    }

    // Recent emails are more important
    let age_days = (chrono::Utc::now() - email.date).num_days();
    if age_days <= 1 {
        score += 0.2;
        reasons.push("today");
    } else if age_days <= 3 {
        score += 0.1;
        reasons.push("recent");
    }

    // Emails that seem to need a reply
    let subject_lower = email.subject.to_lowercase();
    if subject_lower.contains("urgent")
        || subject_lower.contains("asap")
        || subject_lower.contains("action required")
        || subject_lower.contains("please respond")
    {
        score += 0.3;
        reasons.push("urgent keywords");
    }

    // Direct emails (not bulk/automated) are more important
    let from_lower = email.from.to_lowercase();
    let is_automated = from_lower.contains("noreply")
        || from_lower.contains("notification")
        || from_lower.contains("digest");
    if !is_automated {
        score += 0.1;
        reasons.push("personal sender");
    }

    let reason = reasons.join(", ");
    let action = if score >= 0.8 {
        "Reply ASAP".to_string()
    } else if score >= 0.6 {
        "Review when you get a chance".to_string()
    } else {
        "Low priority - check later".to_string()
    };

    (score.min(1.0), reason, action)
}
