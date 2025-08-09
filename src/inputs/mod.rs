//! Universal Input Adapter System for Smart Tree
//!
//! Transform any context source into visualizable trees:
//! - File systems (traditional)
//! - QCP quantum contexts
//! - SSE event streams
//! - OpenAPI specifications
//! - MEM8 consciousness streams
//! - And more...

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod filesystem;
pub mod mem8;
pub mod openapi;
pub mod qcp;
pub mod sse;

/// Represents any kind of context node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNode {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Node type (file, api_endpoint, quantum_state, event, etc.)
    pub node_type: NodeType,

    /// Quantum properties if applicable
    pub quantum_state: Option<QuantumState>,

    /// Child nodes
    pub children: Vec<ContextNode>,

    /// Metadata specific to the input type
    pub metadata: serde_json::Value,

    /// Entanglements with other nodes
    pub entanglements: Vec<Entanglement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    // Traditional
    Directory,
    File,

    // API-related
    ApiEndpoint,
    ApiSchema,
    WebSocketChannel,

    // Quantum
    QuantumWave,
    EntangledState,
    Superposition,

    // Event streams
    EventSource,
    EventType,

    // MEM8
    MemoryWave,
    ConsciousnessStream,
    EmotionalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    /// Wave amplitude (0.0 - 1.0)
    pub amplitude: f64,

    /// Frequency in Hz
    pub frequency: f64,

    /// Phase offset
    pub phase: f64,

    /// Collapse probability
    pub collapse_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entanglement {
    /// ID of entangled node
    pub target_id: String,

    /// Strength of entanglement (0.0 - 1.0)
    pub strength: f64,

    /// Type of relationship
    pub relationship: String,
}

/// Trait for all input adapters
#[async_trait]
pub trait InputAdapter: Send + Sync {
    /// Name of the adapter
    fn name(&self) -> &'static str;

    /// Supported input formats/extensions
    fn supported_formats(&self) -> Vec<&'static str>;

    /// Can this adapter handle the given input?
    async fn can_handle(&self, input: &InputSource) -> bool;

    /// Parse input into context nodes
    async fn parse(&self, input: InputSource) -> Result<ContextNode>;

    /// Get quantum wave signature if applicable
    fn wave_signature(&self) -> Option<String> {
        None
    }
}

/// Represents an input source
#[derive(Debug, Clone)]
pub enum InputSource {
    /// Local file system path
    Path(PathBuf),

    /// URL (HTTP/HTTPS/WSS)
    Url(String),

    /// Raw data with format hint
    Raw {
        data: Vec<u8>,
        format_hint: Option<String>,
    },

    /// QCP endpoint with query
    QcpQuery { endpoint: String, query: String },

    /// MEM8 consciousness stream
    Mem8Stream {
        stream_id: String,
        temporal_range: Option<(i64, i64)>,
    },
}

/// Universal input processor
pub struct InputProcessor {
    adapters: Vec<Box<dyn InputAdapter>>,
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl InputProcessor {
    /// Create a new input processor with all adapters
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Box::new(filesystem::FileSystemAdapter),
                Box::new(qcp::QcpAdapter::new()),
                Box::new(sse::SseAdapter),
                Box::new(openapi::OpenApiAdapter),
                Box::new(mem8::Mem8Adapter),
            ],
        }
    }

    /// Process any input source into context nodes
    pub async fn process(&self, input: InputSource) -> Result<ContextNode> {
        // Find the first adapter that can handle this input
        for adapter in &self.adapters {
            if adapter.can_handle(&input).await {
                eprintln!("🌊 Using {} adapter for input", adapter.name());
                return adapter.parse(input).await;
            }
        }

        anyhow::bail!("No adapter found for input source")
    }

    /// Auto-detect input type from string
    pub fn detect_input_type(input: &str) -> InputSource {
        if input.starts_with("http://") || input.starts_with("https://") {
            InputSource::Url(input.to_string())
        } else if input.starts_with("qcp://") {
            let parts: Vec<&str> = input.splitn(2, "://").collect();
            InputSource::QcpQuery {
                endpoint: "https://qcp.q8.is".to_string(),
                query: parts.get(1).unwrap_or(&"").to_string(),
            }
        } else if input.starts_with("mem8://") {
            let stream_id = input.trim_start_matches("mem8://");
            InputSource::Mem8Stream {
                stream_id: stream_id.to_string(),
                temporal_range: None,
            }
        } else {
            InputSource::Path(PathBuf::from(input))
        }
    }
}

/// Convert context nodes to Smart Tree's FileNode format
pub fn context_to_file_nodes(context: ContextNode) -> Vec<crate::FileNode> {
    let mut nodes = Vec::new();
    convert_node(&context, &mut nodes, 0);
    nodes
}

fn convert_node(context: &ContextNode, nodes: &mut Vec<crate::FileNode>, depth: usize) {
    use crate::scanner::FileType;
    use crate::{FileCategory, FileNode, FilesystemType};
    use std::time::SystemTime;

    let node = FileNode {
        path: PathBuf::from(&context.id),
        is_dir: !context.children.is_empty(),
        size: context
            .metadata
            .get("size")
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
        modified: SystemTime::now(), // Use metadata time if available
        permissions: 0o755,
        uid: 1000,
        gid: 1000,
        is_symlink: false,
        is_hidden: false,
        permission_denied: false,
        is_ignored: false,
        depth,
        file_type: match context.node_type {
            NodeType::Directory => FileType::Directory,
            NodeType::File => FileType::RegularFile,
            NodeType::ApiEndpoint => FileType::RegularFile,
            NodeType::QuantumWave => FileType::RegularFile,
            NodeType::EventSource => FileType::RegularFile,
            NodeType::MemoryWave => FileType::RegularFile,
            _ => FileType::RegularFile,
        },
        category: match context.node_type {
            NodeType::ApiEndpoint => FileCategory::Json,
            NodeType::QuantumWave => FileCategory::Binary,
            NodeType::EventSource => FileCategory::Json,
            NodeType::MemoryWave => FileCategory::Binary,
            _ => FileCategory::Unknown,
        },
        search_matches: None,
        filesystem_type: FilesystemType::Unknown,
    };

    nodes.push(node);

    // Recursively convert children
    for child in &context.children {
        convert_node(child, nodes, depth + 1);
    }
}
