//! Traditional file system input adapter

use super::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;

pub struct FileSystemAdapter;

#[async_trait]
impl InputAdapter for FileSystemAdapter {
    fn name(&self) -> &'static str {
        "FileSystem"
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["dir", "directory", "folder", "path"]
    }

    async fn can_handle(&self, input: &InputSource) -> bool {
        match input {
            InputSource::Path(path) => path.exists(),
            _ => false,
        }
    }

    async fn parse(&self, input: InputSource) -> Result<ContextNode> {
        match input {
            InputSource::Path(path) => {
                let metadata = fs::metadata(&path)?;
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("root")
                    .to_string();

                let mut root = ContextNode {
                    id: path.to_string_lossy().to_string(),
                    name,
                    node_type: if metadata.is_dir() {
                        NodeType::Directory
                    } else {
                        NodeType::File
                    },
                    quantum_state: None,
                    children: vec![],
                    metadata: serde_json::json!({
                        "size": metadata.len(),
                        "modified": metadata.modified().ok(),
                        "readonly": metadata.permissions().readonly(),
                    }),
                    entanglements: vec![],
                };

                if metadata.is_dir() {
                    root.children = self.scan_directory(&path)?;
                }

                Ok(root)
            }
            _ => anyhow::bail!("FileSystem adapter only handles Path inputs"),
        }
    }
}

impl FileSystemAdapter {
    fn scan_directory(&self, path: &std::path::Path) -> Result<Vec<ContextNode>> {
        let mut nodes = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            let node = ContextNode {
                id: path.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                node_type: if metadata.is_dir() {
                    NodeType::Directory
                } else {
                    NodeType::File
                },
                quantum_state: None,
                children: if metadata.is_dir() {
                    self.scan_directory(&path).unwrap_or_default()
                } else {
                    vec![]
                },
                metadata: serde_json::json!({
                    "size": metadata.len(),
                    "modified": metadata.modified().ok(),
                }),
                entanglements: vec![],
            };

            nodes.push(node);
        }

        Ok(nodes)
    }
}
