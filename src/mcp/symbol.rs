//! Smart Tree's in-process CrabCC adapter. The standalone CrabCC CLI and its
//! databases are not required or modified by this tool.

use super::McpConfig;
use anyhow::{bail, Context, Result};
use crabcc_core::{graph::CallGraph, index, outline, query, store::Store};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn canonical_policy_path(path: &Path) -> PathBuf {
    // A blocked directory need not exist yet. Resolve its existing ancestor
    // as well, so aliases such as macOS /var -> /private/var still match.
    let mut ancestor = path.to_path_buf();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        let Some(name) = ancestor.file_name() else {
            break;
        };
        suffix.push(name.to_os_string());
        if !ancestor.pop() {
            break;
        }
    }
    let Ok(mut resolved) = ancestor.canonicalize() else {
        return path.to_path_buf();
    };
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    resolved
}

fn permitted(path: &Path, config: &McpConfig) -> bool {
    !config
        .blocked_paths
        .iter()
        .any(|p| path.starts_with(canonical_policy_path(p)))
        && (config.allowed_paths.is_empty()
            || config
                .allowed_paths
                .iter()
                .any(|p| path.starts_with(canonical_policy_path(p))))
}

fn resolve_root(cwd: &Path, requested: &str, config: &McpConfig) -> Result<PathBuf> {
    let root = cwd
        .join(requested)
        .canonicalize()
        .context("Resolve symbol repository")?;
    if !root.is_dir() || !permitted(&root, config) {
        bail!("Symbol repository is not an allowed directory");
    }
    // The embedded indexer walks the whole root. Do not let a broad root
    // silently include a nested directory that MCP policy explicitly blocks.
    if config
        .blocked_paths
        .iter()
        .any(|p| canonical_policy_path(p).starts_with(&root))
    {
        bail!("Symbol repository contains a blocked directory; choose a narrower root");
    }
    Ok(root)
}

fn source_path(root: &Path, file: &str, config: &McpConfig) -> Result<PathBuf> {
    let path = root
        .join(file)
        .canonicalize()
        .context("Resolve symbol source file")?;
    if !path.starts_with(root) || !permitted(&path, config) || !path.is_file() {
        bail!("Symbol source file is outside the allowed repository");
    }
    Ok(path)
}

fn open_store(root: &Path, config: &McpConfig) -> Result<Store> {
    let mut dir = root.to_path_buf();
    for component in [".st", "symbols"] {
        dir.push(component);
        if dir
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            bail!("Symbol cache directories must not be symlinks");
        }
        if !permitted(&dir, config) {
            bail!("Symbol cache directory is blocked by MCP policy");
        }
        std::fs::create_dir_all(&dir).context("Create symbol cache")?;
    }
    for name in ["index.db", "index.db-wal", "index.db-shm", "fsst.symbols"] {
        if dir
            .join(name)
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            bail!("Symbol cache files must not be symlinks");
        }
    }
    Store::open(&dir.join("index.db"))
}

/// Run a native symbol operation using an explicit project context and MCP policy.
pub fn execute(params: Value, cwd: &Path, config: &McpConfig) -> Result<Value> {
    let operation = params["operation"].as_str().unwrap_or("sym");
    if !matches!(
        operation,
        "sym" | "definition" | "refs" | "references" | "outline" | "index" | "graph"
    ) {
        bail!("Unknown symbol operation '{operation}'");
    }
    let name = params
        .get("name")
        .or_else(|| params.get("symbol"))
        .and_then(Value::as_str);
    if matches!(operation, "sym" | "definition" | "refs" | "references")
        && name.is_none_or(|n| n.trim().is_empty())
    {
        bail!("'name' is required for symbol operation '{operation}'");
    }
    let root = resolve_root(
        cwd,
        params["path"]
            .as_str()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or("."),
        config,
    )?;
    let file = if operation == "outline" {
        let file = params["file"]
            .as_str()
            .context("'file' is required for outline")?;
        Some(
            source_path(&root, file, config)?
                .strip_prefix(&root)?
                .to_string_lossy()
                .replace('\\', "/"),
        )
    } else {
        None
    };
    let include_source = params["include_source"].as_bool().unwrap_or(true);
    // Bound CPU parallelism independently of the caller's global Rayon pool.
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build()?;
    pool.install(|| {
        run(
            &root,
            operation,
            name,
            file.as_deref(),
            include_source,
            config,
        )
    })
}

