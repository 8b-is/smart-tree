//! Moser-Tardos Constructive LLL Splitting tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, MoserTardosParams, run_moser_tardos_splitting};
use serde_json::{json, Value};

pub struct MoserTardosTool;

impl ToolExecutor for MoserTardosTool {
    fn name(&self) -> &'static str {
        "moser_tardos_splitting"
    }

    fn description(&self) -> &'static str {
        "Moser-Tardos algorithmic Lovász Local Lemma. Resamples bad events (clauses) until a satisfying assignment is found. Returns assignment, resample counts, and witness DAG."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "num_variables": { "type": "integer", "minimum": 1 },
                "bad_events": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "max_resamples": { "type": "integer", "default": 10000, "minimum": 1 }
            },
            "required": ["num_variables", "bad_events"]
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: MoserTardosParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_moser_tardos_splitting(params, emitter).map_err(|e| e.to_string())
    }
}