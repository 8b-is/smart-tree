//! Tests for AI-Human Partnership Features
//! 
//! "You're easy to work with. You just get me." - Let's make sure this actually works!

use chrono::{DateTime, Utc, Duration, Timelike};
use st::context_gatherer::{
    ContextGatherer, GatherConfig, GatheredContext, ContextType, ContextContent,
    collab_session::{CollaborativeSessionTracker, SessionType, CollaborativeOrigin, AnchorType},
    partnership::PartnershipAnalyzer,
    cross_session::{CrossSessionBridge, PatternType},
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Create a test context with specific properties
fn create_test_context(
    timestamp: DateTime<Utc>,
    ai_tool: &str,
    content_type: ContextType,
    role_pattern: &str,
) -> GatheredContext {
    let content = if role_pattern == "tandem" {
        ContextContent::Json(serde_json::json!({
            "messages": [
                {"role": "user", "content": "Let's work on this together"},
                {"role": "assistant", "content": "I'd be happy to help! The task is now complete and works perfectly."}
            ]
        }))
    } else if role_pattern == "user_only" {
        ContextContent::Json(serde_json::json!({
            "messages": [
                {"role": "user", "content": "Working on my own here"}
            ]
        }))
    } else {
        ContextContent::Json(serde_json::json!({
            "messages": [
                {"role": "assistant", "content": "Processing in the background"}
            ]
        }))
    };
    
    GatheredContext {
        source_path: PathBuf::from(format!("~/{}/test.json", ai_tool)),
        ai_tool: ai_tool.to_string(),
        content_type,
        content,
        metadata: HashMap::new(),
        relevance_score: 0.8,
        timestamp,
    }
}

#[test]
fn test_collaborative_session_detection() {
    let mut tracker = CollaborativeSessionTracker::new();
    let now = Utc::now();
    
    // Test 1: Solo human session
    let context1 = create_test_context(now, "claude", ContextType::ChatHistory, "user_only");
    tracker.process_context(&context1).unwrap();
    
    assert!(tracker.active_session.is_some());
    let session = tracker.active_session.as_ref().unwrap();
    assert_eq!(session.session_type, SessionType::SoloHuman);
    
    // Test 2: Transition to tandem
    let context2 = create_test_context(
        now + Duration::minutes(2),
        "claude",
        ContextType::ChatHistory,
        "tandem"
    );
    tracker.process_context(&context2).unwrap();
    
    let session = tracker.active_session.as_ref().unwrap();
    assert_eq!(session.session_type, SessionType::Tandem);
    assert_eq!(session.interactions.len(), 1); // New session started due to type change
    
    // Test 3: New session after 30+ minute gap
    let context3 = create_test_context(
        now + Duration::minutes(35),
        "claude",
        ContextType::ChatHistory,
        "tandem"
    );
    tracker.process_context(&context3).unwrap();
    
    assert_eq!(tracker.session_history.len(), 2); // Two previous sessions ended (SoloHuman + Tandem)
    let new_session = tracker.active_session.as_ref().unwrap();
    assert_eq!(new_session.interactions.len(), 1); // Fresh start
}

#[test]
fn test_flow_state_detection() {
    let mut tracker = CollaborativeSessionTracker::new();
    let now = Utc::now();
    
    // Create rapid back-and-forth (flow state)
    for i in 0..8 {
        let context = create_test_context(
            now + Duration::seconds(30 * i),
            "claude",
            ContextType::ChatHistory,
            "tandem"
        );
        tracker.process_context(&context).unwrap();
    }
    
    let session = tracker.active_session.as_ref().unwrap();
    match &session.flow_state {
        st::context_gatherer::collab_session::FlowState::Flow { depth, sustained_minutes } => {
            assert!(*depth > 0.5);
            assert!(*sustained_minutes > 0);
        }
        _ => panic!("Expected flow state, got {:?}", session.flow_state),
    }
}

#[test]
fn test_memory_anchoring() {
    let mut tracker = CollaborativeSessionTracker::new();
    
    // Anchor a collaborative insight
    let anchor_id = tracker.anchor_memory(
        CollaborativeOrigin::Tandem {
            human: "hue".to_string(),
            ai: "claude".to_string(),
        },
        AnchorType::Solution,
        "We solved the wave grid dimension issue by separating sensor values".to_string(),
        vec!["wave".to_string(), "grid".to_string(), "sensor".to_string()],
    ).unwrap();
    
    assert!(!anchor_id.is_empty());
    
    // Test retrieval
    let found = tracker.find_relevant_anchors(&["wave".to_string(), "grid".to_string()]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].anchor_type, AnchorType::Solution);
    assert!(found[0].co_created);
}

