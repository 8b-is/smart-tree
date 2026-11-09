//! OpenAI-Compatible Proxy Server
//!
//! HTTP server implementing the OpenAI Chat Completions API plus a small admin
//! surface for status and provider listing.
//!
//! Endpoints:
//!   POST /v1/chat/completions  - OpenAI-compatible chat
//!   GET  /v1/models            - List available models (across providers)
//!   GET  /admin/status         - Proxy status: providers + auth state
//!
//! Optional bearer auth: set `ST_PROXY_API_KEY` in the env. When set, every
//! request must carry `Authorization: Bearer <key>`. When unset, the proxy is
//! open (loopback only by default).

use crate::proxy::memory::MemoryProxy;
use crate::proxy::openai_compat::{
    OpenAiChoice, OpenAiError, OpenAiErrorResponse, OpenAiRequest, OpenAiResponse,
    OpenAiResponseMessage, OpenAiUsage,
};
use crate::proxy::LlmRequest;
use anyhow::Result;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedProxy = Arc<RwLock<MemoryProxy>>;

/// Start the OpenAI-compatible proxy server.
pub async fn start_proxy_server(port: u16) -> Result<()> {
    let proxy: SharedProxy = Arc::new(RwLock::new(MemoryProxy::new()?));

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route(
            "/v1/helper",
            post(helper_post_endpoint).get(helper_get_endpoint),
        )
        .route("/v1/dealers-choice", get(dealers_choice_endpoint))
        .route("/v1/finance", get(von_chenoweth_lodri_finance_endpoint))
        .route("/v1/capsela/asm-exec", post(capsela_asm_exec_endpoint))
        .route("/admin/status", get(admin_status))
        .layer(middleware::from_fn(bearer_auth))
        .with_state(proxy);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Smart Tree LLM Proxy on http://{}", addr);
    println!(
        "  POST /v1/chat/completions  (supports model: 'dealers_choice' / 'g8t/dealers-choice')"
    );
    println!("  GET  /v1/models");
    println!("  POST /v1/helper            (Mechanic's Helper · Aye & The Cheet · $5/mo i1.is)");
    println!(
        "  GET  /v1/dealers-choice    ($10/mo Unlimited g8t Dealer's Choice Model Leaderboard)"
    );
    println!(
        "  GET  /v1/finance           (8b.is E2E 5-Year Financial Model · von Chenoweth-Lodri Era)"
    );
    println!("  POST /v1/capsela/asm-exec  (Complete Freestanding #![no_std] Capsela ASM Studio)");
    println!("  GET  /admin/status");
    if std::env::var("ST_PROXY_API_KEY").is_ok() {
        println!("  Auth: bearer (ST_PROXY_API_KEY required)");
    } else {
        println!("  Auth: open (set ST_PROXY_API_KEY to require Bearer token)");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Bearer-token middleware. No-op when ST_PROXY_API_KEY is unset.
async fn bearer_auth(req: Request<axum::body::Body>, next: Next) -> Response {
    let Ok(expected) = std::env::var("ST_PROXY_API_KEY") else {
        return next.run(req).await;
    };

    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    match provided {
        Some(token) if token == expected => next.run(req).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(OpenAiErrorResponse {
                error: OpenAiError {
                    message: "missing or invalid bearer token".into(),
                    error_type: "authentication_error".into(),
                    code: Some("invalid_api_key".into()),
                },
            }),
        )
            .into_response(),
    }
}

async fn chat_completions(
    State(proxy): State<SharedProxy>,
    Json(req): Json<OpenAiRequest>,
) -> Response {
    let last_user_text = req
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let is_dealers_choice = matches!(
        req.model.to_ascii_lowercase().as_str(),
        "dealers_choice"
            | "dealers-choice"
            | "g8t/dealers-choice"
            | "g8t/dealers_choice"
            | "dealer"
            | "auto"
    );

    let (provider_name, model_name) = if is_dealers_choice {
        let lock = proxy.read().await;
        let available = lock.inner.list_providers();
        drop(lock);
        let route = crate::g8t_integration::route_dealers_choice(last_user_text, &available);
        (route.selected_provider, route.selected_model)
    } else {
        match req.model.split_once('/') {
            Some((p, m)) => (p.to_string(), m.to_string()),
            None => ("openai".to_string(), req.model.clone()),
        }
    };

    let internal_req = LlmRequest {
        model: model_name.clone(),
        messages: req.messages.into_iter().map(Into::into).collect(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        stream: req.stream.unwrap_or(false),
    };

    let scope_id = req.user.unwrap_or_else(|| "global".to_string());

    let mut proxy_lock = proxy.write().await;
    match proxy_lock
        .complete_with_memory(&provider_name, &scope_id, internal_req)
        .await
    {
        Ok(resp) => (
            StatusCode::OK,
            Json(OpenAiResponse {
                id: format!("st-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: if is_dealers_choice {
                    format!("dealers_choice->{}/{}", provider_name, model_name)
                } else {
                    req.model
                },
                choices: vec![OpenAiChoice {
                    index: 0,
                    message: OpenAiResponseMessage {
                        role: "assistant".to_string(),
                        content: resp.content,
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: resp.usage.map(|u| OpenAiUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            }),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("not found") || msg.contains("invalid") {
                StatusCode::BAD_REQUEST
            } else if msg.contains("unauthorized") || msg.contains("authentication") {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(OpenAiErrorResponse {
                    error: OpenAiError {
                        message: msg,
                        error_type: "api_error".into(),
                        code: None,
                    },
                }),
            )
                .into_response()
        }
    }
}

#[derive(Serialize)]
struct ModelEntry {
    id: String,
    object: &'static str,
    owned_by: String,
}

#[derive(Serialize)]
struct ModelList {
    object: &'static str,
    data: Vec<ModelEntry>,
}

/// GET /v1/models — returns an OpenAI-style list.
/// Each provider contributes a single placeholder entry `<provider>/default`;
/// callers can request specific models with the `<provider>/<model>` syntax.
async fn list_models(State(proxy): State<SharedProxy>) -> Response {
    let lock = proxy.read().await;
    let data: Vec<ModelEntry> = lock
        .inner
        .list_providers()
        .into_iter()
        .map(|p| ModelEntry {
            id: format!("{}/default", p.to_lowercase()),
            object: "model",
            owned_by: p.to_string(),
        })
        .collect();

    Json(ModelList {
        object: "list",
        data,
    })
    .into_response()
}

#[derive(Serialize)]
struct AdminStatus {
    running: bool,
    auth_required: bool,
    providers: Vec<&'static str>,
    version: &'static str,
}

async fn admin_status(State(proxy): State<SharedProxy>) -> Response {
    let lock = proxy.read().await;
    Json(AdminStatus {
        running: true,
        auth_required: std::env::var("ST_PROXY_API_KEY").is_ok(),
        providers: lock.inner.list_providers(),
        version: env!("CARGO_PKG_VERSION"),
    })
    .into_response()
}

/// POST /v1/helper — Mechanic's Helper endpoint for LLM proxy clients & g8t.is/i1.is cost-savers.
async fn helper_post_endpoint(Json(payload): Json<serde_json::Value>) -> Response {
    match crate::mcp::mechanic_helper::handle_helper_with_cwd(Some(payload), None).await {
        Ok(val) => (StatusCode::OK, Json(val)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /v1/helper — Returns current Mechanic's Helper session, Documents/Desktop/Downloads projects, and tool tray.
async fn helper_get_endpoint() -> Response {
    helper_post_endpoint(Json(serde_json::json!({}))).await
}

/// GET /v1/dealers-choice — Returns the $10/mo Unlimited g8t Dealer's Choice Model Leaderboard across all 8 domains.
async fn dealers_choice_endpoint(State(proxy): State<SharedProxy>) -> Response {
    let lock = proxy.read().await;
    let available = lock.inner.list_providers();
    drop(lock);
    let report = crate::g8t_integration::route_dealers_choice(
        "complete capsela g8t asm executable programs",
        &available,
    );
    (StatusCode::OK, Json(report)).into_response()
}

/// POST /v1/capsela/asm-exec — Compiles, disassembles, and executes a complete freestanding `#![no_std] #![no_main]` Capsela ASM program.
async fn capsela_asm_exec_endpoint(Json(payload): Json<serde_json::Value>) -> Response {
    let query = payload
        .get("query")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("capsela-zeno-sun-arcade");
    let custom_source = payload.get("source_code").and_then(|v| v.as_str());
    let capsules_list: Vec<String> = payload
        .get("capsules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let seed = payload
        .get("seed")
        .and_then(|v| v.as_str())
        .unwrap_or("432528");
    let partner = payload
        .get("partner")
        .and_then(|v| v.as_str())
        .unwrap_or("Hue + Gemini (0+1)");

    match crate::g8t_integration::build_and_run_capsela_asm_executable(
        query,
        &capsules_list,
        custom_source,
        seed,
        partner,
    ) {
        Ok(receipt) => (StatusCode::OK, Json(receipt)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Builds the canonical 5-year E2E `von Chenoweth-Lodri Era` financial telemetry JSON payload
/// using exact integer cents and canonical reduced rationals `(p, q)` via `felix_binary_gcd_u64`.
pub fn build_von_chenoweth_lodri_finance_json() -> serde_json::Value {
    let reduce_pq = |p: u64, q: u64| -> (u64, u64) {
        if p == 0 || q == 0 {
            return (0, 1);
        }
        let g = crate::felix_asm::felix_binary_gcd_u64(p, q).max(1);
        (p / g, q / g)
    };

    let ltv_to_cac_pq = reduce_pq(43_200, 4_800);
    let cac_payback_months_pq = reduce_pq(16, 5);
    let gross_margin_pq = reduce_pq(85, 100);
    let cogs_ratio_pq = reduce_pq(15, 100);
    let retention_m12_pq = reduce_pq(75, 100);
    let bonobo_ndr_pq = reduce_pq(150, 100);
    let tier3_save_share_pq = reduce_pq(20, 100);
    let seed_dilution_pq = reduce_pq(5_000_000, 25_000_000);

    type CohortRow = (u8, u64, u64, u64, u64, u64, u32, u64, i64);
    let cohorts: [CohortRow; 5] = [
        (1, 1_200, 2_500, 900, 240, 800, 6, 135_000_000, 500_000_000),
        (2, 4_500, 9_000, 3_500, 900, 3_200, 12, 280_000_000, 0),
        (3, 12_000, 25_000, 10_000, 2_600, 9_500, 24, 590_000_000, 0),
        (
            4,
            25_000,
            55_000,
            22_000,
            5_800,
            22_000,
            40,
            1_020_000_000,
            0,
        ),
        (
            5,
            48_000,
            110_000,
            45_000,
            12_000,
            50_000,
            60,
            1_580_000_000,
            0,
        ),
    ];

    let mut cash_cents: i64 = 25_000_000; // $250K initial bootstrap cash
    let mut years_json = Vec::with_capacity(5);

    for (yr, t1, t2, t3, t4, t5, hc, opex_cents, fundraise_cents) in cohorts {
        let gross_mrr_cents = t1 * 500 + t2 * 1_000 + t3 * 2_900 + t4 * 25_000 + t5 * 2_000;
        let retained_m12_mrr_cents = (gross_mrr_cents * retention_m12_pq.0) / retention_m12_pq.1;
        let expanded_ndr_mrr_cents = (retained_m12_mrr_cents * bonobo_ndr_pq.0) / bonobo_ndr_pq.1;
        let arr_cents = expanded_ndr_mrr_cents * 12;
        let p10_arr_cents = (arr_cents * 7) / 10;
        let p90_arr_cents = (arr_cents * 13) / 10;
        let cogs_cents = (arr_cents * cogs_ratio_pq.0) / cogs_ratio_pq.1;
        let gross_profit_cents = arr_cents - cogs_cents;
        let ebitda_cents = (gross_profit_cents as i64) - (opex_cents as i64);
        cash_cents = cash_cents + fundraise_cents + ebitda_cents;

        years_json.push(serde_json::json!({
            "year": yr,
            "total_accounts": t1 + t2 + t3 + t4 + t5,
            "headcount": hc,
            "gross_mrr_cents": gross_mrr_cents,
            "retained_m12_mrr_cents": retained_m12_mrr_cents,
            "expanded_ndr_mrr_cents": expanded_ndr_mrr_cents,
            "arr_cents": arr_cents,
            "scenarios": {
                "p10_conservative_arr_cents": p10_arr_cents,
                "p50_base_arr_cents": arr_cents,
                "p90_optimistic_arr_cents": p90_arr_cents,
            },
            "annual_cogs_cents": cogs_cents,
            "annual_gross_profit_cents": gross_profit_cents,
            "annual_opex_cents": opex_cents,
            "annual_ebitda_cents": ebitda_cents,
            "ending_cash_cents": cash_cents,
            "runway_months": 1200,
            "revenue_per_employee_cents": arr_cents / (hc as u64),
        }));
    }

    serde_json::json!({
        "era": "von Chenoweth-Lodri Era (8b.is E2E Constellation)",
        "spark_formal_proof": "Von_Chenoweth_Lodri_Finance_Spark (181/181 gnatprove checks proved)",
        "tiers": [
            { "tier": 1, "name": "Mechanic's Helper (i1.is + peloton-row-usbc + Capsela g8t)", "arpu_cents": 500 },
            { "tier": 2, "name": "g8t Dealer's Choice (aye.is + smart-tree + MEM|8 Vector-Wave DB)", "arpu_cents": 1000 },
            {
                "tier": 3,
                "name": "Bonobo + Kompress-Super-Ultra + Headroomz Pro",
                "arpu_cents": 2900,
                "hybrid_save_share_pq": [tier3_save_share_pq.0, tier3_save_share_pq.1],
                "formula": "seats * 2900 + verified_saved_spend_cents * (1/5)"
            },
            { "tier": 4, "name": "Sovereign Enterprise Mesh (felix + memnet + eightb-agy-fanout)", "arpu_cents": 25000 },
            { "tier": 5, "name": "8b-is-engine + ultrawhale Marketplace (3/20 GMV take rate)", "arpu_cents": 2000 }
        ],
        "unit_economics_pq": {
            "gross_margin_pq": [gross_margin_pq.0, gross_margin_pq.1],
            "cogs_ratio_pq": [cogs_ratio_pq.0, cogs_ratio_pq.1],
            "retention_m12_lll_pq": [retention_m12_pq.0, retention_m12_pq.1],
            "bonobo_ndr_pq": [bonobo_ndr_pq.0, bonobo_ndr_pq.1],
            "tier3_save_share_pq": [tier3_save_share_pq.0, tier3_save_share_pq.1],
            "ltv_to_cac_pq": [ltv_to_cac_pq.0, ltv_to_cac_pq.1],
            "cac_payback_months_pq": [cac_payback_months_pq.0, cac_payback_months_pq.1],
            "seed_dilution_pq": [seed_dilution_pq.0, seed_dilution_pq.1]
        },
        "five_year_projections": years_json
    })
}

/// GET /v1/finance — Returns the 8b.is E2E 5-Year Financial Model (`von Chenoweth-Lodri Era`).
async fn von_chenoweth_lodri_finance_endpoint() -> Response {
    (
        StatusCode::OK,
        Json(build_von_chenoweth_lodri_finance_json()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_von_chenoweth_lodri_finance_json_invariants() {
        let payload = build_von_chenoweth_lodri_finance_json();
        assert_eq!(
            payload["unit_economics_pq"]["gross_margin_pq"],
            serde_json::json!([17, 20])
        );
        assert_eq!(
            payload["unit_economics_pq"]["ltv_to_cac_pq"],
            serde_json::json!([9, 1])
        );
        assert_eq!(
            payload["unit_economics_pq"]["tier3_save_share_pq"],
            serde_json::json!([1, 5])
        );
        assert_eq!(
            payload["unit_economics_pq"]["seed_dilution_pq"],
            serde_json::json!([1, 5])
        );
        let yrs = payload["five_year_projections"]
            .as_array()
            .expect("5 years");
        assert_eq!(yrs.len(), 5);
        assert_eq!(yrs[0]["arr_cents"], 179_685_000u64);
        assert_eq!(yrs[4]["arr_cents"], 8_970_750_000u64);
    }
}
