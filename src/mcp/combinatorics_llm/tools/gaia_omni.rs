//! Gaia Omni Avatar Mode tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter};
use crate::gaia_omni::{GaiaOmniParams, run_gaia_omni_mode};
use serde_json::{json, Value};

pub struct GaiaOmniTool;

impl ToolExecutor for GaiaOmniTool {
    fn name(&self) -> &'static str {
        "gaia_omni_avatar_mode"
    }

    fn description(&self) -> &'static str {
        "Gaia Omni planetary superorganism mode with control/holonomy feedback. Runs closed-loop adaptive control with PID gain projection into Jury stability polytope. Returns avatar state and control metrics."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "model_path": { "type": "string" },
                "control_params": {
                    "type": "object",
                    "properties": {
                        "kp": { "type": "number", "default": 1.4 },
                        "ki": { "type": "number", "default": 0.45 },
                        "kd": { "type": "number", "default": 0.12 }
                    }
                },
                "max_steps": { "type": "integer", "default": 100 }
            },
            "required": ["model_path"]
        })
    }

    fn execute(
        &self,
        args: Value,
        model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: GaiaOmniParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_gaia_omni_mode(model_cache, params, emitter).map_err(|e| e.to_string())
    }
}