//! OAuth 2.0 + PKCE framework for proxy providers.
//!
//! No high-level crate — just reqwest + sha2 + base64 + rand. The flow:
//!
//!   1. `begin(provider)` generates code_verifier/challenge, binds a loopback
//!      listener on 127.0.0.1:<random>, returns the authorization URL.
//!   2. User opens the URL, completes login, provider redirects back to the
//!      loopback URL with `?code=...&state=...`.
//!   3. The loopback task exchanges the code for tokens and stores them via
//!      `token_store::save`.
//!
//! Designed to be driven from the proxy admin API and/or the CLI.

use crate::proxy::token_store::{self, StoredToken};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// Static configuration for a single OAuth provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: &'static str,
    pub auth_url: &'static str,
    pub token_url: &'static str,
    /// Only filled from env / user config. We do not ship client_ids.
    pub client_id_env: &'static str,
    /// Some providers (Google desktop apps, GitHub) require a client_secret;
    /// others (public native apps w/ PKCE) don't. None => PKCE-only.
    pub client_secret_env: Option<&'static str>,
    pub scopes: &'static [&'static str],
    /// Extra query params appended to the auth URL (e.g. `access_type=offline`).
    pub extra_auth_params: &'static [(&'static str, &'static str)],
}

impl ProviderConfig {
    fn client_id(&self) -> Result<String> {
        std::env::var(self.client_id_env)
            .map_err(|_| anyhow!("{} not set — register an OAuth app and export the client id", self.client_id_env))
    }

    fn client_secret(&self) -> Option<String> {
        self.client_secret_env.and_then(|k| std::env::var(k).ok())
    }
}

pub mod providers {
    use super::ProviderConfig;

    /// Google AI Studio / Antigravity. Uses standard Google OAuth2.
    /// Register a "Desktop app" OAuth client in GCP and export client id/secret.
    pub const ANTIGRAVITY: ProviderConfig = ProviderConfig {
        name: "antigravity",
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
        token_url: "https://oauth2.googleapis.com/token",
        client_id_env: "ANTIGRAVITY_CLIENT_ID",
        client_secret_env: Some("ANTIGRAVITY_CLIENT_SECRET"),
        scopes: &[
            "https://www.googleapis.com/auth/generative-language",
            "openid",
            "email",
        ],
        extra_auth_params: &[("access_type", "offline"), ("prompt", "consent")],
    };

    /// OpenAI Codex CLI — same public OAuth endpoints as ChatGPT desktop.
    /// Placeholder scopes; refine once we verify against Codex CLI source.
    pub const CODEX: ProviderConfig = ProviderConfig {
        name: "codex",
        auth_url: "https://auth.openai.com/authorize",
        token_url: "https://auth.openai.com/oauth/token",
        client_id_env: "CODEX_CLIENT_ID",
        client_secret_env: None,
        scopes: &["openid", "email", "profile", "offline_access"],
        extra_auth_params: &[],
    };

    pub fn by_name(name: &str) -> Option<ProviderConfig> {
        match name.to_lowercase().as_str() {
            "antigravity" | "google" => Some(ANTIGRAVITY),
            "codex" | "openai" => Some(CODEX),
            _ => None,
        }
    }
}

/// A flow-in-progress: the caller opens `auth_url`, we wait for the callback,
/// exchange the code, and store the resulting token.
pub struct StartedFlow {
    pub auth_url: String,
    pub state: String,
    pub redirect_uri: String,
    done: oneshot::Receiver<Result<StoredToken>>,
}

impl StartedFlow {
    /// Wait up to `timeout` for the user to complete the flow.
    pub async fn wait(self, timeout: Duration) -> Result<StoredToken> {
        match tokio::time::timeout(timeout, self.done).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => bail!("oauth callback channel closed"),
            Err(_) => bail!("oauth flow timed out"),
        }
    }
}

/// Kick off an OAuth flow. Returns a StartedFlow with the auth URL to open.
pub async fn begin(provider: ProviderConfig, account: String) -> Result<StartedFlow> {
    let client_id = provider.client_id()?;
    let client_secret = provider.client_secret();

    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let scope = provider.scopes.join(" ");
    let mut auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        provider.auth_url,
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scope),
        urlencoding::encode(&state),
        urlencoding::encode(&challenge),
    );
    for (k, v) in provider.extra_auth_params {
        auth_url.push_str(&format!("&{}={}", k, urlencoding::encode(v)));
    }

    let (tx, rx) = oneshot::channel::<Result<StoredToken>>();
    let state_expected = state.clone();
    let redirect_uri_cloned = redirect_uri.clone();
    let provider_cloned = provider.clone();
    let account_cloned = account.clone();

    tokio::spawn(async move {
        let res = run_callback(
            listener,
            &state_expected,
            &redirect_uri_cloned,
            &verifier,
            &client_id,
            client_secret.as_deref(),
            &provider_cloned,
            &account_cloned,
        )
        .await;
        let _ = tx.send(res);
    });

    Ok(StartedFlow {
        auth_url,
        state,
        redirect_uri,
        done: rx,
    })
}

