//! Google Drive API client wrapper
//!
//! Provides high-level methods for uploading, downloading, listing,
//! and managing files and folders in Google Drive.

use anyhow::{Context, Result};
use google_drive3::api::File as DriveFile;
use google_drive3::DriveHub;
use std::io::Cursor;

use super::auth::GoogleAuthenticator;
use super::models::DriveFileMetadata;
use super::rate_limiter::RateLimiter;

/// Google Drive API client with rate limiting
pub struct DriveClient {
    hub: DriveHub<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    rate_limiter: RateLimiter,
}

impl DriveClient {
    /// Create a new Drive client with the given authenticator
    pub fn new(auth: GoogleAuthenticator) -> Self {
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .expect("native TLS roots")
                        .https_or_http()
                        .enable_http1()
                        .build(),
                );
        let hub = DriveHub::new(client, auth);
        Self {
            hub,
            rate_limiter: RateLimiter::drive(),
        }
    }

    /// List files in a folder
    pub async fn list_files(
        &self,
        folder_id: &str,
        page_token: Option<&str>,
    ) -> Result<(Vec<DriveFileMetadata>, Option<String>)> {
        self.rate_limiter.acquire().await;

        let query = format!("'{}' in parents and trashed = false", folder_id);
        let mut req = self
            .hub
            .files()
            .list()
            .q(&query)
            .param(
                "fields",
                "nextPageToken,files(id,name,mimeType,size,modifiedTime,md5Checksum,parents)",
            )
            .page_size(100);

        if let Some(token) = page_token {
            req = req.page_token(token);
        }

        let (_, response) = req.doit().await.context("Failed to list Drive files")?;

        let files = response
            .files
            .unwrap_or_default()
            .into_iter()
            .map(Self::parse_drive_file)
            .collect();

        Ok((files, response.next_page_token))
    }

    /// Upload a file to a specific folder
    pub async fn upload_file(
        &self,
        parent_folder_id: &str,
        name: &str,
        content: &[u8],
        mime_type: &str,
    ) -> Result<String> {
        self.rate_limiter.acquire().await;

        let file_meta = DriveFile {
            name: Some(name.to_string()),
            parents: Some(vec![parent_folder_id.to_string()]),
            mime_type: Some(mime_type.to_string()),
            ..Default::default()
        };

        let cursor = Cursor::new(content.to_vec());
        let mime: mime::Mime = mime_type.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);

        let (_, file) = self
            .hub
            .files()
            .create(file_meta)
            .upload(cursor, mime)
            .await
            .context("Failed to upload file to Drive")?;

        file.id.context("Upload succeeded but no file ID returned")
    }

    /// Download a file's content
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        self.rate_limiter.acquire().await;

        let (response, _) = self
            .hub
            .files()
            .get(file_id)
            .param("alt", "media")
            .doit()
            .await
            .context("Failed to download file from Drive")?;

        use http_body_util::BodyExt;
        let body = response.into_body();
        let bytes = body
            .collect()
            .await
            .context("Failed to read response body")?
            .to_bytes();

        Ok(bytes.to_vec())
    }

    /// Create a folder in Drive
    pub async fn create_folder(&self, parent_id: &str, name: &str) -> Result<String> {
        self.rate_limiter.acquire().await;

        let folder = DriveFile {
            name: Some(name.to_string()),
            parents: Some(vec![parent_id.to_string()]),
            mime_type: Some("application/vnd.google-apps.folder".to_string()),
            ..Default::default()
        };

        // For folder creation, use an empty upload with folder mime type
        let cursor = Cursor::new(Vec::<u8>::new());
        let (_, created) = self
            .hub
            .files()
            .create(folder)
            .upload(
                cursor,
                "application/vnd.google-apps.folder".parse().unwrap(),
            )
            .await
            .context("Failed to create Drive folder")?;

        created.id.context("Folder created but no ID returned")
    }

    /// Get file metadata
    pub async fn get_file_metadata(&self, file_id: &str) -> Result<DriveFileMetadata> {
        self.rate_limiter.acquire().await;

        let (_, file) = self
            .hub
            .files()
            .get(file_id)
            .param(
                "fields",
                "id,name,mimeType,size,modifiedTime,md5Checksum,parents",
            )
            .doit()
            .await
            .context("Failed to get file metadata")?;

        Ok(Self::parse_drive_file(file))
    }

    /// Update/overwrite an existing file's content
    pub async fn update_file(&self, file_id: &str, content: &[u8], mime_type: &str) -> Result<()> {
        self.rate_limiter.acquire().await;

        let file_meta = DriveFile::default();
        let cursor = Cursor::new(content.to_vec());
        let mime: mime::Mime = mime_type.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);

        self.hub
            .files()
            .update(file_meta, file_id)
            .upload(cursor, mime)
            .await
            .context("Failed to update file on Drive")?;

        Ok(())
    }

    /// Delete a file or folder
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        self.rate_limiter.acquire().await;

        self.hub
            .files()
            .delete(file_id)
            .doit()
            .await
            .context("Failed to delete Drive file")?;

        Ok(())
    }

    /// Search Drive files by query
    pub async fn search(&self, query: &str) -> Result<Vec<DriveFileMetadata>> {
        self.rate_limiter.acquire().await;

        let search_query = format!("fullText contains '{}' and trashed = false", query);

        let (_, response) = self
            .hub
            .files()
            .list()
            .q(&search_query)
            .param(
                "fields",
                "files(id,name,mimeType,size,modifiedTime,md5Checksum,parents)",
            )
            .page_size(50)
            .doit()
            .await
            .context("Failed to search Drive")?;

        Ok(response
            .files
            .unwrap_or_default()
            .into_iter()
            .map(Self::parse_drive_file)
            .collect())
    }

    /// List all files in a folder recursively
    #[allow(clippy::type_complexity)]
    pub fn list_files_recursive<'a>(
        &'a self,
        folder_id: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<(String, DriveFileMetadata)>>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut all_files = Vec::new();
            let mut page_token = None;

            loop {
                let (files, next) = self.list_files(folder_id, page_token.as_deref()).await?;

                for file in files {
                    let path = if prefix.is_empty() {
                        file.name.clone()
                    } else {
                        format!("{}/{}", prefix, file.name)
                    };

                    if file.is_folder {
                        let sub_files = self.list_files_recursive(&file.id, &path).await?;
                        all_files.extend(sub_files);
                    } else {
                        all_files.push((path, file));
                    }
                }

                match next {
                    Some(token) => page_token = Some(token),
                    None => break,
                }
            }

            Ok(all_files)
        })
    }

    /// Parse a Google Drive File into our metadata struct
    fn parse_drive_file(file: DriveFile) -> DriveFileMetadata {
        let is_folder = file
            .mime_type
            .as_deref()
            .map(|m| m == "application/vnd.google-apps.folder")
            .unwrap_or(false);

        DriveFileMetadata {
            id: file.id.unwrap_or_default(),
            name: file.name.unwrap_or_default(),
            mime_type: file.mime_type.unwrap_or_default(),
            size: file.size.map(|s| s as u64),
            modified_time: file.modified_time,
            md5_checksum: file.md5_checksum,
            parents: file.parents.unwrap_or_default(),
            is_folder,
        }
    }
}
