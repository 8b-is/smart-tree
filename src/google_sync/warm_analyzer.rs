//! Warm Storage Analyzer - Smart email triage and archive suggestions
//!
//! Scores emails 0.0 (definitely keep) to 1.0 (safe to archive) based on:
//! - Newsletter detection (List-Unsubscribe, noreply@ senders)
//! - Automated notifications (alert@, notification@, digest@)
//! - Bulk senders (same From > threshold times)
//! - Old threads with no recent activity
//! - Large attachments
//! - Read but never replied (and old enough)
//!
//! "Don't worry about the noise. I'll help you find what matters." - Liquid

use std::collections::HashMap;

use super::models::{ArchiveCategory, ArchiveReason, EmailMetadata, WarmStorageScore};

/// Configurable thresholds for warm storage scoring
pub struct WarmConfig {
    pub newsletter_score: f64,
    pub automated_score: f64,
    pub old_thread_days: u32,
    pub old_thread_score: f64,
    pub large_attachment_bytes: u64,
    pub large_attachment_score: f64,
    pub read_not_replied_days: u32,
    pub read_not_replied_score: f64,
    pub bulk_sender_threshold: u32,
    pub bulk_sender_score: f64,
    pub no_activity_days: u32,
    pub no_activity_score: f64,
    pub min_score_to_suggest: f64,
}

impl Default for WarmConfig {
    fn default() -> Self {
        Self {
            newsletter_score: 0.8,
            automated_score: 0.7,
            old_thread_days: 180,
            old_thread_score: 0.6,
            large_attachment_bytes: 10 * 1024 * 1024, // 10 MB
            large_attachment_score: 0.5,
            read_not_replied_days: 90,
            read_not_replied_score: 0.4,
            bulk_sender_threshold: 50,
            bulk_sender_score: 0.6,
            no_activity_days: 180,
            no_activity_score: 0.5,
            min_score_to_suggest: 0.5,
        }
    }
}

/// Analyze emails and produce archive suggestions
pub struct WarmAnalyzer {
    config: WarmConfig,
}

impl WarmAnalyzer {
    pub fn new(config: WarmConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(WarmConfig::default())
    }

