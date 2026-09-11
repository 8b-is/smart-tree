use super::indexing::{self, EmbedRequest, Embeddings};
use super::storage::{hash_token, new_token, now, Repository, Store};
use super::Hub;
use anyhow::{Context, Result};
use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

type ApiResult = std::result::Result<Json<Value>, ApiError>;

pub(super) struct ApiError(StatusCode, String);
impl ApiError {
    fn bad(message: impl ToString) -> Self {
        Self(StatusCode::BAD_REQUEST, message.to_string())
    }
    fn unauthorized() -> Self {
        Self(
            StatusCode::UNAUTHORIZED,
            "A valid access token is required".into(),
        )
    }
    fn missing() -> Self {
        Self(StatusCode::NOT_FOUND, "Repository not found".into())
    }
    fn busy() -> Self {
        Self(
            StatusCode::TOO_MANY_REQUESTS,
            "Please try again shortly".into(),
        )
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        tracing::error!(%error,"Hub request failed");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The request could not be completed".into(),
        )
    }
}
impl From<rusqlite::Error> for ApiError {
    fn from(error: rusqlite::Error) -> Self {
        anyhow::Error::from(error).into()
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}

async fn with_store<T, F>(hub: Arc<Hub>, operation: F) -> std::result::Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(&mut Store) -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut store = hub.lock()?;
        operation(&mut store)
    })
    .await
    .context("Storage task failed")?
    .map_err(Into::into)
}

fn auth_hash(headers: &HeaderMap) -> String {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return String::new();
    };
    if let Some(token) = value.strip_prefix("Bearer ") {
        return hash_token(token);
    }
    if let Some(encoded) = value.strip_prefix("Basic ") {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) {
            if let Ok(value) = std::str::from_utf8(&bytes) {
                if let Some((_, token)) = value.split_once(':') {
                    return hash_token(token);
                }
            }
        }
    }
    String::new()
}

fn is_admin(hub: &Hub, hash: &str) -> bool {
    hash.len() == hub.admin_hash.len()
        && hash
            .bytes()
            .zip(hub.admin_hash.bytes())
            .fold(0, |difference, (left, right)| difference | (left ^ right))
            == 0
}

fn require_admin(hub: &Hub, headers: &HeaderMap) -> std::result::Result<(), ApiError> {
    if is_admin(hub, &auth_hash(headers)) {
        Ok(())
    } else {
        Err(ApiError::unauthorized())
    }
}

#[derive(Default)]
pub(super) struct RateLimits {
    entries: HashMap<(String, &'static str), (Instant, u32)>,
}
impl RateLimits {
    fn check(&mut self, ip: String, bucket: &'static str, maximum: u32) -> bool {
        let now = Instant::now();
        self.entries
            .retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(3600));
        if self.entries.len() >= 10000 {
            return false;
        }
        let entry = self.entries.entry((ip, bucket)).or_insert((now, 0));
        if entry.1 >= maximum {
            return false;
        }
        entry.1 += 1;
        true
    }
}

fn rate_limit(
    hub: &Hub,
    headers: &HeaderMap,
    peer: SocketAddr,
    bucket: &'static str,
    maximum: u32,
) -> std::result::Result<(), ApiError> {
    let ip = if hub.config.trust_proxy && peer.ip().is_loopback() {
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .and_then(|v| v.trim().parse::<std::net::IpAddr>().ok())
            .unwrap_or(peer.ip())
    } else {
        peer.ip()
    };
    let mut limits = hub.limits.lock().map_err(|_| ApiError::busy())?;
    if limits.check(ip.to_string(), bucket, maximum) {
        Ok(())
    } else {
        Err(ApiError::busy())
    }
}

