//! Smart Tree's in-process CrabCC adapter. The standalone CrabCC CLI and its
//! databases are not required or modified by this tool.

use super::McpConfig;
use anyhow::{bail, Context, Result};
use crabcc_core::{graph::CallGraph, index, outline, query, store::Store};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// WUPZ ABI v1 magic bytes (`"WUPZ"` = `0x57, 0x55, 0x50, 0x5A`).
pub const WUPZ_MAGIC_V1: [u8; 4] = *b"WUPZ";
/// WUPZ ABI version 1 for 32-byte metadata witness sealing.
pub const WUPZ_ABI_V1: u8 = 1;
/// Default integer denominator `q` for `p/q` ("Counting Divided") graph kernels.
pub const GRAPH_PQ_Q: u64 = 1000;

/// Exact deterministic integer square root `floor(sqrt(n))` with zero IEEE-754 floats.
#[inline]
pub const fn isqrt_pq(n: u128) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x as u64;
        }
        x = y;
    }
}

/// Fold a byte slice with a 64-bit golden-ratio XOR rotation (`wupz_xor_fold`).
#[inline]
pub fn wupz_xor_fold_bytes(bytes: &[u8]) -> u64 {
    let mut acc: u64 = 0x9E37_79B9_7F4A_7C15;
    for (idx, &b) in bytes.iter().enumerate() {
        let lane = (b as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        acc ^= lane.rotate_left(((idx & 7) * 8) as u32);
        acc = acc.rotate_left(11).wrapping_add(0x94D0_49BB_1331_11EB);
    }
    acc
}

/// Seal a Smart Tree / CrabCC metadata frame with a 32-byte `WUPZ` ABI v1 witness (`[u8; 32]`).
///
/// Layout (`32` bytes total):
/// - `0..4`:   `"WUPZ"` magic (`0x57, 0x55, 0x50, 0x5A`)
/// - `4`:      ABI version (`1`)
/// - `5`:      balanced-ternary `AND` gate (`{-1, 0, +1}` encoded as `i8 as u8`)
/// - `6`:      balanced-ternary `NAND` gate (`{-1, 0, +1}` encoded as `i8 as u8`)
/// - `7`:      `QuantTern` gate byte (`0x55` + trit offset)
/// - `8..16`:  64-bit `xor_fold` (little-endian)
/// - `16..20`: bitplane `popcount` (`u32`, little-endian)
/// - `20..24`: payload byte length (`u32`, little-endian)
/// - `24..32`: 64-bit golden witness tag (`u64`, little-endian)
pub fn seal_wupz_metadata_frame_v1(payload: &[u8]) -> ([u8; 32], String) {
    let xor_fold = wupz_xor_fold_bytes(payload);
    let popcount: u32 = payload.iter().map(|b| b.count_ones()).sum();
    let len_u32 = payload.len() as u32;
    let t0: i8 = match xor_fold % 3 {
        0 => -1,
        1 => 0,
        _ => 1,
    };
    let t1: i8 = match (xor_fold >> 8) % 3 {
        0 => -1,
        1 => 0,
        _ => 1,
    };
    let and_gate: i8 = t0.min(t1);
    let nand_gate: i8 = -and_gate;
    let qt_byte: u8 = (85i16 + (t0 as i16) * 9 + (t1 as i16) * 3 + (and_gate as i16)) as u8;
    let witness_tag =
        xor_fold.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((popcount as u64) << 32) ^ (len_u32 as u64);

    let mut seal = [0u8; 32];
    seal[0..4].copy_from_slice(&WUPZ_MAGIC_V1);
    seal[4] = WUPZ_ABI_V1;
    seal[5] = and_gate as u8;
    seal[6] = nand_gate as u8;
    seal[7] = qt_byte;
    seal[8..16].copy_from_slice(&xor_fold.to_le_bytes());
    seal[16..20].copy_from_slice(&popcount.to_le_bytes());
    seal[20..24].copy_from_slice(&len_u32.to_le_bytes());
    seal[24..32].copy_from_slice(&witness_tag.to_le_bytes());

    let mut hex_out = String::with_capacity(66);
    hex_out.push_str("0x");
    for b in &seal {
        use std::fmt::Write as _;
        let _ = write!(&mut hex_out, "{b:02x}");
    }
    (seal, hex_out)
}

/// Verify a 32-byte `WUPZ` ABI v1 witness seal against `payload`.
#[inline]
pub fn verify_wupz_metadata_frame_v1(payload: &[u8], seal: &[u8; 32]) -> bool {
    let (expected, _) = seal_wupz_metadata_frame_v1(payload);
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= expected[i] ^ seal[i];
    }
    diff == 0
}

