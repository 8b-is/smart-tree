//! Persistent file facts and local keyword retrieval. No model call required.

use crate::mem8::path_serde::StoredPath;
use crate::mem8::record_store::RecordStore;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

const PREFIX: &str = "file/v1/";
const CONTENT_LIMIT: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    #[serde(with = "crate::mem8::path_serde")]
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub terms: BTreeSet<String>,
    pub preview: String,
}

pub struct FileIndex {
    store: RecordStore,
    view: Arc<IndexView>,
    storage_directory: PathBuf,
    pending: BTreeSet<PathBuf>,
}

impl FileIndex {
    pub fn open(directory: &Path) -> Result<Self> {
        let mut store = RecordStore::open(&directory.join("file_index.t8"))?;
        let mut files = BTreeMap::new();
        let mut pending = BTreeSet::new();
        for key in store.keys_with_prefix("pending/v1/") {
            if let Some(Some(path)) = store.get::<Option<StoredPath>>(&key)? {
                pending.insert(path.0);
            }
        }
        for key in store.keys_with_prefix(PREFIX) {
            if let Some(Some(file)) = store.get::<Option<IndexedFile>>(&key)? {
                anyhow::ensure!(key == file_key(&file.path), "Invalid file index key");
                files.insert(file.path.clone(), file);
            }
        }
        let mut view = IndexView {
            files: BTreeMap::new(),
            postings: BTreeMap::new(),
        };
        for (path, file) in files {
            for term in &file.terms {
                Arc::make_mut(view.postings.entry(term.clone()).or_default()).insert(path.clone());
            }
            view.files.insert(path, Arc::new(file));
        }
        Ok(Self {
            store,
            view: Arc::new(view),
            storage_directory: directory.canonicalize()?,
            pending,
        })
    }

