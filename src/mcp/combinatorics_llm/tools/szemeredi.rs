//! Szemerédi Regularity Compression tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, SzemerediParams, run_szemeredi_compression};
use serde_json::{json, Value};

pub struct SzemerediTool;

impl ToolExecutor for SzemerediTool {
    fn name(&self) -> &'static str {
        "szemeredi_compression"
    }

    fn description(&self) -> &'static str {
        "Szemerédi Regularity Lemma graph compression. Partitions vertices into ε-regular pairs with energy increment. Returns partition, irregular pairs, and energy metrics."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "num_vertices": { "type": "integer", "minimum": 1 },
                "edges": { "type": "array", "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 } },
                "epsilon": { "type": "number", "default": 0.1, "minimum": 0.001, "maximum": 0.5 },
                "min_parts": { "type": "integer", "default": 4, "minimum": 1 }
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
        let params: SzemerediParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_szemeredi_compression(params, emitter).map_err(|e| e.to_string())
    }
}