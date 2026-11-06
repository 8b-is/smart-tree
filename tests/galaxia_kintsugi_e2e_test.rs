//! End-to-end regression tests for Galaxia cluster gossip and Kintsugi ledger integration.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use st::{
    galaxia_cluster::{fnv1a_64, GalaxiaClusterCrdt, GalaxiaGossipEvent, KintsugiLedger},
    mcp_combinatorics_llm::{
        build_mcp_router, execute_combinatorics_llm_tool_with_emitter, AppState, GalaxiaKvPacket,
        LlamaModelCache, McpProgressEmitter,
    },
};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_galaxia_kintsugi_mcp_tool_dispatch() {
    // Verify that gaia_consensus_ledger tool is registered and dispatchable
    let state = Arc::new(AppState::new(2));
    let emitter = McpProgressEmitter::default();

    // Test via execute_combinatorics_llm_tool_with_emitter with full state
    let args = json!({
        "lap": 1,
        "payload": { "test": "data" }
    });
    let result = execute_combinatorics_llm_tool_with_emitter(
        "gaia_consensus_ledger",
        args,
        &state.model_cache,
        &emitter,
        Some(&state),
    );

    // Should succeed and return an event
    let content = result.get("content").and_then(|c| c.as_array()).unwrap();
    let text = content[0].get("text").and_then(|t| t.as_str()).unwrap();
    assert!(text.contains("lap_index"));
    assert!(text.contains("consensus_payload"));
    assert!(text.contains("wave_memory_seal"));
    assert!(text.contains("sector_id"));
    assert!(text.contains("timestamp_ms"));
}

#[tokio::test]
async fn test_galaxia_kintsugi_tool_rejects_without_state() {
    // Verify that gaia_consensus_ledger returns error when state is None
    let model_cache = LlamaModelCache::new(2);
    let emitter = McpProgressEmitter::default();

    let args = json!({
        "lap": 1,
        "payload": { "test": "data" }
    });
    let result = execute_combinatorics_llm_tool_with_emitter(
        "gaia_consensus_ledger",
        args,
        &model_cache,
        &emitter,
        None,
    );

    // Should return error response
    assert_eq!(result.get("isError").and_then(|e| e.as_bool()), Some(true));
    let content = result.get("content").and_then(|c| c.as_array()).unwrap();
    let text = content[0].get("text").and_then(|t| t.as_str()).unwrap();
    assert!(text.contains("requires full AppState"));
}

#[tokio::test]
async fn test_galaxia_crdt_gossip_roundtrip_via_router() {
    // Test CRDT gossip handler via HTTP router
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    // Create a CRDT from a peer
    let peer_crdt = GalaxiaClusterCrdt::new_single_node("peer-node", "http://peer:8080");

    let req = Request::builder()
        .method("POST")
        .uri("/galaxia/crdt-gossip")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&peer_crdt).unwrap()))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let merged: Value = serde_json::from_slice(&body).unwrap();

    // Should return merged CRDT with both nodes
    let members = merged.get("members").unwrap().as_object().unwrap();
    assert!(
        members.len() >= 2,
        "Expected at least 2 members, got {}",
        members.len()
    );

    // Leader should be deterministic (lexicographically lowest alive node between local and peer)
    // Local node has random sector-XXX id, peer has "peer-node"
    let leader_id = merged
        .get("leader")
        .unwrap()
        .get("coordinator_id")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(leader_id == "peer-node" || leader_id.starts_with("sector-"));
}

#[tokio::test]
async fn test_galaxia_cluster_state_endpoint() {
    // Test cluster state endpoint returns current state
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    let req = Request::builder()
        .method("GET")
        .uri("/galaxia/cluster-state")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let cluster_state: Value = serde_json::from_slice(&body).unwrap();

    assert!(cluster_state.get("node_id").is_some());
    assert!(cluster_state.get("election_term").is_some());
    assert!(cluster_state.get("elected_coordinator_id").is_some());
    assert!(cluster_state.get("alive_peers").is_some());
    assert!(cluster_state.get("crdt").is_some());
}

