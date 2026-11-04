use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
#[allow(unused_imports)]
use st::mcp_combinatorics_llm::{
    self, build_mcp_router, evaluate_jsonrpc_payload,
    kernels::{
        ffi_progress_trampoline, run_erdos_renyi_scanning, run_erdos_steiner_pruning,
        run_mlx_quant, run_moser_tardos_splitting, run_ramsey_clique_analysis,
        run_szemeredi_compression, BitSet, FfiCallbackState, StackBitMatrix, StackBitSet,
    },
    llama_cpp_2,
    model_cache::{KvForkMode, McpProgressEmitter},
    AppState, ErdosRenyiParams, ErdosSteinerParams, MlxQuantParams, MlxQuantScheme,
    MoserTardosParams, RamseyCliqueParams, SzemerediParams,
};
#[allow(unused_imports)]
use std::{collections::HashSet, ffi::CString, fs, sync::Arc};
use tokio::sync::mpsc;
use tower::ServiceExt;

// ============================================================================
// 1. MCP JSON-RPC Lifecycle, Batching & Axum SSE Transport Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_initialize_and_tool_registry() {
    let state = Arc::new(AppState::new(2));

    // 1. Verify `initialize` handshake
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let init_resp = evaluate_jsonrpc_payload(Arc::clone(&state), init_req, None)
        .await
        .expect("initialize must return a response");
    assert_eq!(init_resp["result"]["protocolVersion"], "2025-03-26");

    // 2. Verify one-way notification returns None
    let notif_req = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    assert!(
        evaluate_jsonrpc_payload(Arc::clone(&state), notif_req, None)
            .await
            .is_none(),
        "MCP notifications must not emit a response"
    );

    // 3. Verify `tools/list` exposes all 7 tools
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    let list_resp = evaluate_jsonrpc_payload(state, list_req, None)
        .await
        .unwrap();
    let tools = list_resp["result"]["tools"].as_array().unwrap();
    let names: HashSet<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    for expected in [
        "szemeredi_compression",
        "erdos_steiner_pruning",
        "moser_tardos_splitting",
        "erdos_renyi_scanning",
        "ramsey_clique_analysis",
        "mlx_quant",
        "llama_rust",
    ] {
        assert!(names.contains(expected), "Missing tool schema: {expected}");
    }
}

