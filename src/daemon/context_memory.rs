//! Native MEM8 context, persisted watches, and evidence-backed file recall.

use super::search::{tokenize, FileIndex, IndexView};
use super::{create_directory_info, detect_project, scan_system_context, SystemContext};
use crate::hot_watcher::{HotWatcher, WatchSnapshot};
use crate::mem8::record_store::RecordStore;
use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const ROOT_PREFIX: &str = "directory/v1/";
const ASSOCIATION_PREFIX: &str = "association/v1/";

#[derive(Clone, Serialize, Deserialize)]
struct RootMemory {
    context: SystemContext,
    watch: WatchSnapshot,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Association {
    #[serde(with = "crate::mem8::path_serde")]
    pub path: PathBuf,
    #[serde(default)]
    pub people: Vec<String>,
    pub notes: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RecallRequest {
    pub query: String,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct RecallHit {
    #[serde(with = "crate::mem8::path_serde")]
    pub path: PathBuf,
    pub score: usize,
    pub modified_at: Option<DateTime<Utc>>,
    pub evidence: Vec<String>,
    pub preview: String,
}

pub struct ContextMemory {
    store: RecordStore,
    roots: BTreeMap<PathBuf, RootMemory>,
    associations: Arc<BTreeMap<String, Association>>,
    pub watcher: HotWatcher,
    index: FileIndex,
    storage_directory: PathBuf,
    dirty: BTreeSet<PathBuf>,
    applied_context: BTreeSet<PathBuf>,
}

impl ContextMemory {
    /// Restore saved context before any filesystem reconciliation.
    pub fn open(directory: &Path) -> Result<Self> {
        let mut store = RecordStore::open_memory(&directory.join("directory_context.m8"))?;
        let mut index = FileIndex::open(directory)?;
        let mut roots = BTreeMap::new();
        let mut watcher = HotWatcher::new();
        for key in store.keys_with_prefix(ROOT_PREFIX) {
            if let Some(Some(mut root)) = store.get::<Option<RootMemory>>(&key)? {
                ensure!(
                    key == root_key(&root.watch.path),
                    "Invalid directory memory key"
                );
                root.watch.wave = store
                    .get_wave(&key)?
                    .context("Missing directory memory wave")?;
                if let Err(error) = watcher.restore(&root.watch) {
                    tracing::warn!(%error, path = %root.watch.path.display(), "Saved watch is currently unavailable; retaining context");
                }
                roots.insert(root.watch.path.clone(), root);
            }
        }
        let mut associations = BTreeMap::new();
        for key in store.keys_with_prefix(ASSOCIATION_PREFIX) {
            if let Some(association) = store.get::<Association>(&key)? {
                associations.insert(key, association);
            }
        }
        index.retain_roots(&roots.keys().cloned().collect::<Vec<_>>())?;
        Ok(Self {
            store,
            roots,
            associations: Arc::new(associations),
            watcher,
            index,
            storage_directory: directory.canonicalize()?,
            dirty: BTreeSet::new(),
            applied_context: BTreeSet::new(),
        })
    }

    pub fn context(&self) -> SystemContext {
        let mut combined = SystemContext::default();
        for root in self.roots.values() {
            combined.projects.extend(root.context.projects.clone());
            combined
                .consciousnesses
                .extend(root.context.consciousnesses.clone());
            combined.last_scan = combined.last_scan.max(root.context.last_scan);
        }
        combined
    }

    pub fn indexed_files(&self) -> usize {
        self.index.len()
    }

    pub fn watch(&mut self, path: &Path) -> Result<PathBuf> {
        let path = path
            .canonicalize()
            .context("Cannot resolve watch directory")?;
        ensure!(path.is_dir(), "Watch path must be a directory");
        ensure!(
            !path.starts_with(&self.storage_directory),
            "Cannot watch the daemon memory directory"
        );
        if let Some(root) = self.roots.get(&path) {
            if self.watcher.snapshot(&path).is_none() {
                self.watcher.restore(&root.watch)?;
            }
            return Ok(path);
        }
        self.watcher.watch(&path)?;
        let result = (|| -> Result<RootMemory> {
            let mut context = SystemContext::default();
            scan_system_context(&mut context, std::slice::from_ref(&path))?;
            self.index.reconcile(&path)?;
            let watch = self
                .watcher
                .snapshot(&path)
                .context("Directory watch was not registered")?;
            let root = RootMemory { context, watch };
            self.store
                .put_with_wave(&root_key(&path), &Some(&root), &root.watch.wave)?;
            Ok(root)
        })();
        match result {
            Ok(root) => {
                self.roots.insert(path.clone(), root);
                Ok(path)
            }
            Err(error) => {
                if let Err(rollback) = self.watcher.unwatch(&path) {
                    tracing::error!(%rollback, "Could not roll back directory watch");
                }
                Err(error)
            }
        }
    }

    /// Startup defaults must not resurrect a root removed by DELETE /watch.
    pub fn watch_initial(&mut self, path: &Path) -> Result<()> {
        let path = path.canonicalize()?;
        if self
            .store
            .get::<Option<RootMemory>>(&root_key(&path))?
            .is_none()
        {
            self.watch(&path)?;
        }
        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<PathBuf> {
        let path = normalized_path(path)?;
        let Some(root) = self.roots.get(&path).cloned() else {
            self.index
                .retain_roots(&self.roots.keys().cloned().collect::<Vec<_>>())?;
            return Ok(path);
        };
        self.watcher.unwatch(&path)?;
        if let Err(error) = self.store.forget::<RootMemory>(&root_key(&path)) {
            if let Err(rollback) = self.watcher.restore(&root.watch) {
                tracing::error!(%rollback, "Could not restore directory watch after storage failure");
            }
            return Err(error);
        }
        self.roots.remove(&path);
        self.dirty.remove(&path);
        self.index
            .retain_roots(&self.roots.keys().cloned().collect::<Vec<_>>())?;
        Ok(path)
    }

    /// Process only changed paths. Periodic reconciliation covers notifications
    /// missed while the daemon was stopped; it preserves unchanged index entries.
    pub fn maintain(&mut self, reconcile: bool, checkpoint: bool) -> Result<()> {
        self.watcher.process_pending_events();
        let mut changed = BTreeSet::new();
        if reconcile {
            for (path, root) in &self.roots {
                if self.watcher.snapshot(path).is_none() && path.is_dir() {
                    self.watcher.restore(&root.watch)?;
                }
                changed.extend(self.index.reconcile(path)?);
            }
        }
        for path in self.watcher.take_changed_paths() {
            if !path.starts_with(&self.storage_directory)
                && self
                    .roots
                    .keys()
                    .any(|root| path.starts_with(root) && root.is_dir())
            {
                changed.extend(self.index.update_path(&path)?);
            }
        }
        changed.extend(
            self.index
                .pending_context()
                .difference(&self.applied_context)
                .cloned(),
        );
        for path in changed {
            let relevant: Vec<_> = self
                .roots
                .keys()
                .filter(|root| path.starts_with(root))
                .collect();
            if !relevant.is_empty() && relevant.iter().all(|root| !root.is_dir()) {
                continue;
            }
            let is_directory = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata.is_dir(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => {
                    tracing::warn!(%error, path = %path.display(), "Directory context unavailable; pending update retained");
                    continue;
                }
            };
            if is_directory && std::fs::read_dir(&path).is_err() {
                continue;
            }
            for (root_path, root) in &mut self.roots {
                if !path.starts_with(root_path) {
                    continue;
                }
                if !is_directory {
                    root.context
                        .projects
                        .retain(|known, _| !known.starts_with(&path));
                    root.context
                        .consciousnesses
                        .retain(|known, _| !known.starts_with(&path));
                } else {
                    root.context.projects.remove(&path);
                    if let Some(project) = detect_project(&path) {
                        root.context.projects.insert(path.clone(), project);
                    }
                    if let Some(info) = create_directory_info(&path) {
                        root.context.consciousnesses.insert(path.clone(), info);
                    }
                }
                root.context.last_scan = Some(std::time::SystemTime::now());
                self.dirty.insert(root_path.clone());
            }
            self.applied_context.insert(path);
        }
        if checkpoint {
            self.checkpoint()?;
        }
        Ok(())
    }

    pub fn checkpoint(&mut self) -> Result<()> {
        for (path, root) in &mut self.roots {
            // Idle decay is a live ranking hint, not a reason to rewrite memory.
            if !self.dirty.contains(path) {
                continue;
            }
            if let Some(watch) = self.watcher.snapshot(path) {
                root.watch = watch;
            }
            self.store
                .put_with_wave(&root_key(path), &Some(&root), &root.watch.wave)?;
            self.dirty.remove(path);
        }
        self.index.acknowledge_context(&self.applied_context)?;
        self.applied_context.clear();
        Ok(())
    }

    pub fn remember(&mut self, mut association: Association) -> Result<String> {
        ensure!(
            !association.notes.trim().is_empty() || !association.people.is_empty(),
            "Provide context or people to remember"
        );
        ensure!(
            association.notes.len() <= 64 * 1024
                && association.people.len() <= 100
                && association.people.iter().all(|person| person.len() <= 1024),
            "Context association is too large"
        );
        association.path = normalized_path(&association.path)?;
        let id = uuid::Uuid::new_v4().to_string();
        let key = format!("{ASSOCIATION_PREFIX}{id}");
        self.store.put(&key, &association)?;
        Arc::make_mut(&mut self.associations).insert(key, association);
        Ok(id)
    }

    pub fn recall_view(&self) -> RecallView {
        RecallView {
            index: self.index.snapshot(),
            associations: Arc::clone(&self.associations),
        }
    }

    #[cfg(test)]
    pub fn recall(&self, request: &RecallRequest, now: DateTime<Utc>) -> Vec<RecallHit> {
        self.recall_view().recall(request, now)
    }
}

#[derive(Clone)]
pub struct RecallView {
    index: Arc<IndexView>,
    associations: Arc<BTreeMap<String, Association>>,
}

impl RecallView {
    pub fn recall(&self, request: &RecallRequest, now: DateTime<Utc>) -> Vec<RecallHit> {
        let query = request.query.to_lowercase();
        let stop_words = [
            "a", "an", "the", "what", "which", "was", "is", "i", "we", "they", "my", "with", "on",
            "of", "for", "find", "document", "file", "worked", "last", "week", "year", "old",
        ];
        let terms: Vec<_> = tokenize(&query)
            .into_iter()
            .filter(|term| !stop_words.contains(&term.as_str()))
            .collect();
        let last_week = query.contains("last week");
        let monday = now.date_naive()
            - chrono::Duration::days(i64::from(now.weekday().num_days_from_monday()));
        let week_end = monday.and_hms_opt(0, 0, 0).map(|date| date.and_utc());
        let after = request.after.or_else(|| {
            if last_week {
                week_end.map(|end| end - chrono::Duration::days(7))
            } else {
                None
            }
        });
        let before = request.before.or(if last_week { week_end } else { None });
        let in_window = |time: DateTime<Utc>| {
            after.is_none_or(|start| time >= start) && before.is_none_or(|end| time < end)
        };
        let mut candidates = if terms.is_empty() {
            self.index.all_paths()
        } else {
            self.index.candidates(&terms)
        };
        let mut memories: BTreeMap<PathBuf, Vec<(&Association, usize)>> = BTreeMap::new();
        for association in self.associations.values() {
            let words = tokenize(&format!(
                "{} {} {}",
                association.path.display(),
                association.people.join(" "),
                association.notes
            ));
            let score = terms.iter().filter(|term| words.contains(*term)).count();
            if (score > 0 || terms.is_empty()) && in_window(association.occurred_at) {
                candidates.insert(association.path.clone());
                memories
                    .entry(association.path.clone())
                    .or_default()
                    .push((association, score));
            }
        }
        let mut hits = Vec::new();
        for path in candidates {
            let file = self.index.get(&path);
            let modified_at = file
                .and_then(|file| file.modified)
                .map(DateTime::<Utc>::from);
            let associations = memories.get(&path);
            if associations.is_none()
                && (after.is_some() || before.is_some())
                && !modified_at.is_some_and(in_window)
            {
                continue;
            }
            let mut score = file
                .map(|file| {
                    terms
                        .iter()
                        .filter(|term| file.terms.contains(*term))
                        .count()
                })
                .unwrap_or(0);
            let mut evidence = Vec::new();
            if let Some(associations) = associations {
                for (association, matched) in associations {
                    score += matched * 3;
                    evidence.push(format!(
                        "{}: {} [{}]",
                        association.occurred_at.to_rfc3339(),
                        association.notes,
                        association.people.join(", ")
                    ));
                }
            }
            if file.is_some() && score > 0 {
                evidence.push("Matched indexed path or text".into());
            }
            hits.push(RecallHit {
                path,
                score,
                modified_at,
                evidence,
                preview: file
                    .map(|file| file.preview.chars().take(240).collect())
                    .unwrap_or_default(),
            });
        }
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| b.modified_at.cmp(&a.modified_at))
                .then_with(|| a.path.cmp(&b.path))
        });
        hits.truncate(request.limit.unwrap_or(10).min(100));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unfinished_context_update_replays_without_rescanning_the_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        std::fs::write(root.join("README.md"), "Old context").unwrap();
        let root = root.canonicalize().unwrap();
        let storage = temp.path().join("memory");
        let mut memory = ContextMemory::open(&storage).unwrap();
        memory.watch(&root).unwrap();
        memory.maintain(false, true).unwrap();
        std::fs::write(
            root.join("README.md"),
            "New context saved only to file index",
        )
        .unwrap();
        memory.index.update_path(&root.join("README.md")).unwrap();
        drop(memory);
        let mut memory = ContextMemory::open(&storage).unwrap();
        assert_eq!(memory.context().projects[&root].essence, "Old context");
        memory.maintain(false, true).unwrap();
        assert_eq!(
            memory.context().projects[&root].essence,
            "New context saved only to file index"
        );
        assert!(memory.index.pending_context().is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn native_paths_survive_context_and_index_restarts() {
        use std::os::unix::ffi::OsStringExt;
        let temp = tempfile::tempdir().unwrap();
        let root = temp
            .path()
            .join(std::ffi::OsString::from_vec(b"project-\xff".to_vec()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        for name in [b"document-\xff.md", b"document-\xfe.md"] {
            std::fs::write(
                root.join(std::ffi::OsString::from_vec(name.to_vec())),
                "unique document",
            )
            .unwrap();
        }
        let storage = temp.path().join("memory");
        let mut memory = ContextMemory::open(&storage).unwrap();
        let root = memory.watch(&root).unwrap();
        drop(memory);
        let memory = ContextMemory::open(&storage).unwrap();
        assert!(memory.context().projects.contains_key(&root));
        let hits = memory.recall(
            &RecallRequest {
                query: "unique".into(),
                after: None,
                before: None,
                limit: None,
            },
            Utc::now(),
        );
        assert_eq!(hits.len(), 2);
        assert_ne!(hits[0].path, hits[1].path);
        assert!(serde_json::to_string(&hits).is_ok());
    }

    #[test]
    fn restores_context_without_rebuilding_and_persists_incremental_changes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        std::fs::write(root.join("README.md"), "Original directory context").unwrap();
        let root = root.canonicalize().unwrap();
        let storage = temp.path().join("memory");
        let mut memory = ContextMemory::open(&storage).unwrap();
        memory.watch(&root).unwrap();
        assert_eq!(
            memory.context().projects[&root].essence,
            "Original directory context"
        );
        drop(memory);
        std::fs::write(root.join("README.md"), "Changed after daemon stopped").unwrap();
        let mut memory = ContextMemory::open(&storage).unwrap();
        assert_eq!(
            memory.context().projects[&root].essence,
            "Original directory context"
        );
        assert_eq!(memory.watcher.summary().total_watched, 1);
        // Registering an existing root must also use its saved context.
        memory.watch(&root).unwrap();
        assert_eq!(
            memory.context().projects[&root].essence,
            "Original directory context"
        );
        memory.maintain(true, true).unwrap();
        assert_eq!(
            memory.context().projects[&root].essence,
            "Changed after daemon stopped"
        );
        std::fs::remove_file(root.join("Cargo.toml")).unwrap();
        memory.maintain(true, true).unwrap();
        assert!(!memory.context().projects.contains_key(&root));
        memory.unwatch(&root).unwrap();
        drop(memory);
        let mut memory = ContextMemory::open(&storage).unwrap();
        memory.watch_initial(&root).unwrap();
        assert!(memory.context().consciousnesses.is_empty());
        assert_eq!(memory.watcher.summary().total_watched, 0);
        assert_eq!(memory.indexed_files(), 0);
    }

    #[test]
    fn recall_links_people_and_time_to_evidence_across_restart() {
        let temp = tempfile::tempdir().unwrap();
        let document = temp.path().join("moon-homework.pdf");
        std::fs::write(&document, b"binary fixture").unwrap();
        let storage = temp.path().join("memory");
        let mut memory = ContextMemory::open(&storage).unwrap();
        let now: DateTime<Utc> = "2026-09-11T12:00:00Z".parse().unwrap();
        memory
            .remember(Association {
                path: document.clone(),
                people: vec!["daughter".into()],
                notes: "Worked on lunar homework with my 10 year old daughter".into(),
                occurred_at: "2026-09-03T16:00:00Z".parse().unwrap(),
            })
            .unwrap();
        memory
            .remember(Association {
                path: temp.path().join("old-project.pdf"),
                people: vec!["daughter".into()],
                notes: "Earlier homework".into(),
                occurred_at: "2026-08-10T16:00:00Z".parse().unwrap(),
            })
            .unwrap();
        drop(memory);
        let memory = ContextMemory::open(&storage).unwrap();
        let request = RecallRequest {
            query: "what document was I worked on with my 10 year old daughter last week".into(),
            after: None,
            before: None,
            limit: None,
        };
        let hits = memory.recall(&request, now);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, document.canonicalize().unwrap());
        assert!(hits[0].evidence[0].contains("2026-09-03"));
        assert!(hits[0].evidence[0].contains("daughter"));
    }

    #[test]
    fn idle_checkpoints_do_not_rewrite_memory() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("documents");
        std::fs::create_dir(&root).unwrap();
        let storage = temp.path().join("memory");
        let mut memory = ContextMemory::open(&storage).unwrap();
        memory.watch(&root).unwrap();
        let before = std::fs::metadata(storage.join("directory_context.m8"))
            .unwrap()
            .len();
        memory.maintain(true, true).unwrap();
        assert_eq!(
            std::fs::metadata(storage.join("directory_context.m8"))
                .unwrap()
                .len(),
            before
        );
    }
}

fn root_key(path: &Path) -> String {
    format!(
        "{ROOT_PREFIX}{}",
        hex::encode(path.as_os_str().as_encoded_bytes())
    )
}

fn normalized_path(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    loop {
        if let Ok(mut resolved) = ancestor.canonicalize() {
            for name in suffix.iter().rev() {
                resolved.push(name);
            }
            return Ok(resolved);
        }
        match (ancestor.parent(), ancestor.file_name()) {
            (Some(parent), Some(name)) => {
                suffix.push(name.to_os_string());
                ancestor = parent;
            }
            _ => return Ok(absolute),
        }
    }
}
