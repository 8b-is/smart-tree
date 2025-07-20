//! Server-Sent Events (SSE) input adapter
//!
//! Visualizes real-time event streams as living trees

use super::*;
use anyhow::Result;
use async_trait::async_trait;
use reqwest;
use std::time::{Duration, SystemTime};

pub struct SseAdapter;

#[async_trait]
impl InputAdapter for SseAdapter {
    fn name(&self) -> &'static str {
        "SSE"
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["sse", "events", "stream"]
    }

    async fn can_handle(&self, input: &InputSource) -> bool {
        match input {
            InputSource::Url(url) => {
                url.contains("/events") || url.contains("/stream") || url.contains("sse")
            }
            InputSource::Raw { format_hint, .. } => {
                format_hint.as_ref().map(|h| h == "sse").unwrap_or(false)
            }
            _ => false,
        }
    }

    async fn parse(&self, input: InputSource) -> Result<ContextNode> {
        match input {
            InputSource::Url(url) => self.parse_sse_stream(&url).await,
            InputSource::Raw { data, .. } => self.parse_sse_data(&data).await,
            _ => anyhow::bail!("SSE adapter requires URL or raw data"),
        }
    }
}

impl SseAdapter {
    async fn parse_sse_stream(&self, url: &str) -> Result<ContextNode> {
        let _client = reqwest::Client::new();

        // Create root node for event stream
        let mut root = ContextNode {
            id: url.to_string(),
            name: "Event Stream".to_string(),
            node_type: NodeType::EventSource,
            quantum_state: Some(QuantumState {
                amplitude: 1.0,
                frequency: 1.0, // 1 Hz update rate
                phase: 0.0,
                collapse_probability: 0.0, // Never collapses, always streaming
            }),
            children: vec![],
            metadata: serde_json::json!({
                "url": url,
                "status": "connecting",
                "event_count": 0
            }),
            entanglements: vec![],
        };

        // For demo, just show structure
        // In real implementation, would stream events
        root.children = vec![
            ContextNode {
                id: format!("{}/types", url),
                name: "Event Types".to_string(),
                node_type: NodeType::Directory,
                quantum_state: None,
                children: vec![
                    self.create_event_type_node("user_action", 10.5),
                    self.create_event_type_node("system_update", 2.0),
                    self.create_event_type_node("error", 0.1),
                ],
                metadata: serde_json::json!({}),
                entanglements: vec![],
            },
            ContextNode {
                id: format!("{}/timeline", url),
                name: "Timeline".to_string(),
                node_type: NodeType::Directory,
                quantum_state: None,
                children: self.create_timeline_nodes(),
                metadata: serde_json::json!({}),
                entanglements: vec![],
            },
        ];

        Ok(root)
    }

    async fn parse_sse_data(&self, data: &[u8]) -> Result<ContextNode> {
        let content = String::from_utf8_lossy(data);
        let mut events = Vec::new();

        // Parse SSE format
        for line in content.lines() {
            if line.starts_with("data: ") {
                let event_data = &line[6..];
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(event_data) {
                    events.push(json);
                }
            }
        }

        Ok(ContextNode {
            id: "sse_data".to_string(),
            name: "SSE Events".to_string(),
            node_type: NodeType::EventSource,
            quantum_state: None,
            children: events
                .iter()
                .enumerate()
                .map(|(i, event)| ContextNode {
                    id: format!("event_{}", i),
                    name: event
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("event")
                        .to_string(),
                    node_type: NodeType::EventType,
                    quantum_state: None,
                    children: vec![],
                    metadata: event.clone(),
                    entanglements: vec![],
                })
                .collect(),
            metadata: serde_json::json!({
                "event_count": events.len()
            }),
            entanglements: vec![],
        })
    }

    fn create_event_type_node(&self, event_type: &str, frequency: f64) -> ContextNode {
        ContextNode {
            id: format!("type_{}", event_type),
            name: event_type.to_string(),
            node_type: NodeType::EventType,
            quantum_state: Some(QuantumState {
                amplitude: frequency / 10.0, // Normalize
                frequency,
                phase: 0.0,
                collapse_probability: 0.5,
            }),
            children: vec![],
            metadata: serde_json::json!({
                "average_per_second": frequency,
                "total_count": (frequency * 3600.0) as u64, // Last hour
            }),
            entanglements: vec![],
        }
    }

    fn create_timeline_nodes(&self) -> Vec<ContextNode> {
        let now = SystemTime::now();
        let mut nodes = Vec::new();

        // Create nodes for last 5 time buckets
        for i in 0..5 {
            let bucket_time = now - Duration::from_secs(i * 60);
            nodes.push(ContextNode {
                id: format!("bucket_{}", i),
                name: format!("{} min ago", i),
                node_type: NodeType::Directory,
                quantum_state: Some(QuantumState {
                    amplitude: 1.0 / (i as f64 + 1.0), // Decay over time
                    frequency: 1.0,
                    phase: i as f64 * std::f64::consts::PI / 5.0,
                    collapse_probability: i as f64 / 5.0,
                }),
                children: vec![],
                metadata: serde_json::json!({
                    "timestamp": bucket_time,
                    "event_count": 42 - (i * 8), // Simulated decay
                }),
                entanglements: if i > 0 {
                    vec![Entanglement {
                        target_id: format!("bucket_{}", i - 1),
                        strength: 0.8,
                        relationship: "temporal_sequence".to_string(),
                    }]
                } else {
                    vec![]
                },
            });
        }

        nodes
    }
}
