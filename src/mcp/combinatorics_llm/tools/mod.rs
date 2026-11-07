//! Per-tool MCP executor registry.
//!
//! Each combinatorics/LLM tool implements `ToolExecutor` and registers itself.
//! The dispatcher looks up the tool by name and delegates execution.

pub mod szemeredi;
pub mod erdos_renyi;
pub mod erdos_steiner;
pub mod moser_tardos;
pub mod ramsey_clique;
pub mod mlx_quant;
pub mod llama_rust;
pub mod ultra_zeno;
pub mod gaia_omni;
pub mod ternary_deck;
pub mod kintsugi;

use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter};
use crate::mcp_combinatorics_llm::AppState;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Result of a tool execution.
pub type ToolResult = Result<Value, String>;

/// Trait for executing a single MCP tool.
///
/// Each tool handles its own argument parsing, validation, and execution.
/// The registry manages the mapping from tool name to executor.
pub trait ToolExecutor: Send + Sync {
    /// Unique tool name (must match MCP tool registry).
    fn name(&self) -> &'static str;

    /// Human-readable description for `tools/list`.
    fn description(&self) -> &'static str;

    /// JSON Schema for the tool's input arguments.
    fn input_schema(&self) -> Value;

    /// Whether this tool requires full `AppState` (not just `LlamaModelCache`).
    /// Tools that need cluster/ledger access return `true`.
    fn requires_full_state(&self) -> bool {
        false
    }

    /// Execute the tool with the given arguments.
    ///
    /// - `args`: Raw JSON arguments from the MCP call.
    /// - `model_cache`: The model cache (always available).
    /// - `emitter`: Progress emitter for streaming updates.
    /// - `state`: Full `AppState` if `requires_full_state()` is true, else `None`.
    fn execute(
        &self,
        args: Value,
        model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        state: Option<&AppState>,
    ) -> ToolResult;
}

/// Registry of all available tools.
pub struct ToolRegistry {
    tools: HashMap<&'static str, Arc<dyn ToolExecutor>>,
}

impl ToolRegistry {
    /// Create a new registry and register all built-in tools.
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register_builtin_tools();
        registry
    }

    /// Register all built-in combinatorics/LLM tools.
    fn register_builtin_tools(&mut self) {
        // Combinatorial solvers
        self.register(Box::new(szemeredi::SzemerediTool));
        self.register(Box::new(erdos_renyi::ErdosRenyiTool));
        self.register(Box::new(erdos_steiner::ErdosSteinerTool));
        self.register(Box::new(moser_tardos::MoserTardosTool));
        self.register(Box::new(ramsey_clique::RamseyCliqueTool));

        // Quantization & LLM
        self.register(Box::new(mlx_quant::MlxQuantTool));
        self.register(Box::new(llama_rust::LlamaRustTool));

        // Advanced cognitive engines
        self.register(Box::new(ultra_zeno::UltraZenoTool));
        self.register(Box::new(gaia_omni::GaiaOmniTool));

        // Ternary verification
        self.register(Box::new(ternary_deck::TernaryDeckTool));

        // Cluster & consensus (require full state)
        self.register(Box::new(kintsugi::KintsugiTool));
    }

    /// Register a tool executor.
    pub fn register(&mut self, tool: Box<dyn ToolExecutor>) {
        let name = tool.name();
        self.tools.insert(name, Arc::from(tool));
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolExecutor>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names.
    pub fn names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    /// Get tool schemas for `tools/list`.
    pub fn list_tools(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.input_schema(),
                })
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a tool by name using the registry.
pub fn execute_tool(
    registry: &ToolRegistry,
    name: &str,
    args: Value,
    model_cache: &LlamaModelCache,
    emitter: &McpProgressEmitter,
    state: Option<&AppState>,
) -> ToolResult {
    let tool = registry
        .get(name)
        .ok_or_else(|| format!("Unknown combinatorics/LLM tool: {}", name))?;

    if tool.requires_full_state() && state.is_none() {
        return Err(format!(
            "Tool '{}' requires full AppState (not available in this context)",
            name
        ));
    }

    tool.execute(args, model_cache, emitter, state)
}