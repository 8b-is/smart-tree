use super::storage::{now, Passage, Repository};
use super::Hub;
use anyhow::{bail, ensure, Context, Result};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

pub(super) fn source_url(value: &str) -> Result<(String, String, String)> {
    let url = reqwest::Url::parse(value).context("Use an HTTPS GitHub repository URL")?;
    ensure!(
        url.scheme() == "https"
            && url.host_str() == Some("github.com")
            && url.port().is_none()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "Only public HTTPS GitHub URLs without credentials are supported"
    );
    let parts: Vec<_> = url.path().trim_matches('/').split('/').collect();
    ensure!(
        parts.len() == 2,
        "Use a repository URL, not a file or branch URL"
    );
    let owner = parts[0];
    let name = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    for part in [owner, name] {
        ensure!(
            !part.is_empty()
                && part.len() <= 100
                && !part.starts_with('.')
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c)),
            "Invalid repository name"
        );
    }
    Ok((
        format!("https://github.com/{owner}/{name}.git"),
        owner.to_owned(),
        name.to_owned(),
    ))
}

pub(super) fn git(directory: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .args([
            "--no-optional-locks",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "-c",
            "http.followRedirects=false",
        ])
        .arg(format!("--git-dir={}", directory.display()))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn git_text(directory: &Path, args: &[&str]) -> Result<String> {
    let output = git(directory).args(args).output()?;
    ensure!(output.status.success(), "Git object lookup failed");
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

pub(super) fn import(hub: &Hub, root: &Path, public: bool, recall: bool) -> Result<usize> {
    let root = root.canonicalize().context("Import root does not exist")?;
    ensure!(
        hub.config
            .import_roots
            .iter()
            .any(|allowed| allowed.canonicalize().ok().as_ref() == Some(&root)),
        "Import root is not configured for this hub"
    );
    let mut imported = 0;
    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let candidate = entry.path();
        let directory = if candidate.join(".git").is_dir() {
            candidate.join(".git")
        } else if candidate.join("HEAD").is_file() {
            candidate
        } else {
            continue;
        };
        let directory = directory.canonicalize()?;
        ensure!(
            directory.starts_with(&root),
            "Repository escaped import root"
        );
        let origin = match git_text(&directory, &["config", "--get", "remote.origin.url"]) {
            Ok(origin) => origin,
            Err(_) => continue,
        };
        let Ok((url, collection, name)) = source_url(&origin) else {
            continue;
        };
        let store = hub.lock()?;
        let exists: bool = store.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM repositories WHERE git_dir=?)",
            [directory.to_string_lossy()],
            |row| row.get(0),
        )?;
        if exists {
            continue;
        }
        store.add_repository(&Repository {
            id: uuid::Uuid::new_v4().to_string(),
            collection,
            name,
            source_url: url,
            git_dir: directory.to_string_lossy().into_owned(),
            owner_hash: hub.admin_hash.clone(),
            public,
            recall,
            approved: true,
            status: "queued".into(),
            generation: String::new(),
            commit: String::new(),
            files: 0,
            chunks: 0,
            skipped: 0,
            updated_at: now(),
            message: String::new(),
        })?;
        imported += 1;
    }
    Ok(imported)
}

