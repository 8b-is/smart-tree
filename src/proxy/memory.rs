//! 🧠 Memory Proxy - Scoped conversation history for LLMs
//!
//! This module adds memory capabilities to the LLM proxy, allowing for
//! persistent, scoped conversation history.
//!
//! "A proxy that remembers is a proxy that cares!" - The Cheet 😺

use crate::mem8::record_store::{memory_dir, RecordStore};
use crate::proxy::{LlmMessage, LlmProxy, LlmRequest, LlmResponse, LlmRole};
use anyhow::{ensure, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SCOPE_PREFIX: &str = "scope/v1/";
const MIGRATION_KEY: &str = "migration/proxy-json-v1";

/// 🧠 Scoped memory for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationScope {
    pub id: String,
    pub messages: Vec<LlmMessage>,
    pub last_updated: chrono::DateTime<Utc>,
}

/// 🗄️ Persistent memory storage for the proxy
pub struct ProxyMemory {
    store: RecordStore,
    scopes: HashMap<String, ConversationScope>,
}

impl ProxyMemory {
    pub fn new() -> Result<Self> {
        Self::open(&memory_dir()?)
    }

    /// Open compressed conversation records, importing the legacy JSON once.
    /// The original file is preserved; interrupted imports resume without
    /// overwriting newer records or resurrecting cleared conversations.
    pub fn open(directory: &Path) -> Result<Self> {
        let mut store = RecordStore::open_memory(&directory.join("proxy_memory.m8"))?;
        if store.get::<bool>(MIGRATION_KEY)? != Some(true) {
            let legacy_path = directory.join("proxy_memory.json");
            let legacy = match fs::read(&legacy_path) {
                Ok(content) => {
                    serde_json::from_slice::<HashMap<String, ConversationScope>>(&content)
                        .with_context(|| {
                            format!("Invalid conversation memory: {}", legacy_path.display())
                        })?
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
                Err(error) => return Err(error).context("Cannot read legacy conversation memory"),
            };
            for (id, scope) in legacy {
                ensure!(
                    id == scope.id,
                    "Conversation scope ID does not match its key"
                );
                let key = format!("{SCOPE_PREFIX}{id}");
                if store.get::<Option<ConversationScope>>(&key)?.is_none() {
                    store.put(&key, &Some(scope))?;
                }
            }
            store.put(MIGRATION_KEY, &true)?;
        }
        let mut scopes = HashMap::new();
        for key in store.keys_with_prefix(SCOPE_PREFIX) {
            if let Some(Some(scope)) = store.get::<Option<ConversationScope>>(&key)? {
                ensure!(
                    key == format!("{SCOPE_PREFIX}{}", scope.id),
                    "Invalid conversation scope key"
                );
                scopes.insert(scope.id.clone(), scope);
            }
        }
        Ok(Self { store, scopes })
    }

    pub fn get_scope(&self, scope_id: &str) -> Option<&ConversationScope> {
        self.scopes.get(scope_id)
    }

    pub fn update_scope(&mut self, scope_id: &str, messages: Vec<LlmMessage>) -> Result<()> {
        let mut scope = self
            .scopes
            .get(scope_id)
            .cloned()
            .unwrap_or_else(|| ConversationScope {
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

        self.store
            .put(&format!("{SCOPE_PREFIX}{scope_id}"), &Some(&scope))?;
        self.scopes.insert(scope_id.to_string(), scope);
        Ok(())
    }

    pub fn clear_scope(&mut self, scope_id: &str) -> Result<()> {
        self.store
            .forget::<ConversationScope>(&format!("{SCOPE_PREFIX}{scope_id}"))?;
        self.scopes.remove(scope_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(content: &str) -> LlmMessage {
        LlmMessage {
            role: LlmRole::User,
            content: content.into(),
        }
    }

    #[test]
    fn conversations_migrate_once_and_clears_survive_restart() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = HashMap::from([(
            "family".to_string(),
            ConversationScope {
                id: "family".into(),
                messages: vec![message("lunar homework with my daughter")],
                last_updated: Utc::now(),
            },
        )]);
        let bytes = serde_json::to_vec(&legacy).unwrap();
        let legacy_path = temp.path().join("proxy_memory.json");
        fs::write(&legacy_path, &bytes).unwrap();
        let mut memory = ProxyMemory::open(temp.path()).unwrap();
        assert!(memory.get_scope("family").unwrap().messages[0]
            .content
            .contains("daughter"));
        memory
            .update_scope("work", vec![message("independent conversation")])
            .unwrap();
        memory.clear_scope("family").unwrap();
        memory.store.compact().unwrap();
        drop(memory);
        let memory = ProxyMemory::open(temp.path()).unwrap();
        assert!(memory.get_scope("family").is_none());
        assert_eq!(memory.get_scope("work").unwrap().messages.len(), 1);
        assert_eq!(fs::read(legacy_path).unwrap(), bytes);
        let native = fs::read(temp.path().join("proxy_memory.m8")).unwrap();
        assert_eq!(&native[8..12], &0x4d454d38u32.to_le_bytes());
    }

    #[test]
    fn partial_import_preserves_newer_scopes_and_tombstones() {
        let temp = tempfile::tempdir().unwrap();
        let old_scope = |id: &str| ConversationScope {
            id: id.into(),
            messages: vec![message("old")],
            last_updated: Utc::now(),
        };
        fs::write(
            temp.path().join("proxy_memory.json"),
            serde_json::to_vec(&HashMap::from([
                ("updated", old_scope("updated")),
                ("cleared", old_scope("cleared")),
                ("pending", old_scope("pending")),
            ]))
            .unwrap(),
        )
        .unwrap();
        let mut store = RecordStore::open_memory(&temp.path().join("proxy_memory.m8")).unwrap();
        let mut newer = old_scope("updated");
        newer.messages = vec![message("new")];
        store.put("scope/v1/updated", &Some(newer)).unwrap();
        store
            .put("scope/v1/cleared", &None::<ConversationScope>)
            .unwrap();
        drop(store);
        let memory = ProxyMemory::open(temp.path()).unwrap();
        assert_eq!(
            memory.get_scope("updated").unwrap().messages[0].content,
            "new"
        );
        assert!(memory.get_scope("cleared").is_none());
        assert!(memory.get_scope("pending").is_some());
    }

    #[test]
    fn malformed_legacy_memory_is_not_silently_replaced() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("proxy_memory.json");
        fs::write(&path, b"{broken").unwrap();
        assert!(ProxyMemory::open(temp.path()).is_err());
        assert_eq!(fs::read(path).unwrap(), b"{broken");
    }

    #[test]
    fn retention_is_scoped_and_failed_writes_do_not_publish() {
        let temp = tempfile::tempdir().unwrap();
        let mut memory = ProxyMemory::open(temp.path()).unwrap();
        memory
            .update_scope("first", (0..25).map(|n| message(&n.to_string())).collect())
            .unwrap();
        memory
            .update_scope("second", vec![message("keep")])
            .unwrap();
        assert_eq!(memory.get_scope("first").unwrap().messages.len(), 20);
        assert_eq!(memory.get_scope("first").unwrap().messages[0].content, "5");
        memory.store.make_read_only_for_test().unwrap();
        assert!(memory
            .update_scope("second", vec![message("uncommitted")])
            .is_err());
        assert_eq!(memory.get_scope("second").unwrap().messages.len(), 1);
        assert!(memory.clear_scope("second").is_err());
        assert!(memory.get_scope("second").is_some());
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

    /// Create a new MemoryProxy with auto-detection of local LLMs (Ollama, LM Studio)
    pub async fn with_local_detection() -> Result<Self> {
        Ok(Self {
            inner: LlmProxy::with_local_detection().await,
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
            if let Some(system_msg) = request
                .messages
                .iter()
                .find(|m| m.role == LlmRole::System)
                .cloned()
            {
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
        if let Some(last_user_msg) = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == LlmRole::User)
        {
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
