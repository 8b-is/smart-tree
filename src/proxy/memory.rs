//! 🧠 Memory Proxy - Scoped conversation history for LLMs
//!
//! This module adds memory capabilities to the LLM proxy, allowing for
//! persistent, scoped conversation history.
//!
//! "A proxy that remembers is a proxy that cares!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmProxy, LlmRequest, LlmResponse, LlmRole};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

/// 🧠 Scoped memory for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationScope {
    pub id: String,
    pub messages: Vec<LlmMessage>,
    pub last_updated: chrono::DateTime<Utc>,
}

/// 🗄️ Persistent memory storage for the proxy
pub struct ProxyMemory {
    storage_path: PathBuf,
    scopes: HashMap<String, ConversationScope>,
}

impl ProxyMemory {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let storage_path = Path::new(&home)
            .join(".mem8")
            .join("proxy_memory.json");

        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut memory = Self {
            storage_path,
            scopes: HashMap::new(),
        };

        memory.load()?;
        Ok(memory)
    }

    pub fn get_scope(&self, scope_id: &str) -> Option<&ConversationScope> {
        self.scopes.get(scope_id)
    }

    pub fn update_scope(&mut self, scope_id: &str, messages: Vec<LlmMessage>) -> Result<()> {
        let scope = self.scopes.entry(scope_id.to_string()).or_insert_with(|| ConversationScope {
            id: scope_id.to_string(),
            messages: Vec::new(),
            last_updated: Utc::now(),
        });

        scope.messages.extend(messages);
        scope.last_updated = Utc::now();
        
        // Limit history to last 20 messages to keep it manageable
        if scope.messages.len() > 20 {
            scope.messages = scope.messages.split_off(scope.messages.len() - 20);
        }

        self.save()?;
        Ok(())
    }

    pub fn clear_scope(&mut self, scope_id: &str) -> Result<()> {
        self.scopes.remove(scope_id);
        self.save()?;
        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        if self.storage_path.exists() {
            let content = fs::read_to_string(&self.storage_path)?;
            self.scopes = serde_json::from_str(&content).unwrap_or_default();
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.scopes)?;
        fs::write(&self.storage_path, content)?;
        Ok(())
    }
}

/// 🛠️ Enhanced proxy with memory support
pub struct MemoryProxy {
    pub inner: LlmProxy,
    pub memory: ProxyMemory,
}

impl MemoryProxy {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: LlmProxy::default(),
            memory: ProxyMemory::new()?,
        })
    }

    pub async fn complete_with_memory(
        &mut self,
        provider_name: &str,
        scope_id: &str,
        mut request: LlmRequest,
    ) -> Result<LlmResponse> {
        // 1. Retrieve history from scope
        if let Some(scope) = self.memory.get_scope(scope_id) {
            // Prepend history to current messages (after system message if present)
            let mut new_messages = Vec::new();
            
            // Keep existing system message at the top
            if let Some(system_msg) = request.messages.iter().find(|m| m.role == LlmRole::System).cloned() {
                new_messages.push(system_msg);
            }
            
            // Add history
            for msg in &scope.messages {
                if msg.role != LlmRole::System {
                    new_messages.push(msg.clone());
                }
            }
            
            // Add current user message(s)
            for msg in request.messages {
                if msg.role != LlmRole::System {
                    new_messages.push(msg);
                }
            }
            
            request.messages = new_messages;
        }

        // 2. Call the inner proxy
        let response = self.inner.complete(provider_name, request.clone()).await?;

        // 3. Update memory with the new exchange
        let mut new_history = Vec::new();
        // Add the last user message
        if let Some(last_user_msg) = request.messages.iter().rev().find(|m| m.role == LlmRole::User) {
            new_history.push(last_user_msg.clone());
        }
        // Add the assistant response
        new_history.push(LlmMessage {
            role: LlmRole::Assistant,
            content: response.content.clone(),
        });

        self.memory.update_scope(scope_id, new_history)?;

        Ok(response)
    }
}