#[tokio::test]
async fn test_axum_sse_progress_stream_end_to_end() {
    let state = Arc::new(AppState::new(2));
    let app = build_mcp_router(state);

    let rpc_body = json!({
        "jsonrpc": "2.0",
        "id": "sse-test-1",
        "method": "tools/call",
        "params": {
            "name": "erdos_renyi_scanning",
            "arguments": { "n": 120, "p": 0.08, "seed": 42 },
            "_meta": { "progressToken": "er-token-99" }
        }
    });

    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "text/event-stream")
        .body(Body::from(rpc_body.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/event-stream"));

    let raw_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let sse_text = String::from_utf8_lossy(&raw_bytes);

    let events: Vec<Value> = sse_text
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .collect();

    // Must contain multiple `notifications/progress` frames followed by the final `result` frame
    assert!(
        events.len() >= 3,
        "Expected progress frames + final response, got {}",
        events.len()
    );

    let progress_frames: Vec<&Value> = events
        .iter()
        .filter(|e| e["method"] == "notifications/progress")
        .collect();
    assert!(!progress_frames.is_empty());
    assert_eq!(
        progress_frames[0]["params"]["progressToken"],
        json!("er-token-99")
    );

    let final_frame = events.last().unwrap();
    assert_eq!(final_frame["id"], json!("sse-test-1"));
    assert_eq!(final_frame["result"]["isError"], json!(false));
}

// ============================================================================
// 2. Combinatorial Solver Verification & Client Disconnect Cancellation
// ============================================================================

#[test]
fn test_szemeredi_compression_regularity_and_energy() {
    let n = 64;
    let mut edges = Vec::new();
    // Construct a dense complete bipartite graph K_{32, 32}
    for u in 0..32 {
        for v in 32..64 {
            edges.push((u, v));
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    let emitter = McpProgressEmitter::new(Some(tx), Some(json!("sz-prog")));

    let res = run_szemeredi_compression(
        SzemerediParams {
            num_vertices: n,
            edges,
            epsilon: 0.15,
            min_parts: 2,
        },
        &emitter,
    )
    .expect("Szemerédi compression should succeed");

    assert_eq!(res["status"], "compressed");
    assert!(res["energy_potential"].as_f64().unwrap() > 0.0);

    let mut progress_count = 0;
    while let Ok(frame) = rx.try_recv() {
        assert_eq!(frame["method"], "notifications/progress");
        progress_count += 1;
    }
    assert!(
        progress_count >= 2,
        "Expected initialized + refinement_step events"
    );
}

#[test]
fn test_erdos_steiner_pruning_and_disconnect_cancellation() {
    // 1. Verify leaf pruning, degree-2 contraction, and triangle metric dominance
    // Triangle (0, 1, w=2.0), (1, 2, w=2.0), direct (0, 2, w=9.0 -> dominated!)
    // Plus non-terminal leaf (2, 3, w=1.0) where 3 is not a terminal -> pruned!
    let params = ErdosSteinerParams {
        edges: vec![(0, 1, 2.0), (1, 2, 2.0), (0, 2, 9.0), (2, 3, 1.0)],
        terminals: vec![0, 1, 2],
        pruning_threshold: 10.0,
    };

    let res = run_erdos_steiner_pruning(params, &McpProgressEmitter::default()).unwrap();
    assert_eq!(res["pruned_leaf_vertices"], 1); // Vertex 3 removed
    assert_eq!(res["retained_edge_count"], 2); // Edge (0,2,9.0) dominated by 0->1->2 (cost 4.0)
    assert!((res["total_retained_weight"].as_f64().unwrap() - 4.0).abs() < 1e-9);

    // 2. Verify cooperative cancellation when SSE receiver drops mid-flight
    let (tx, rx) = mpsc::unbounded_channel();
    drop(rx); // Simulate immediate client SSE disconnect
    let dead_emitter = McpProgressEmitter::new(Some(tx), Some(json!("cancel-steiner")));

    let cancel_res = run_erdos_steiner_pruning(
        ErdosSteinerParams {
            edges: vec![(0, 1, 1.0), (1, 2, 1.0)],
            terminals: vec![0, 2],
            pruning_threshold: 5.0,
        },
        &dead_emitter,
    );
    assert!(cancel_res.is_err());
    assert!(cancel_res.unwrap_err().contains("Client disconnected"));
}

#[test]
fn test_moser_tardos_parallel_mis_disjointness_and_reproducibility() {
    // Construct a 3-CNF formula with 40 variables and 30 clauses
    let mut bad_events = Vec::new();
    for i in 1..=30 {
        let v1 = i as i32;
        let v2 = ((i % 40) + 1) as i32;
        let v3 = (((i + 7) % 40) + 1) as i32;
        bad_events.push(vec![v1, -v2, v3]);
    }

    let params1 = MoserTardosParams {
        num_variables: 40,
        bad_events: bad_events.clone(),
        max_resamples: 500,
    };
    let params2 = MoserTardosParams {
        num_variables: 40,
        bad_events: bad_events.clone(),
        max_resamples: 500,
    };

    let res1 = run_moser_tardos_splitting(params1, &McpProgressEmitter::default()).unwrap();
    let res2 = run_moser_tardos_splitting(params2, &McpProgressEmitter::default()).unwrap();

    assert_eq!(res1["status"], "converged");
    // Thread-schedule invariant PRNG guarantees identical assignments across runs
    assert_eq!(res1["assignment"], res2["assignment"]);

    // Verify that every parallel MIS layer contains strictly variable-disjoint clauses
    if let Some(layers) = res1["witness_parallel_layers"].as_array() {
        for layer in layers {
            let mut seen_vars = HashSet::new();
            for c_val in layer.as_array().unwrap() {
                let c_idx = c_val.as_u64().unwrap() as usize;
                for &lit in &bad_events[c_idx] {
                    let v = lit.unsigned_abs() as usize;
                    assert!(
                        seen_vars.insert(v),
                        "MIS invariant violated: variable {v} appeared twice in the same parallel layer"
                    );
                }
            }
        }
    }
}

#[test]
fn test_erdos_renyi_phase_transitions() {
    let sub = run_erdos_renyi_scanning(
        ErdosRenyiParams {
            n: 200,
            p: 0.001,
            seed: Some(99),
        },
        &McpProgressEmitter::default(),
    )
    .unwrap();
    assert_eq!(sub["regime"], "subcritical");

    let sup = run_erdos_renyi_scanning(
        ErdosRenyiParams {
            n: 200,
            p: 0.06,
            seed: Some(99),
        },
        &McpProgressEmitter::default(),
    )
    .unwrap();
    assert_eq!(sup["regime"], "connected_supercritical");
}

#[test]
fn test_ramsey_clique_stack_tiers_and_degeneracy() {
    // Test across N=1 (40 vertices) and N=2 (90 vertices) with a planted 6-clique on {0..5}
    for &num_vertices in &[40usize, 90usize] {
        let mut edges = Vec::new();
        for u in 0..6 {
            for v in (u + 1)..6 {
                edges.push((u, v));
            }
        }

        let res = run_ramsey_clique_analysis(
            RamseyCliqueParams {
                num_vertices,
                edges,
                k_clique: 6,
                l_independent: 10,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();

        assert_eq!(res["heap_allocations_in_search"], 0);
        assert_eq!(res["graph_degeneracy"], 5);
        assert_eq!(res["k_clique_found"], true);
        assert_eq!(res["l_independent_found"], true);
        assert_eq!(res["largest_clique_witness"], json!([0, 1, 2, 3, 4, 5]));
    }
}

// ============================================================================
// 3. MLX Quantization Checkpoint Inspection & C-FFI Callback Trampoline Test
// ============================================================================

#[test]
fn test_mlx_quant_ffi_trampoline_and_disconnect_abort() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let emitter = McpProgressEmitter::new(Some(tx), Some(json!("mlx-token")));

    let mut state = FfiCallbackState {
        emitter: &emitter,
        scheme: "mxfp4",
        bits: 4,
        group_size: 64,
        source_bytes: 1_000_000_000,
        cancelled: false,
    };

    let tensor_name = CString::new("model.layers.3.self_attn.q_proj.weight").unwrap();
    let rc = ffi_progress_trampoline(
        4,
        32,
        tensor_name.as_ptr(),
        32_000_000,
        (&mut state as *mut FfiCallbackState<'_>) as *mut libc::c_void,
    );
    assert_eq!(rc, 0, "Trampoline must return 0 when SSE stream is healthy");

    let frame = rx.try_recv().expect("Expected layer_quantized SSE frame");
    assert_eq!(frame["params"]["data"]["phase"], "layer_quantized");
    assert_eq!(frame["params"]["data"]["layer"], 4);
    assert!(
        frame["params"]["data"]["live_compression_ratio"]
            .as_f64()
            .unwrap()
            > 3.5
    );

    // Drop receiver to simulate client disconnect and verify trampoline returns 1 (abort)
    drop(rx);
    let rc_abort = ffi_progress_trampoline(
        5,
        32,
        tensor_name.as_ptr(),
        40_000_000,
        (&mut state as *mut FfiCallbackState<'_>) as *mut libc::c_void,
    );
    assert_eq!(
        rc_abort, 1,
        "Trampoline must return 1 to abort C FFI when client disconnects"
    );
    assert!(state.cancelled);
}

// ============================================================================
// 4. Tree-Attention Speculative Decoding, N-Gram Drafter & PID Controller Tests
// ============================================================================

#[test]
fn test_speculative_tree_attention_seq_id_propagation() {
    use llama_cpp_2::token::LlamaToken;
    use mcp_combinatorics_llm::model_cache::{DraftTreeNode, SpeculativeDraftTree};

    // Build a 4-node tree rooted at Node 0 (pos 10):
    //   Node 0 (root) -> Node 1 (branch seq 2) -> Node 3 (leaf seq 2)
    //                 -> Node 2 (leaf seq 9)
    let mut tree = SpeculativeDraftTree {
        root_pos: 10,
        borrowed_aux_seqs: vec![9],
        nodes: vec![
            DraftTreeNode {
                token: LlamaToken(100),
                draft_prob: 1.0,
                pos: 10,
                depth: 0,
                parent_idx: None,
                children: vec![1, 2],
                leaf_seq_ids: vec![],
                target_batch_idx: None,
            },
            DraftTreeNode {
                token: LlamaToken(101),
                draft_prob: 0.6,
                pos: 11,
                depth: 1,
                parent_idx: Some(0),
                children: vec![3],
                leaf_seq_ids: vec![],
                target_batch_idx: None,
            },
            DraftTreeNode {
                token: LlamaToken(102),
                draft_prob: 0.3,
                pos: 11,
                depth: 1,
                parent_idx: Some(0),
                children: vec![],
                leaf_seq_ids: vec![9],
                target_batch_idx: None,
            },
            DraftTreeNode {
                token: LlamaToken(103),
                draft_prob: 0.8,
                pos: 12,
                depth: 2,
                parent_idx: Some(1),
                children: vec![],
                leaf_seq_ids: vec![2],
                target_batch_idx: None,
            },
        ],
    };

    tree.finalize_tree_attention_masks();

    // Root Node 0 must be visible to both leaf sequence 2 and auxiliary leaf sequence 9
    assert_eq!(tree.nodes[0].leaf_seq_ids, vec![2, 9]);
    // Internal Node 1 must be visible only to leaf sequence 2
    assert_eq!(tree.nodes[1].leaf_seq_ids, vec![2]);
    // Sibling Leaf Node 2 must be visible only to auxiliary leaf sequence 9
    assert_eq!(tree.nodes[2].leaf_seq_ids, vec![9]);
}

#[test]
fn test_ngram_prompt_lookup_tree_drafter() {
    use llama_cpp_2::token::LlamaToken;
    use mcp_combinatorics_llm::model_cache::PromptLookupDrafter;

    // Construct a token history containing two distinct continuations after the 3-gram [10, 20, 30]:
    //   Occurrence 1: [10, 20, 30] -> [40, 41, 42]
    //   Occurrence 2: [10, 20, 30] -> [50, 51, 52]
    //   Occurrence 3: [10, 20, 30] -> [40, 41, 42] (makes branch 40 twice as frequent as 50)
    //   Current suffix ending at `root_token = 30`: committed ends with [..., 10, 20]
    let committed: Vec<LlamaToken> = vec![
        10, 20, 30, 40, 41, 42, 99, 10, 20, 30, 50, 51, 52, 99, 10, 20, 30, 40, 41, 42, 99, 10, 20,
    ]
    .into_iter()
    .map(LlamaToken)
    .collect();

    let root_token = LlamaToken(30);
    let root_pos = committed.len() as i32;
    let mut aux_pool = vec![11, 10, 9, 8];

    let drafter = PromptLookupDrafter::default();
    let tree = drafter
        .build_ngram_draft_tree(
            2, // primary seq_id = 2
            &committed,
            root_token,
            root_pos,
            3, // depth_k = 3
            2, // width_w = 2
            4096,
            |tok| tok.0 == 999,
            &mut aux_pool,
        )
        .expect("N-gram prompt lookup drafter should build a 2-branch speculative tree");

    // Borrowed 1 auxiliary sequence ID (8) for the second branch
    assert_eq!(tree.borrowed_aux_seqs, vec![8]);
    // Root node must carry both leaf sequence IDs [2, 8]
    assert_eq!(tree.nodes[0].leaf_seq_ids, vec![2, 8]);
    assert_eq!(tree.nodes[0].children.len(), 2);

    let branch_a_idx = tree.nodes[0].children[0];
    let branch_b_idx = tree.nodes[0].children[1];

    // Branch A must be the higher-frequency continuation [40, 41, 42] on primary seq 2
    assert_eq!(tree.nodes[branch_a_idx].token, LlamaToken(40));
    assert_eq!(tree.nodes[branch_a_idx].leaf_seq_ids, vec![2]);
    assert!(tree.nodes[branch_a_idx].draft_prob > tree.nodes[branch_b_idx].draft_prob);

    // Branch B must be the secondary continuation [50, 51, 52] on auxiliary seq 8
    assert_eq!(tree.nodes[branch_b_idx].token, LlamaToken(50));
    assert_eq!(tree.nodes[branch_b_idx].leaf_seq_ids, vec![8]);
}

#[test]
fn test_pid_ewma_speculative_controller_regimes() {
    use mcp_combinatorics_llm::model_cache::AdaptiveSpeculativeController;

    let mut ctrl = AdaptiveSpeculativeController::new(3, 2);

    // 1. High-Acceptance Regime: 100% of drafted depth accepted over 5 steps
    //    Controller must scale K upward toward max_k (8) and collapse W to 1 (laser chain)
    for _ in 0..5 {
        let (k, _w) = ctrl.select_k_and_w(100, 1, 16, 128);
        ctrl.record_verification_outcome(k, k, k + 1);
    }
    let (high_k, high_w) = ctrl.select_k_and_w(100, 1, 16, 128);
    assert!(
        high_k >= 6,
        "Expected PID loop to expand K >= 6 under 100% acceptance, got {high_k}"
    );
    assert_eq!(
        high_w, 1,
        "Expected W == 1 (single chain) when ewma_accept_rate > 0.78"
    );

    // 2. Moderate-Uncertainty Regime: ~45% acceptance rate
    //    Controller must widen tree_width W >= 2 to hedge depth-1 ambiguity
    let mut mod_ctrl = AdaptiveSpeculativeController::new(4, 2);
    for _ in 0..4 {
        mod_ctrl.record_verification_outcome(4, 2, 7);
    }
    let (mod_k, mod_w) = mod_ctrl.select_k_and_w(100, 1, 16, 128);
    assert!((2..=5).contains(&mod_k));
    assert!(
        mod_w >= 2,
        "Expected controller to widen W >= 2 in moderate acceptance regime, got {mod_w}"
    );

    // 3. Zero-Acceptance Collapse & Cooldown Probe Recovery:
    //    Repeated 0-token acceptances must trigger K=0 cooldown bypass, then recover with a (1, 1) probe
    for _ in 0..6 {
        mod_ctrl.record_verification_outcome(4, 0, 5);
    }
    assert!(mod_ctrl.ewma_accept_rate < 0.18);

    // Next two calls during cooldown must bypass speculation (K = 0)
    let (cd1_k, _) = mod_ctrl.select_k_and_w(100, 1, 16, 128);
    let (cd2_k, _) = mod_ctrl.select_k_and_w(100, 1, 16, 128);
    assert_eq!(cd1_k, 0);
    assert_eq!(cd2_k, 0);

    // Third call expires cooldown and fires a (K = 1, W = 1) recovery probe
    let (probe_k, probe_w) = mod_ctrl.select_k_and_w(100, 1, 16, 128);
    assert_eq!(probe_k, 1, "Expected K=1 recovery probe after cooldown");
    assert_eq!(probe_w, 1, "Expected W=1 recovery probe after cooldown");
    assert_eq!(mod_ctrl.stats().probe_recoveries, 1);
}
