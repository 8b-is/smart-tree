//! Kintsugi Consensus Ledger tool.

use crate::mcp_combinatorics_llm::tools::ToolExecutor;
use crate::mcp_combinatorics_llm::{LlamaModelCache, McpProgressEmitter, AppState, KintsugiLedger, fnv1a_64};
use serde_json::{json, Value};

pub struct KintsugiTool;

impl ToolExecutor for KintsugiTool {
    fn name(&self) -> &'static str {
        "gaia_consensus_ledger"
    }

    fn description(&self) -> &'static str {
        "Kintsugi append-only consensus ledger. Finalizes events with FNV-1a 64-bit memory seal. Each event contains lap index, consensus payload, sector ID, timestamp, and hash chain. Requires full AppState for ledger access."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "lap": { "type": "integer", "minimum": 0 },
                "payload": { "type": "object" },
                "mem8_state": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 }, "description": "Raw memory state bytes for seal computation" },
                "sector_id": { "type": "string", "description": "Optional sector override (defaults to local node ID)" }
            },
            "required": ["lap", "payload"]
        })
    }

    fn requires_full_state(&self) -> bool {
        true
    }

    fn execute(
        &self,
        args: Value,
        _model_cache: &LlamaModelCache,
        _emitter: &McpProgressEmitter,
        state: Option<&AppState>,
    ) -> crate::mcp_combinatorics_llm::tools::ToolResult {
        let state = state.ok_or("gaia_consensus_ledger requires full AppState")?;
        let lap = args["lap"].as_u64().ok_or("Missing lap")?;
        let payload = args["payload"].clone();

        // Use provided mem8_state or derive from wave memory
        let mem8_state: Vec<u8> = args["mem8_state"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|b| b as u8)).collect())
            .unwrap_or_else(|| vec![0u8; 1024]);

        let sector_id = args["sector_id"]
            .as_str()
            .unwrap_or(&state.gossip_bus.node_id)
            .to_string();

        let mut ledger = state
            .wave_ledger
            .write()
            .map_err(|_| "Ledger locked")?;

        let event = ledger.finalize_event(lap, payload, &mem8_state, &sector_id);
        Ok(json!(event))
    }
}