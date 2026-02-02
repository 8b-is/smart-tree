//! Authentication Module - GitHub OAuth for Collaboration
//!
//! Supports i1.is and other services via GitHub login.
//!
//! Flow:
//! 1. Client requests /auth/github/login
//! 2. Redirect to GitHub OAuth
//! 3. GitHub redirects back with code
//! 4. Exchange code for access token
//! 5. Fetch user info from GitHub API
//! 6. Create session token

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// GitHub OAuth configuration
#[derive(Debug, Clone)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

impl GitHubOAuthConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            client_id: std::env::var("GITHUB_CLIENT_ID").ok()?,
            client_secret: std::env::var("GITHUB_CLIENT_SECRET").ok()?,
            redirect_uri: std::env::var("GITHUB_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8420/auth/github/callback".to_string()),
            scope: std::env::var("GITHUB_SCOPE")
                .unwrap_or_else(|_| "read:user user:email".to_string()),
        })
    }

    pub fn authorization_url(&self, state: &str) -> String {
        // Simple URL encoding for OAuth parameters
        let encode = |s: &str| -> String {
            s.chars()
                .map(|c| match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                    ' ' => "+".to_string(),
                    _ => format!("%{:02X}", c as u8),
                })
                .collect()
        };
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            self.client_id,
            encode(&self.redirect_uri),
            encode(&self.scope),
            state
        )
    }
}

/// GitHub user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

/// Authenticated user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub github_user: GitHubUser,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Associated collaboration participant ID
    pub participant_id: Option<String>,
}

impl UserSession {
    pub fn new(github_user: GitHubUser) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            github_user,
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            participant_id: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn display_name(&self) -> String {
        self.github_user
            .name
            .clone()
            .unwrap_or_else(|| self.github_user.login.clone())
    }
}

/// Session store
pub struct SessionStore {
    sessions: HashMap<String, UserSession>,
    /// Pending OAuth states (state -> timestamp)
    pending_states: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            pending_states: HashMap::new(),
        }
    }

    /// Generate a new OAuth state token
    pub fn create_oauth_state(&mut self) -> String {
        let state = Uuid::new_v4().to_string();
        self.pending_states.insert(state.clone(), chrono::Utc::now());
        state
    }

    /// Validate and consume an OAuth state
    pub fn validate_oauth_state(&mut self, state: &str) -> bool {
        if let Some(created) = self.pending_states.remove(state) {
            // State is valid for 10 minutes
            chrono::Utc::now() - created < chrono::Duration::minutes(10)
        } else {
            false
        }
    }

    /// Store a session
    pub fn create_session(&mut self, github_user: GitHubUser) -> UserSession {
        let session = UserSession::new(github_user);
        self.sessions.insert(session.session_id.clone(), session.clone());
        session
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&UserSession> {
        self.sessions.get(session_id).filter(|s| !s.is_expired())
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut UserSession> {
        self.sessions.get_mut(session_id).filter(|s| !s.is_expired())
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        self.sessions.retain(|_, s| s.expires_at > now);
        self.pending_states
            .retain(|_, created| now - *created < chrono::Duration::minutes(10));
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe session store
pub type SharedSessionStore = Arc<RwLock<SessionStore>>;

/// Create a new shared session store
pub fn create_session_store() -> SharedSessionStore {
    Arc::new(RwLock::new(SessionStore::new()))
}

/// Exchange GitHub OAuth code for access token
pub async fn exchange_code_for_token(
    config: &GitHubOAuthConfig,
    code: &str,
) -> Result<String> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let client = reqwest::Client::new();
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("code", &code.to_string()),
            ("redirect_uri", &config.redirect_uri),
        ])
        .send()
        .await?;

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
}

/// Fetch GitHub user info using access token
pub async fn fetch_github_user(access_token: &str) -> Result<GitHubUser> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "Smart-Tree-Daemon")
        .send()
        .await?;

    let user: GitHubUser = response.json().await?;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let user = GitHubUser {
            id: 12345,
            login: "testuser".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            avatar_url: "https://example.com/avatar.png".to_string(),
        };

        let session = UserSession::new(user);
        assert!(!session.is_expired());
        assert_eq!(session.display_name(), "Test User");
    }

    #[test]
    fn test_oauth_state() {
        let mut store = SessionStore::new();
        let state = store.create_oauth_state();
        assert!(store.validate_oauth_state(&state));
        // Can't reuse state
        assert!(!store.validate_oauth_state(&state));
    }
}