pub(super) fn router(hub: Arc<Hub>) -> Router {
    Router::new()
        .route("/internal/tls-allow", get(tls_allow))
        .route("/", get(|| async { Html(include_str!("web/index.html")) }))
        .route(
            "/hub.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("web/hub.css"),
                )
            }),
        )
        .route(
            "/hub.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("web/hub.js"),
                )
            }),
        )
        .route(
            "/docs",
            get(|| async { Html(include_str!("web/docs.html")) }),
        )
        .route("/llms.txt", get(|| async { include_str!("web/llms.txt") }))
        .route(
            "/health",
            get(|| async { Json(json!({"status":"ok","service":"smart-tree-hub"})) }),
        )
        .route("/api/v1/stats", get(stats))
        .route("/api/v1/repositories", get(repositories))
        .route(
            "/api/v1/repositories/:id",
            get(repository).patch(update_repository),
        )
        .route("/api/v1/recall", post(recall))
        .route("/api/v1/archive-requests", post(request_archive))
        .route("/api/v1/admin/import", post(import))
        .route("/api/v1/admin/repositories/:id/approve", post(approve))
        .route("/api/v1/admin/repositories/:id/reindex", post(reindex))
        .route("/api/v1/admin/feedback", get(feedback_list))
        .route("/api/feedback", post(feedback))
        .route("/feedback", post(feedback))
        .route("/api/tool-request", post(feedback))
        .route("/api/smart-tree/latest", get(latest_version))
        .route(
            "/git/:id/*rest",
            get(git_get)
                .post(git_post)
                .layer(DefaultBodyLimit::max(8 * 1024 * 1024)),
        )
        .fallback(|| async { (StatusCode::NOT_FOUND, Json(json!({"error":"Not found"}))) })
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(hub)
}

#[derive(Deserialize)]
struct TlsQuery {
    domain: String,
}

async fn tls_allow(
    State(hub): State<Arc<Hub>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(query): Query<TlsQuery>,
) -> std::result::Result<StatusCode, ApiError> {
    if !peer.ip().is_loopback() {
        return Ok(StatusCode::FORBIDDEN);
    }
    let domain = query.domain.to_ascii_lowercase();
    let fixed = matches!(
        domain.as_str(),
        "8s.is" | "www.8s.is" | "api.8s.is" | "feedback.8s.is" | "f.8s.is" | "git.8s.is"
    );
    let valid = domain.strip_suffix(".8s.is").is_some_and(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
    });
    if !fixed && !valid {
        return Ok(StatusCode::FORBIDDEN);
    }
    if fixed {
        return Ok(StatusCode::OK);
    }
    let accepted = with_store(hub, move |store| {
        let known: bool = store.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM tls_names WHERE domain=?)",
            [&domain],
            |row| row.get(0),
        )?;
        if known {
            return Ok(true);
        }
        let recent: usize = store.db.query_row(
            "SELECT count(*) FROM tls_names WHERE requested_at>?",
            [chrono::Utc::now().timestamp() - 7 * 86400],
            |row| row.get(0),
        )?;
        if recent >= 35 {
            return Ok(false);
        }
        store.db.execute(
            "INSERT INTO tls_names VALUES (?,?)",
            params![domain, chrono::Utc::now().timestamp()],
        )?;
        Ok(true)
    })
    .await?;
    Ok(if accepted {
        StatusCode::OK
    } else {
        StatusCode::TOO_MANY_REQUESTS
    })
}

async fn stats(State(hub): State<Arc<Hub>>) -> ApiResult {
    let mut value = with_store(hub.clone(), |store| store.stats()).await?;
    value["semantic_configured"] = json!(hub.config.embed_url.is_some());
    value["storage"] = json!({"archives":filesystem_space(&hub.config.archive_dir),"index":filesystem_space(&hub.config.state_dir)});
    value["coverage"] = json!({"revision":"HEAD at indexing time","max_file_bytes":1048576,"max_files_per_repository":hub.config.max_files,"max_passages_per_repository":hub.config.max_chunks,"archive_intake":"operator_review","semantic_engine":"local embeddings + cosine retrieval over MEM8","text_index":"persistent FTS5 tokens + T8R passages"});
    Ok(Json(value))
}

fn filesystem_space(path: &std::path::Path) -> Value {
    #[cfg(unix)]
    {
        let Ok(name) = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) else {
            return Value::Null;
        };
        let mut info = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // statvfs initializes the complete structure on success.
        if unsafe { libc::statvfs(name.as_ptr(), info.as_mut_ptr()) } == 0 {
            let info = unsafe { info.assume_init() };
            return json!({"capacity_bytes":(info.f_blocks as u128)*(info.f_frsize as u128),"available_bytes":(info.f_bavail as u128)*(info.f_frsize as u128)});
        }
    }
    Value::Null
}