/// Compute deterministic zero-float `p/q` graph & index metrics + 32-byte `WUPZ` ABI v1 witness seal.
pub fn compute_graph_stats_pq(files: usize, symbols: usize, edges: usize, root_tag: &str) -> Value {
    let files_q = (files as u64).max(1);
    let symbols_q = (symbols as u64).max(1);
    let sym_per_file_p = (symbols as u64).saturating_mul(GRAPH_PQ_Q) / files_q;
    let edges_per_sym_p = (edges as u64).saturating_mul(GRAPH_PQ_Q) / symbols_q;
    // Spectral complexity = isqrt(symbols * edges * Q^2) / Q
    let spectral_p = isqrt_pq(
        (symbols as u128)
            .saturating_mul(edges as u128)
            .saturating_mul((GRAPH_PQ_Q as u128) * (GRAPH_PQ_Q as u128)),
    );
    let frame_descriptor = format!(
        "crabcc:v1:root={root_tag}:files={files}:symbols={symbols}:edges={edges}:spf={sym_per_file_p}/{GRAPH_PQ_Q}:eps={edges_per_sym_p}/{GRAPH_PQ_Q}:spec={spectral_p}/{GRAPH_PQ_Q}"
    );
    let (seal_bytes, seal_hex) = seal_wupz_metadata_frame_v1(frame_descriptor.as_bytes());
    let verified = verify_wupz_metadata_frame_v1(frame_descriptor.as_bytes(), &seal_bytes);
    let xor_fold = u64::from_le_bytes(seal_bytes[8..16].try_into().unwrap_or([0u8; 8]));
    let popcount = u32::from_le_bytes(seal_bytes[16..20].try_into().unwrap_or([0u8; 4]));

    json!({
        "symbols_per_file_pq": format!("{sym_per_file_p}/{GRAPH_PQ_Q}"),
        "edges_per_symbol_pq": format!("{edges_per_sym_p}/{GRAPH_PQ_Q}"),
        "spectral_complexity_pq": format!("{spectral_p}/{GRAPH_PQ_Q}"),
        "no_float_law": true,
        "wupz_abi_version": WUPZ_ABI_V1,
        "wupz_seal_32b": seal_hex,
        "wupz_xor_fold": format!("0x{xor_fold:016x}"),
        "wupz_and_gate": seal_bytes[5] as i8,
        "wupz_nand_gate": seal_bytes[6] as i8,
        "wupz_quant_tern": format!("qt:{:02x}", seal_bytes[7]),
        "wupz_popcount": popcount,
        "wupz_verified": verified,
    })
}

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
    if matches!(operation, "suggest" | "best" | "g8t") {
        let query = params
            .get("name")
            .or_else(|| params.get("query"))
            .or_else(|| params.get("symbol"))
            .and_then(Value::as_str)
            .unwrap_or("*");
        let yolo = params.get("yolo").and_then(Value::as_bool).unwrap_or(false);
        let top_n = params
            .get("top_n")
            .and_then(Value::as_u64)
            .map(|n| n as usize)
            .unwrap_or(3);
        let arch = params.get("arch").and_then(Value::as_str);
        let report = crate::g8t_integration::query_best_functions(query, top_n, yolo, arch)?;
        return Ok(serde_json::to_value(report)?);
    }
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
    let root_tag = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string());
    if operation == "index" || store.needs_reindex || store.meta_get("edges_populated")?.is_none() {
        store.clear_all()?;
        let stats = index::build_index(root, &store)?;
        store.meta_set("edges_populated", "1")?;
        let files_cnt = store.list_files()?.len();
        let syms_cnt = store.iter_all_symbols()?.len();
        let pq_meta = compute_graph_stats_pq(files_cnt, syms_cnt, 0, &root_tag);
        if let Some(seal_hex) = pq_meta["wupz_seal_32b"].as_str() {
            let _ = store.meta_set("wupz_seal_v1", seal_hex);
        }
        if operation == "index" {
            return Ok(json!({
                "engine": "crabcc-core",
                "built_in": true,
                "stats": stats,
                "pq_kernel": pq_meta,
            }));
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
            let files_cnt = store.list_files()?.len();
            let syms_cnt = store.iter_all_symbols()?.len();
            let pq_meta = compute_graph_stats_pq(files_cnt, syms_cnt, graph.edge_count, &root_tag);
            if let Some(seal_hex) = pq_meta["wupz_seal_32b"].as_str() {
                let _ = store.meta_set("wupz_seal_v1", seal_hex);
            }
            Ok(json!({
                "engine": "crabcc-core",
                "built_in": true,
                "files": files_cnt,
                "symbols": syms_cnt,
                "edges": graph.edge_count,
                "pq_kernel": pq_meta,
            }))
        }
        _ => unreachable!("operation validated before opening the store"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pq_graph_kernel_and_wupz_32b_seal_verify() {
        assert_eq!(isqrt_pq(0), 0);
        assert_eq!(isqrt_pq(10_000), 100);
        assert_eq!(isqrt_pq(2_000_000), 1414);

        let payload = b"smart-tree:crabcc:wupz-v1:test-frame";
        let (seal, seal_hex) = seal_wupz_metadata_frame_v1(payload);
        assert_eq!(&seal[0..4], b"WUPZ");
        assert_eq!(seal[4], 1);
        assert_eq!(seal_hex.len(), 66); // "0x" + 64 hex chars (32 bytes)
        assert!(verify_wupz_metadata_frame_v1(payload, &seal));
        let mut tampered = seal;
        tampered[31] ^= 0x01;
        assert!(!verify_wupz_metadata_frame_v1(payload, &tampered));

        let stats = compute_graph_stats_pq(10, 45, 90, "smart-tree");
        assert_eq!(stats["symbols_per_file_pq"], "4500/1000");
        assert_eq!(stats["edges_per_symbol_pq"], "2000/1000");
        assert_eq!(stats["wupz_verified"], true);
        assert_eq!(stats["wupz_abi_version"], 1);
    }

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
