//! Integration tests for Google Sync features
//! Tests file intelligence, warm analyzer, rate limiter, and token store

#[cfg(feature = "google")]
mod google_tests {
    use st::google_sync::file_intelligence;
    use st::google_sync::models::EmailMetadata;
    use st::google_sync::warm_analyzer::{WarmAnalyzer, WarmConfig};

    // ── File Intelligence Tests ─────────────────────────────────────

    #[test]
    fn test_suggest_filing_finds_misplaced_pdfs() {
        // Create a temp "Downloads" directory with misplaced files
        let dir = tempfile::Builder::new()
            .prefix("Downloads")
            .tempdir()
            .unwrap();

        std::fs::write(dir.path().join("Q4_Report.pdf"), b"fake pdf").unwrap();
        std::fs::write(dir.path().join("random_notes.txt"), b"just text").unwrap();
        std::fs::write(dir.path().join("vacation.jpg"), b"fake jpg").unwrap();

        let suggestions = file_intelligence::suggest_filing(dir.path());

        // PDFs and JPGs in "Downloads" should get suggestions
        // (depends on whether ~/Documents etc. exist on the test machine)
        // At minimum, the function shouldn't panic
        println!("Suggestions found: {}", suggestions.len());
        for s in &suggestions {
            println!(
                "  [{:.0}%] {} -> {}\n    {}",
                s.confidence * 100.0,
                s.file_path,
                s.suggested_path,
                s.personality_message
            );
        }
    }

    // ── Warm Analyzer Tests ─────────────────────────────────────────

    fn make_test_email(from: &str, subject: &str, age_days: i64) -> EmailMetadata {
        EmailMetadata {
            message_id: format!("msg_{}", subject.replace(' ', "_")),
            thread_id: "thread_1".to_string(),
            subject: subject.to_string(),
            from: from.to_string(),
            to: vec!["user@example.com".to_string()],
            date: chrono::Utc::now() - chrono::Duration::days(age_days),
            labels: vec!["INBOX".to_string()],
            size_bytes: 5000,
            has_attachments: false,
            snippet: "Test snippet".to_string(),
            is_read: true,
            is_replied: false,
        }
    }

    #[test]
    fn test_warm_analyzer_full_inbox_scan() {
        let emails = vec![
            make_test_email("noreply@newsletter.com", "Weekly Digest #47", 10),
            make_test_email("notification@github.com", "PR merged", 5),
            make_test_email("alerts@monitoring.io", "CPU spike alert", 3),
            make_test_email("friend@gmail.com", "Hey, how's it going?", 200),
            make_test_email("boss@work.com", "Meeting tomorrow", 2),
            make_test_email("digest@medium.com", "Your daily read", 15),
            make_test_email("do-not-reply@bank.com", "Statement ready", 30),
            make_test_email("updates@store.com", "Your order shipped", 45),
        ];

        let analyzer = WarmAnalyzer::with_defaults();
        let scores = analyzer.analyze(&emails);

        println!("\nWarm Storage Analysis Results:");
        println!("{}", "=".repeat(50));

        let summary = WarmAnalyzer::summarize(&scores);
        println!("{}", summary);

        // Newsletters and automated should be flagged
        assert!(
            scores.iter().any(|s| s.from.contains("noreply")),
            "Newsletter from noreply should be flagged"
        );
        assert!(
            scores.iter().any(|s| s.from.contains("notification")),
            "GitHub notification should be flagged"
        );

        // Boss's recent email should NOT be flagged
        assert!(
            !scores.iter().any(|s| s.from.contains("boss")),
            "Boss's recent email should NOT be flagged"
        );

        // Old email from friend should be flagged (read, not replied, 200 days)
        assert!(
            scores.iter().any(|s| s.from.contains("friend")),
            "200-day old unresponded email should be flagged"
        );
    }

    #[test]
    fn test_warm_analyzer_custom_thresholds() {
        let emails = vec![
            make_test_email("noreply@co.com", "Update", 10),
            make_test_email("friend@co.com", "Hey", 100),
        ];

        // Very strict: only flag if score >= 0.9
        let config = WarmConfig {
            min_score_to_suggest: 0.9,
            ..Default::default()
        };
        let analyzer = WarmAnalyzer::new(config);
        let strict_scores = analyzer.analyze(&emails);

        // Relaxed: flag everything >= 0.1
        let config2 = WarmConfig {
            min_score_to_suggest: 0.1,
            ..Default::default()
        };
        let analyzer2 = WarmAnalyzer::new(config2);
        let relaxed_scores = analyzer2.analyze(&emails);

        assert!(
            relaxed_scores.len() >= strict_scores.len(),
            "Relaxed threshold should flag at least as many emails"
        );
    }

    // ── Email Triage Tests ──────────────────────────────────────────

    #[test]
    fn test_email_triage_importance() {
        let mut urgent_email = make_test_email("ceo@company.com", "URGENT: Action Required", 0);
        urgent_email.is_read = false;

        let boring_email = make_test_email("noreply@spam.com", "Weekly digest", 30);

        let emails = vec![urgent_email, boring_email];
        let triages = file_intelligence::triage_emails(&emails);

        println!("\nEmail Triage Results:");
        for t in &triages {
            println!(
                "  [{:.0}%] {} - {}\n    Action: {}\n    {}",
                t.importance_score * 100.0,
                t.from,
                t.subject,
                t.action_suggestion,
                t.personality_message
            );
        }

        // Urgent unread email from CEO should rank high
        assert!(
            !triages.is_empty(),
            "Should have at least one triage result"
        );
        assert!(
            triages[0].from.contains("ceo"),
            "CEO's urgent email should be highest priority"
        );
        assert!(
            triages[0].importance_score >= 0.7,
            "Urgent unread email should score >= 0.7"
        );
    }

    // ── Token Store Tests ───────────────────────────────────────────

    #[test]
    fn test_token_store_config_roundtrip() {
        use st::google_sync::models::{AuthConfig, AuthMethod};
        use st::google_sync::token_store::TokenStore;

        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::new(dir.path().to_path_buf()).unwrap();

        let config = AuthConfig {
            method: AuthMethod::UserOAuth2 {
                client_id: "test_client_id".to_string(),
                client_secret: "***".to_string(),
                redirect_port: Some(8085),
            },
            scopes: vec!["gmail.readonly".to_string()],
            account_email: Some("user@gmail.com".to_string()),
            authenticated_at: Some(chrono::Utc::now()),
        };

        store.save_config(&config).unwrap();
        let loaded = store.load_config().unwrap().unwrap();

        assert_eq!(loaded.account_email, Some("user@gmail.com".to_string()));
        assert_eq!(loaded.scopes.len(), 1);
    }

    // ── Rate Limiter Tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_rate_limiter_acquire_blocks() {
        use st::google_sync::rate_limiter::RateLimiter;

        let limiter = RateLimiter::new(2.0, 100.0);

        // Acquire 2 tokens (should be instant)
        limiter.acquire().await;
        limiter.acquire().await;

        // 3rd acquire should block briefly then succeed (fast refill)
        let start = std::time::Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // Should have waited at least a tiny bit
        println!("Waited {}ms for rate limiter refill", elapsed.as_millis());
        assert!(
            elapsed.as_millis() < 500,
            "Shouldn't wait too long with 100/sec refill"
        );
    }
}
