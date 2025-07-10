use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub tool_name: String,
    pub importance_score: f32,
    pub last_accessed: u64,
    pub access_count: u64,
    pub compressed_data: Vec<u8>,
}

impl Context {
    pub fn new(tool_name: String, data: Vec<u8>) -> Self {
        let id = blake3::hash(&data).to_hex().to_string();
        Self {
            id,
            tool_name,
            importance_score: 1.0,
            last_accessed: 0,
            access_count: 0,
            compressed_data: data,
        }
    }
    
    pub fn update_importance(&mut self, delta: f32) {
        self.importance_score = (self.importance_score + delta).clamp(0.0, 100.0);
        self.access_count += 1;
    }
    
    pub fn should_offload(&self, threshold: f32) -> bool {
        self.importance_score < threshold
    }
}

pub struct ContextManager {
    contexts: dashmap::DashMap<String, Arc<Context>>,
    max_memory: usize,
    current_memory: std::sync::atomic::AtomicUsize,
}

impl ContextManager {
    pub fn new(max_memory: usize) -> Self {
        Self {
            contexts: dashmap::DashMap::new(),
            max_memory,
            current_memory: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    pub fn add_context(&self, context: Context) -> Result<(), String> {
        let size = context.compressed_data.len();
        let current = self.current_memory.load(std::sync::atomic::Ordering::Relaxed);
        
        if current + size > self.max_memory {
            return Err("Memory limit exceeded".to_string());
        }
        
        self.current_memory.fetch_add(size, std::sync::atomic::Ordering::Relaxed);
        self.contexts.insert(context.id.clone(), Arc::new(context));
        Ok(())
    }
    
    pub fn get_context(&self, id: &str) -> Option<Arc<Context>> {
        self.contexts.get(id).map(|c| c.clone())
    }
    
    pub fn offload_contexts(&self, threshold: f32) -> Vec<Context> {
        let mut offloaded = Vec::new();
        
        self.contexts.retain(|_, context| {
            if context.should_offload(threshold) {
                offloaded.push(context.as_ref().clone());
                self.current_memory.fetch_sub(
                    context.compressed_data.len(),
                    std::sync::atomic::Ordering::Relaxed
                );
                false
            } else {
                true
            }
        });
        
        offloaded
    }
}