#[derive(Default, Deserialize)]
struct CatalogQuery {
    #[serde(default)]
    collection: String,
    #[serde(default)]
    q: String,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn repositories(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Query(query): Query<CatalogQuery>,
) -> ApiResult {
    if query.collection.len() > 100 || query.q.len() > 200 || query.offset.unwrap_or(0) > 100000 {
        return Err(ApiError::bad("Catalogue query exceeds limits"));
    }
    let hash = auth_hash(&headers);
    let admin = is_admin(&hub, &hash);
    let limit = query.limit.unwrap_or(30).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let rows = with_store(hub, move |store| {
        store.repositories(&hash, admin, &query.collection, &query.q, limit, offset)
    })
    .await?;
    Ok(Json(
        json!({"repositories":rows.iter().map(Repository::public_view).collect::<Vec<_>>(),"limit":limit,"offset":offset,"has_more":rows.len()==limit}),
    ))
}

async fn repository(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult {
    let hash = auth_hash(&headers);
    let admin = is_admin(&hub, &hash);
    let repo = with_store(hub, move |store| store.repository(&id))
        .await?
        .filter(|repo| repo.visible(&hash, admin))
        .ok_or_else(ApiError::missing)?;
    Ok(Json(repo.public_view()))
}

#[derive(Deserialize)]
struct ArchiveRequest {
    source_url: String,
    #[serde(default)]
    public: bool,
    #[serde(default)]
    recall_opt_in: bool,
}

async fn request_archive(
    State(hub): State<Arc<Hub>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<ArchiveRequest>,
) -> ApiResult {
    rate_limit(&hub, &headers, peer, "archive", 5)?;
    let (source_url, collection, name) =
        indexing::source_url(&request.source_url).map_err(ApiError::bad)?;
    let token = new_token();
    let repo = Repository {
        id: uuid::Uuid::new_v4().to_string(),
        collection,
        name,
        source_url,
        git_dir: String::new(),
        owner_hash: hash_token(&token),
        public: request.public,
        recall: request.recall_opt_in,
        approved: false,
        status: "pending_approval".into(),
        generation: String::new(),
        commit: String::new(),
        files: 0,
        chunks: 0,
        skipped: 0,
        updated_at: now(),
        message: "Awaiting operator review".into(),
    };
    let response = json!({"repository":repo.public_view(),"manage_token":token,"message":"Save this token. It controls visibility and recall permission. An operator reviews requests before cloning."});
    with_store(hub, move |store| {
        let pending: usize = store.db.query_row(
            "SELECT count(*) FROM repositories WHERE status='pending_approval'",
            [],
            |row| row.get(0),
        )?;
        anyhow::ensure!(pending < 1000, "Archive request queue is full");
        store.add_repository(&repo)
    })
    .await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
struct Consent {
    public: bool,
    recall_opt_in: bool,
}

async fn update_repository(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<Consent>,
) -> ApiResult {
    let hash = auth_hash(&headers);
    let admin = is_admin(&hub, &hash);
    let repo = with_store(hub.clone(), {
        let id = id.clone();
        move |store| store.repository(&id)
    })
    .await?
    .ok_or_else(ApiError::missing)?;
    if !admin && (hash.is_empty() || repo.owner_hash != hash) {
        return Err(ApiError::unauthorized());
    }
    let repo = with_store(hub, move |store| {
        store.change_consent(&id, request.public, request.recall_opt_in)?;
        store.repository(&id)?.context("Repository missing")
    })
    .await?;
    Ok(Json(repo.public_view()))
}

#[derive(Deserialize)]
struct ImportRequest {
    root: std::path::PathBuf,
    #[serde(default)]
    public: bool,
    #[serde(default)]
    recall_opt_in: bool,
}

async fn import(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Json(request): Json<ImportRequest>,
) -> ApiResult {
    require_admin(&hub, &headers)?;
    let count = tokio::task::spawn_blocking(move || {
        indexing::import(&hub, &request.root, request.public, request.recall_opt_in)
    })
    .await
    .context("Import task failed")??;
    Ok(Json(json!({"imported":count,"status":"queued"})))
}

async fn approve(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult {
    require_admin(&hub, &headers)?;
    with_store(hub,move|store|{
        let changed=store.db.execute("UPDATE repositories SET approved=1,status='queued',updated_at=?2 WHERE id=?1 AND status='pending_approval'",params![id,now()])?;
        anyhow::ensure!(changed==1,"No pending archive request found"); Ok(json!({"status":"queued","id":id}))
    }).await.map(Json)
}

async fn reindex(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult {
    require_admin(&hub, &headers)?;
    with_store(hub,move|store|{
        let changed=store.db.execute("UPDATE repositories SET status='queued',updated_at=?2 WHERE id=?1 AND approved=1 AND status!='indexing'",params![id,now()])?;
        anyhow::ensure!(changed==1,"Repository missing or already indexing"); Ok(json!({"status":"queued","id":id}))
    }).await.map(Json)
}

async fn feedback(
    State(hub): State<Arc<Hub>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(value): Json<Value>,
) -> ApiResult {
    rate_limit(&hub, &headers, peer, "feedback", 30)?;
    super::storage::validate_feedback(&value).map_err(ApiError::bad)?;
    with_store(hub, move |store| store.save_feedback(value))
        .await
        .map(Json)
}

async fn feedback_list(State(hub): State<Arc<Hub>>, headers: HeaderMap) -> ApiResult {
    require_admin(&hub, &headers)?;
    let items = with_store(hub, |store| store.feedback_list()).await?;
    Ok(Json(json!({"feedback":items})))
}

#[derive(Deserialize)]
struct RecallRequest {
    query: String,
    #[serde(default)]
    collection: String,
    #[serde(default)]
    repository: String,
    #[serde(default)]
    mode: String,
    limit: Option<usize>,
}

async fn recall(
    State(hub): State<Arc<Hub>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<RecallRequest>,
) -> ApiResult {
    if request.query.trim().is_empty()
        || request.query.len() > 512
        || request.collection.len() > 100
        || request.repository.len() > 100
        || !matches!(request.mode.as_str(), "" | "hybrid" | "keyword")
    {
        return Err(ApiError::bad(
            "Use a 1–512 byte query and keyword or hybrid mode",
        ));
    }
    rate_limit(&hub, &headers, peer, "search", 600)?;
    let _permit = hub
        .reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::busy())?;
    let started = Instant::now();
    let hash = auth_hash(&headers);
    let admin = is_admin(&hub, &hash);
    let mut embedding = None;
    if request.mode != "keyword" {
        if let Some(url) = &hub.config.embed_url {
            let texts = vec![request.query.clone()];
            let response = hub
                .http
                .post(url)
                .json(&EmbedRequest {
                    texts: &texts,
                    query: true,
                })
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            if let Ok(response) = response {
                if response.status().is_success() {
                    if let Ok(mut result) = response.json::<Embeddings>().await {
                        if result.validate(1).is_ok() {
                            embedding = Some(result);
                        }
                    }
                }
            }
        }
    }
    let mode = if embedding.is_some() {
        "hybrid"
    } else {
        "keyword"
    };
    let results = with_store(hub, move |store| {
        search(store, &request, &hash, admin, embedding.as_ref())
    })
    .await?;
    Ok(Json(
        json!({"mode":mode,"elapsed_ms":started.elapsed().as_millis(),"results":results,"content_is_untrusted":true}),
    ))
}

fn search(
    store: &mut Store,
    request: &RecallRequest,
    hash: &str,
    admin: bool,
    embedding: Option<&Embeddings>,
) -> Result<Vec<Value>> {
    let allowed = {
        let mut statement = store.db.prepare(
            "SELECT c.id FROM chunks c JOIN repositories r ON r.id=c.repo_id
             WHERE r.approved=1 AND r.recall=1 AND c.generation=r.generation
             AND (r.public=1 OR (r.owner_hash=?1 AND ?1!='') OR ?2)
             AND (?3='' OR r.collection=?3) AND (?4='' OR r.id=?4)",
        )?;
        let rows = statement.query_map(
            params![hash, admin, request.collection, request.repository],
            |row| row.get::<_, i64>(0),
        )?;
        rows.collect::<rusqlite::Result<HashSet<_>>>()?
    };
    let terms: Vec<_> = request
        .query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .take(16)
        .map(|word| format!("\"{word}\""))
        .collect();
    let mut scores: HashMap<i64, f32> = HashMap::new();
    if !terms.is_empty() {
        let mut statement=store.db.prepare(
            "SELECT t.rowid FROM chunk_terms t JOIN chunks c ON c.id=t.rowid JOIN repositories r ON r.id=c.repo_id
             WHERE chunk_terms MATCH ?1 AND r.approved=1 AND r.recall=1 AND c.generation=r.generation
             AND (r.public=1 OR (r.owner_hash=?2 AND ?2!='') OR ?3)
             AND (?4='' OR r.collection=?4) AND (?5='' OR r.id=?5)
             ORDER BY bm25(chunk_terms,2.0,1.0) LIMIT 100"
        )?;
        let rows = statement.query_map(
            params![
                terms.join(" OR "),
                hash,
                admin,
                request.collection,
                request.repository
            ],
            |row| row.get::<_, i64>(0),
        )?;
        for (rank, id) in rows.enumerate() {
            scores.insert(id?, 1.0 / (60 + rank) as f32);
        }
    }
    if let Some(query) = embedding {
        let vector = &query.vectors[0];
        let mut nearest: Vec<_> = store
            .vectors
            .iter()
            .filter(|(id, memory)| allowed.contains(id) && memory.model == query.model)
            .map(|(&id, memory)| {
                (
                    id,
                    memory
                        .vector
                        .iter()
                        .zip(vector)
                        .map(|(left, right)| left * right)
                        .sum::<f32>(),
                )
            })
            .filter(|(_, score)| *score > 0.25)
            .collect();
        nearest.sort_unstable_by(|left, right| right.1.total_cmp(&left.1));
        for (rank, (id, _)) in nearest.into_iter().take(100).enumerate() {
            *scores.entry(id).or_default() += 1.0 / (60 + rank) as f32;
        }
    }
    let mut candidates: Vec<_> = scores.into_iter().collect();
    candidates
        .sort_unstable_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(&right.0)));
    let mut output = Vec::new();
    let mut per_file: HashMap<(String, String), usize> = HashMap::new();
    for (id, score) in candidates {
        if output.len() >= request.limit.unwrap_or(10).clamp(1, 50) {
            break;
        }
        if !allowed.contains(&id) {
            continue;
        }
        let passage = store.passage(id)?;
        let count = per_file
            .entry((passage.repo_id.clone(), passage.path.clone()))
            .or_default();
        if *count >= 2 {
            continue;
        }
        *count += 1;
        let repo = store
            .repository(&passage.repo_id)?
            .context("Passage repository missing")?;
        let mut source = reqwest::Url::parse(repo.source_url.trim_end_matches(".git"))?;
        source
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Invalid source URL"))?
            .extend(["blob", &passage.commit])
            .extend(passage.path.split('/'));
        source.set_fragment(Some(&format!(
            "L{}-L{}",
            passage.line_start, passage.line_end
        )));
        output.push(json!({"repository_id":repo.id,"repository":format!("{}/{}",repo.collection,repo.name),"path":passage.path,"commit":passage.commit,"line_start":passage.line_start,"line_end":passage.line_end,"text":passage.text,"score":score,"source_url":source.as_str()}));
    }
    Ok(output)
}

async fn latest_version(State(hub): State<Arc<Hub>>) -> ApiResult {
    let mut cache = hub.latest_release.lock().await;
    if let Some((fetched, value)) = cache.as_ref() {
        if fetched.elapsed() < Duration::from_secs(3600) {
            return Ok(Json(value.clone()));
        }
    }
    let response = hub
        .http
        .get("https://i1.is/releases/smart-tree/latest.json")
        .header(header::USER_AGENT, "smart-tree-hub")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "Release information is temporarily unavailable".into(),
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Release information is temporarily unavailable".into(),
        ));
    }
    let value: Value = response.json().await.context("Read release metadata")?;
    let version = value["tag_name"]
        .as_str()
        .context("Release tag missing")?
        .trim_start_matches('v');
    let result = json!({"version":version,"release_date":value["published_at"],"download_url":value["html_url"],"release_notes_url":value["html_url"],"features":[],"ai_benefits":[]});
    *cache = Some((Instant::now(), result.clone()));
    Ok(Json(result))
}