fn run(
    root: &Path,
    operation: &str,
    name: Option<&str>,
    file: Option<&str>,
    include_source: bool,
    config: &McpConfig,
) -> Result<Value> {
    let store = open_store(root, config)?;
    if operation == "index" || store.needs_reindex || store.meta_get("edges_populated")?.is_none() {
        store.clear_all()?;
        let stats = index::build_index(root, &store)?;
        store.meta_set("edges_populated", "1")?;
        if operation == "index" {
            return Ok(json!({"engine": "crabcc-core", "built_in": true, "stats": stats}));
        }
    } else {
        // CrabCC's mtime shortcut uses whole seconds. Editors can save twice
        // within that interval (or preserve timestamps), so invalidate changed
        // content before its incremental parser refreshes. Unchanged files
        // keep their parsed symbols and edges.
        for (file, (hash, _)) in store.list_files_with_meta()? {
            let path = match source_path(root, &file, config) {
                Ok(path) => path,
                Err(_) => {
                    store.delete_file(&file)?;
                    continue;
                }
            };
            // Match the core indexer's per-file memory limit.
            if std::fs::metadata(&path)?.len() > 2 * 1024 * 1024 {
                store.delete_file(&file)?;
                continue;
            }
            if crabcc_core::hash::sha256_hex(&std::fs::read(path)?) != hash {
                store.touch_mtime(&file, -1)?;
            }
        }
        index::refresh(root, &store)?;
    }
    match operation {
        "sym" | "definition" => {
            let symbols = query::find_symbol(&store, name.context("Missing symbol name")?)?;
            let mut results = Vec::new();
            for symbol in symbols.into_iter().take(100) {
                let mut value = serde_json::to_value(&symbol)?;
                if include_source {
                    let path = source_path(root, &symbol.file, config)?;
                    let text = std::fs::read_to_string(path).context("Read symbol source")?;
                    let start = symbol.line_start.saturating_sub(1) as usize;
                    let count = symbol
                        .line_end
                        .saturating_sub(symbol.line_start)
                        .saturating_add(1)
                        .min(65) as usize;
                    value["source"] = text
                        .lines()
                        .skip(start)
                        .take(count)
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into();
                }
                results.push(value);
            }
            Ok(json!(results))
        }
        "refs" | "references" => {
            let mut value = serde_json::to_value(query::query_refs(
                &store,
                root,
                name.context("Missing symbol name")?,
                query::Mode::Hits { limit: Some(100) },
                None,
            )?)?;
            if !include_source {
                if let Some(hits) = value.as_array_mut() {
                    for hit in hits {
                        if let Some(fields) = hit.as_object_mut() {
                            fields.remove("snippet");
                        }
                    }
                }
            }
            Ok(value)
        }
        "outline" => Ok(serde_json::to_value(outline::outline(
            &store,
            file.context("Missing outline file")?,
        )?)?),
        "graph" => {
            let graph = CallGraph::build(&store, root)?;
            Ok(
                json!({"engine": "crabcc-core", "built_in": true, "files": store.list_files()?.len(), "symbols": store.iter_all_symbols()?.len(), "edges": graph.edge_count}),
            )
        }
        _ => unreachable!("operation validated before opening the store"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mcp_remembers_the_project_between_symbol_calls() -> Result<()> {
        let workspace = tempfile::tempdir()?;
        let project = workspace.path().join("project");
        std::fs::create_dir(&project)?;
        std::fs::write(project.join("hello.rs"), "pub fn greet() -> u8 { 7 }\n")?;
        let server = super::super::McpServer::new(McpConfig {
            allowed_paths: vec![workspace.path().to_path_buf()],
            ..McpConfig::default()
        });
        *server.context.cwd.lock().unwrap() = workspace.path().to_path_buf();
        let request = |id, args| {
            json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
                "params":{"name":"symbol","arguments":args}})
            .to_string()
        };
        let indexed: Value = serde_json::from_str(
            &server
                .handle_request(&request(1, json!({"operation":"index","path":"project"})))
                .await?,
        )?;
        assert!(indexed.get("error").is_none(), "{indexed}");
        let response: Value = serde_json::from_str(
            &server
                .handle_request(&request(2, json!({"operation":"sym","name":"greet"})))
                .await?,
        )?;
        let symbols: Value = serde_json::from_str(
            response["result"]["content"][0]["text"]
                .as_str()
                .context("MCP symbol response")?,
        )?;
        assert_eq!(symbols[0]["name"], "greet");
        assert_eq!(symbols[0]["file"], "hello.rs");
        assert!(!workspace.path().join(".st").exists());
        assert!(project.join(".st/symbols/index.db").exists());
        Ok(())
    }

    #[test]
    fn native_index_lookup_references_outline_and_graph() -> Result<()> {
        let repo = tempfile::tempdir()?;
        std::fs::write(
            repo.path().join("hello.rs"),
            "pub fn greet() -> u8 { 7 }\npub fn caller() -> u8 { greet() }\n",
        )?;
        let config = McpConfig::default();
        let call = |params| execute(params, repo.path(), &config);
        assert_eq!(call(json!({"operation":"index"}))?["built_in"], true);
        let symbols = call(json!({"name":"greet"}))?;
        assert_eq!(symbols[0]["file"], "hello.rs");
        assert!(symbols[0]["source"].as_str().unwrap().contains("7"));
        assert!(call(json!({"name":"greet","include_source":false}))?[0]
            .get("source")
            .is_none());
        let refs = call(json!({"operation":"refs","name":"greet"}))?;
        assert!(!refs.as_array().unwrap().is_empty());
        assert_eq!(
            call(json!({"operation":"outline","file":"hello.rs"}))?
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(
            call(json!({"operation":"graph"}))?["edges"]
                .as_u64()
                .unwrap()
                > 0
        );
        let source = repo.path().join("hello.rs");
        let modified = std::fs::metadata(&source)?.modified()?;
        std::fs::write(&source, "pub fn newer() -> u8 { 9 }\n")?;
        std::fs::File::options()
            .write(true)
            .open(&source)?
            .set_modified(modified)?;
        assert_eq!(call(json!({"name":"greet"}))?, json!([]));
        assert_eq!(call(json!({"name":"newer"}))?[0]["name"], "newer");
        std::fs::remove_file(&source)?;
        assert_eq!(call(json!({"name":"newer"}))?, json!([]));
        assert!(!repo.path().join(".crabcc").exists());
        Ok(())
    }

    #[test]
    fn validates_roots_and_outline_before_creating_an_index() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let other = tempfile::tempdir()?;
        std::fs::write(other.path().join("outside.rs"), "fn outside() {}")?;
        let config = McpConfig {
            allowed_paths: vec![repo.path().to_path_buf()],
            ..McpConfig::default()
        };
        assert!(execute(
            json!({"operation":"index", "path":other.path()}),
            repo.path(),
            &config
        )
        .is_err());
        assert!(execute(
            json!({"operation":"outline", "file":other.path().join("outside.rs")}),
            repo.path(),
            &config
        )
        .is_err());
        assert!(execute(json!({"name":""}), repo.path(), &config).is_err());
        assert!(!repo.path().join(".st").exists());
        let blocked = McpConfig {
            blocked_paths: vec![repo.path().join("private")],
            ..config
        };
        assert!(execute(json!({"operation":"index"}), repo.path(), &blocked).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn refuses_external_symlink_sources_and_cache() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let other = tempfile::tempdir()?;
        std::fs::write(other.path().join("secret.rs"), "fn secret() {}")?;
        std::os::unix::fs::symlink(other.path().join("secret.rs"), repo.path().join("link.rs"))?;
        let config = McpConfig::default();
        assert!(execute(
            json!({"operation":"outline", "file":"link.rs"}),
            repo.path(),
            &config
        )
        .is_err());
        std::os::unix::fs::symlink(other.path(), repo.path().join(".st"))?;
        assert!(execute(json!({"operation":"index"}), repo.path(), &config).is_err());
        assert!(!other.path().join("symbols").exists());
        Ok(())
    }
}
