//! REST API endpoints for file browser and system state

use super::{FileTreeNode, SharedState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    path: Option<String>,
    depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    connections: usize,
}

#[derive(Debug, Serialize)]
pub struct FileContent {
    path: String,
    content: String,
    is_binary: bool,
    size: u64,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    path: String,
    content: String,
}

/// Health check endpoint
pub async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let connections = state.read().await.connections;
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        connections,
    })
}

/// List files in a directory
pub async fn list_files(
    State(state): State<SharedState>,
    Query(query): Query<PathQuery>,
) -> Result<Json<Vec<FileTreeNode>>, (StatusCode, String)> {
    let base_path = {
        let s = state.read().await;
        s.cwd.clone()
    };

    let path = match &query.path {
        Some(p) => {
            let requested = PathBuf::from(p);
            if requested.is_absolute() {
                requested
            } else {
                base_path.join(requested)
            }
        }
        None => base_path,
    };

    let path = path.canonicalize().map_err(|e| {
        (StatusCode::NOT_FOUND, format!("Path not found: {}", e))
    })?;

    let mut entries = Vec::new();

    let read_dir = fs::read_dir(&path).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read directory: {}", e))
    })?;

    for entry in read_dir.flatten() {
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let name = entry.file_name().to_string_lossy().to_string();
        let file_type = if is_dir {
            "directory".to_string()
        } else {
            get_file_type(&name)
        };

        entries.push(FileTreeNode {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
            file_type,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(Json(entries))
}

/// Read file content
pub async fn read_file(
    Query(query): Query<PathQuery>,
) -> Result<Json<FileContent>, (StatusCode, String)> {
    let path = query.path.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "Missing path parameter".to_string())
    })?;

    let path = PathBuf::from(&path);

    let metadata = fs::metadata(&path).map_err(|e| {
        (StatusCode::NOT_FOUND, format!("File not found: {}", e))
    })?;

    if metadata.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Path is a directory".to_string()));
    }

    let size = metadata.len();

    // Check if binary
    let is_binary = is_binary_file(&path);

    let content = if is_binary {
        "[Binary file]".to_string()
    } else if size > 1_000_000 {
        "[File too large to display]".to_string()
    } else {
        fs::read_to_string(&path).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e))
        })?
    };

    let mime_type = get_mime_type(&path);

    Ok(Json(FileContent {
        path: path.to_string_lossy().to_string(),
        content,
        is_binary,
        size,
        mime_type,
    }))
}

/// Write file content
pub async fn write_file(
    Json(request): Json<WriteFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let path = PathBuf::from(&request.path);

    fs::write(&path, &request.content).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write file: {}", e))
    })?;

    Ok((StatusCode::OK, "File saved"))
}

/// Get directory tree
pub async fn get_tree(
    State(state): State<SharedState>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<Vec<FileTreeNode>>, (StatusCode, String)> {
    let base_path = {
        let s = state.read().await;
        s.cwd.clone()
    };

    let path = match &query.path {
        Some(p) => PathBuf::from(p),
        None => base_path,
    };

    let depth = query.depth.unwrap_or(3);

    let nodes = collect_tree(&path, depth, 0)?;

    Ok(Json(nodes))
}

fn collect_tree(path: &PathBuf, max_depth: usize, current_depth: usize) -> Result<Vec<FileTreeNode>, (StatusCode, String)> {
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let read_dir = fs::read_dir(path).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read directory: {}", e))
    })?;

    for entry in read_dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and common ignored directories
        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "__pycache__" {
            continue;
        }

        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let file_type = if is_dir {
            "directory".to_string()
        } else {
            get_file_type(&name)
        };

        entries.push(FileTreeNode {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
            file_type,
        });
    }

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// Render markdown to HTML
pub async fn render_markdown(
    Query(query): Query<PathQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let path = query.path.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "Missing path parameter".to_string())
    })?;

    let content = fs::read_to_string(&path).map_err(|e| {
        (StatusCode::NOT_FOUND, format!("File not found: {}", e))
    })?;

    // Return raw markdown - client will render with marked.js
    Ok(content)
}

fn get_file_type(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" | "jsx" => "react",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" => "css",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "markdown" => "markdown",
        "sh" | "bash" | "zsh" => "shell",
        "go" => "go",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" => "cpp",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "sql" => "sql",
        "txt" => "text",
        "lock" => "lock",
        "gitignore" | "dockerignore" => "ignore",
        _ => "file",
    }
    .to_string()
}

fn get_mime_type(path: &std::path::Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" | "rb" | "php" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "json" => "application/json",
        "md" => "text/markdown",
        "txt" => "text/plain",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn is_binary_file(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "ico" | "webp" |
        "mp3" | "mp4" | "wav" | "avi" | "mkv" |
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" |
        "exe" | "dll" | "so" | "dylib" |
        "pdf" | "doc" | "docx" | "xls" | "xlsx" |
        "ttf" | "woff" | "woff2" | "eot" |
        "sqlite" | "db"
    )
}
