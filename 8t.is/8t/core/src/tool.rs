use crate::{EightyTool, Protocol, Result, ToolError, ToolInfo};
use std::sync::Arc;

pub struct ToolRegistry {
    tools: dashmap::DashMap<String, Arc<dyn EightyTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: dashmap::DashMap::new(),
        }
    }
    
    pub fn register(&self, tool: Arc<dyn EightyTool>) -> Result<()> {
        let info = tool.info();
        if self.tools.contains_key(&info.name) {
            return Err(ToolError::ToolError(
                format!("Tool '{}' already registered", info.name)
            ));
        }
        self.tools.insert(info.name.clone(), tool);
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn EightyTool>> {
        self.tools.get(name).map(|t| t.clone())
    }
    
    pub fn list(&self) -> Vec<ToolInfo> {
        self.tools
            .iter()
            .map(|entry| entry.value().info())
            .collect()
    }
    
    pub fn process(
        &self,
        tool_name: &str,
        input: &[u8],
        protocol: Protocol,
    ) -> Result<Vec<u8>> {
        let tool = self.get(tool_name)
            .ok_or_else(|| ToolError::ToolError(format!("Tool '{}' not found", tool_name)))?;
        
        // Check if protocol is supported
        let info = tool.info();
        if !info.supported_protocols.contains(&protocol) {
            return Err(ToolError::ProtocolError(
                format!("Tool '{}' doesn't support {:?} protocol", tool_name, protocol)
            ));
        }
        
        tool.process(input, protocol)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Base implementation helper
pub struct BaseTool {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub supported_protocols: Vec<Protocol>,
}

impl BaseTool {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(), // Using workspace version
            capabilities: Vec::new(),
            supported_protocols: vec![Protocol::Json],
        }
    }
    
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        if !self.supported_protocols.contains(&protocol) {
            self.supported_protocols.push(protocol);
        }
        self
    }
    
    pub fn info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            capabilities: self.capabilities.clone(),
            supported_protocols: self.supported_protocols.clone(),
        }
    }
}