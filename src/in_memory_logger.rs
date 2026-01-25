//! In-Memory Logger for Smart Tree
//!
//! A custom tracing layer that captures log events and stores them in a
//! thread-safe, capped in-memory buffer. This allows the daemon to expose
//! recent logs via an API endpoint.

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::Subscriber;
use tracing_subscriber::Layer;

const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct InMemoryLogStore {
    pub entries: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl Default for InMemoryLogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLogStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
        }
    }
}

pub struct InMemoryLoggerLayer {
    store: InMemoryLogStore,
}

impl InMemoryLoggerLayer {
    pub fn new(store: InMemoryLogStore) -> Self {
        Self { store }
    }
}

// A visitor to format the log message.
struct LogVisitor {
    message: String,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}

impl<S> Layer<S> for InMemoryLoggerLayer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = LogVisitor { message: String::new() };
        event.record(&mut visitor);

        let metadata = event.metadata();
        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: metadata.level().to_string(),
            message: visitor.message,
        };

        if let Ok(mut entries) = self.store.entries.lock() {
            if entries.len() >= MAX_LOG_ENTRIES {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }
}
