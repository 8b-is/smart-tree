use crate::{Feedback, FeedbackCategory, FeedbackStore, Priority, AgentOrchestrator};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Continuous feedback system that monitors AI interactions and generates improvements
pub struct ContinuousFeedbackSystem {
    store: Arc<FeedbackStore>,
    orchestrator: Arc<AgentOrchestrator>,
    metrics: Arc<RwLock<SystemMetrics>>,
    config: FeedbackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    pub min_confidence_threshold: f32,
    pub batch_size: usize,
    pub processing_interval: Duration,
    pub auto_prioritize: bool,
    pub g8t_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_feedback_processed: u64,
    pub improvements_submitted: u64,
    pub average_processing_time: Duration,
    pub success_rate: f32,
    pub active_agents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInteraction {
    pub timestamp: u64,
    pub input_size: usize,
    pub output_size: usize,
    pub context_type: String,
    pub extraction_ratio: f32,
    pub user_satisfaction: Option<f32>,
    pub ai_observations: Vec<String>,
}

impl ContinuousFeedbackSystem {
    pub fn new(store_path: &str, config: FeedbackConfig) -> Self {
        let store = Arc::new(FeedbackStore::new(store_path));
        let orchestrator = Arc::new(AgentOrchestrator::new(config.g8t_endpoint.clone()));
        
        Self {
            store,
            orchestrator,
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            config,
        }
    }
    
    /// Start the continuous feedback loop
    pub async fn start(&self) -> Result<()> {
        // Initialize storage
        self.store.init().await?;
        
        // Initialize improvement agents
        self.orchestrator.init_agents().await?;
        
        // Start monitoring loop
        let system = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = system.process_feedback_batch().await {
                    tracing::error!("Feedback processing error: {}", e);
                }
                tokio::time::sleep(system.config.processing_interval).await;
            }
        });
        
        // Start orchestration loop
        let orchestrator = self.orchestrator.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = orchestrator.orchestrate().await {
                    tracing::error!("Orchestration error: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
        
        Ok(())
    }
    
    /// Process an AI interaction and generate feedback
    pub async fn process_interaction(&self, interaction: AIInteraction) -> Result<()> {
        let mut feedback_items = Vec::new();
        
        // Analyze extraction ratio
        if interaction.extraction_ratio > 50.0 {
            feedback_items.push(
                Feedback::new(
                    "contextualizer",
                    FeedbackCategory::Performance,
                    "High extraction ratio detected",
                    format!(
                        "Extraction ratio of {:.1}% for {} suggests opportunity for better filtering",
                        interaction.extraction_ratio, interaction.context_type
                    ),
                )
                .with_priority(Priority::Medium)
                .with_tags(vec!["auto-detected".to_string(), "extraction-ratio".to_string()])
            );
        }
        
        // Check for specific patterns in AI observations
        for observation in &interaction.ai_observations {
            if let Some(feedback) = self.analyze_observation(observation, &interaction.context_type).await {
                feedback_items.push(feedback);
            }
        }
        
        // Analyze user satisfaction if available
        if let Some(satisfaction) = interaction.user_satisfaction {
            if satisfaction < 0.7 {
                feedback_items.push(
                    Feedback::new(
                        "contextualizer",
                        FeedbackCategory::Feature,
                        "Low user satisfaction",
                        format!(
                            "User satisfaction of {:.1}% suggests improvements needed for {} context",
                            satisfaction * 100.0, interaction.context_type
                        ),
                    )
                    .with_priority(Priority::High)
                );
            }
        }
        
        // Store all feedback
        for feedback in feedback_items {
            self.store.store(&feedback).await?;
            self.orchestrator.process_feedback(feedback).await?;
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_feedback_processed += 1;
        
        Ok(())
    }
    
    /// Analyze AI observations for improvement opportunities
    async fn analyze_observation(&self, observation: &str, context_type: &str) -> Option<Feedback> {
        let obs_lower = observation.to_lowercase();
        
        // Pattern matching for common improvement opportunities
        if obs_lower.contains("could be more specific") || obs_lower.contains("too much data") {
            return Some(
                Feedback::new(
                    "contextualizer",
                    FeedbackCategory::Feature,
                    "Context specificity improvement",
                    format!("AI suggests more specific filtering for {}: {}", context_type, observation),
                )
                .with_priority(Priority::Medium)
                .with_tags(vec!["ai-suggestion".to_string(), "specificity".to_string()])
            );
        }
        
        if obs_lower.contains("missing") || obs_lower.contains("should include") {
            return Some(
                Feedback::new(
                    "contextualizer",
                    FeedbackCategory::Bug,
                    "Missing context data",
                    format!("AI identified missing data in {}: {}", context_type, observation),
                )
                .with_priority(Priority::High)
                .with_tags(vec!["ai-suggestion".to_string(), "missing-data".to_string()])
            );
        }
        
        if obs_lower.contains("pattern") || obs_lower.contains("repeated") {
            return Some(
                Feedback::new(
                    "contextualizer",
                    FeedbackCategory::Performance,
                    "Pattern optimization opportunity",
                    format!("AI detected pattern in {}: {}", context_type, observation),
                )
                .with_priority(Priority::Medium)
                .with_tags(vec!["ai-suggestion".to_string(), "pattern".to_string()])
            );
        }
        
        None
    }
    
    /// Process a batch of feedback items
    async fn process_feedback_batch(&self) -> Result<()> {
        // Get pending feedback
        let feedback_list = self.store.list_feedback(None, None).await?;
        
        // Process in batches
        for chunk in feedback_list.chunks(self.config.batch_size) {
            for feedback in chunk {
                if self.config.auto_prioritize {
                    // Re-prioritize based on frequency and impact
                    let priority = self.calculate_priority(feedback).await;
                    let mut updated = feedback.clone();
                    updated.priority = priority;
                    self.orchestrator.process_feedback(updated).await?;
                } else {
                    self.orchestrator.process_feedback(feedback.clone()).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate dynamic priority based on various factors
    async fn calculate_priority(&self, feedback: &Feedback) -> Priority {
        // Factors to consider:
        // 1. How many times similar feedback has been received
        // 2. Potential impact
        // 3. Category importance
        // 4. User satisfaction correlation
        
        match feedback.category {
            FeedbackCategory::Bug | FeedbackCategory::Security => Priority::High,
            FeedbackCategory::Performance if feedback.description.contains("slow") => Priority::High,
            FeedbackCategory::Feature if feedback.tags.contains(&"ai-suggestion".to_string()) => Priority::Medium,
            _ => Priority::Low,
        }
    }
    
    /// Get system metrics
    pub async fn get_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            total_feedback_processed: 0,
            improvements_submitted: 0,
            average_processing_time: Duration::from_secs(0),
            success_rate: 0.0,
            active_agents: 0,
        }
    }
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.7,
            batch_size: 10,
            processing_interval: Duration::from_secs(60),
            auto_prioritize: true,
            g8t_endpoint: "https://g8t.is/api".to_string(),
        }
    }
}

impl Clone for ContinuousFeedbackSystem {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            orchestrator: self.orchestrator.clone(),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
        }
    }
}

/// Integration point for AI assistants to submit observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFeedbackSubmission {
    pub model: String,
    pub conversation_id: String,
    pub observations: Vec<Observation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub category: ObservationCategory,
    pub description: String,
    pub confidence: f32,
    pub suggested_improvement: Option<String>,
    pub example_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationCategory {
    Inefficiency,
    MissingFeature,
    UserStruggle,
    PatternDetected,
    ImprovementOpportunity,
}