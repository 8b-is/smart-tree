//! Erdős-Rényi Phase Transition Scanning tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, ErdosRenyiParams, run_erdos_renyi_scanning};
use serde_json::{json, Value};

pub struct ErdosRenyiTool;

impl ToolExecutor for ErdosRenyiTool {
    fn name(&self) -> &'static str {
        "erdos_renyi_scanning"
    }

    fn description(&self) -> &'static str {
        "Erdős-Rényi random graph phase transition scanning. Detects giant component emergence, connectivity threshold, and subgraph counts across edge probabilities."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "num_vertices": { "type": "integer", "minimum": 1 },
                "edge_probabilities": { "type": "array", "items": { "type": "number" } },
                "trials_per_probability": { "type": "integer", "default": 10, "minimum": 1 }
            },
            "required": ["num_vertices", "edge_probabilities"]
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: ErdosRenyiParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_erdos_renyi_scanning(params, emitter).map_err(|e| e.to_string())
    }
}