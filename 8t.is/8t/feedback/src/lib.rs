pub mod agent;
pub mod continuous;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub use agent::{ImprovementAgent, AgentOrchestrator, AgentSpecialization, AgentStatus};
pub use continuous::{ContinuousFeedbackSystem, AIInteraction, AIFeedbackSubmission};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,
    pub tool_name: String,
    pub category: FeedbackCategory,
    pub title: String,
    pub description: String,
    pub code_suggestion: Option<String>,
    pub priority: Priority,
    pub source: FeedbackSource,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackCategory {
    Feature,
    Bug,
    Performance,
    Documentation,
    Refactor,
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSource {
    pub ai_model: String,
    pub conversation_id: Option<String>,
    pub user_id: Option<String>,
}

pub struct FeedbackStore {
    base_path: PathBuf,
}

impl FeedbackStore {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
    
    pub async fn init(&self) -> Result<()> {
        // Create directory structure
        tokio::fs::create_dir_all(&self.base_path).await?;
        
        for category in &["features", "bugs", "performance", "docs", "refactor", "security"] {
            tokio::fs::create_dir_all(self.base_path.join(category)).await?;
        }
        
        Ok(())
    }
    
    pub async fn store(&self, feedback: &Feedback) -> Result<PathBuf> {
        let category_dir = match feedback.category {
            FeedbackCategory::Feature => "features",
            FeedbackCategory::Bug => "bugs",
            FeedbackCategory::Performance => "performance",
            FeedbackCategory::Documentation => "docs",
            FeedbackCategory::Refactor => "refactor",
            FeedbackCategory::Security => "security",
        };
        
        let tool_dir = self.base_path
            .join(category_dir)
            .join(&feedback.tool_name);
        
        tokio::fs::create_dir_all(&tool_dir).await?;
        
        // Create filename: YYYY-MM-DD_priority_id.json
        let filename = format!(
            "{}_{:?}_{}.json",
            feedback.created_at.format("%Y-%m-%d"),
            feedback.priority,
            &feedback.id[..8]
        );
        
        let file_path = tool_dir.join(filename);
        
        // Write feedback as JSON
        let json = serde_json::to_string_pretty(feedback)?;
        tokio::fs::write(&file_path, json).await?;
        
        // If there's a code suggestion, create a separate file
        if let Some(code) = &feedback.code_suggestion {
            let code_filename = format!("{}.suggestion", &feedback.id[..8]);
            let code_path = tool_dir.join(code_filename);
            tokio::fs::write(&code_path, code).await?;
        }
        
        Ok(file_path)
    }
    
    pub async fn create_git_branch(&self, feedback: &Feedback) -> Result<String> {
        let repo = git2::Repository::open(&self.base_path)?;
        
        let branch_name = format!(
            "{}/{}-{}",
            match feedback.category {
                FeedbackCategory::Feature => "feature",
                FeedbackCategory::Bug => "fix",
                FeedbackCategory::Performance => "perf",
                FeedbackCategory::Documentation => "docs",
                FeedbackCategory::Refactor => "refactor",
                FeedbackCategory::Security => "security",
            },
            feedback.tool_name,
            feedback.title.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
        );
        
        let head = repo.head()?;
        let oid = head.target().ok_or_else(|| anyhow::anyhow!("No HEAD"))?;
        let commit = repo.find_commit(oid)?;
        
        repo.branch(&branch_name, &commit, false)?;
        
        Ok(branch_name)
    }
    
    pub async fn list_feedback(
        &self,
        tool_name: Option<&str>,
        category: Option<FeedbackCategory>,
    ) -> Result<Vec<Feedback>> {
        let mut feedback_list = Vec::new();
        
        let categories = if let Some(cat) = category {
            vec![cat]
        } else {
            vec![
                FeedbackCategory::Feature,
                FeedbackCategory::Bug,
                FeedbackCategory::Performance,
                FeedbackCategory::Documentation,
                FeedbackCategory::Refactor,
                FeedbackCategory::Security,
            ]
        };
        
        for cat in categories {
            let cat_dir = match cat {
                FeedbackCategory::Feature => "features",
                FeedbackCategory::Bug => "bugs",
                FeedbackCategory::Performance => "performance",
                FeedbackCategory::Documentation => "docs",
                FeedbackCategory::Refactor => "refactor",
                FeedbackCategory::Security => "security",
            };
            
            let dir_path = if let Some(tool) = tool_name {
                self.base_path.join(cat_dir).join(tool)
            } else {
                self.base_path.join(cat_dir)
            };
            
            if !dir_path.exists() {
                continue;
            }
            
            let mut entries = tokio::fs::read_dir(dir_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = tokio::fs::read_to_string(&path).await?;
                    if let Ok(feedback) = serde_json::from_str::<Feedback>(&content) {
                        feedback_list.push(feedback);
                    }
                }
            }
        }
        
        // Sort by priority and date
        feedback_list.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(b.created_at.cmp(&a.created_at))
        });
        
        Ok(feedback_list)
    }
}

impl Feedback {
    pub fn new(
        tool_name: impl Into<String>,
        category: FeedbackCategory,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tool_name: tool_name.into(),
            category,
            title: title.into(),
            description: description.into(),
            code_suggestion: None,
            priority: Priority::Medium,
            source: FeedbackSource {
                ai_model: "unknown".to_string(),
                conversation_id: None,
                user_id: None,
            },
            created_at: Utc::now(),
            tags: Vec::new(),
        }
    }
    
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code_suggestion = Some(code.into());
        self
    }
    
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_source(mut self, ai_model: impl Into<String>) -> Self {
        self.source.ai_model = ai_model.into();
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}