    /// Reconcile metadata against saved entries; unchanged files are not read
    /// or rewritten. Missing roots retain their cache for removable volumes.
    pub fn reconcile(&mut self, root: &Path) -> Result<BTreeSet<PathBuf>> {
        if !root.is_dir() {
            return Ok(BTreeSet::new());
        }
        let storage_directory = self.storage_directory.clone();
        let entries = walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                !entry.path().starts_with(&storage_directory)
                    && (entry.depth() == 0 || !ignored_directory(entry.path()))
            });
        let mut seen = BTreeSet::new();
        let mut changed = BTreeSet::new();
        let mut inaccessible = BTreeSet::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    inaccessible.insert(error.path().unwrap_or(root).to_path_buf());
                    tracing::warn!(%error, "Skipping inaccessible index path; saved entries retained");
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.into_path();
            seen.insert(path.clone());
            if self.update_file(&path)? {
                if let Some(parent) = path.parent() {
                    changed.insert(parent.to_path_buf());
                }
            }
        }
        let removed: Vec<_> = self
            .view
            .files
            .keys()
            .filter(|path| {
                path.starts_with(root)
                    && !seen.contains(*path)
                    && !inaccessible.iter().any(|skipped| path.starts_with(skipped))
            })
            .cloned()
            .collect();
        for path in removed {
            self.remove_file(&path)?;
            if let Some(parent) = path.parent() {
                changed.insert(parent.to_path_buf());
            }
        }
        Ok(changed)
    }

    pub fn update_path(&mut self, path: &Path) -> Result<BTreeSet<PathBuf>> {
        if path.starts_with(&self.storage_directory) || path.ancestors().any(ignored_directory) {
            return Ok(BTreeSet::new());
        }
        if path.is_dir() {
            let mut changed = self.reconcile(path)?;
            changed.insert(path.to_path_buf());
            return Ok(changed);
        }
        let mut changed = BTreeSet::new();
        if path.is_file() {
            if self.update_file(path)? {
                if let Some(parent) = path.parent() {
                    changed.insert(parent.to_path_buf());
                }
            }
        } else if !path.exists() {
            let removed: Vec<_> = self
                .view
                .files
                .keys()
                .filter(|known| known.starts_with(path))
                .cloned()
                .collect();
            for removed in removed {
                self.remove_file(&removed)?;
            }
            if let Some(parent) = path.parent() {
                changed.insert(parent.to_path_buf());
            }
            changed.insert(path.to_path_buf());
        }
        Ok(changed)
    }

    pub fn retain_roots(&mut self, roots: &[PathBuf]) -> Result<()> {
        let removed: Vec<_> = self
            .view
            .files
            .keys()
            .filter(|path| !roots.iter().any(|root| path.starts_with(root)))
            .cloned()
            .collect();
        for path in removed {
            self.remove_file(&path)?;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Arc<IndexView> {
        Arc::clone(&self.view)
    }
    pub fn len(&self) -> usize {
        self.view.len()
    }

    pub fn pending_context(&self) -> BTreeSet<PathBuf> {
        self.pending.clone()
    }

    /// Called only after native directory context has been synced.
    pub fn acknowledge_context(&mut self, applied: &BTreeSet<PathBuf>) -> Result<()> {
        for path in self
            .pending
            .intersection(applied)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.store.forget::<StoredPath>(&pending_key(&path))?;
            self.pending.remove(&path);
        }
        Ok(())
    }

    fn mark_pending(&mut self, path: &Path) -> Result<()> {
        if !self.pending.contains(path) {
            self.store
                .put(&pending_key(path), &Some(StoredPath(path.to_path_buf())))?;
            self.pending.insert(path.to_path_buf());
        }
        Ok(())
    }

    fn update_file(&mut self, path: &Path) -> Result<bool> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                tracing::warn!(%error, path = %path.display(), "Cannot read indexed file metadata; saved entry retained");
                return Ok(false);
            }
        };
        if !metadata.is_file() {
            return Ok(false);
        }
        let modified = metadata.modified().ok();
        if modified.is_some()
            && self
                .view
                .files
                .get(path)
                .is_some_and(|old| old.size == metadata.len() && old.modified == modified)
        {
            return Ok(false);
        }
        let mut preview = String::new();
        // Index a bounded text prefix. Binary documents remain searchable by
        // path and explicit contextual annotations, without lossy extraction.
        if is_text(path) {
            use std::io::Read;
            let mut bytes = Vec::new();
            let mut options = std::fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let read = options
                .open(path)
                .and_then(|file| {
                    if !file.metadata()?.is_file() {
                        return Err(std::io::Error::other("Index requires a regular file"));
                    }
                    Ok(file)
                })
                .and_then(|file| file.take(CONTENT_LIMIT).read_to_end(&mut bytes));
            if read.is_ok() && !bytes.contains(&0) {
                // A prefix may end part way through a UTF-8 codepoint.
                preview = String::from_utf8_lossy(&bytes).into_owned();
            }
        }
        let mut terms = tokenize(&path.to_string_lossy());
        terms.extend(tokenize(&preview));
        let file = IndexedFile {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            terms,
            preview,
        };
        if let Some(parent) = path.parent() {
            self.mark_pending(parent)?;
        }
        self.store.put(&file_key(path), &Some(&file))?;
        let view = Arc::make_mut(&mut self.view);
        view.remove_postings(path);
        for term in &file.terms {
            Arc::make_mut(view.postings.entry(term.clone()).or_default())
                .insert(path.to_path_buf());
        }
        view.files.insert(path.to_path_buf(), Arc::new(file));
        Ok(true)
    }

    fn remove_file(&mut self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            self.mark_pending(parent)?;
        }
        self.store.put(&file_key(path), &None::<IndexedFile>)?;
        let view = Arc::make_mut(&mut self.view);
        view.remove_postings(path);
        view.files.remove(path);
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct IndexView {
    files: BTreeMap<PathBuf, Arc<IndexedFile>>,
    postings: BTreeMap<String, Arc<BTreeSet<PathBuf>>>,
}

impl IndexView {
    pub fn candidates(&self, terms: &[String]) -> BTreeSet<PathBuf> {
        terms
            .iter()
            .filter_map(|term| self.postings.get(term))
            .flat_map(|paths| paths.iter())
            .cloned()
            .collect()
    }