async fn git_get(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path((id, rest)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> std::result::Result<Response, ApiError> {
    if rest != "info/refs" || query.get("service").map(String::as_str) != Some("git-upload-pack") {
        return Err(ApiError::missing());
    }
    serve_git(hub, headers, id, &rest, "GET", Bytes::new()).await
}

async fn git_post(
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    Path((id, rest)): Path<(String, String)>,
    body: Bytes,
) -> std::result::Result<Response, ApiError> {
    if rest != "git-upload-pack" {
        return Err(ApiError::missing());
    }
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        != Some("application/x-git-upload-pack-request")
    {
        return Err(ApiError::bad("Git upload-pack content type required"));
    }
    serve_git(hub, headers, id, &rest, "POST", body).await
}

async fn serve_git(
    hub: Arc<Hub>,
    headers: HeaderMap,
    id: String,
    rest: &str,
    method: &str,
    body: Bytes,
) -> std::result::Result<Response, ApiError> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
    let _permit = hub
        .git_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::busy())?;
    let hash = auth_hash(&headers);
    let admin = is_admin(&hub, &hash);
    let id = id.strip_suffix(".git").unwrap_or(&id).to_owned();
    let repo = with_store(hub, move |store| store.repository(&id))
        .await?
        .ok_or_else(ApiError::missing)?;
    if !repo.visible(&hash, admin) {
        return Ok((
            StatusCode::UNAUTHORIZED,
            [(
                header::WWW_AUTHENTICATE,
                "Basic realm=\"Smart Tree archive\"",
            )],
            "An archive receipt token is required",
        )
            .into_response());
    }
    if !repo.approved || repo.git_dir.is_empty() {
        return Err(ApiError::missing());
    }
    let translated = std::path::Path::new(&repo.git_dir).join(rest);
    let mut command = tokio::process::Command::new("git");
    command
        // Imported collections can belong to the server operator. Trust only
        // this validated, registered Git directory for the upload-pack process.
        .arg("-c")
        .arg(format!("safe.directory={}", repo.git_dir))
        .args([
            "-c",
            "http.receivepack=false",
            "-c",
            "core.hooksPath=/dev/null",
            "http-backend",
        ])
        .env_remove("GIT_PROJECT_ROOT")
        .env("GIT_HTTP_EXPORT_ALL", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("PATH_TRANSLATED", translated)
        .env("REQUEST_METHOD", method)
        .env(
            "QUERY_STRING",
            if method == "GET" {
                "service=git-upload-pack"
            } else {
                ""
            },
        )
        .env("CONTENT_TYPE", "application/x-git-upload-pack-request")
        .env("CONTENT_LENGTH", body.len().to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    if let Some(protocol) = headers
        .get("git-protocol")
        .and_then(|v| v.to_str().ok())
        .filter(|v| matches!(*v, "version=1" | "version=2"))
    {
        command.env("GIT_PROTOCOL", protocol);
    }
    let mut child = command.spawn().context("Start Git upload-pack")?;
    let mut input = child.stdin.take().context("Open Git stdin")?;
    tokio::spawn(async move {
        let _ = input.write_all(&body).await;
    });
    let output = child.stdout.take().context("Open Git stdout")?;
    let mut output = tokio::io::BufReader::new(output);
    let mut response = Response::builder().status(StatusCode::OK);
    let mut total = 0;
    loop {
        let mut line = String::new();
        let count = tokio::time::timeout(Duration::from_secs(30), output.read_line(&mut line))
            .await
            .context("Git headers timed out")?
            .context("Read Git headers")?;
        total += count;
        if total > 16384 || count == 0 {
            return Err(ApiError::bad("Invalid Git response"));
        }
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            if name.eq_ignore_ascii_case("status") {
                if let Some(code) = value
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u16>().ok())
                {
                    response = response.status(code);
                }
            } else if ["content-type", "cache-control", "expires", "pragma"]
                .contains(&name.to_ascii_lowercase().as_str())
            {
                response = response.header(name, value);
            }
        }
    }
    let stream = futures::stream::try_unfold(
        (output, child, _permit),
        |(mut output, child, permit)| async move {
            let mut buffer = vec![0; 65536];
            let count = tokio::time::timeout(Duration::from_secs(120), output.read(&mut buffer))
                .await
                .map_err(std::io::Error::other)??;
            if count == 0 {
                Ok::<_, std::io::Error>(None)
            } else {
                buffer.truncate(count);
                Ok(Some((Bytes::from(buffer), (output, child, permit))))
            }
        },
    );
    response
        .body(Body::from_stream(stream))
        .context("Build Git response")
        .map_err(Into::into)
}
