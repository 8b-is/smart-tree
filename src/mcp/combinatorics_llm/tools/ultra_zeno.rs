//! UltraZeno M1-M2-M3 Information Bottleneck tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter};
use crate::ultra_zeno::{UltraZenoParams, run_ultra_zeno_engine};
use serde_json::{json, Value};

pub struct UltraZenoTool;

impl ToolExecutor for UltraZenoTool {
    fn name(&self) -> &'static str {
        "ultra_zeno_structural_compression"
    }

    fn description(&self) -> &'static str {
        "UltraZeno M1-M2-M3 Information Bottleneck structural compression. Degree-sorted dyadic partitioning with MDL objective. M1=entropy, M2=mutual info, M3=interaction complexity. Returns cognitive mode and partition hierarchy."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "weights": { "type": "array", "items": { "type": "number" } },
                "max_partitions": { "type": "integer", "default": 8, "minimum": 1 },
                "epsilon": { "type": "number", "default": 0.1 }
            },
            "required": ["weights"]
        })
    }

    fn execute(
        &self,
        args: Value,
        model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: UltraZenoParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_ultra_zeno_engine(model_cache, params, emitter).map_err(|e| e.to_string())
    }
}