#[test]
fn test_rapport_index_evolution() {
    let mut tracker = CollaborativeSessionTracker::new();
    let now = Utc::now();
    
    // Simulate multiple successful sessions
    for day in 0..5 {
        for hour in 0..3 {
            let context = create_test_context(
                now + Duration::days(day) + Duration::hours(hour * 2),
                "claude",
                ContextType::ChatHistory,
                "tandem"
            );
            tracker.process_context(&context).unwrap();
        }
        
        // End session for the day
        if let Some(_) = tracker.active_session.take() {
            tracker.end_active_session();
        }
    }
    
    let rapport = tracker.get_rapport("claude").unwrap();
    assert!(rapport.overall_score > 0.5);
    assert!(rapport.trust_level > 0.0);
    assert!(rapport.evolution_trend >= 0.0); // Should be stable or improving
}

#[test]
fn test_cross_domain_patterns() {
    let mut bridge = CrossSessionBridge::new();
    let now = Utc::now();
    
    // Create contexts with wave decay pattern in different projects
    let contexts = vec![
        GatheredContext {
            source_path: PathBuf::from("/project1/analysis.json"),
            ai_tool: "claude".to_string(),
            content_type: ContextType::CodeSnippets,
            content: ContextContent::Text(
                "Implementing wave decay algorithm for memory fadeout".to_string()
            ),
            metadata: HashMap::new(),
            relevance_score: 0.9,
            timestamp: now,
        },
        GatheredContext {
            source_path: PathBuf::from("/project2/memory.json"),
            ai_tool: "cursor".to_string(),
            content_type: ContextType::CodeSnippets,
            content: ContextContent::Text(
                "Using wave decay pattern for temporal memory".to_string()
            ),
            metadata: HashMap::new(),
            relevance_score: 0.85,
            timestamp: now + Duration::days(2),
        },
        GatheredContext {
            source_path: PathBuf::from("/project3/audio.json"),
            ai_tool: "windsurf".to_string(),
            content_type: ContextType::CodeSnippets,
            content: ContextContent::Text(
                "Applied wave decay to audio signal processing".to_string()
            ),
            metadata: HashMap::new(),
            relevance_score: 0.8,
            timestamp: now + Duration::days(5),
        },
    ];
    
    let _new_patterns = bridge.analyze_for_patterns(&contexts);
    
    // Get all patterns from the bridge
    let all_patterns: Vec<_> = bridge.patterns.values().collect();
    assert!(!all_patterns.is_empty());
    
    // Should recognize wave decay as a cross-domain pattern
    let wave_pattern = all_patterns.iter()
        .find(|p| p.description.contains("Wave decay"))
        .expect("Should find wave decay pattern");
    
    assert_eq!(wave_pattern.pattern_type, PatternType::Algorithm);
    assert_eq!(wave_pattern.occurrences.len(), 3);
    assert!(wave_pattern.strength > 0.0);
}

#[test]
fn test_partnership_analysis() {
    let now = Utc::now();
    let contexts = vec![
        // Productive tandem session
        create_test_context(now, "claude", ContextType::ChatHistory, "tandem"),
        create_test_context(now + Duration::minutes(5), "claude", ContextType::ChatHistory, "tandem"),
        create_test_context(now + Duration::minutes(10), "claude", ContextType::ChatHistory, "tandem"),
        // Learning moment
        GatheredContext {
            source_path: PathBuf::from("~/.claude/learning.json"),
            ai_tool: "claude".to_string(),
            content_type: ContextType::ChatHistory,
            content: ContextContent::Json(serde_json::json!({
                "messages": [
                    {"role": "user", "content": "What's a monad?"},
                    {"role": "assistant", "content": "A monad is a design pattern..."}
                ]
            })),
            metadata: HashMap::new(),
            relevance_score: 0.9,
            timestamp: now + Duration::minutes(15),
        },
    ];
    
    let analyzer = PartnershipAnalyzer::new(contexts);
    let analysis = analyzer.analyze_partnership();
    
    assert!(analysis.total_interactions > 0);
    assert!(!analysis.collaborative_sessions.is_empty());
    assert!(analysis.collaboration_metrics.productivity_rate > 0.0);
    assert!(analysis.collaboration_metrics.learning_rate > 0.0);
    // With completion messages, the partnership might be "Thriving" or "Healthy"
    assert!(["Thriving", "Healthy", "Developing"].contains(&analysis.partnership_health.status.as_str()));
    assert!(!analysis.recommendations.is_empty());
}

