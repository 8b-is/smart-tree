//! Claude API Error Types
//!
//! Typed errors matching every HTTP status code the Claude API returns.
//! Each variant carries the error message from the API response body.
//!
//! Retryable errors: RateLimited (429), InternalError (500), Overloaded (529)
//! Non-retryable: everything else (fix the request before retrying)

use serde::Deserialize;
use std::fmt;

// ---------------------------------------------------------------------------
// Typed API error enum
// ---------------------------------------------------------------------------

/// Every error the Claude Messages API can return, plus network/parse errors.
///
/// # Example
/// ```rust,no_run
/// match err {
///     ClaudeApiError::RateLimited { message } => { /* back off and retry */ }
///     ClaudeApiError::InvalidRequest { message } => { /* fix request */ }
///     _ => { /* log and bail */ }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum ClaudeApiError {
    /// 400 - Malformed JSON, missing params, invalid values
    #[error("Invalid request (400): {message}")]
    InvalidRequest { message: String },

    /// 401 - Missing or invalid API key
    #[error("Authentication failed (401): {message}")]
    AuthenticationError { message: String },

    /// 403 - API key lacks permission for the requested model/feature
    #[error("Permission denied (403): {message}")]
    PermissionDenied { message: String },

    /// 404 - Bad endpoint or invalid model ID
    #[error("Not found (404): {message}")]
    NotFound { message: String },

    /// 413 - Request body too large (too many tokens, huge images, etc.)
    #[error("Request too large (413): {message}")]
    RequestTooLarge { message: String },

    /// 429 - Rate limited (retryable - check `retry-after` header)
    #[error("Rate limited (429): {message}")]
    RateLimited { message: String },

    /// 500 - Anthropic server issue (retryable)
    #[error("Internal server error (500): {message}")]
    InternalError { message: String },

    /// 529 - API overloaded (retryable)
    #[error("API overloaded (529): {message}")]
    Overloaded { message: String },

    /// Network-level failure (DNS, TLS, connection refused, etc.)
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Failed to parse the API response JSON
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// SSE stream delivered an error event or broke mid-stream
    #[error("Stream error: {message}")]
    StreamError { message: String },

    /// Catch-all for unexpected HTTP status codes
    #[error("Unexpected API error ({status}): {message}")]
    Unknown { status: u16, message: String },
}

impl ClaudeApiError {
    /// Returns true if the error is safe to retry with backoff.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ClaudeApiError::RateLimited { .. }
                | ClaudeApiError::InternalError { .. }
                | ClaudeApiError::Overloaded { .. }
        )
    }
}

// ---------------------------------------------------------------------------
// Raw API error response (for deserializing the JSON body)
// ---------------------------------------------------------------------------

/// The top-level error envelope from the Claude API:
/// `{ "type": "error", "error": { "type": "...", "message": "..." } }`
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub error: ApiErrorBody,
}

/// The inner error object with machine-readable type and human-readable message.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    /// Machine-readable error type, e.g. "invalid_request_error"
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-readable description of what went wrong
    pub message: String,
}

impl fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

// ---------------------------------------------------------------------------
// Parsing: HTTP status + body -> ClaudeApiError
// ---------------------------------------------------------------------------

impl ClaudeApiError {
    /// Parse an API error from the HTTP status code and response body text.
    ///
    /// Tries to deserialize the structured error JSON first; falls back to
    /// using the raw body text if parsing fails.
    pub fn from_response(status: u16, body: &str) -> Self {
        // Try to extract the structured error message
        let message = serde_json::from_str::<ApiErrorResponse>(body)
            .map(|r| r.error.message)
            .unwrap_or_else(|_| body.to_string());

        match status {
            400 => ClaudeApiError::InvalidRequest { message },
            401 => ClaudeApiError::AuthenticationError { message },
            403 => ClaudeApiError::PermissionDenied { message },
            404 => ClaudeApiError::NotFound { message },
            413 => ClaudeApiError::RequestTooLarge { message },
            429 => ClaudeApiError::RateLimited { message },
            500 => ClaudeApiError::InternalError { message },
            529 => ClaudeApiError::Overloaded { message },
            _ => ClaudeApiError::Unknown { status, message },
        }
    }
}
