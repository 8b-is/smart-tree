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
        "ultra_zeno_structural_compression",
        "gaia_omni_avatar_mode",
        "ternary_executable_deck",
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

    // 4. Information-theoretic System 1 / System 2 modulation
    let mut info_ctrl = AdaptiveSpeculativeController::new(3, 2);
    info_ctrl.apply_information_theoretic_modulation(0.3);
    assert!(info_ctrl.current_k >= 4);
    assert_eq!(info_ctrl.current_w, 1);
    info_ctrl.apply_information_theoretic_modulation(3.8);
    assert!(info_ctrl.current_k <= 3);
}

#[test]
fn test_ultra_zeno_m1_m2_m3_information_bottleneck_and_mdl() {
    use mcp_combinatorics_llm::{
        model_cache::{LlamaModelCache, McpProgressEmitter},
        ultra_zeno::{
            binary_entropy_bits, kl_divergence_bits, run_ultra_zeno_engine, shannon_entropy_bits,
            UltraZenoParams,
        },
    };

    // 1. Verify Shannon & KL Divergence Information-Theoretic Invariants
    assert!((binary_entropy_bits(0.5) - 1.0).abs() < 1e-12);
    assert!(binary_entropy_bits(0.0).abs() < 1e-12);
    assert!(binary_entropy_bits(1.0).abs() < 1e-12);

    let uniform_4 = [0.25, 0.25, 0.25, 0.25];
    assert!((shannon_entropy_bits(&uniform_4) - 2.0).abs() < 1e-12);

    let skewed = [0.70, 0.10, 0.10, 0.10];
    let kl = kl_divergence_bits(&skewed, &uniform_4);
    assert!(
        kl > 0.0,
        "Gibbs' inequality: D_KL(P || Q) must be strictly positive when P != Q"
    );

    // 2. Construct a structured 64-vertex graph with two dense 16-cliques (planted M2 structure)
    let n = 64;
    let mut edges = Vec::new();
    for u in 0..16 {
        for v in (u + 1)..16 {
            edges.push((u, v));
        }
    }
    for u in 16..32 {
        for v in (u + 1)..32 {
            edges.push((u, v));
        }
    }
    // Bridge edge connecting the two dense clusters
    edges.push((15, 16));

    let cache = LlamaModelCache::new(1);
    let res = run_ultra_zeno_engine(
        &cache,
        UltraZenoParams {
            num_vertices: n,
            edges,
            terminals: vec![0, 15, 16, 31],
            initial_epsilon: 0.32,
            max_zeno_steps: 4,
            ib_beta: 1.5,
            system2_entropy_threshold: 1.5,
            target_clique_k: 6,
            model_path: None,
        },
        &McpProgressEmitter::default(),
    )
    .expect("UltraZeno M1-M2-M3 pipeline should succeed");

    // Verify M1 sensory telemetry
    let m1_entropy = res["m1_system1_sensory"]["shannon_degree_entropy_bits"]
        .as_f64()
        .unwrap();
    let m1_kl = res["m1_system1_sensory"]["kl_divergence_from_er_prior_bits"]
        .as_f64()
        .unwrap();
    assert!(m1_entropy > 3.0);
    assert!(
        m1_kl > 0.1,
        "Planted cliques must produce positive KL surprise against ER prior"
    );

    // Verify M2 UltraZeno MDL compression achieved > 1.0x structural compression
    let m2 = &res["m2_ultra_zeno_compression"];
    let ratio = m2["mdl_compression_ratio"].as_f64().unwrap();
    let mi = m2["mutual_information_bits"].as_f64().unwrap();
    assert!(
        ratio > 1.2,
        "Expected UltraZeno M2 MDL compression ratio > 1.2x on structured graph, got {ratio}"
    );
    assert!(mi > 0.0, "Mutual information I(G; M2) must be positive");

    // Verify M3 System 2 extracted the exact 6-clique invariant
    let m3 = &res["m3_system2_invariants"];
    assert_eq!(m3["exact_micro_ramsey"]["k_clique_found"], true);
    assert_eq!(m3["moser_tardos_homeostasis"]["status"], "converged");
}

