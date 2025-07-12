pub mod context;
pub mod contextualizer;
pub mod contextualizer_engine;
pub mod protocol;
pub mod quantize;
pub mod tool;

pub use context::{Context, ContextManager};
pub use contextualizer::{Contextualizer, ContextQuery, ContextDepth};
pub use contextualizer_engine::{ContextEngine, ProcessedContext};
pub use tool::ToolRegistry;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub supported_protocols: Vec<Protocol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Json,
    MessagePack,
    Qcp, // Quantum Context Protocol for later
}

pub trait EightyTool: Send + Sync {
    fn info(&self) -> ToolInfo;
    
    fn process(&self, input: &[u8], protocol: Protocol) -> Result<Vec<u8>>;
    
    fn quantize(&self, data: &[u8]) -> Result<Vec<u8>> {
        quantize::quantize_8bit(data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Quantization error: {0}")]
    QuantizationError(String),
    
    #[error("Tool error: {0}")]
    ToolError(String),
}

pub type Result<T> = std::result::Result<T, ToolError>;