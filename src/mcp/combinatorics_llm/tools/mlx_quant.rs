//! MLX Quantization tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, MlxQuantParams, run_mlx_quant};
use serde_json::{json, Value};

pub struct MlxQuantTool;

impl ToolExecutor for MlxQuantTool {
    fn name(&self) -> &'static str {
        "mlx_quant"
    }

    fn description(&self) -> &'static str {
        "Apple Silicon MLX quantization (affine, mxfp4, mxfp8, nvfp4). Quantizes weight matrices using Metal-accelerated MLX kernels. Returns quantized data and error metrics."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "weights_path": { "type": "string" },
                "scheme": { "type": "string", "enum": ["affine", "mxfp4", "mxfp8", "nvfp4"], "default": "affine" },
                "group_size": { "type": "integer", "default": 32, "minimum": 1 },
                "output_path": { "type": "string" }
            },
            "required": ["weights_path"]
        })
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        emitter: &McpProgressEmitter,
        _state: Option<&crate::mcp_combinatorics_llm::AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let params: MlxQuantParams = serde_json::from_value(args).map_err(|e| e.to_string())?;
        run_mlx_quant(params, emitter).map_err(|e| e.to_string())
    }
}