//! Erdős-Steiner Pruning tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, ErdosSteinerParams, run_erdos_steiner_pruning};
use serde_json::{json, Value};

pub struct ErdosSteinerTool;

impl ToolExecutor for ErdosSteinerTool {
    fn name(&self) -> &'static str {
        "erdos_steiner_pruning"
    }

    fn description(&self) -> &'static str {
        "Erdős-Steiner 2-approximation minimum Steiner tree pruning. Finds near-minimal tree connecting focal terminals via Voronoi clustering on shortest-path metric."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "num_vertices": { "type": "integer", "minimum": 1 },
                "edges": { "type": "array", "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 } },
                "terminals": { "type": "array", "items": { "type": "integer" } },
                "tokens_per_symbol": { "type": "integer", "default": 120 }
            },
            "required": ["num_vertices", "edges", "terminals"]
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: ErdosSteinerParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_erdos_steiner_pruning(params, emitter).map_err(|e| e.to_string())
    }
}