    pub fn get(&self, path: &Path) -> Option<&IndexedFile> {
        self.files.get(path).map(AsRef::as_ref)
    }
    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn all_paths(&self) -> BTreeSet<PathBuf> {
        self.files.keys().cloned().collect()
    }

    fn remove_postings(&mut self, path: &Path) {
        if let Some(old) = self.files.get(path) {
            for term in &old.terms {
                if let Some(paths) = self.postings.get_mut(term) {
                    Arc::make_mut(paths).remove(path);
                    if paths.is_empty() {
                        self.postings.remove(term);
                    }
                }
            }
        }
    }
}

pub fn tokenize(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty() && word.len() <= 80)
        .map(str::to_lowercase)
        .collect()
}

fn file_key(path: &Path) -> String {
    format!(
        "{PREFIX}{}",
        hex::encode(path.as_os_str().as_encoded_bytes())
    )
}

fn pending_key(path: &Path) -> String {
    format!(
        "pending/v1/{}",
        hex::encode(path.as_os_str().as_encoded_bytes())
    )
}

pub(super) fn ignored_directory(path: &Path) -> bool {
    path.is_dir()
        && matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some(".git" | "node_modules" | "target" | ".cache")
        )
}

fn is_text(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "txt"
                | "md"
                | "markdown"
                | "rst"
                | "csv"
                | "tsv"
                | "json"
                | "toml"
                | "yaml"
                | "yml"
                | "rs"
                | "py"
                | "js"
                | "ts"
                | "html"
                | "css"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn inaccessible_branch_keeps_cached_files_and_other_branches_update() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("documents");
        let private = root.join("private");
        std::fs::create_dir_all(&private).unwrap();
        let saved = private.join("saved.md");
        std::fs::write(&saved, "remembered astronomy").unwrap();
        let mut index = FileIndex::open(&temp.path().join("memory")).unwrap();
        index.reconcile(&root).unwrap();
        struct RestoreAccess(PathBuf);
        impl Drop for RestoreAccess {
            fn drop(&mut self) {
                let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700));
            }
        }
        let _restore = RestoreAccess(private.clone());
        std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0)).unwrap();
        if std::fs::read_dir(&private).is_ok() {
            return;
        } // Privileged test runners bypass file permissions.
        let visible = root.join("new.md");
        std::fs::write(&visible, "new garden plan").unwrap();
        index.reconcile(&root).unwrap();
        assert!(index.view.get(&saved).is_some());
        assert!(index.view.get(&visible).is_some());
    }

    #[test]
    fn restart_uses_saved_content_and_reconciliation_changes_only_modified_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("documents");
        std::fs::create_dir(&root).unwrap();
        let document = root.join("moon.md");
        std::fs::write(&document, "daughter astronomy homework").unwrap();
        let storage = temp.path().join("memory");
        let mut index = FileIndex::open(&storage).unwrap();
        index.reconcile(&root).unwrap();
        let before = std::fs::metadata(storage.join("file_index.t8"))
            .unwrap()
            .len();
        assert!(index.reconcile(&root).unwrap().is_empty());
        assert_eq!(
            before,
            std::fs::metadata(storage.join("file_index.t8"))
                .unwrap()
                .len()
        );
        drop(index);
        std::fs::remove_file(&document).unwrap();
        let mut index = FileIndex::open(&storage).unwrap();
        assert_eq!(
            index.view.candidates(&["daughter".into()]),
            BTreeSet::from([document.clone()])
        );
        assert_eq!(
            index.view.get(&document).unwrap().preview,
            "daughter astronomy homework"
        );
        index.reconcile(&root).unwrap();
        assert!(index.view.candidates(&["daughter".into()]).is_empty());
        std::fs::write(&document, "garden plan").unwrap();
        index.update_path(&document).unwrap();
        assert_eq!(index.view.candidates(&["garden".into()]).len(), 1);
    }
}