    /// Analyze a batch of emails and return scored suggestions
    pub fn analyze(&self, emails: &[EmailMetadata]) -> Vec<WarmStorageScore> {
        // Pre-compute sender frequency for bulk detection
        let sender_counts = self.count_senders(emails);

        let mut scores: Vec<WarmStorageScore> = emails
            .iter()
            .filter_map(|email| {
                let score = self.score_email(email, &sender_counts);
                if score.total_score >= self.config.min_score_to_suggest {
                    Some(score)
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending (most archivable first)
        scores.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scores
    }

    /// Score a single email across all heuristics
    fn score_email(
        &self,
        email: &EmailMetadata,
        sender_counts: &HashMap<String, u32>,
    ) -> WarmStorageScore {
        let mut reasons = Vec::new();
        let mut max_score = 0.0_f64;

        // Newsletter detection
        if let Some(reason) = self.detect_newsletter(email) {
            max_score = max_score.max(self.config.newsletter_score);
            reasons.push(reason);
        }

        // Automated notification detection
        if let Some(reason) = self.detect_automated(email) {
            max_score = max_score.max(self.config.automated_score);
            reasons.push(reason);
        }

        // Bulk sender detection
        if let Some(reason) = self.detect_bulk_sender(email, sender_counts) {
            max_score = max_score.max(self.config.bulk_sender_score);
            reasons.push(reason);
        }

        // Old thread detection
        if let Some(reason) = self.detect_old_thread(email) {
            max_score = max_score.max(self.config.old_thread_score);
            reasons.push(reason);
        }

        // Large attachment detection
        if let Some(reason) = self.detect_large_attachment(email) {
            max_score = max_score.max(self.config.large_attachment_score);
            reasons.push(reason);
        }

        // Read but not replied detection
        if let Some(reason) = self.detect_read_not_replied(email) {
            max_score = max_score.max(self.config.read_not_replied_score);
            reasons.push(reason);
        }

        // Determine category from highest-scoring reason
        let category = self.categorize(&reasons);

        WarmStorageScore {
            message_id: email.message_id.clone(),
            subject: email.subject.clone(),
            from: email.from.clone(),
            date: email.date,
            total_score: max_score,
            reasons,
            category,
        }
    }

    fn detect_newsletter(&self, email: &EmailMetadata) -> Option<ArchiveReason> {
        let from_lower = email.from.to_lowercase();

        // Check for newsletter patterns
        let is_newsletter = from_lower.contains("noreply")
            || from_lower.contains("no-reply")
            || from_lower.contains("newsletter")
            || from_lower.contains("digest")
            || from_lower.contains("updates@")
            || from_lower.contains("news@")
            || email
                .labels
                .iter()
                .any(|l| l.to_lowercase().contains("newsletter"));

        if is_newsletter {
            Some(ArchiveReason::Newsletter {
                sender: email.from.clone(),
            })
        } else {
            None
        }
    }

    fn detect_automated(&self, email: &EmailMetadata) -> Option<ArchiveReason> {
        let from_lower = email.from.to_lowercase();

        let patterns = [
            "notification@",
            "alert@",
            "alerts@",
            "mailer-daemon",
            "postmaster@",
            "system@",
            "automated@",
            "do-not-reply",
            "donotreply",
            "notify@",
            "noreply@",
        ];

        for pattern in &patterns {
            if from_lower.contains(pattern) {
                return Some(ArchiveReason::AutomatedNotification {
                    pattern: pattern.to_string(),
                });
            }
        }

        None
    }

    fn detect_bulk_sender(
        &self,
        email: &EmailMetadata,
        sender_counts: &HashMap<String, u32>,
    ) -> Option<ArchiveReason> {
        let sender = Self::normalize_sender(&email.from);
        let count = sender_counts.get(&sender).copied().unwrap_or(0);

        if count >= self.config.bulk_sender_threshold {
            Some(ArchiveReason::BulkSender { count })
        } else {
            None
        }
    }

    fn detect_old_thread(&self, email: &EmailMetadata) -> Option<ArchiveReason> {
        let age = chrono::Utc::now().signed_duration_since(email.date);
        let age_days = age.num_days().max(0) as u32;

        if age_days >= self.config.old_thread_days {
            Some(ArchiveReason::OldThread { age_days })
        } else {
            None
        }
    }

    fn detect_large_attachment(&self, email: &EmailMetadata) -> Option<ArchiveReason> {
        if email.has_attachments && email.size_bytes >= self.config.large_attachment_bytes {
            Some(ArchiveReason::LargeAttachment {
                size_bytes: email.size_bytes,
            })
        } else {
            None
        }
    }

    fn detect_read_not_replied(&self, email: &EmailMetadata) -> Option<ArchiveReason> {
        if !email.is_read || email.is_replied {
            return None;
        }

        let age = chrono::Utc::now().signed_duration_since(email.date);
        let age_days = age.num_days().max(0) as u32;

        if age_days >= self.config.read_not_replied_days {
            Some(ArchiveReason::ReadNotReplied { age_days })
        } else {
            None
        }
    }

    fn categorize(&self, reasons: &[ArchiveReason]) -> ArchiveCategory {
        // Pick category from highest-priority reason
        if let Some(reason) = reasons.first() {
            match reason {
                ArchiveReason::Newsletter { .. } => ArchiveCategory::Newsletter,
                ArchiveReason::AutomatedNotification { .. } => ArchiveCategory::Automated,
                ArchiveReason::BulkSender { .. } => ArchiveCategory::Bulk,
                ArchiveReason::OldThread { .. } | ArchiveReason::NoRecentActivity { .. } => {
                    ArchiveCategory::Stale
                }
                ArchiveReason::LargeAttachment { .. } => ArchiveCategory::LargeAttachment,
                ArchiveReason::ReadNotReplied { .. } => ArchiveCategory::ReadNoAction,
            }
        } else {
            ArchiveCategory::Stale // fallback
        }
    }

    fn count_senders(&self, emails: &[EmailMetadata]) -> HashMap<String, u32> {
        let mut counts = HashMap::new();
        for email in emails {
            let sender = Self::normalize_sender(&email.from);
            *counts.entry(sender).or_insert(0) += 1;
        }
        counts
    }

    /// Normalize sender: extract email address and lowercase
    fn normalize_sender(from: &str) -> String {
        // Extract email from "Name <email@domain.com>" format
        if let Some(start) = from.find('<') {
            if let Some(end) = from.find('>') {
                return from[start + 1..end].to_lowercase();
            }
        }
        from.to_lowercase()
    }

    /// Generate a human-readable summary of suggestions
    pub fn summarize(scores: &[WarmStorageScore]) -> String {
        if scores.is_empty() {
            return "No emails suggested for archiving. Your inbox looks clean!".to_string();
        }

        let mut summary = format!("Found {} emails to consider archiving:\n\n", scores.len());

        // Group by category
        let mut by_category: HashMap<String, Vec<&WarmStorageScore>> = HashMap::new();
        for score in scores {
            let cat = format!("{:?}", score.category);
            by_category.entry(cat).or_default().push(score);
        }

        for (category, items) in &by_category {
            summary.push_str(&format!("## {} ({} emails)\n", category, items.len()));
            for item in items.iter().take(5) {
                summary.push_str(&format!(
                    "  - [{:.0}%] {} (from: {})\n",
                    item.total_score * 100.0,
                    item.subject,
                    item.from
                ));
            }
            if items.len() > 5 {
                summary.push_str(&format!("  ... and {} more\n", items.len() - 5));
            }
            summary.push('\n');
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_email(from: &str, subject: &str, age_days: i64) -> EmailMetadata {
        EmailMetadata {
            message_id: "test".to_string(),
            thread_id: "thread".to_string(),
            subject: subject.to_string(),
            from: from.to_string(),
            to: vec!["me@example.com".to_string()],
            date: Utc::now() - chrono::Duration::days(age_days),
            labels: vec![],
            size_bytes: 1000,
            has_attachments: false,
            snippet: String::new(),
            is_read: true,
            is_replied: false,
        }
    }

    #[test]
    fn test_newsletter_detection() {
        let analyzer = WarmAnalyzer::with_defaults();
        let email = make_email("noreply@company.com", "Weekly Update", 30);
        let scores = analyzer.analyze(&[email]);
        assert_eq!(scores.len(), 1);
        assert!(scores[0].total_score >= 0.7);
    }

    #[test]
    fn test_old_thread_detection() {
        let analyzer = WarmAnalyzer::with_defaults();
        let email = make_email("friend@example.com", "Old conversation", 200);
        let scores = analyzer.analyze(&[email]);
        assert_eq!(scores.len(), 1);
    }

    #[test]
    fn test_recent_email_not_flagged() {
        let analyzer = WarmAnalyzer::with_defaults();
        let email = make_email("friend@example.com", "Hey!", 5);
        let scores = analyzer.analyze(&[email]);
        assert!(scores.is_empty());
    }
}
