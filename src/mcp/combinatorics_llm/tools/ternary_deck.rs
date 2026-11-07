//! Ternary Executable Deck tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter};
use crate::ternary_deck::{TernaryDeckParams, run_ternary_executable_deck};
use serde_json::{json, Value};

pub struct TernaryDeckTool;

impl ToolExecutor for TernaryDeckTool {
    fn name(&self) -> &'static str {
        "ternary_executable_deck"
    }

    fn description(&self) -> &'static str {
        "5-card ternary verification deck: (1) 4-trit vs 5-trit base-3 frame packing, (2) drift-free integer frame timing, (3) u64 ternary quantizer MSE optimization, (4) prerequisite DAG critical-path scheduling, (5) discrete-time proportional stability & Jury limit-cycle analysis."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "width": { "type": "integer", "default": 3840, "minimum": 1 },
                "height": { "type": "integer", "default": 2160, "minimum": 1 },
                "fps": { "type": "integer", "default": 60, "minimum": 1 },
                "sample_render_ns": { "type": "integer", "default": 16500000, "minimum": 0 },
                "plant_a": { "type": "number", "default": 0.90 },
                "plant_b": { "type": "number", "default": 0.25 },
                "gain_k": { "type": "number", "default": 2.40 }
            }
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: TernaryDeckParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_ternary_executable_deck(params).map_err(|e| e.to_string())
    }
}