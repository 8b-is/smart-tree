//! Llama Rust tool for local LLM inference.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, LlamaRustParams, run_llama_rust};
use serde_json::{json, Value};

pub struct LlamaRustTool;

impl ToolExecutor for LlamaRustTool {
    fn name(&self) -> &'static str {
        "llama_rust"
    }

    fn description(&self) -> &'static str {
        "Local LLM inference via llama-cpp-2. Singleflight LRU model cache with multi-sequence KV radix pool (8 primary + 24 auxiliary). Supports speculative decoding, continuous batching, and tree attention."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "model_path": { "type": "string" },
                "prompt": { "type": "string" },
                "n_predict": { "type": "integer", "default": 128, "minimum": 1 },
                "temperature": { "type": "number", "default": 0.7 },
                "top_k": { "type": "integer", "default": 40 },
                "top_p": { "type": "number", "default": 0.95 },
                "n_gpu_layers": { "type": "integer", "default": -1 },
                "speculative": { "type": "boolean", "default": false },
                "draft_model_path": { "type": "string" }
            },
            "required": ["model_path", "prompt"]
        })
    }

    fn execute(
        &self,
        args: Value,
        model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: LlamaRustParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_llama_rust(model_cache, params, emitter).map_err(|e| e.to_string())
    }
}