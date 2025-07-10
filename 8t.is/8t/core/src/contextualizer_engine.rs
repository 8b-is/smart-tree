use crate::{Result, ToolError, Contextualizer, ContextQuery, ContextDepth};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use blake3::Hasher;

/// The main contextualization engine that powers 8t
/// This is where the magic happens - taking 80x context and reducing it to just what you need
pub struct ContextEngine {
    contextualizer: Contextualizer,
    learned_patterns: HashMap<String, ContextPattern>,
    conversation_context: ConversationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPattern {
    pub id: String,
    pub source_type: String,
    pub common_queries: Vec<String>,
    pub field_importance: HashMap<String, f32>,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub current_topic: Option<String>,
    pub recent_keywords: Vec<String>,
    pub interaction_history: Vec<Interaction>,
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub query: String,
    pub timestamp: u64,
    pub satisfaction_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_depth: ContextDepth,
    pub max_response_size: usize,
    pub technical_level: TechnicalLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalLevel {
    Beginner,
    Intermediate,
    Expert,
}

impl ContextEngine {
    pub fn new() -> Self {
        Self {
            contextualizer: Contextualizer::new(),
            learned_patterns: Self::init_common_patterns(),
            conversation_context: ConversationContext::default(),
        }
    }
    
    /// The main entry point - takes any data and returns just what's needed
    pub fn process(&mut self, data: &[u8], hint: Option<&str>) -> Result<ProcessedContext> {
        // Step 1: Identify the data source type
        let source_type = self.identify_source_type(data)?;
        
        // Step 2: Build context query based on conversation and hint
        let query = self.build_context_query(&source_type, hint);
        
        // Step 3: Apply contextualizer
        let filtered_data = self.contextualizer.contextualize(data, &query)?;
        
        // Step 4: Apply additional processing based on user preferences
        let final_data = self.apply_user_preferences(filtered_data)?;
        
        // Step 5: Update learning patterns
        self.update_patterns(&source_type, &query);
        
        let extraction_ratio = self.calculate_extraction_ratio(data.len(), final_data.len());
        
        Ok(ProcessedContext {
            data: final_data,
            source_type,
            extraction_ratio,
            metadata: self.generate_metadata(&query),
        })
    }
    
    /// Smart detection of data source type
    fn identify_source_type(&self, data: &[u8]) -> Result<String> {
        // Try parsing as JSON first
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) {
            // Look for telltale signs of different APIs
            if let Some(obj) = value.as_object() {
                // Hetzner server API
                if obj.contains_key("server_type") && obj.contains_key("datacenter") {
                    return Ok("hetzner_server".to_string());
                }
                
                // DNS records
                if obj.contains_key("zone_id") || obj.contains_key("rrsets") {
                    return Ok("dns_records".to_string());
                }
                
                // Kubernetes resources
                if obj.contains_key("apiVersion") && obj.contains_key("kind") {
                    return Ok("kubernetes".to_string());
                }
                
                // Docker/Container info
                if obj.contains_key("Config") && obj.contains_key("State") {
                    return Ok("docker_container".to_string());
                }
                
                // Git/GitHub
                if obj.contains_key("commits") || obj.contains_key("pull_request") {
                    return Ok("github".to_string());
                }
                
                // Cloud provider responses
                if obj.contains_key("Instances") || obj.contains_key("instanceId") {
                    return Ok("aws_ec2".to_string());
                }
                
                if obj.contains_key("compute") && obj.contains_key("project") {
                    return Ok("gcp_compute".to_string());
                }
            }
        }
        
        // Check for other formats
        if data.starts_with(b"<?xml") {
            return Ok("xml".to_string());
        }
        
        if data.starts_with(b"---\n") {
            return Ok("yaml".to_string());
        }
        
        // Default to generic JSON
        Ok("json".to_string())
    }
    
    fn build_context_query(&self, source_type: &str, hint: Option<&str>) -> ContextQuery {
        let mut query = ContextQuery {
            topic: self.conversation_context.current_topic.clone()
                .unwrap_or_else(|| source_type.to_string()),
            keywords: self.conversation_context.recent_keywords.clone(),
            depth: self.conversation_context.user_preferences.preferred_depth,
            max_output_size: Some(self.conversation_context.user_preferences.max_response_size),
        };
        
        // Add hint-based keywords
        if let Some(hint) = hint {
            let hint_words: Vec<String> = hint.split_whitespace()
                .map(|s| s.to_lowercase())
                .collect();
            query.keywords.extend(hint_words);
        }
        
        // Add source-specific patterns
        if let Some(pattern) = self.learned_patterns.get(source_type) {
            query.keywords.extend(pattern.common_queries.clone());
        }
        
        query
    }
    
    fn apply_user_preferences(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        match self.conversation_context.user_preferences.technical_level {
            TechnicalLevel::Beginner => {
                // Add explanatory comments
                self.add_explanations(data)
            }
            TechnicalLevel::Intermediate => {
                // Keep as is
                Ok(data)
            }
            TechnicalLevel::Expert => {
                // Further compress with technical notation
                self.apply_technical_compression(data)
            }
        }
    }
    
    fn add_explanations(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        // For beginners, add helpful context
        if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&data) {
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "_explanation".to_string(),
                    serde_json::json!({
                        "note": "This data has been filtered to show only relevant information",
                        "original_size": "Much larger",
                        "extraction_method": "8t contextual filtering"
                    })
                );
            }
            return serde_json::to_vec_pretty(&value)
                .map_err(|e| ToolError::ToolError(e.to_string()));
        }
        Ok(data)
    }
    
    fn apply_technical_compression(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        // For experts, use more aggressive compression
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&data) {
            // Convert to compact notation
            let compact = self.to_compact_notation(&value);
            return Ok(compact.into_bytes());
        }
        Ok(data)
    }
    
    fn to_compact_notation(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(map) => {
                let pairs: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}:{}", k, self.to_compact_notation(v)))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            }
            serde_json::Value::Array(arr) => {
                let items: Vec<String> = arr.iter()
                    .map(|v| self.to_compact_notation(v))
                    .collect();
                format!("[{}]", items.join(","))
            }
            serde_json::Value::String(s) => {
                if s.len() > 20 {
                    format!("\"{}...\"", &s[..17])
                } else {
                    format!("\"{}\"", s)
                }
            }
            _ => value.to_string(),
        }
    }
    
    fn update_patterns(&mut self, source_type: &str, query: &ContextQuery) {
        if let Some(pattern) = self.learned_patterns.get_mut(source_type) {
            pattern.access_count += 1;
            
            // Update common queries with new keywords
            for keyword in &query.keywords {
                if !pattern.common_queries.contains(keyword) {
                    pattern.common_queries.push(keyword.clone());
                    
                    // Keep only the most recent/relevant
                    if pattern.common_queries.len() > 20 {
                        pattern.common_queries.remove(0);
                    }
                }
            }
        } else {
            // Create new pattern
            let mut hasher = Hasher::new();
            hasher.update(source_type.as_bytes());
            let id = hasher.finalize().to_hex().to_string();
            
            self.learned_patterns.insert(
                source_type.to_string(),
                ContextPattern {
                    id,
                    source_type: source_type.to_string(),
                    common_queries: query.keywords.clone(),
                    field_importance: HashMap::new(),
                    access_count: 1,
                }
            );
        }
    }
    
    fn calculate_extraction_ratio(&self, original: usize, extracted: usize) -> f32 {
        if original == 0 {
            0.0
        } else {
            (extracted as f32 / original as f32) * 100.0
        }
    }
    
    fn generate_metadata(&self, query: &ContextQuery) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("topic".to_string(), query.topic.clone());
        metadata.insert("depth".to_string(), format!("{:?}", query.depth));
        metadata.insert("keywords".to_string(), query.keywords.join(", "));
        metadata
    }
    
    fn init_common_patterns() -> HashMap<String, ContextPattern> {
        let mut patterns = HashMap::new();
        
        // Hetzner Server Pattern
        patterns.insert("hetzner_server".to_string(), ContextPattern {
            id: "hetzner_001".to_string(),
            source_type: "hetzner_server".to_string(),
            common_queries: vec![
                "id".to_string(), "name".to_string(), "status".to_string(),
                "public_net".to_string(), "server_type".to_string()
            ],
            field_importance: HashMap::from([
                ("id".to_string(), 1.0),
                ("name".to_string(), 0.9),
                ("status".to_string(), 0.9),
                ("public_net".to_string(), 0.8),
            ]),
            access_count: 0,
        });
        
        // DNS Pattern
        patterns.insert("dns_records".to_string(), ContextPattern {
            id: "dns_001".to_string(),
            source_type: "dns_records".to_string(),
            common_queries: vec![
                "name".to_string(), "type".to_string(), "value".to_string(),
                "ttl".to_string(), "priority".to_string()
            ],
            field_importance: HashMap::from([
                ("name".to_string(), 1.0),
                ("type".to_string(), 1.0),
                ("value".to_string(), 1.0),
                ("ttl".to_string(), 0.5),
            ]),
            access_count: 0,
        });
        
        patterns
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContext {
    pub data: Vec<u8>,
    pub source_type: String,
    pub extraction_ratio: f32,
    pub metadata: HashMap<String, String>,
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self {
            current_topic: None,
            recent_keywords: Vec::new(),
            interaction_history: Vec::new(),
            user_preferences: UserPreferences::default(),
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_depth: ContextDepth::Relevant,
            max_response_size: 8192, // 8KB default
            technical_level: TechnicalLevel::Intermediate,
        }
    }
}