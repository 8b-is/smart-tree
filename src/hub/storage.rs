use crate::mem8::record_store::RecordStore;
use anyhow::{ensure, Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

pub(super) fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub(super) fn new_token() -> String {
    use rand::RngCore;
    let mut bytes = [0; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub(super) fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug)]
pub(super) struct Repository {
    pub id: String,
    pub collection: String,
    pub name: String,
    pub source_url: String,
    pub git_dir: String,
    pub owner_hash: String,
    pub public: bool,
    pub recall: bool,
    pub approved: bool,
    pub status: String,
    pub generation: String,
    pub commit: String,
    pub files: usize,
    pub chunks: usize,
    pub skipped: usize,
    pub updated_at: String,
    pub message: String,
}

impl Repository {
    pub fn visible(&self, hash: &str, admin: bool) -> bool {
        admin || (!hash.is_empty() && self.owner_hash == hash) || (self.public && self.approved)
    }

    pub fn public_view(&self) -> Value {
        json!({
            "id": self.id, "collection": self.collection, "name": self.name,
            "source_url": self.source_url, "public": self.public,
            "recall_opt_in": self.recall, "status": self.status,
            "commit": self.commit, "indexed_files": self.files,
            "indexed_passages": self.chunks, "skipped_files": self.skipped,
            "updated_at": self.updated_at, "message": self.message,
            "clone_path": format!("/git/{}.git", self.id),
        })
    }
}

fn repository_row(row: &Row<'_>) -> rusqlite::Result<Repository> {
    Ok(Repository {
        id: row.get(0)?,
        collection: row.get(1)?,
        name: row.get(2)?,
        source_url: row.get(3)?,
        git_dir: row.get(4)?,
        owner_hash: row.get(5)?,
        public: row.get(6)?,
        recall: row.get(7)?,
        approved: row.get(8)?,
        status: row.get(9)?,
        generation: row.get(10)?,
        commit: row.get(11)?,
        files: row.get(12)?,
        chunks: row.get(13)?,
        skipped: row.get(14)?,
        updated_at: row.get(15)?,
        message: row.get(16)?,
    })
}

const REPO_COLUMNS: &str = "id,collection,name,source_url,git_dir,owner_hash,public,recall,approved,status,generation,commit_sha,files,chunks,skipped,updated_at,message";

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Passage {
    pub repo_id: String,
    pub path: String,
    pub commit: String,
    pub line_start: usize,
    pub line_end: usize,
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct SemanticMemory {
    pub chunk_id: i64,
    pub model: String,
    pub vector: Vec<f32>,
}

pub(super) struct Store {
    pub db: Connection,
    passages: RecordStore,
    memories: RecordStore,
    feedback: RecordStore,
    pub vectors: HashMap<i64, SemanticMemory>,
}

impl Store {
    pub fn open(directory: &Path) -> Result<Self> {
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
        }
        // Opening the journals first also holds an exclusive process lock.
        let passages = RecordStore::open(&directory.join("passages.t8"))?;
        let mut memories = RecordStore::open_memory(&directory.join("recall.m8"))?;
        let feedback = RecordStore::open(&directory.join("feedback.t8"))?;
        let db = Connection::open(directory.join("catalog.sqlite3"))?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS repositories (
               id TEXT PRIMARY KEY, collection TEXT NOT NULL, name TEXT NOT NULL,
               source_url TEXT NOT NULL, git_dir TEXT NOT NULL DEFAULT '', owner_hash TEXT NOT NULL,
               public INTEGER NOT NULL DEFAULT 0, recall INTEGER NOT NULL DEFAULT 0,
               approved INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL,
               generation TEXT NOT NULL DEFAULT '', commit_sha TEXT NOT NULL DEFAULT '',
               files INTEGER NOT NULL DEFAULT 0, chunks INTEGER NOT NULL DEFAULT 0,
               skipped INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL, message TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS repository_collection ON repositories(collection, name);
             CREATE TABLE IF NOT EXISTS chunks (
               id INTEGER PRIMARY KEY AUTOINCREMENT, repo_id TEXT NOT NULL REFERENCES repositories(id),
               generation TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS chunks_repository ON chunks(repo_id, generation);
             CREATE VIRTUAL TABLE IF NOT EXISTS chunk_terms USING fts5(
               path, body, content='', contentless_delete=1, tokenize='unicode61'
             );
             CREATE TABLE IF NOT EXISTS feedback (
               id TEXT PRIMARY KEY, title TEXT NOT NULL, category TEXT NOT NULL, created_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS tls_names (domain TEXT PRIMARY KEY, requested_at INTEGER NOT NULL);
             UPDATE repositories SET status='queued', message='Resuming interrupted indexing' WHERE status='indexing';"
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                directory.join("catalog.sqlite3"),
                std::fs::Permissions::from_mode(0o600),
            )?;
        }
        let mut vectors = HashMap::new();
        for key in memories.keys_with_prefix("chunk/") {
            if let Some(memory) = memories.get::<Option<SemanticMemory>>(&key)?.flatten() {
                // The catalogue is the authority for consent and published generations.
                let allowed: bool = db.query_row(
                    "SELECT EXISTS(SELECT 1 FROM chunks c JOIN repositories r ON r.id=c.repo_id
                     WHERE c.id=? AND r.recall=1 AND r.approved=1 AND c.generation=r.generation)",
                    [memory.chunk_id],
                    |row| row.get(0),
                )?;
                if allowed {
                    vectors.insert(memory.chunk_id, memory);
                }
            }
        }
        Ok(Self {
            db,
            passages,
            memories,
            feedback,
            vectors,
        })
    }

    pub fn repository(&self, id: &str) -> Result<Option<Repository>> {
        Ok(self
            .db
            .query_row(
                &format!("SELECT {REPO_COLUMNS} FROM repositories WHERE id=?"),
                [id],
                repository_row,
            )
            .optional()?)
    }

    pub fn repositories(
        &self,
        hash: &str,
        admin: bool,
        collection: &str,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Repository>> {
        let mut statement = self.db.prepare(&format!(
            "SELECT {REPO_COLUMNS} FROM repositories
             WHERE ((public=1 AND approved=1) OR (owner_hash=?1 AND ?1!='') OR ?2)
             AND (?3='' OR collection=?3) AND (?4='' OR instr(lower(name),lower(?4))>0)
             ORDER BY collection,name LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = statement.query_map(
            params![hash, admin, collection, query, limit, offset],
            repository_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn add_repository(&self, repo: &Repository) -> Result<()> {
        self.db.execute(
            "INSERT INTO repositories (id,collection,name,source_url,git_dir,owner_hash,public,recall,approved,status,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![repo.id,repo.collection,repo.name,repo.source_url,repo.git_dir,repo.owner_hash,repo.public,repo.recall,repo.approved,repo.status,repo.updated_at],
        )?;
        Ok(())
    }

    pub fn next_job(&self) -> Result<Option<Repository>> {
        let repo = self.db.query_row(&format!("SELECT {REPO_COLUMNS} FROM repositories WHERE status='queued' AND approved=1 ORDER BY updated_at LIMIT 1"), [], repository_row).optional()?;
        if let Some(repo) = &repo {
            self.db.execute("UPDATE repositories SET status='indexing',message='Reading committed Git objects' WHERE id=?", [&repo.id])?;
        }
        Ok(repo)
    }

    pub fn progress(&self, id: &str, message: &str) -> Result<()> {
        self.db.execute(
            "UPDATE repositories SET message=?2,updated_at=?3 WHERE id=?1",
            params![id, message, now()],
        )?;
        Ok(())
    }

    pub fn put_passage(
        &mut self,
        generation: &str,
        passage: &Passage,
        memory: Option<(&str, Vec<f32>)>,
    ) -> Result<()> {
        let transaction = self.db.transaction()?;
        transaction.execute(
            "INSERT INTO chunks(repo_id,generation) VALUES (?,?)",
            params![passage.repo_id, generation],
        )?;
        let id = transaction.last_insert_rowid();
        let key = format!("chunk/{id}");
        // Source and memory reach disk before publishing their index references.
        self.passages.put(&key, &Some(passage))?;
        let memory = memory.map(|(model, vector)| SemanticMemory {
            chunk_id: id,
            model: model.to_owned(),
            vector,
        });
        if let Some(memory) = &memory {
            self.memories.put(&key, &Some(memory))?;
        } else if self
            .memories
            .get::<Option<SemanticMemory>>(&key)?
            .flatten()
            .is_some()
        {
            self.memories.forget::<SemanticMemory>(&key)?;
            self.vectors.remove(&id);
        }
        transaction.execute(
            "INSERT INTO chunk_terms(rowid,path,body) VALUES (?,?,?)",
            params![id, passage.path, passage.text],
        )?;
        transaction.commit()?;
        if let Some(memory) = memory {
            self.vectors.insert(id, memory);
        }
        Ok(())
    }

    pub fn passage(&mut self, id: i64) -> Result<Passage> {
        self.passages
            .get::<Option<Passage>>(&format!("chunk/{id}"))?
            .flatten()
            .context("Missing source passage")
    }

    pub fn publish(
        &self,
        id: &str,
        generation: &str,
        commit: &str,
        files: usize,
        chunks: usize,
        skipped: usize,
    ) -> Result<()> {
        self.db.execute(
            "UPDATE repositories SET generation=?2,commit_sha=?3,files=?4,chunks=?5,skipped=?6,status='ready',message=?7,updated_at=?8 WHERE id=?1 AND recall=1",
            params![id,generation,commit,files,chunks,skipped,if skipped>0 { "Indexed supported text; some files were skipped (see coverage)" } else { "Indexed committed text" },now()],
        )?;
        Ok(())
    }

    pub fn change_consent(&mut self, id: &str, public: bool, recall: bool) -> Result<()> {
        let repo = self.repository(id)?.context("Repository not found")?;
        self.db.execute(
            "UPDATE repositories SET public=?2,recall=?3,updated_at=?4 WHERE id=?1",
            params![id, public, recall, now()],
        )?;
        if !recall {
            let ids = {
                let mut statement = self.db.prepare("SELECT id FROM chunks WHERE repo_id=?")?;
                let rows = statement.query_map([id], |row| row.get::<_, i64>(0))?;
                rows.collect::<rusqlite::Result<Vec<_>>>()?
            };
            for chunk in ids {
                let key = format!("chunk/{chunk}");
                self.memories.forget::<SemanticMemory>(&key)?;
                self.passages.forget::<Passage>(&key)?;
                self.vectors.remove(&chunk);
                self.db
                    .execute("DELETE FROM chunk_terms WHERE rowid=?", [chunk])?;
                self.db.execute("DELETE FROM chunks WHERE id=?", [chunk])?;
            }
            self.db.execute("UPDATE repositories SET generation='',files=0,chunks=0,commit_sha='',status=CASE WHEN approved=1 THEN 'archived' ELSE status END,message='Recall disabled by owner' WHERE id=?", [id])?;
        } else if !repo.recall && repo.approved {
            self.db.execute("UPDATE repositories SET status='queued',message='Recall enabled by owner' WHERE id=?", [id])?;
        }
        Ok(())
    }

    pub fn save_feedback(&mut self, mut value: Value) -> Result<Value> {
        validate_feedback(&value)?;
        let category = value["category"].as_str().unwrap_or("tool_request");
        ensure!(
            ["bug", "nice_to_have", "critical", "tool_request"].contains(&category),
            "Invalid feedback category"
        );
        let title = value["title"]
            .as_str()
            .or_else(|| value["tool_name"].as_str())
            .context("Title is required")?;
        let description = value["description"]
            .as_str()
            .context("Description is required")?;
        ensure!(
            !title.trim().is_empty() && title.len() <= 200,
            "Title must be 1–200 bytes"
        );
        ensure!(
            !description.trim().is_empty() && description.len() <= 20000,
            "Description must be 1–20000 bytes"
        );
        for field in ["impact_score", "frequency_score"] {
            if let Some(score) = value.get(field) {
                ensure!(
                    score
                        .as_u64()
                        .is_some_and(|score| (1..=10).contains(&score)),
                    "Scores must be between 1 and 10"
                );
            }
        }
        let title = title.to_owned();
        let category = category.to_owned();
        let id = uuid::Uuid::new_v4().to_string();
        let created = now();
        value["received_at"] = json!(created);
        self.feedback
            .put(&format!("feedback/{id}"), &value.to_string())?;
        self.db.execute(
            "INSERT INTO feedback VALUES (?,?,?,?)",
            params![id, title, category, created],
        )?;
        Ok(
            json!({"feedback_id":id,"status":"received","message":"Saved to Smart Tree Hub for review."}),
        )
    }

    pub fn feedback_list(&mut self) -> Result<Vec<Value>> {
        let ids = {
            let mut statement = self
                .db
                .prepare("SELECT id FROM feedback ORDER BY created_at DESC LIMIT 100")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.into_iter()
            .map(|id| {
                let raw = self
                    .feedback
                    .get::<String>(&format!("feedback/{id}"))?
                    .context("Missing feedback record")?;
                Ok(json!({"id":id,"data":serde_json::from_str::<Value>(&raw)?}))
            })
            .collect()
    }

    pub fn stats(&self) -> Result<Value> {
        let (repos, ready, files, chunks, skipped): (usize,usize,usize,usize,usize) = self.db.query_row(
            "SELECT count(*),coalesce(sum(status='ready'),0),coalesce(sum(files),0),coalesce(sum(chunks),0),coalesce(sum(skipped),0)
             FROM repositories WHERE public=1 AND approved=1", [],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)),
        )?;
        let mut statement = self.db.prepare(
            "SELECT collection,count(*),coalesce(sum(files),0),coalesce(sum(chunks),0) FROM repositories WHERE public=1 AND approved=1 GROUP BY collection ORDER BY collection"
        )?;
        let collections = statement.query_map([], |row| Ok(json!({"name":row.get::<_,String>(0)?,"repositories":row.get::<_,usize>(1)?,"files":row.get::<_,usize>(2)?,"passages":row.get::<_,usize>(3)?})))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(
            json!({"repositories":repos,"ready_repositories":ready,"indexed_files":files,"indexed_passages":chunks,"skipped_files":skipped,"collections":collections}),
        )
    }
}

pub(super) fn validate_feedback(value: &Value) -> Result<()> {
    let category = value["category"].as_str().unwrap_or("tool_request");
    ensure!(
        ["bug", "nice_to_have", "critical", "tool_request"].contains(&category),
        "Invalid feedback category"
    );
    let title = value["title"]
        .as_str()
        .or_else(|| value["tool_name"].as_str())
        .context("Title is required")?;
    let description = value["description"]
        .as_str()
        .context("Description is required")?;
    ensure!(
        !title.trim().is_empty() && title.len() <= 200,
        "Title must be 1–200 bytes"
    );
    ensure!(
        !description.trim().is_empty() && description.len() <= 20000,
        "Description must be 1–20000 bytes"
    );
    for field in ["impact_score", "frequency_score"] {
        if let Some(score) = value.get(field) {
            ensure!(
                score
                    .as_u64()
                    .is_some_and(|score| (1..=10).contains(&score)),
                "Scores must be between 1 and 10"
            );
        }
    }
    Ok(())
}
