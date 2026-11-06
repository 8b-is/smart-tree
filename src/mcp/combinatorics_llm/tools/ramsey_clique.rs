//! Ramsey Clique Analysis tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, RamseyCliqueParams, run_ramsey_clique_analysis};
use serde_json::{json, Value};

pub struct RamseyCliqueTool;

impl ToolExecutor for RamseyCliqueTool {
    fn name(&self) -> &'static str {
        "ramsey_clique_analysis"
    }

    fn description(&self) -> &'static str {
        "Ramsey clique analysis via Bron-Kerbosch with degeneracy ordering. Finds maximum cliques, degeneracy cores, and stack vs heap performance comparison."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "num_vertices": { "type": "integer", "minimum": 1 },
                "edges": { "type": "array", "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 } },
                "target_clique_size": { "type": "integer", "minimum": 2 }
            },
            "required": ["num_vertices", "edges"]
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: RamseyCliqueParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_ramsey_clique_analysis(params, emitter).map_err(|e| e.to_string())
    }
}