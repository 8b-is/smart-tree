use crate::{Protocol, Result, ToolError};
use serde_json;

pub trait ProtocolHandler: Send + Sync {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

pub struct JsonHandler;
pub struct MessagePackHandler;
pub struct QcpHandler;

impl ProtocolHandler for JsonHandler {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For JSON, we just validate it's valid JSON and pass through
        serde_json::from_slice::<serde_json::Value>(data)
            .map_err(|e| ToolError::ProtocolError(format!("Invalid JSON: {}", e)))?;
        Ok(data.to_vec())
    }
    
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For JSON decoding, validate and pretty-print
        let value: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| ToolError::ProtocolError(format!("JSON decode error: {}", e)))?;
        serde_json::to_vec_pretty(&value)
            .map_err(|e| ToolError::ProtocolError(format!("JSON encode error: {}", e)))
    }
}

impl ProtocolHandler for MessagePackHandler {
    fn encode(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // Placeholder for MessagePack implementation
        Err(ToolError::ProtocolError("MessagePack not yet implemented".to_string()))
    }
    
    fn decode(&self, _data: &[u8]) -> Result<Vec<u8>> {
        Err(ToolError::ProtocolError("MessagePack not yet implemented".to_string()))
    }
}

impl ProtocolHandler for QcpHandler {
    fn encode(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // Quantum Context Protocol - future implementation
        // Will use semantic tokenization and delta encoding
        Err(ToolError::ProtocolError("QCP not yet implemented".to_string()))
    }
    
    fn decode(&self, _data: &[u8]) -> Result<Vec<u8>> {
        Err(ToolError::ProtocolError("QCP not yet implemented".to_string()))
    }
}

pub fn get_handler(protocol: Protocol) -> Box<dyn ProtocolHandler> {
    match protocol {
        Protocol::Json => Box::new(JsonHandler),
        Protocol::MessagePack => Box::new(MessagePackHandler),
        Protocol::Qcp => Box::new(QcpHandler),
    }
}