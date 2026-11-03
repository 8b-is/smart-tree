use super::Formatter;
use crate::mem8_binary::LoudsSmartTree;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

pub struct DigestFormatter;
pub struct SmtrCapsuleFormatter;

impl Default for DigestFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Calculate a SHA256 hash of the tree structure for consistency verification
    fn calculate_tree_hash(&self, nodes: &[FileNode]) -> String {
        let mut hasher = Sha256::new();

        // Hash each node's key properties in a deterministic way
        for node in nodes {
            // Hash: depth, name, type (dir/file), size, permissions
            hasher.update(node.depth.to_le_bytes());
            hasher.update(
                node.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_bytes(),
            );
            hasher.update([if node.is_dir { 1 } else { 0 }]);
            hasher.update(node.size.to_le_bytes());
            hasher.update(node.permissions.to_le_bytes());
        }

        // Return first 16 chars of hex for brevity
        let result = hasher.finalize();
        hex::encode(&result[..8])
    }
}

impl Formatter for DigestFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // Calculate SHA256 hash of the tree structure
        let tree_hash = self.calculate_tree_hash(nodes);

        // Compute 8B-META-WP-v2.1 LoudsSmartTree sufficient projection & 64B SMTR capsule
        let key = *b"8b-is::smart-tree::smtr-key::v21";
        let raw_tree = LoudsSmartTree::from_file_nodes(nodes, &key);
        let proj_tree = raw_tree.project_sufficient(false);
        let capsule = proj_tree.pack_m8_capsule(&key);

        // First line: Hash, basic stats, and 64B SMTR capsule
        write!(
            writer,
            "HASH: {} F:{} D:{} S:{:x} LOUDS:{}/{} SMTR_V2:{}",
            tree_hash,
            stats.total_files,
            stats.total_dirs,
            stats.total_size,
            proj_tree.node_count,
            raw_tree.node_count,
            hex::encode(capsule),
        )?;

        // File type summary (top 5) - super compact
        if !stats.file_types.is_empty() {
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));

            let types_str: Vec<String> = types
                .iter()
                .take(5)
                .map(|(ext, count)| format!("{}:{}", ext, count))
                .collect();

            write!(writer, " TYPES: {}", types_str.join(" "))?;
        }

        // Add newline at the end
        writeln!(writer)?;

        Ok(())
    }
}

impl Default for SmtrCapsuleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtrCapsuleFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for SmtrCapsuleFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let key = *b"8b-is::smart-tree::smtr-key::v21";
        let raw_tree = LoudsSmartTree::from_file_nodes(nodes, &key);
        let proj_tree = raw_tree.project_sufficient(false);
        let capsule = proj_tree.pack_m8_capsule(&key);
        let indra_root = proj_tree.subtree_digest(0, &key);

        let payload = serde_json::json!({
            "spec": "8B-META-WP-v2.1",
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "louds_raw_nodes": raw_tree.node_count,
            "louds_sufficient_nodes": proj_tree.node_count,
            "louds_popcnt_edges": proj_tree.rank1(255),
            "louds_bits_w0_hex": format!("{:016x}", proj_tree.louds_bits[0]),
            "indra_net_root_hex": hex::encode(indra_root),
            "smtr_m8_capsule_64b_hex": hex::encode(capsule),
            "dharma_mudra_verified": LoudsSmartTree::verify_m8_capsule(&capsule, &key),
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileNode, FileType, FilesystemType, TreeStats};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn sample_node(name: &str, is_dir: bool, is_ignored: bool) -> FileNode {
        FileNode {
            path: PathBuf::from(name),
            is_dir,
            size: 128,
            permissions: 0o644,
            uid: 501,
            gid: 20,
            modified: SystemTime::UNIX_EPOCH,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored,
            depth: if is_dir { 0 } else { 1 },
            file_type: if is_dir {
                FileType::Directory
            } else {
                FileType::RegularFile
            },
            category: FileCategory::Rust,
            search_matches: None,
            filesystem_type: FilesystemType::Apfs,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        }
    }

    #[test]
    fn test_digest_and_smtr_capsule_formatter_emit_verified_capsule() {
        let nodes = vec![
            sample_node("smart-tree", true, false),
            sample_node("lib.rs", false, false),
            sample_node("target", true, true),
        ];
        let stats = TreeStats::default();

        let mut digest_out = Vec::new();
        DigestFormatter::new()
            .format(&mut digest_out, &nodes, &stats, Path::new("."))
            .unwrap();
        let digest_str = String::from_utf8(digest_out).unwrap();
        assert!(digest_str.contains("LOUDS:2/3"));
        assert!(digest_str.contains("SMTR_V2:534d545202"));

        let mut smtr_out = Vec::new();
        SmtrCapsuleFormatter::new()
            .format(&mut smtr_out, &nodes, &stats, Path::new("."))
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&smtr_out).unwrap();
        assert_eq!(val["spec"], "8B-META-WP-v2.1");
        assert_eq!(val["louds_raw_nodes"], 3);
        assert_eq!(val["louds_sufficient_nodes"], 2);
        assert_eq!(val["dharma_mudra_verified"], true);
    }
}