pub(super) async fn worker(hub: Arc<Hub>) {
    loop {
        if hub.stopping.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        let result = hub.lock().and_then(|store| store.next_job());
        match result {
            Ok(Some(repo)) => {
                let id = repo.id.clone();
                let worker_hub = hub.clone();
                let result =
                    tokio::task::spawn_blocking(move || index_repository(&worker_hub, repo)).await;
                match result {
                    Ok(Ok(())) => tracing::info!(repository=%id,"Repository job completed"),
                    error => {
                        tracing::error!(repository=%id,?error,"Repository job failed");
                        if let Ok(store) = hub.lock() {
                            let _ = store.db.execute("UPDATE repositories SET status='failed',message='Indexing failed; an operator can retry',updated_at=?2 WHERE id=?1 AND status='indexing'", params![id,now()]);
                        }
                    }
                }
            }
            Ok(None) => tokio::time::sleep(Duration::from_secs(2)).await,
            Err(error) => {
                tracing::error!(%error,"Read archive queue");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

fn clone_archive(hub: &Hub, repo: &Repository) -> Result<PathBuf> {
    let (url, _, _) = source_url(&repo.source_url)?;
    let destination = hub.config.archive_dir.join(format!("{}.git", repo.id));
    let partial = hub.config.archive_dir.join(format!("{}.partial", repo.id));
    if destination.join("HEAD").is_file() {
        return Ok(destination);
    }
    // Only this job's generated directory can be removed on retry.
    if partial.exists() {
        std::fs::remove_dir_all(&partial)?;
    }
    let mut child = Command::new("git")
        .args([
            "-c",
            "core.fsync=all",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "-c",
            "http.followRedirects=false",
            "clone",
            "--mirror",
            "--",
            &url,
        ])
        .arg(&partial)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            ensure!(status.success(), "Public repository clone failed");
            break;
        }
        let bytes: u64 = walkdir::WalkDir::new(&partial)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.metadata().ok())
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.len())
            .sum();
        if bytes > 10 * 1024 * 1024 * 1024
            || start.elapsed() > Duration::from_secs(900)
            || hub.stopping.load(std::sync::atomic::Ordering::Relaxed)
        {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Archive exceeded the 10 GiB or 15 minute intake limit");
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    std::fs::rename(&partial, &destination)?;
    std::fs::File::open(&hub.config.archive_dir)?.sync_all()?;
    Ok(destination)
}

struct BlobReader {
    process: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl BlobReader {
    fn new(directory: &Path) -> Result<Self> {
        let mut process = git(directory)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input = process.stdin.take().context("Open Git input")?;
        let output = BufReader::new(process.stdout.take().context("Open Git output")?);
        Ok(Self {
            process,
            input,
            output,
        })
    }

    fn read(&mut self, oid: &str, size: usize) -> Result<Vec<u8>> {
        ensure!(
            matches!(oid.len(), 40 | 64) && oid.bytes().all(|c| c.is_ascii_hexdigit()),
            "Invalid Git object ID"
        );
        writeln!(self.input, "{oid}")?;
        self.input.flush()?;
        let mut header = String::new();
        self.output.read_line(&mut header)?;
        let fields: Vec<_> = header.split_whitespace().collect();
        ensure!(
            fields.len() == 3
                && fields[0] == oid
                && fields[1] == "blob"
                && fields[2].parse::<usize>()? == size,
            "Unexpected Git blob response"
        );
        let mut content = vec![0; size];
        self.output.read_exact(&mut content)?;
        let mut end = [0];
        self.output.read_exact(&mut end)?;
        ensure!(end == *b"\n", "Invalid Git blob separator");
        Ok(content)
    }
}

impl Drop for BlobReader {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

pub(super) fn supported(path: &str) -> bool {
    let path = Path::new(path);
    if path.components().any(|part| {
        let name = part.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            "node_modules" | "vendor" | "target" | "dist" | ".git" | ".env" | ".ssh"
        ) || name.starts_with(".env.")
    }) {
        return false;
    }
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if matches!(
        name.as_str(),
        "license" | "readme" | "copying" | "makefile" | "dockerfile"
    ) {
        return true;
    }
    if name.ends_with(".min.js") || name.ends_with(".lock") {
        return false;
    }
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "md" | "mdx"
            | "rst"
            | "txt"
            | "rs"
            | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "go"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "java"
            | "rb"
            | "sh"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "html"
            | "css"
            | "sql"
    )
}

pub(super) fn passages(repo: &str, commit: &str, path: &str, text: &str) -> Vec<Passage> {
    let mut passages = Vec::new();
    let mut buffer = String::new();
    let mut start = 1;
    let mut end = 1;
    for (index, line) in text.lines().enumerate() {
        let mut part = String::new();
        // Bound long generated lines without splitting a UTF-8 scalar.
        for character in line.chars().chain(std::iter::once('\n')) {
            part.push(character);
            if part.len() >= 1500 {
                if !buffer.is_empty() {
                    passages.push(Passage {
                        repo_id: repo.into(),
                        path: path.into(),
                        commit: commit.into(),
                        line_start: start,
                        line_end: end,
                        text: std::mem::take(&mut buffer),
                    });
                }
                passages.push(Passage {
                    repo_id: repo.into(),
                    path: path.into(),
                    commit: commit.into(),
                    line_start: index + 1,
                    line_end: index + 1,
                    text: std::mem::take(&mut part),
                });
                start = index + 1;
            }
        }
        if !buffer.is_empty() && buffer.len() + part.len() > 1800 {
            passages.push(Passage {
                repo_id: repo.into(),
                path: path.into(),
                commit: commit.into(),
                line_start: start,
                line_end: end,
                text: std::mem::take(&mut buffer),
            });
        }
        if buffer.is_empty() {
            start = index + 1;
        }
        buffer.push_str(&part);
        end = index + 1;
    }
    if !buffer.trim().is_empty() {
        passages.push(Passage {
            repo_id: repo.into(),
            path: path.into(),
            commit: commit.into(),
            line_start: start,
            line_end: end,
            text: buffer,
        });
    }
    passages
}

#[derive(Serialize)]
pub(super) struct EmbedRequest<'a> {
    pub texts: &'a [String],
    pub query: bool,
}

#[derive(Deserialize)]
pub(super) struct Embeddings {
    pub model: String,
    pub vectors: Vec<Vec<f32>>,
}

impl Embeddings {
    pub fn validate(&mut self, count: usize) -> Result<()> {
        ensure!(
            !self.model.is_empty() && self.model.len() <= 256 && self.vectors.len() == count,
            "Invalid embedding response"
        );
        for vector in &mut self.vectors {
            ensure!(
                vector.len() == 384 && vector.iter().all(|v| v.is_finite()),
                "Expected 384 finite embedding dimensions"
            );
            let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
            ensure!(norm > 0.00001, "Empty embedding vector");
            vector.iter_mut().for_each(|v| *v /= norm);
        }
        Ok(())
    }
}

fn index_repository(hub: &Hub, mut repo: Repository) -> Result<()> {
    let directory = if repo.git_dir.is_empty() {
        clone_archive(hub, &repo)?
    } else {
        PathBuf::from(&repo.git_dir)
    };
    repo.git_dir = directory.to_string_lossy().into_owned();
    let commit = git_text(&directory, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    ensure!(
        matches!(commit.len(), 40 | 64) && commit.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid commit ID"
    );
    {
        let store = hub.lock()?;
        store.db.execute(
            "UPDATE repositories SET git_dir=?2 WHERE id=?1",
            params![repo.id, repo.git_dir],
        )?;
        let current = store.repository(&repo.id)?.context("Repository removed")?;
        if !current.recall {
            store.db.execute("UPDATE repositories SET status='archived',commit_sha=?2,message='Git archive available; recall is off',updated_at=?3 WHERE id=?1",params![repo.id,commit,now()])?;
            return Ok(());
        }
        if repo.commit == commit && !repo.generation.is_empty() {
            store.db.execute("UPDATE repositories SET status='ready',message='Commit unchanged; saved index reused' WHERE id=?",[&repo.id])?;
            return Ok(());
        }
    }
    let generation = uuid::Uuid::new_v4().to_string();
    let mut tree = git(&directory)
        .args(["ls-tree", "-rlz", "--full-tree", &commit])
        .stdout(Stdio::piped())
        .spawn()?;
    let output = tree.stdout.take().context("Read Git tree")?;
    let mut tree_reader = BufReader::new(output);
    let mut blobs = BlobReader::new(&directory)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let mut batch = Vec::new();
    let (mut files, mut chunks, mut skipped) = (0, 0, 0);
    let result = (|| -> Result<()> {
        loop {
            ensure!(
                !hub.stopping.load(std::sync::atomic::Ordering::Relaxed),
                "Hub is stopping"
            );
            let mut entry = Vec::new();
            if tree_reader.read_until(0, &mut entry)? == 0 {
                break;
            }
            ensure!(entry.len() < 16384, "Git tree entry is too large");
            entry.pop();
            let Some(tab) = entry.iter().position(|b| *b == b'\t') else {
                skipped += 1;
                continue;
            };
            let Ok(metadata) = std::str::from_utf8(&entry[..tab]) else {
                skipped += 1;
                continue;
            };
            let Ok(path) = std::str::from_utf8(&entry[tab + 1..]) else {
                skipped += 1;
                continue;
            };
            let fields: Vec<_> = metadata.split_whitespace().collect();
            if fields.len() != 4
                || !matches!(fields[0], "100644" | "100755")
                || fields[1] != "blob"
                || !supported(path)
            {
                skipped += 1;
                continue;
            }
            let size = fields[3].parse::<usize>()?;
            if size == 0
                || size > 1024 * 1024
                || files >= hub.config.max_files
                || chunks >= hub.config.max_chunks
            {
                skipped += 1;
                continue;
            }
            let bytes = blobs.read(fields[2], size)?;
            if bytes.contains(&0) {
                skipped += 1;
                continue;
            }
            let Ok(text) = std::str::from_utf8(&bytes) else {
                skipped += 1;
                continue;
            };
            files += 1;
            for passage in passages(&repo.id, &commit, path, text) {
                if chunks >= hub.config.max_chunks {
                    skipped += 1;
                    break;
                }
                batch.push(passage);
                chunks += 1;
                if batch.len() >= 24 {
                    save_batch(hub, &client, &repo.id, &generation, &mut batch)?;
                }
            }
            if files % 50 == 0 {
                hub.lock()?.progress(
                    &repo.id,
                    &format!("Indexing: {files} files, {chunks} passages"),
                )?;
            }
        }
        save_batch(hub, &client, &repo.id, &generation, &mut batch)?;
        ensure!(tree.wait()?.success(), "Git tree traversal failed");
        hub.lock()?
            .publish(&repo.id, &generation, &commit, files, chunks, skipped)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = tree.kill();
        let _ = tree.wait();
    }
    result
}

fn save_batch(
    hub: &Hub,
    client: &reqwest::blocking::Client,
    repo_id: &str,
    generation: &str,
    batch: &mut Vec<Passage>,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    ensure!(
        !hub.stopping.load(std::sync::atomic::Ordering::Relaxed),
        "Hub is stopping"
    );
    let embeddings = if let Some(url) = &hub.config.embed_url {
        let texts: Vec<_> = batch
            .iter()
            .map(|passage| format!("{}\n{}", passage.path, passage.text))
            .collect();
        let mut result: Embeddings = client
            .post(url)
            .json(&EmbedRequest {
                texts: &texts,
                query: false,
            })
            .send()?
            .error_for_status()?
            .json()?;
        result.validate(batch.len())?;
        Some(result)
    } else {
        None
    };
    let mut store = hub.lock()?;
    ensure!(
        store.repository(repo_id)?.is_some_and(|repo| repo.recall),
        "Recall consent was withdrawn during indexing"
    );
    for (index, passage) in batch.drain(..).enumerate() {
        let memory = embeddings
            .as_ref()
            .map(|data| (data.model.as_str(), data.vectors[index].clone()));
        store.put_passage(generation, &passage, memory)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_urls_cannot_address_local_services_or_carry_credentials() {
        for url in [
            "http://github.com/a/b",
            "https://github.com@localhost/a/b",
            "https://github.com.evil/a/b",
            "https://github.com/a/b?x=1",
            "https://user:pass@github.com/a/b",
            "file:///etc/passwd",
            "https://github.com/a/b/tree/main",
            "https://github.com/a/%2e%2e",
        ] {
            assert!(source_url(url).is_err(), "{url}");
        }
        assert_eq!(
            source_url("https://github.com/standardgalactic/abacus")
                .unwrap()
                .0,
            "https://github.com/standardgalactic/abacus.git"
        );
    }
    #[test]
    fn chunking_preserves_unicode_and_line_evidence() {
        let text = format!("one\n{}\nthree\n", "🌲".repeat(1000));
        let chunks = passages("repo", "commit", "README.md", &text);
        assert_eq!(
            chunks.iter().map(|p| p.text.as_str()).collect::<String>(),
            text
        );
        assert!(chunks
            .iter()
            .all(|p| p.text.len() <= 1800 && p.line_start <= p.line_end));
        assert_eq!(chunks.first().unwrap().line_start, 1);
        assert_eq!(chunks.last().unwrap().line_end, 3);
    }
    #[test]
    fn credentials_and_build_artifacts_are_excluded() {
        for path in [
            ".env",
            ".env.production",
            "node_modules/a/index.js",
            "id_rsa",
            "test.pem",
            "dist/app.js",
        ] {
            assert!(!supported(path));
        }
        assert!(supported("docs/MEM8.md"));
        assert!(supported("src/lib.rs"));
    }
}