async fn run_callback(
    listener: TcpListener,
    state_expected: &str,
    redirect_uri: &str,
    verifier: &str,
    client_id: &str,
    client_secret: Option<&str>,
    provider: &ProviderConfig,
    account: &str,
) -> Result<StoredToken> {
    let (mut stream, _) = listener.accept().await?;
    let (code, state_got) = read_callback_query(&mut stream).await?;
    if state_got != state_expected {
        write_plain(&mut stream, "oauth state mismatch — possible CSRF, aborting").await;
        bail!("oauth state mismatch");
    }

    let token = exchange_code(
        provider,
        client_id,
        client_secret,
        &code,
        redirect_uri,
        verifier,
    )
    .await;

    match &token {
        Ok(_) => {
            write_plain(
                &mut stream,
                "Smart Tree proxy: sign-in complete. You can close this tab.",
            )
            .await
        }
        Err(e) => {
            write_plain(&mut stream, &format!("sign-in failed: {}", e)).await;
        }
    }

    let token = token?;
    token_store::save(provider.name, account, &token)?;
    Ok(token)
}

async fn read_callback_query(
    stream: &mut tokio::net::TcpStream,
) -> Result<(String, String)> {
    use tokio::io::AsyncReadExt;
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().context("empty HTTP request")?;
    // "GET /callback?code=...&state=... HTTP/1.1"
    let path = first_line
        .split_whitespace()
        .nth(1)
        .context("malformed request line")?;
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut code = None;
    let mut state = None;
    let mut error = None;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let v = urlencoding::decode(v).unwrap_or_default().into_owned();
            match k {
                "code" => code = Some(v),
                "state" => state = Some(v),
                "error" => error = Some(v),
                _ => {}
            }
        }
    }
    if let Some(e) = error {
        bail!("oauth provider returned error: {}", e);
    }
    Ok((
        code.context("missing code in callback")?,
        state.context("missing state in callback")?,
    ))
}

async fn write_plain(stream: &mut tokio::net::TcpStream, body: &str) {
    use tokio::io::AsyncWriteExt;
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes()).await;
    let _ = stream.shutdown().await;
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
}

#[derive(Serialize)]
struct TokenExchange<'a> {
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
    client_id: &'a str,
    code_verifier: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<&'a str>,
}

async fn exchange_code(
    provider: &ProviderConfig,
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<StoredToken> {
    let body = TokenExchange {
        grant_type: "authorization_code",
        code,
        redirect_uri,
        client_id,
        code_verifier: verifier,
        client_secret,
    };

    let res = Client::new()
        .post(provider.token_url)
        .form(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        bail!("token endpoint returned error: {}", text);
    }

    let t: TokenResponse = res.json().await?;
    let expires_at = t
        .expires_in
        .map(|s| chrono::Utc::now() + chrono::Duration::seconds(s));
    Ok(StoredToken {
        access_token: t.access_token,
        refresh_token: t.refresh_token,
        expires_at,
        scope: t.scope,
        token_type: t.token_type,
    })
}

/// Refresh an existing stored token in place. Returns the refreshed token.
pub async fn refresh(provider: ProviderConfig, account: &str) -> Result<StoredToken> {
    let current = token_store::load(provider.name, account)?
        .ok_or_else(|| anyhow!("no stored token for {}:{}", provider.name, account))?;
    let refresh_token = current
        .refresh_token
        .as_deref()
        .ok_or_else(|| anyhow!("stored token has no refresh_token"))?;

    let client_id = provider.client_id()?;
    let client_secret = provider.client_secret();

    #[derive(Serialize)]
    struct RefreshBody<'a> {
        grant_type: &'a str,
        refresh_token: &'a str,
        client_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_secret: Option<&'a str>,
    }

    let res = Client::new()
        .post(provider.token_url)
        .form(&RefreshBody {
            grant_type: "refresh_token",
            refresh_token,
            client_id: &client_id,
            client_secret: client_secret.as_deref(),
        })
        .send()
        .await?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        bail!("refresh failed: {}", text);
    }

    let t: TokenResponse = res.json().await?;
    let expires_at = t
        .expires_in
        .map(|s| chrono::Utc::now() + chrono::Duration::seconds(s));
    let refreshed = StoredToken {
        access_token: t.access_token,
        // Google omits refresh_token on refresh responses; keep the original.
        refresh_token: t.refresh_token.or(current.refresh_token),
        expires_at,
        scope: t.scope.or(current.scope),
        token_type: t.token_type.or(current.token_type),
    };
    token_store::save(provider.name, account, &refreshed)?;
    Ok(refreshed)
}

/// Load a token, transparently refreshing if it's expired.
pub async fn load_fresh(provider: ProviderConfig, account: &str) -> Result<StoredToken> {
    match token_store::load(provider.name, account)? {
        Some(t) if !t.is_expired() => Ok(t),
        Some(_) => refresh(provider, account).await,
        None => bail!("no stored token for {}:{}", provider.name, account),
    }
}

// Small dependency-free urlencoding shim so we don't pull another crate.
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{:02X}", b)),
            }
        }
        out
    }

    pub fn decode(s: &str) -> Option<std::borrow::Cow<'_, str>> {
        let mut out = Vec::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'+' => {
                    out.push(b' ');
                    i += 1;
                }
                b'%' if i + 2 < bytes.len() => {
                    let hi = (bytes[i + 1] as char).to_digit(16)?;
                    let lo = (bytes[i + 2] as char).to_digit(16)?;
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                }
                c => {
                    out.push(c);
                    i += 1;
                }
            }
        }
        Some(String::from_utf8(out).ok()?.into())
    }
}