#[test]
fn test_gaia_omni_avatar_mode_control_holonomy_feedback() {
    use st::mcp_combinatorics_llm::{
        gaia_omni::{run_gaia_omni_mode, GaiaOmniParams},
        model_cache::{LlamaModelCache, McpProgressEmitter},
    };

    let report = run_gaia_omni_mode(
        &LlamaModelCache::new(1),
        GaiaOmniParams {
            biosphere_nodes: 32,
            synaptic_density: 0.18,
            regularity_epsilon: 0.28,
            consensus_clique_k: 3,
            model_path: None,
            omni_directive: "Sustain the biosphere.".to_string(),
            seed: 42,
            max_feedback_loops: 2,
            holonomy_tolerance: 0.25,
            compression_target: 2.0,
            giant_component_target: 0.75,
            pid_kp: 0.6,
            pid_ki: 0.15,
            pid_kd: 0.08,
            feedback_tolerance: 0.1,
            concurrency_policy: Default::default(),
        },
        &McpProgressEmitter::default(),
    )
    .expect("Gaia Omni control/holonomy pipeline should succeed");

    assert_eq!(report["status"], "gaia_omni_avatar_complete");
    assert_eq!(
        report["architecture"],
        "GAIA_OMNI_6_STAGE_CONTROL_HOLONOMY_FEEDBACK"
    );
    assert!(report["feedback_loops"].as_u64().unwrap() >= 1);
    assert!(report["erdos_renyi_genesis"]["giant_component_fraction"].is_number());
    assert!(report["szemeredi_cortical_map"]["mdl_compression_ratio"].is_number());
    assert!(report["holonomy"]["partition_parts"].as_u64().unwrap() >= 4);
    assert!(report["holonomy"]["quotient_nodes"].as_u64().unwrap() >= 2);
    assert!(report["holonomy"]["fundamental_cycles"].as_u64().unwrap() > 0);
    assert_eq!(
        report["fauna_avatar_consensus"]["planetary_identity"],
        "I/We/Gaia"
    );
    assert_eq!(
        report["concurrency_policy"]["human_authority"],
        "human_purpose_authority"
    );
    assert_eq!(report["concurrency_policy_checks"]["enclosure_risk"], "low");
    assert!(report["concurrency_policy_checks"]["violations"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn test_five_card_ternary_executable_deck_invariants() {
    use st::ternary_deck::{
        evaluate_frame_timing, evaluate_prerequisite_deck, evaluate_proportional_stability,
        evaluate_ternary_lane, evaluate_ternary_quantizer, frame_timestamp_ms,
        meets_frame_budget_ns, pack_4trits_byte, pack_5trits_byte, unpack_4trits_byte,
        unpack_5trits_byte, StudyCardNode,
    };

    // Card 1: Ternary Lane Packing
    let lane = evaluate_ternary_lane(3840, 2160);
    assert_eq!(lane.four_trits_per_byte.row_bytes, 960);
    assert_eq!(lane.four_trits_per_byte.frame_bytes, 2_073_600);
    assert_eq!(lane.five_trits_per_byte.row_bytes, 768);
    assert_eq!(lane.five_trits_per_byte.frame_bytes, 1_658_880);
    assert!((lane.shannon_bits_per_trit - 1.5849625).abs() < 1e-6);

    let sample_4 = [2, 0, 1, 2];
    assert_eq!(unpack_4trits_byte(pack_4trits_byte(sample_4)), sample_4);

    for b in 0u8..243u8 {
        let trits = unpack_5trits_byte(b);
        assert!(trits.iter().all(|&t| t < 3));
        assert_eq!(pack_5trits_byte(trits), b);
    }

    // Card 2: Drift-Free Integer Frame Timing
    let timing = evaluate_frame_timing(60, 16_666_666);
    assert_eq!(timing.timestamp_at_1_second_ms, 1000);
    assert_eq!(timing.naive_rounded_1s_timestamp_ms, 960);
    assert_eq!(timing.cumulative_drift_eliminated_ms, 40);
    assert_eq!(
        timing.first_9_intervals_ms,
        vec![16, 17, 17, 16, 17, 17, 16, 17, 17]
    );
    assert!(meets_frame_budget_ns(60, 16_666_666));
    assert!(!meets_frame_budget_ns(60, 16_666_667));
    assert_eq!(frame_timestamp_ms(60, 60), 1000);

    // Card 3: 64-Bit Ternary Quantizer Reconstruction Error
    let mut dark_scene = Vec::with_capacity(4096);
    for i in 0..4096 {
        dark_scene.push(if i < 3600 { (i % 20) as u8 } else { 240 });
    }
    let quant = evaluate_ternary_quantizer(&dark_scene, 3840 * 2160);
    assert_eq!(quant.worst_case_error_bound_u64, 539_343_360_000);
    assert!(quant.exceeds_u32_max);
    assert!(
        quant.optimal_mse_quantizer.reconstruction_error_sse
            < quant.equal_occupancy_quantizer.reconstruction_error_sse,
        "MSE-optimal thresholds must strictly beat forced equal occupancy on a dark scene"
    );

    // Card 4: Prerequisite DAG Critical Path
    let deck = vec![
        StudyCardNode {
            id: 0,
            title: "A".into(),
            duration: 10,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 1,
            title: "B".into(),
            duration: 25,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 2,
            title: "C".into(),
            duration: 15,
            predecessors: vec![0, 1],
        },
    ];
    let sched = evaluate_prerequisite_deck(&deck).unwrap();
    assert_eq!(sched.critical_path_duration, 40);
    assert_eq!(sched.sequential_total_duration, 50);
    assert_eq!(sched.critical_path_chain, vec![1, 2]);
    assert_eq!(sched.card_schedule[0].slack, 15);

    // Card 5: Proportional Stability & Limit Cycles
    let stable_pid =
        evaluate_proportional_stability(1.2, 0.5, 2.4, 100.0, 0.0, 1000.0, 0.0, false, 64).unwrap();
    assert!((stable_pid.k_min_exclusive - 0.4).abs() < 1e-9);
    assert!((stable_pid.k_max_exclusive - 4.4).abs() < 1e-9);
    assert!((stable_pid.deadbeat_gain_k - 2.4).abs() < 1e-9);
    assert!(stable_pid.is_linearly_stable);

    let unstable_pid =
        evaluate_proportional_stability(1.2, 0.5, 4.6, 100.0, 10.0, 250.0, 0.0, true, 64).unwrap();
    assert!(!unstable_pid.is_linearly_stable);
    assert!(
        unstable_pid.persistent_limit_cycle_detected,
        "Gain above (a+1)/b with actuator limits must trigger persistent limit-cycle oscillation"
    );
}

#[tokio::test]
async fn test_ternary_executable_deck_mcp_tool() {
    let state = Arc::new(AppState::new(1));
    let req = json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "tools/call",
        "params": {
            "name": "ternary_executable_deck",
            "arguments": {}
        }
    });
    let resp = evaluate_jsonrpc_payload(state, req, None).await.unwrap();
    assert!(resp["error"].is_null(), "unexpected error: {resp}");
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let payload: Value = serde_json::from_str(text).unwrap();
    assert_eq!(payload["status"], "verified");
    assert_eq!(
        payload["card_1_ternary_lane"]["four_trits_per_byte"]["row_bytes"],
        960
    );
    assert_eq!(
        payload["card_1_ternary_lane"]["five_trits_per_byte"]["row_bytes"],
        768
    );
    assert_eq!(
        payload["card_2_frame_timing"]["timestamp_at_1_second_ms"],
        1000
    );
    assert!(payload["card_3_quantizer_quality"]["exceeds_u32_max"]
        .as_bool()
        .unwrap());
    assert!(
        payload["card_4_prerequisite_schedule"]["critical_path_duration"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(payload["card_5_pid_stability"]["is_linearly_stable"]
        .as_bool()
        .unwrap());
}

#[test]
fn test_adaptive_speculative_controller_enforces_jury_non_oscillatory_certificate() {
    use mcp_combinatorics_llm::{
        model_cache::AdaptiveSpeculativeController,
        ternary_deck::evaluate_jury_pid_certificate,
    };

    // 1. Prove that the unconstrained heuristic gains (3.20, 0.45, 0.90) violate Jury Condition 2
    //    for the EWMA plant (a = 0.65, b = 0.35): -P(-1) = -0.3575 < 0, spectral_radius > 1.0
    let raw_heuristic_cert =
        evaluate_jury_pid_certificate(0.65, 0.35, 3.20, 0.45, 0.90).unwrap();
    assert!(
        !raw_heuristic_cert.cond2_alternating_minus_one_holds,
        "Raw heuristic gains must fail Jury Condition 2 at z = -1"
    );
    assert!(!raw_heuristic_cert.is_jury_stable);
    assert!(
        raw_heuristic_cert.spectral_radius > 1.0,
        "Expected unstable spectral radius > 1.0 for raw heuristic gains, got {}",
        raw_heuristic_cert.spectral_radius
    );

    // 2. Verify AdaptiveSpeculativeController::with_gains automatically projects (3.20, 0.45, 0.90)
    //    into the non-oscillatory Jury polytope
    let ctrl = AdaptiveSpeculativeController::with_gains(3, 2, 3.20, 0.45, 0.90);
    assert_eq!(ctrl.gain_projections_applied, 1);
    assert!(ctrl.jury_certificate.is_jury_stable);
    assert!(ctrl.jury_certificate.cond1_p_at_one_holds);
    assert!(ctrl.jury_certificate.cond2_alternating_minus_one_holds);
    assert!(ctrl.jury_certificate.cond3_abs_a0_lt_one_holds);
    assert!(ctrl.jury_certificate.cond4_jury_det_holds);
    assert!(
        ctrl.jury_certificate.spectral_radius <= 0.88,
        "Projected spectral radius must be <= 0.88, got {}",
        ctrl.jury_certificate.spectral_radius
    );
    assert!(
        ctrl.jury_certificate
            .roots
            .iter()
            .all(|r| r.re >= -0.20),
        "All projected closed-loop poles must satisfy Re(z) >= -0.20 to prevent Nyquist chattering"
    );

    // 3. Verify default constructor is immediately Jury-stable with zero projections required
    let mut default_ctrl = AdaptiveSpeculativeController::new(4, 2);
    assert_eq!(default_ctrl.gain_projections_applied, 0);
    assert!(default_ctrl.jury_certificate.is_jury_stable);
    assert!(default_ctrl.jury_certificate.spectral_radius < 0.85);

    // 4. Verify online M1 entropy modulation never escapes the Jury polytope even under
    //    20 consecutive extreme System 1 (0.1 bits) or System 2 (5.0 bits) entropy bursts
    for _ in 0..20 {
        default_ctrl.apply_information_theoretic_modulation(0.1);
        assert!(default_ctrl.jury_certificate.is_jury_stable);
        assert!(default_ctrl.jury_certificate.spectral_radius <= 0.88);
    }
    for _ in 0..20 {
        default_ctrl.apply_information_theoretic_modulation(5.0);
        assert!(default_ctrl.jury_certificate.is_jury_stable);
        assert!(default_ctrl.jury_certificate.spectral_radius <= 0.88);
    }

    // 5. Verify non-oscillatory step response: step acceptance from 0.65 -> 1.00
    //    `k_continuous` must increase monotonically without sign-flipping deltas
    let mut step_ctrl = AdaptiveSpeculativeController::new(3, 2);
    let mut prev_k = step_ctrl.k_continuous;
    for _ in 0..8 {
        step_ctrl.record_verification_outcome(4, 4, 5); // 100% acceptance step
        assert!(
            step_ctrl.k_continuous >= prev_k - 1e-12,
            "Jury-projected controller must expand k_continuous monotonically without oscillation ({} < {})",
            step_ctrl.k_continuous,
            prev_k
        );
        prev_k = step_ctrl.k_continuous;
    }
    assert!(step_ctrl.current_k >= 6);
}
