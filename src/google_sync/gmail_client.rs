//! Gmail API client wrapper
//!
//! Provides high-level methods for listing, searching, and downloading
//! Gmail messages. Uses the google-gmail1 crate for API calls.

use anyhow::{Context, Result};
use google_gmail1::api::{Message, ModifyMessageRequest};
use google_gmail1::Gmail;

use super::auth::GoogleAuthenticator;
use super::models::EmailMetadata;
use super::rate_limiter::RateLimiter;

/// Gmail API client with rate limiting
pub struct GmailClient {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    rate_limiter: RateLimiter,
}

impl GmailClient {
    /// Create a new Gmail client with the given authenticator
    pub fn new(auth: GoogleAuthenticator) -> Self {
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .expect("native TLS roots")
                    .https_or_http()
                    .enable_http1()
                    .build(),
            );
        let hub = Gmail::new(client, auth);
        Self {
            hub,
            rate_limiter: RateLimiter::gmail(),
        }
    }

    /// List messages with optional query filter
    pub async fn list_messages(
        &self,
        query: Option<&str>,
        label_ids: Option<&[&str]>,
        max_results: u32,
        page_token: Option<&str>,
    ) -> Result<(Vec<EmailMetadata>, Option<String>)> {
        self.rate_limiter.acquire().await;

        let mut req = self.hub.users().messages_list("me").max_results(max_results);

        if let Some(q) = query {
            req = req.q(q);
        }
        if let Some(labels) = label_ids {
            for label in labels {
                req = req.add_label_ids(label);
            }
        }
        if let Some(token) = page_token {
            req = req.page_token(token);
        }

        let (_, response) = req.doit().await.context("Failed to list messages")?;

        let next_page = response.next_page_token;
        let messages = response.messages.unwrap_or_default();

        // Fetch metadata for each message
        let mut metadata = Vec::with_capacity(messages.len());
        for msg in &messages {
            if let Some(id) = &msg.id {
                match self.get_message_metadata(id).await {
                    Ok(meta) => metadata.push(meta),
                    Err(e) => tracing::warn!("Failed to fetch metadata for {}: {}", id, e),
                }
            }
        }

        Ok((metadata, next_page))
    }

    /// Get message metadata (lightweight, no body content)
    pub async fn get_message_metadata(&self, message_id: &str) -> Result<EmailMetadata> {
        self.rate_limiter.acquire().await;

        let (_, msg) = self
            .hub
            .users()
            .messages_get("me", message_id)
            .format("metadata")
            .add_metadata_headers("From")
            .add_metadata_headers("To")
            .add_metadata_headers("Subject")
            .add_metadata_headers("Date")
            .add_metadata_headers("List-Unsubscribe")
            .doit()
            .await
            .context("Failed to get message metadata")?;

        Self::parse_message_metadata(&msg)
    }

    /// Download full message as raw RFC 2822 bytes (.eml format)
    pub async fn get_message_raw(&self, message_id: &str) -> Result<Vec<u8>> {
        self.rate_limiter.acquire().await;

        let (_, msg) = self
            .hub
            .users()
            .messages_get("me", message_id)
            .format("raw")
            .doit()
            .await
            .context("Failed to get raw message")?;

        let raw = msg.raw.context("Message has no raw content")?;
        // Gmail returns base64url encoded raw content
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&raw)
            .context("Failed to decode raw message")?;

        Ok(decoded)
    }

    /// Search messages using Gmail query syntax
    pub async fn search(&self, query: &str, max_results: u32) -> Result<Vec<EmailMetadata>> {
        let (results, _) = self.list_messages(Some(query), None, max_results, None).await?;
        Ok(results)
    }

    /// List all labels in the account
    pub async fn list_labels(&self) -> Result<Vec<(String, String)>> {
        self.rate_limiter.acquire().await;

        let (_, response) = self
            .hub
            .users()
            .labels_list("me")
            .doit()
            .await
            .context("Failed to list labels")?;

        Ok(response
            .labels
            .unwrap_or_default()
            .into_iter()
            .filter_map(|l| Some((l.id?, l.name?)))
            .collect())
    }

    /// Modify labels on a message (for archiving: remove INBOX, add label)
    pub async fn modify_labels(
        &self,
        message_id: &str,
        add_labels: &[&str],
        remove_labels: &[&str],
    ) -> Result<()> {
        self.rate_limiter.acquire().await;

        let req = ModifyMessageRequest {
            add_label_ids: Some(add_labels.iter().map(|s| s.to_string()).collect()),
            remove_label_ids: Some(remove_labels.iter().map(|s| s.to_string()).collect()),
        };

        self.hub
            .users()
            .messages_modify(req, "me", message_id)
            .doit()
            .await
            .context("Failed to modify message labels")?;

        Ok(())
    }

    /// Get all messages in a thread
    pub async fn get_thread_messages(&self, thread_id: &str) -> Result<Vec<EmailMetadata>> {
        self.rate_limiter.acquire().await;

        let (_, thread) = self
            .hub
            .users()
            .threads_get("me", thread_id)
            .format("metadata")
            .add_metadata_headers("From")
            .add_metadata_headers("To")
            .add_metadata_headers("Subject")
            .add_metadata_headers("Date")
            .doit()
            .await
            .context("Failed to get thread")?;

        let messages = thread.messages.unwrap_or_default();
        let mut metadata = Vec::with_capacity(messages.len());
        for msg in &messages {
            match Self::parse_message_metadata(msg) {
                Ok(meta) => metadata.push(meta),
                Err(e) => tracing::warn!("Failed to parse thread message: {}", e),
            }
        }

        Ok(metadata)
    }

    /// Parse a Gmail API Message into our EmailMetadata struct
    fn parse_message_metadata(msg: &Message) -> Result<EmailMetadata> {
        let payload = msg.payload.as_ref();
        let headers = payload.and_then(|p| p.headers.as_ref());

        let get_header = |name: &str| -> String {
            headers
                .and_then(|hs| {
                    hs.iter()
                        .find(|h| h.name.as_deref() == Some(name))
                        .and_then(|h| h.value.clone())
                })
                .unwrap_or_default()
        };

        let labels = msg.label_ids.clone().unwrap_or_default();
        let is_read = !labels.contains(&"UNREAD".to_string());
        let is_replied = labels.contains(&"SENT".to_string());

        let date_str = get_header("Date");
        let date = chrono::DateTime::parse_from_rfc2822(&date_str)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| {
                // Fallback: use internal date
                chrono::Utc::now()
            });

        let to_str = get_header("To");
        let to: Vec<String> = to_str.split(',').map(|s| s.trim().to_string()).collect();

        Ok(EmailMetadata {
            message_id: msg.id.clone().unwrap_or_default(),
            thread_id: msg.thread_id.clone().unwrap_or_default(),
            subject: get_header("Subject"),
            from: get_header("From"),
            to,
            date,
            labels,
            size_bytes: msg.size_estimate.unwrap_or(0) as u64,
            has_attachments: payload
                .and_then(|p| p.parts.as_ref())
                .map(|parts| {
                    parts
                        .iter()
                        .any(|p| p.filename.as_ref().is_some_and(|f| !f.is_empty()))
                })
                .unwrap_or(false),
            snippet: msg.snippet.clone().unwrap_or_default(),
            is_read,
            is_replied,
        })
    }
}
