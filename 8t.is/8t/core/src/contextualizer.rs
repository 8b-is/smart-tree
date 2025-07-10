use crate::{Result, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuery {
    pub topic: String,
    pub keywords: Vec<String>,
    pub depth: ContextDepth,
    pub max_output_size: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextDepth {
    Summary,    // Just the essentials
    Relevant,   // Related fields only
    Deep,       // Include nested related data
    Everything, // Full data (fallback)
}

pub struct Contextualizer {
    patterns: HashMap<String, Vec<String>>,
}

impl Contextualizer {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // DNS-related patterns
        patterns.insert("dns".to_string(), vec![
            "dns".to_string(),
            "nameserver".to_string(),
            "domain".to_string(),
            "record".to_string(),
            "zone".to_string(),
            "resolver".to_string(),
            "a_record".to_string(),
            "aaaa_record".to_string(),
            "mx_record".to_string(),
            "txt_record".to_string(),
            "cname".to_string(),
        ]);
        
        // Server patterns
        patterns.insert("server".to_string(), vec![
            "server".to_string(),
            "host".to_string(),
            "ip".to_string(),
            "ipv4".to_string(),
            "ipv6".to_string(),
            "name".to_string(),
            "status".to_string(),
            "datacenter".to_string(),
            "location".to_string(),
        ]);
        
        // Network patterns
        patterns.insert("network".to_string(), vec![
            "network".to_string(),
            "subnet".to_string(),
            "interface".to_string(),
            "port".to_string(),
            "firewall".to_string(),
            "route".to_string(),
            "gateway".to_string(),
        ]);
        
        // Error/Issue patterns
        patterns.insert("error".to_string(), vec![
            "error".to_string(),
            "failure".to_string(),
            "issue".to_string(),
            "problem".to_string(),
            "warning".to_string(),
            "critical".to_string(),
            "alert".to_string(),
            "status".to_string(),
        ]);
        
        Self { patterns }
    }
    
    pub fn contextualize(&self, data: &[u8], query: &ContextQuery) -> Result<Vec<u8>> {
        // Parse JSON data
        let value: Value = serde_json::from_slice(data)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        // Extract relevant fields based on query
        let filtered = match query.depth {
            ContextDepth::Summary => self.extract_summary(&value, query),
            ContextDepth::Relevant => self.extract_relevant(&value, query),
            ContextDepth::Deep => self.extract_deep(&value, query),
            ContextDepth::Everything => value,
        };
        
        // Convert back to bytes
        let output = serde_json::to_vec_pretty(&filtered)
            .map_err(|e| ToolError::ToolError(format!("Serialization error: {}", e)))?;
        
        // Apply size limit if specified
        if let Some(max_size) = query.max_output_size {
            if output.len() > max_size {
                // Truncate intelligently by re-filtering with Summary depth
                let summary_query = ContextQuery {
                    depth: ContextDepth::Summary,
                    ..query.clone()
                };
                return self.contextualize(data, &summary_query);
            }
        }
        
        Ok(output)
    }
    
    fn extract_summary(&self, value: &Value, query: &ContextQuery) -> Value {
        let mut summary = serde_json::Map::new();
        
        if let Value::Object(map) = value {
            // For Hetzner-style server JSON, extract key identifiers
            if let Some(id) = map.get("id") {
                summary.insert("id".to_string(), id.clone());
            }
            if let Some(name) = map.get("name") {
                summary.insert("name".to_string(), name.clone());
            }
            if let Some(status) = map.get("status") {
                summary.insert("status".to_string(), status.clone());
            }
            
            // Look for topic-specific fields
            let topic_patterns = self.get_topic_patterns(&query.topic);
            for (key, value) in map {
                if topic_patterns.iter().any(|p| key.contains(p)) {
                    summary.insert(key.clone(), self.simplify_value(value));
                }
            }
        }
        
        Value::Object(summary)
    }
    
    fn extract_relevant(&self, value: &Value, query: &ContextQuery) -> Value {
        self.filter_object(value, |key, _value| {
            let key_lower = key.to_lowercase();
            
            // Check against topic patterns
            let topic_patterns = self.get_topic_patterns(&query.topic);
            let matches_topic = topic_patterns.iter().any(|p| key_lower.contains(p));
            
            // Check against query keywords
            let matches_keywords = query.keywords.iter().any(|k| key_lower.contains(&k.to_lowercase()));
            
            matches_topic || matches_keywords
        })
    }
    
    fn extract_deep(&self, value: &Value, query: &ContextQuery) -> Value {
        // Like relevant, but includes parent objects if children match
        self.deep_filter_object(value, |key, _value| {
            let key_lower = key.to_lowercase();
            
            let topic_patterns = self.get_topic_patterns(&query.topic);
            let matches_topic = topic_patterns.iter().any(|p| key_lower.contains(p));
            let matches_keywords = query.keywords.iter().any(|k| key_lower.contains(&k.to_lowercase()));
            
            matches_topic || matches_keywords
        })
    }
    
    fn get_topic_patterns(&self, topic: &str) -> Vec<String> {
        let topic_lower = topic.to_lowercase();
        
        // Check if we have predefined patterns
        if let Some(patterns) = self.patterns.get(&topic_lower) {
            return patterns.clone();
        }
        
        // Otherwise, use the topic itself as a pattern
        vec![topic_lower]
    }
    
    fn filter_object<F>(&self, value: &Value, predicate: F) -> Value
    where
        F: Fn(&str, &Value) -> bool + Copy,
    {
        match value {
            Value::Object(map) => {
                let mut filtered = serde_json::Map::new();
                for (key, val) in map {
                    if predicate(key, val) {
                        filtered.insert(key.clone(), self.filter_object(val, predicate));
                    }
                }
                Value::Object(filtered)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.filter_object(v, predicate)).collect())
            }
            _ => value.clone(),
        }
    }
    
    fn deep_filter_object<F>(&self, value: &Value, predicate: F) -> Value
    where
        F: Fn(&str, &Value) -> bool + Copy,
    {
        match value {
            Value::Object(map) => {
                let mut filtered = serde_json::Map::new();
                let mut has_matching_child = false;
                
                // First pass: check children
                for (key, val) in map {
                    if predicate(key, val) {
                        has_matching_child = true;
                        filtered.insert(key.clone(), self.deep_filter_object(val, predicate));
                    } else if let Value::Object(_) | Value::Array(_) = val {
                        let child_result = self.deep_filter_object(val, predicate);
                        if !child_result.is_null() && 
                           !(child_result.is_object() && child_result.as_object().unwrap().is_empty()) &&
                           !(child_result.is_array() && child_result.as_array().unwrap().is_empty()) {
                            has_matching_child = true;
                            filtered.insert(key.clone(), child_result);
                        }
                    }
                }
                
                if has_matching_child {
                    Value::Object(filtered)
                } else {
                    Value::Null
                }
            }
            Value::Array(arr) => {
                let filtered: Vec<_> = arr.iter()
                    .map(|v| self.deep_filter_object(v, predicate))
                    .filter(|v| !v.is_null())
                    .collect();
                    
                if filtered.is_empty() {
                    Value::Null
                } else {
                    Value::Array(filtered)
                }
            }
            _ => value.clone(),
        }
    }
    
    fn simplify_value(&self, value: &Value) -> Value {
        match value {
            Value::Object(map) if map.len() > 3 => {
                // For large objects, just show count
                Value::String(format!("<{} fields>", map.len()))
            }
            Value::Array(arr) if arr.len() > 5 => {
                // For large arrays, show first few items and count
                let preview: Vec<_> = arr.iter().take(3).cloned().collect();
                Value::Array(vec![
                    Value::Array(preview),
                    Value::String(format!("... {} more items", arr.len() - 3)),
                ])
            }
            _ => value.clone(),
        }
    }
}

impl Default for Contextualizer {
    fn default() -> Self {
        Self::new()
    }
}

// Example usage for different scenarios
impl ContextQuery {
    pub fn dns_issue() -> Self {
        Self {
            topic: "dns".to_string(),
            keywords: vec!["error".to_string(), "failure".to_string(), "resolver".to_string()],
            depth: ContextDepth::Relevant,
            max_output_size: Some(4096), // 4KB max
        }
    }
    
    pub fn server_status() -> Self {
        Self {
            topic: "server".to_string(),
            keywords: vec!["status".to_string(), "health".to_string()],
            depth: ContextDepth::Summary,
            max_output_size: Some(1024), // 1KB max
        }
    }
    
    pub fn network_debug() -> Self {
        Self {
            topic: "network".to_string(),
            keywords: vec!["interface".to_string(), "route".to_string(), "firewall".to_string()],
            depth: ContextDepth::Deep,
            max_output_size: None, // No limit for debugging
        }
    }
}