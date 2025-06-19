use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::{DateTime, Local};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct JsonFormatter {
    pub compact: bool,
}

impl JsonFormatter {
    pub fn new(compact: bool) -> Self {
        Self { compact }
    }

    fn build_json_tree(&self, nodes: &[FileNode], root_path: &Path) -> Value {
        // Build parent-child relationships
        let mut children_map: HashMap<PathBuf, Vec<&FileNode>> = HashMap::new();
        let mut root_node = None;
        
        for node in nodes {
            if node.path == root_path {
                root_node = Some(node);
            } else if let Some(parent) = node.path.parent() {
                children_map.entry(parent.to_path_buf()).or_default().push(node);
            }
        }
        
        fn node_to_json(
            node: &FileNode,
            children_map: &HashMap<PathBuf, Vec<&FileNode>>,
            root_path: &Path,
        ) -> Value {
            let mut obj = json!({
                "name": node.path.file_name().unwrap_or(node.path.as_os_str()).to_string_lossy(),
                "path": node.path.strip_prefix(root_path).unwrap_or(&node.path).to_string_lossy(),
                "type": match node.file_type {
                    crate::scanner::FileType::Directory => "directory",
                    crate::scanner::FileType::RegularFile => "file",
                    crate::scanner::FileType::Symlink => "symlink",
                    crate::scanner::FileType::Executable => "executable",
                    crate::scanner::FileType::Socket => "socket",
                    crate::scanner::FileType::Pipe => "pipe",
                    crate::scanner::FileType::BlockDevice => "block_device",
                    crate::scanner::FileType::CharDevice => "char_device",
                },
                "size": node.size,
                "permissions": format!("{:o}", node.permissions),
                "uid": node.uid,
                "gid": node.gid,
                "modified": DateTime::<Local>::from(node.modified).to_rfc3339(),
            });
            
            if node.permission_denied {
                obj["permission_denied"] = json!(true);
            }
            
            if let Some(children) = children_map.get(&node.path) {
                let mut sorted_children = children.to_vec();
                sorted_children.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.path.file_name().cmp(&b.path.file_name()),
                    }
                });
                
                obj["children"] = json!(
                    sorted_children
                        .iter()
                        .map(|child| node_to_json(child, children_map, root_path))
                        .collect::<Vec<_>>()
                );
            }
            
            obj
        }
        
        if let Some(root) = root_node {
            node_to_json(root, &children_map, root_path)
        } else {
            json!({})
        }
    }
}

impl Formatter for JsonFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let json_tree = self.build_json_tree(nodes, root_path);
        
        if self.compact {
            writeln!(writer, "{}", serde_json::to_string(&json_tree)?)?;
        } else {
            writeln!(writer, "{}", serde_json::to_string_pretty(&json_tree)?)?;
        }
        
        Ok(())
    }
}