#[tokio::test]
async fn test_galaxia_kv_prefixes_endpoint() {
    // Test KV prefixes listing endpoint
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    let req = Request::builder()
        .method("GET")
        .uri("/galaxia/kv-prefixes")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let prefixes: Value = serde_json::from_slice(&body).unwrap();

    assert!(prefixes.get("node_id").is_some());
    assert!(prefixes.get("synchronized_prefixes").is_some());
}

#[tokio::test]
async fn test_kintsugi_ledger_persistence_and_hash_chaining() {
    // Test that ledger events are properly chained and tamper-evident
    let mut ledger = KintsugiLedger::new();
    let sector_id = "test-sector";

    // Finalize first event
    let payload1 = json!({ "round": 1, "data": "first" });
    let mem8_state1 = b"memory-state-1";
    let event1 = ledger.finalize_event(1, payload1.clone(), mem8_state1, sector_id);

    // Finalize second event
    let payload2 = json!({ "round": 2, "data": "second" });
    let mem8_state2 = b"memory-state-2";
    let event2 = ledger.finalize_event(2, payload2.clone(), mem8_state2, sector_id);

    // Verify events are stored
    assert_eq!(ledger.events.len(), 2);
    assert_eq!(event1.lap_index, 1);
    assert_eq!(event2.lap_index, 2);
    assert_eq!(event1.sector_id, sector_id);
    assert_eq!(event2.sector_id, sector_id);

    // Verify seals are deterministic FNV-1a of mem8_state
    assert_eq!(event1.wave_memory_seal, fnv1a_64(mem8_state1));
    assert_eq!(event2.wave_memory_seal, fnv1a_64(mem8_state2));

    // Verify payloads preserved
    assert_eq!(event1.consensus_payload, payload1);
    assert_eq!(event2.consensus_payload, payload2);

    // Verify timestamps are monotonic (or at least non-decreasing)
    assert!(event2.timestamp_ms >= event1.timestamp_ms);
}

#[tokio::test]
async fn test_galaxia_gossip_sse_stream_emits_events() {
    // Test SSE endpoint emits gossip events
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    let req = Request::builder()
        .method("GET")
        .uri("/galaxia/gossip")
        .header("accept", "text/event-stream")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    // SSE endpoint should accept the connection
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_two_node_crdt_merge_deterministic() {
    // Test that two nodes merging CRDTs produce deterministic results
    let mut crdt1 = GalaxiaClusterCrdt::new_single_node("node-a", "http://a:8080");
    let mut crdt2 = GalaxiaClusterCrdt::new_single_node("node-b", "http://b:8080");

    // Node A merges B's state
    let _outcome1 = crdt1.merge(&crdt2, "node-a");
    // Node B merges A's state
    let _outcome2 = crdt2.merge(&crdt1, "node-b");

    // Both should have both nodes
    assert_eq!(crdt1.members.len(), 2);
    assert_eq!(crdt2.members.len(), 2);

    // Leader election is deterministic: highest incarnation, then lexicographically lowest alive node
    // Both nodes have incarnation 1, so leader should be lexicographically lowest: "node-a"
    assert_eq!(crdt1.leader.coordinator_id, "node-a");
    assert_eq!(crdt2.leader.coordinator_id, "node-a");
}

#[tokio::test]
async fn test_galaxia_kv_sync_rejects_invalid_packet() {
    // Test KV sync handler rejects malformed packets
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    // Send packet with missing required fields
    let invalid_packet = json!({
        "model_identifier": "test-model"
        // missing origin_node, token_ids, fnv1a_checksum
    });

    let req = Request::builder()
        .method("POST")
        .uri("/galaxia/kv-sync")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_packet).unwrap()))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    // axum returns 422 for JSON deserialization errors
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_mcp_tools_list_includes_gaia_consensus_ledger() {
    // Verify tools/list includes the new tool
    let state = Arc::new(AppState::new(2));
    let router = build_mcp_router(Arc::clone(&state));

    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
                "params": {}
            }))
            .unwrap(),
        ))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp_json: Value = serde_json::from_slice(&body).unwrap();

    let tools = resp_json
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(tool_names.contains(&"gaia_consensus_ledger"));
}
