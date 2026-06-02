//! Google OAuth2 authentication for Gmail and Drive
//!
//! Supports two modes:
//! - User OAuth2: Opens browser for consent (local) or prints URL (remote/SSH)
//! - Service Account: Headless auth with JSON key file
//!
//! Token refresh is handled automatically by yup-oauth2.

use anyhow::{Context, Result};
use chrono::Utc;
use yup_oauth2::{
    authenticator::Authenticator, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
    ServiceAccountAuthenticator, ServiceAccountKey,
};

use super::models::{AuthConfig, AuthMethod};
use super::token_store::TokenStore;

/// Required Gmail API scopes
pub const GMAIL_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/gmail.labels",
];

/// Required Drive API scopes
pub const DRIVE_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/drive.file",
];

/// All scopes needed for full Gmail + Drive integration
pub fn all_scopes() -> Vec<&'static str> {
    let mut scopes = Vec::new();
    scopes.extend_from_slice(GMAIL_SCOPES);
    scopes.extend_from_slice(DRIVE_SCOPES);
    scopes
}

/// Google authentication manager
pub struct GoogleAuth {
    token_store: TokenStore,
}

/// Opaque authenticator handle for making API calls
pub type GoogleAuthenticator =
    Authenticator<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>;

impl GoogleAuth {
    /// Create a new auth manager using the given storage directory
    pub fn new(store_dir: std::path::PathBuf) -> Result<Self> {
        let token_store = TokenStore::new(store_dir)?;
        Ok(Self { token_store })
    }

    /// Create using the default storage directory (~/.st/google/)
    pub fn default_store() -> Result<Self> {
        Self::new(super::google_state_dir())
    }

    /// Start user OAuth2 authentication flow
    ///
    /// - `local_browser`: If true, opens browser and runs local redirect server.
    ///   If false, prints URL for manual copy (SSH/remote scenarios).
    /// - `port`: Optional port for the local redirect server (useful for port forwarding).
    pub async fn authenticate_user(
        &self,
        client_id: &str,
        client_secret: &str,
        local_browser: bool,
        port: Option<u16>,
    ) -> Result<GoogleAuthenticator> {
        let secret = yup_oauth2::ApplicationSecret {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uris: vec![format!("http://localhost:{}", port.unwrap_or(8085))],
            ..Default::default()
        };

        let return_method = if local_browser {
            InstalledFlowReturnMethod::HTTPRedirect
        } else {
            InstalledFlowReturnMethod::Interactive
        };

        // Token cache path for yup-oauth2's built-in persistence
        let token_cache_path = super::google_state_dir().join("oauth2_tokens.json");

        let auth = InstalledFlowAuthenticator::builder(secret, return_method)
            .persist_tokens_to_disk(&token_cache_path)
            .build()
            .await
            .context("Failed to build OAuth2 authenticator")?;

        // Save config for status checks
        let config = AuthConfig {
            method: AuthMethod::UserOAuth2 {
                client_id: client_id.to_string(),
                client_secret: "***".to_string(), // Don't store secret in config
                redirect_port: port,
            },
            scopes: all_scopes().iter().map(|s| s.to_string()).collect(),
            account_email: None,
            authenticated_at: Some(Utc::now()),
        };
        self.token_store.save_config(&config)?;

        Ok(auth)
    }

    /// Authenticate with a Google service account
    pub async fn authenticate_service_account(
        &self,
        key_path: &str,
        delegate_email: Option<&str>,
    ) -> Result<GoogleAuthenticator> {
        let key_data =
            std::fs::read_to_string(key_path).context("Failed to read service account key")?;
        let key: ServiceAccountKey =
            serde_json::from_str(&key_data).context("Failed to parse service account key")?;

        let mut builder = ServiceAccountAuthenticator::builder(key);

        if let Some(email) = delegate_email {
            builder = builder.subject(email.to_string());
        }

        let auth = builder
            .build()
            .await
            .context("Failed to build service account authenticator")?;

        // Save config
        let config = AuthConfig {
            method: AuthMethod::ServiceAccount {
                key_path: key_path.to_string(),
                delegate_email: delegate_email.map(String::from),
            },
            scopes: all_scopes().iter().map(|s| s.to_string()).collect(),
            account_email: delegate_email.map(String::from),
            authenticated_at: Some(Utc::now()),
        };
        self.token_store.save_config(&config)?;

        Ok(auth)
    }

    /// Check if we have stored auth configuration
    pub fn has_config(&self) -> bool {
        self.token_store.load_config().ok().flatten().is_some()
    }

    /// Get the stored auth configuration (if any)
    pub fn get_config(&self) -> Result<Option<AuthConfig>> {
        self.token_store.load_config()
    }

    /// Check if OAuth2 tokens are cached on disk
    pub fn has_cached_tokens(&self) -> bool {
        super::google_state_dir()
            .join("oauth2_tokens.json")
            .exists()
    }

    /// Clear all stored credentials
    pub fn logout(&self) -> Result<()> {
        // Remove yup-oauth2 token cache
        let token_path = super::google_state_dir().join("oauth2_tokens.json");
        if token_path.exists() {
            std::fs::remove_file(&token_path).context("Failed to remove token cache")?;
        }

        // Remove encrypted tokens
        self.token_store.clear("tokens")?;

        // Remove config
        let config_path = super::google_state_dir().join("config.json");
        if config_path.exists() {
            std::fs::remove_file(&config_path).context("Failed to remove config")?;
        }

        Ok(())
    }

    /// Get a status summary of the current auth state
    pub fn status_summary(&self) -> String {
        match self.get_config() {
            Ok(Some(config)) => {
                let method = match &config.method {
                    AuthMethod::UserOAuth2 { client_id, .. } => {
                        format!(
                            "User OAuth2 (client: {}...)",
                            &client_id[..8.min(client_id.len())]
                        )
                    }
                    AuthMethod::ServiceAccount { key_path, .. } => {
                        format!("Service Account ({})", key_path)
                    }
                };
                let email = config.account_email.as_deref().unwrap_or("unknown");
                let tokens = if self.has_cached_tokens() {
                    "cached"
                } else {
                    "not cached"
                };
                let auth_time = config
                    .authenticated_at
                    .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                format!(
                    "Authenticated\n  Method: {method}\n  Account: {email}\n  Tokens: {tokens}\n  Since: {auth_time}"
                )
            }
            _ => "Not authenticated. Use auth_login to connect your Google account.".to_string(),
        }
    }
}