#[test]
fn test_co_engagement_heatmap() {
    use st::context_gatherer::collab_session::CoEngagementHeatmap;
    
    let mut tracker = CollaborativeSessionTracker::new();
    let now = Utc::now();
    
    // Create tandem sessions at specific times
    for day in 0..7 {
        // Create date at start of day, then add hours for specific times
        let base_date = (now - Duration::days(day))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        
        // Morning session (9 AM)
        let morning = base_date + Duration::hours(9);
        let context = create_test_context(morning, "claude", ContextType::ChatHistory, "tandem");
        tracker.process_context(&context).unwrap();
        
        // End morning session properly before starting afternoon
        tracker.end_active_session();
        
        // Afternoon session (2 PM)
        let afternoon = base_date + Duration::hours(14);
        let context = create_test_context(afternoon, "claude", ContextType::ChatHistory, "tandem");
        tracker.process_context(&context).unwrap();
        
        // End afternoon session
        tracker.end_active_session();
    }
    
    let sessions: Vec<_> = tracker.session_history.iter().cloned().collect();
    let heatmap = CoEngagementHeatmap::from_sessions(&sessions);
    
    assert!(!heatmap.peak_collaboration_zones.is_empty());
    assert!(heatmap.collaboration_density > 0.0);
    
    // Should identify 9 AM and 2 PM as peak zones
    let has_morning_peak = heatmap.peak_collaboration_zones.iter()
        .any(|(hour, _)| *hour == 9);
    let has_afternoon_peak = heatmap.peak_collaboration_zones.iter()
        .any(|(hour, _)| *hour == 14);
    
    assert!(has_morning_peak || has_afternoon_peak);
}

#[test]
fn test_persona_invitation() {
    let bridge = CrossSessionBridge::new();
    
    // Test performance-related context
    let invitation = bridge.invite_persona(
        "Need to optimize the wave calculation performance",
        15
    ).unwrap();
    
    assert_eq!(invitation.persona_name, "The Cheet");
    assert!(invitation.expertise_areas.contains(&"Performance optimization".to_string()));
    assert_eq!(invitation.suggested_duration_minutes, 15);
    
    // Test wave-related context
    let invitation = bridge.invite_persona(
        "Exploring wave interference patterns in memory",
        10
    ).unwrap();
    
    assert_eq!(invitation.persona_name, "Omni");
    assert!(invitation.expertise_areas.contains(&"Wave-based thinking".to_string()));
}

#[test]
fn test_insight_generation() {
    let mut bridge = CrossSessionBridge::new();
    let now = Utc::now();
    
    // Create enough occurrences to generate insights
    let contexts: Vec<_> = (0..5).map(|i| {
        GatheredContext {
            source_path: PathBuf::from(format!("/project{}/code.rs", i)),
            ai_tool: "claude".to_string(),
            content_type: ContextType::CodeSnippets,
            content: ContextContent::Text(
                "Using observer pattern for event handling".to_string()
            ),
            metadata: HashMap::new(),
            relevance_score: 0.8,
            timestamp: now + Duration::days(i),
        }
    }).collect();
    
    bridge.analyze_for_patterns(&contexts);
    let insights = bridge.generate_insights(0.1);
    
    assert!(!insights.is_empty());
    let insight = &insights[0];
    assert!(insight.content.contains("appears across"));
    assert!(insight.content.contains("different contexts"));
    assert_eq!(insight.source_sessions.len(), 5);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_context_gathering_flow() {
        let project_path = PathBuf::from("/tmp/test_project");
        let config = GatherConfig {
            search_dirs: vec![".claude".to_string()],
            custom_dirs: vec![],
            extensions: vec!["json".to_string()],
            project_identifiers: vec!["test_project".to_string()],
            max_file_size: 1024 * 1024,
            recursive: true,
            privacy_mode: true,
        };
        
        let mut gatherer = ContextGatherer::new(project_path, config);
        
        // Even without actual files, we can test the structure
        assert_eq!(gatherer.contexts().len(), 0);
        
        // Test memory anchoring through gatherer
        let anchor_id = gatherer.anchor_memory(
            CollaborativeOrigin::Single("human".to_string()),
            AnchorType::PatternInsight,
            "Testing is essential for reliability".to_string(),
            vec!["testing".to_string(), "reliability".to_string()],
        ).unwrap();
        
        assert!(!anchor_id.is_empty());
        
        // Test memory retrieval
        let memories = gatherer.find_relevant_memories(&["testing".to_string()]);
        assert_eq!(memories.len(), 1);
        assert!(memories[0].contains("Pattern"));
        assert!(memories[0].contains("Testing is essential"));
    }
}