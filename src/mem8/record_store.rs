//! Durable native MEM8 memory and token record storage for the daemon.
//!
//! Adapts Aye's `mem8-storage/src/wal.rs` append/flush/replay design to variable
//! sized records, with compression, checksums and atomic consolidation.
//! Memory uses Aye's MEM8/RAW8 v1.1.1 blocks. Exact application data uses
//! dictionary tokens and compressed T8R frames, also available independently
//! for data that does not represent a memory wave.

use super::{native_block, token_codec};
use crate::mem8_lite::Wave;
use anyhow::{bail, ensure, Context, Result};
use bincode::Options;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"T8R\0\x01\0\0\0";
const NATIVE_APPLICATION: &[u8; 4] = b"ST8\x01";
const FRAME_HEADER: usize = 16;
const MAX_RECORD: u64 = 64 * 1024 * 1024;
const COMPACT_AFTER: u64 = 4 * 1024 * 1024;

/// Shared location for daemon conversation and directory memory.
pub fn memory_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("ST_MEMORY_DIR") {
        ensure!(!path.is_empty(), "ST_MEMORY_DIR must not be empty");
        return Ok(PathBuf::from(path));
    }
    Ok(dirs::home_dir()
        .context("Cannot locate the home directory for MEM8 storage")?
        .join(".st"))
}

#[derive(Serialize, Deserialize)]
struct Record {
    key: String,
    value: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Location {
    offset: u64,
    length: u64,
}

pub struct RecordStore {
    path: PathBuf,
    file: File,
    // Separate inode keeps the writer lock valid during atomic replacement.
    _lock: File,
    index: HashMap<String, Location>,
    length: u64,
    live_bytes: u64,
    poisoned: bool,
    native: bool,
}

impl RecordStore {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_format(path, false)
    }

    /// Native MEM8 wave blocks paired with tokenized, exact RAW8 source data.
    pub fn open_memory(path: &Path) -> Result<Self> {
        Self::open_format(path, true)
    }

    fn open_format(path: &Path, native: bool) -> Result<Self> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent)?;
        let lock = open_private(&path.with_extension("lock"), false)?;
        lock.try_lock()
            .context("MEM8 record store is already in use")?;
        let mut file = open_private(path, false)?;
        if file.metadata()?.len() == 0 {
            if !native {
                file.write_all(MAGIC)?;
            }
            file.sync_all()?;
            sync_parent(path)?;
        }
        if !native {
            file.seek(SeekFrom::Start(0))?;
            let mut magic = [0u8; 8];
            file.read_exact(&mut magic)
                .context("Incomplete token record header")?;
            ensure!(
                &magic == MAGIC,
                "Unsupported token record format or version"
            );
        } else if file.metadata()?.len() >= 12 {
            let mut magic = [0; 4];
            file.seek(SeekFrom::Start(8))?;
            file.read_exact(&mut magic)?;
            ensure!(
                magic == 0x4d454d38u32.to_le_bytes(),
                "Unsupported native MEM8 format; existing file was preserved"
            );
        }
        let mut store = Self {
            path: path.to_path_buf(),
            file,
            _lock: lock,
            index: HashMap::new(),
            length: if native { 0 } else { 8 },
            live_bytes: 0,
            poisoned: false,
            native,
        };
        store.replay()?;
        Ok(store)
    }

    pub fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>> {
        ensure!(
            !self.poisoned,
            "MEM8 store needs to be reopened after a failed write"
        );
        let Some(location) = self.index.get(key).copied() else {
            return Ok(None);
        };
        let record = self.read_record(location.offset)?;
        Ok(Some(
            codec()
                .deserialize(&record.value)
                .context("Invalid stored record payload")?,
        ))
    }

    /// Enumerate a namespace without decompressing unrelated records.
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.index
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Acknowledged writes have reached disk before the in-memory index changes.
    pub fn put<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        self.put_with_wave(key, value, &Wave::default())
    }

    pub fn put_with_wave<T: Serialize>(&mut self, key: &str, value: &T, wave: &Wave) -> Result<()> {
        self.put_record(key, value, wave, false)
    }

    /// Persist a logical deletion and mark its native wave as tombstoned.
    pub fn forget<T: Serialize>(&mut self, key: &str) -> Result<()> {
        self.put_record(key, &None::<T>, &Wave::default(), true)
    }

    fn put_record<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        wave: &Wave,
        tombstoned: bool,
    ) -> Result<()> {
        ensure!(
            !self.poisoned,
            "MEM8 store needs to be reopened after a failed write"
        );
        let record = Record {
            key: key.to_string(),
            value: codec().serialize(value)?,
        };
        let raw = codec().serialize(&record)?;
        ensure!(
            raw.len() as u64 <= MAX_RECORD,
            "MEM8 record exceeds size limit"
        );
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&token_codec::encode(&raw))?;
        let compressed = encoder.finish()?;
        ensure!(
            compressed.len() as u64 <= MAX_RECORD,
            "Compressed MEM8 record exceeds size limit"
        );
        let mut header = [0u8; FRAME_HEADER];
        header[..4].copy_from_slice(&(compressed.len() as u32).to_le_bytes());
        header[4..8].copy_from_slice(&(raw.len() as u32).to_le_bytes());
        header[8..12].copy_from_slice(&crc32fast::hash(&compressed).to_le_bytes());
        let header_crc = crc32fast::hash(&header[..12]);
        header[12..].copy_from_slice(&header_crc.to_le_bytes());
        let mut frame = Vec::new();
        if self.native {
            frame.extend_from_slice(NATIVE_APPLICATION);
        }
        frame.extend_from_slice(&header);
        frame.extend_from_slice(&compressed);
        let frame = if self.native {
            native_block::encode(&frame, key_location(key), wave, tombstoned)?
        } else {
            frame
        };
        let result = (|| -> std::io::Result<()> {
            self.file.seek(SeekFrom::Start(self.length))?;
            self.file.write_all(&frame)?;
            self.file.sync_data()
        })();
        if let Err(error) = result {
            self.poisoned = true;
            // Keep the previous committed prefix if rollback is possible.
            let _ = self.file.set_len(self.length);
            return Err(error).context("Failed to persist MEM8 record");
        }
        let location = Location {
            offset: self.length,
            length: frame.len() as u64,
        };
        if let Some(old) = self.index.insert(key.to_string(), location) {
            self.live_bytes -= old.length;
        }
        self.live_bytes += location.length;
        self.length += location.length;
        if self.length > COMPACT_AFTER
            && self.length.saturating_sub(self.header_size()) > self.live_bytes.saturating_mul(2)
        {
            // The append is already committed; a failed consolidation can be retried.
            if let Err(error) = self.compact() {
                tracing::warn!(%error, "MEM8 consolidation failed; journal remains committed");
            }
        }
        Ok(())
    }

    pub fn get_wave(&mut self, key: &str) -> Result<Option<Wave>> {
        ensure!(self.native, "Wave reads require native MEM8 storage");
        ensure!(
            !self.poisoned,
            "MEM8 store needs to be reopened after a failed write"
        );
        let Some(location) = self.index.get(key) else {
            return Ok(None);
        };
        self.file.seek(SeekFrom::Start(location.offset))?;
        let mut pair = [0; native_block::PAIR_SIZE];
        self.file.read_exact(&mut pair)?;
        Ok(Some(native_block::decode_pair(&pair)?.1))
    }

    fn header_size(&self) -> u64 {
        if self.native {
            0
        } else {
            8
        }
    }

    #[cfg(test)]
    pub(crate) fn make_read_only_for_test(&mut self) -> Result<()> {
        self.file = File::open(&self.path)?;
        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        ensure!(
            !self.poisoned,
            "MEM8 store needs to be reopened after a failed write"
        );
        let temporary = self
            .path
            .with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
        let result = self.compact_to(&temporary);
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }

    fn compact_to(&mut self, temporary: &Path) -> Result<()> {
        let mut output = open_private(temporary, true)?;
        if !self.native {
            output.write_all(MAGIC)?;
        }
        let mut index = HashMap::new();
        let mut offset = self.header_size();
        let mut entries: Vec<_> = self.index.iter().collect();
        entries.sort_by_key(|(key, _)| *key);
        for (key, location) in entries {
            self.file.seek(SeekFrom::Start(location.offset))?;
            let copied = std::io::copy(&mut (&mut self.file).take(location.length), &mut output)?;
            ensure!(
                copied == location.length,
                "Truncated MEM8 record during consolidation"
            );
            index.insert(
                key.clone(),
                Location {
                    offset,
                    length: copied,
                },
            );
            offset += copied;
        }
        output.sync_all()?;
        std::fs::rename(temporary, &self.path).context("Failed to replace MEM8 checkpoint")?;
        self.file = output;
        self.index = index;
        self.length = offset;
        self.live_bytes = offset - self.header_size();
        sync_parent(&self.path)?;
        Ok(())
    }

    fn replay(&mut self) -> Result<()> {
        let file_length = self.file.metadata()?.len();
        while self.length < file_length {
            let minimum = if self.native {
                native_block::PAIR_SIZE
            } else {
                FRAME_HEADER
            } as u64;
            if file_length - self.length < minimum {
                break; // Interrupted final header.
            }
            self.file.seek(SeekFrom::Start(self.length))?;
            let length = if self.native {
                let mut pair = [0; native_block::PAIR_SIZE];
                self.file.read_exact(&mut pair)?;
                let (_, _, payload) = native_block::decode_pair(&pair)?;
                ensure!(
                    payload.starts_with(NATIVE_APPLICATION),
                    "Unsupported MEM8 application payload"
                );
                let (compressed, _) = read_header(&mut &payload[4..])?;
                let bytes = 4 + FRAME_HEADER + compressed as usize;
                (bytes.div_ceil(native_block::PAYLOAD_SIZE) * native_block::PAIR_SIZE) as u64
            } else {
                let (compressed, _) = read_header(&mut self.file)?;
                FRAME_HEADER as u64 + u64::from(compressed)
            };
            if length > file_length - self.length {
                break; // Interrupted final payload.
            }
            let record = self.read_record(self.length)?;
            let location = Location {
                offset: self.length,
                length,
            };
            if let Some(old) = self.index.insert(record.key, location) {
                self.live_bytes -= old.length;
            }
            self.live_bytes += length;
            self.length += length;
        }
        if self.length != file_length {
            self.file.set_len(self.length)?;
            self.file.sync_all()?;
            tracing::warn!(
                discarded_bytes = file_length - self.length,
                "Recovered incomplete MEM8 journal tail"
            );
        }
        Ok(())
    }

    fn read_record(&mut self, offset: u64) -> Result<Record> {
        self.file.seek(SeekFrom::Start(offset))?;
        let (frame, location) = if self.native {
            let mut pair = [0; native_block::PAIR_SIZE];
            self.file.read_exact(&mut pair)?;
            let (location, _, payload) = native_block::decode_pair(&pair)?;
            ensure!(
                payload.starts_with(NATIVE_APPLICATION),
                "Unsupported MEM8 application payload"
            );
            let (compressed, _) = read_header(&mut &payload[4..])?;
            let expected = 4 + FRAME_HEADER + compressed as usize;
            let mut frame = payload.to_vec();
            while frame.len() < expected {
                ensure!(
                    frame.len() % native_block::PAYLOAD_SIZE == 0,
                    "Short MEM8 continuation block"
                );
                self.file.read_exact(&mut pair)?;
                let (next_location, _, payload) = native_block::decode_pair(&pair)?;
                ensure!(location == next_location, "Mismatched MEM8 continuation");
                frame.extend_from_slice(payload);
            }
            ensure!(frame.len() == expected, "Invalid MEM8 application length");
            (frame[4..].to_vec(), Some(location))
        } else {
            let mut header = [0; FRAME_HEADER];
            self.file.read_exact(&mut header)?;
            let (compressed, _) = read_header(&mut header.as_slice())?;
            let mut frame = header.to_vec();
            frame.resize(FRAME_HEADER + compressed as usize, 0);
            self.file.read_exact(&mut frame[FRAME_HEADER..])?;
            (frame, None)
        };
        let (_, raw_length) = read_header(&mut frame.as_slice())?;
        let checksum = u32::from_le_bytes(frame[8..12].try_into()?);
        let compressed = &frame[FRAME_HEADER..];
        ensure!(
            crc32fast::hash(compressed) == checksum,
            "MEM8 record checksum mismatch at byte {offset}"
        );
        let mut decoder = ZlibDecoder::new(compressed);
        let mut tokens = Vec::new();
        (&mut decoder)
            .take(u64::from(raw_length) * 3 + 1)
            .read_to_end(&mut tokens)?;
        ensure!(
            tokens.len() <= raw_length as usize * 3
                && decoder.total_in() == compressed.len() as u64,
            "Invalid MEM8 record length"
        );
        let raw = token_codec::decode(&tokens, raw_length as usize)?;
        let record: Record = codec().deserialize(&raw).context("Invalid stored record")?;
        if let Some(location) = location {
            ensure!(
                key_location(&record.key) == location,
                "MEM8 record location mismatch"
            );
        }
        Ok(record)
    }
}

fn codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .with_limit(MAX_RECORD)
        .reject_trailing_bytes()
}

fn read_header(file: &mut impl Read) -> Result<(u32, u32)> {
    let mut header = [0u8; FRAME_HEADER];
    file.read_exact(&mut header)?;
    ensure!(
        crc32fast::hash(&header[..12]) == u32::from_le_bytes(header[12..16].try_into()?),
        "MEM8 frame header checksum mismatch"
    );
    let compressed = u32::from_le_bytes(header[..4].try_into()?);
    let raw = u32::from_le_bytes(header[4..8].try_into()?);
    if compressed == 0
        || raw == 0
        || u64::from(compressed) > MAX_RECORD
        || u64::from(raw) > MAX_RECORD
    {
        bail!("Invalid MEM8 frame size");
    }
    Ok((compressed, raw))
}

fn key_location(key: &str) -> u64 {
    let hash = Sha256::digest(key.as_bytes());
    let mut prefix = [0; 8];
    prefix.copy_from_slice(&hash[..8]);
    u64::from_le_bytes(prefix)
}

fn open_private(path: &Path, exclusive: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if exclusive {
        options.create_new(true);
    } else {
        options.create(true).truncate(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .with_context(|| format!("Cannot open MEM8 storage {}", path.display()))?;
    ensure!(
        file.metadata()?.is_file(),
        "MEM8 storage requires a regular file"
    );
    Ok(file)
}

fn sync_parent(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?
    .sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_memory_roundtrip_compaction_and_partial_append_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("native.m8");
        let mut store = RecordStore::open_memory(&path).unwrap();
        store.put("committed", &"original".to_string()).unwrap();
        let committed = store.length as usize;
        let mut random = 0x12345678u32;
        let content: Vec<u8> = (0..20000)
            .map(|_| {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                random as u8
            })
            .collect();
        let wave = Wave::new(440.0, -0.25, 0.9);
        store.put_with_wave("large", &content, &wave).unwrap();
        assert_eq!(
            store.get::<Vec<u8>>("large").unwrap(),
            Some(content.clone())
        );
        drop(store);
        let bytes = std::fs::read(&path).unwrap();
        for length in [
            committed,
            committed + 1,
            committed + 4095,
            committed + 4096,
            committed + 8191,
            committed + 8192,
            bytes.len() - 1,
        ] {
            let recovery = temp.path().join("recovery.m8");
            std::fs::write(&recovery, &bytes[..length]).unwrap();
            let mut store = RecordStore::open_memory(&recovery).unwrap();
            assert_eq!(
                store.get::<String>("committed").unwrap().as_deref(),
                Some("original")
            );
            assert!(store.get::<Vec<u8>>("large").unwrap().is_none());
            assert_eq!(std::fs::metadata(recovery).unwrap().len(), committed as u64);
        }
        let mut store = RecordStore::open_memory(&path).unwrap();
        let restored = store.get_wave("large").unwrap().unwrap();
        assert!((restored.arousal - wave.arousal).abs() < 0.0001);
        store.put("committed", &"new".to_string()).unwrap();
        store.compact().unwrap();
        assert!(RecordStore::open_memory(&path).is_err());
        drop(store);
        let mut store = RecordStore::open_memory(&path).unwrap();
        assert_eq!(store.get::<Vec<u8>>("large").unwrap(), Some(content));
        assert_eq!(
            store.get::<String>("committed").unwrap().as_deref(),
            Some("new")
        );
    }

    #[test]
    fn append_reopen_and_compact_preserve_latest_values() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("records.m8");
        let mut store = RecordStore::open(&path).unwrap();
        store.put("first", &"original".to_string()).unwrap();
        store.put("second", &vec![42u32, 7]).unwrap();
        store.put("first", &"replacement".to_string()).unwrap();
        assert_eq!(
            store.get::<String>("first").unwrap().as_deref(),
            Some("replacement")
        );
        let original_size = std::fs::metadata(&path).unwrap().len();
        store.compact().unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() < original_size);
        // Consolidation must not release the process-wide writer lock.
        assert!(RecordStore::open(&path).is_err());
        drop(store);
        let mut store = RecordStore::open(&path).unwrap();
        assert_eq!(
            store.get::<String>("first").unwrap().as_deref(),
            Some("replacement")
        );
        assert_eq!(store.get::<Vec<u32>>("second").unwrap(), Some(vec![42, 7]));
        assert!(store.get::<String>("missing").unwrap().is_none());
        store.put("third", &3u64).unwrap();
    }

    #[test]
    fn every_truncated_tail_recovers_the_committed_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source.m8");
        let mut store = RecordStore::open(&path).unwrap();
        store.put("first", &1u64).unwrap();
        let committed = store.length as usize;
        store.put("last", &2u64).unwrap();
        drop(store);
        let bytes = std::fs::read(&path).unwrap();
        let recovery = temp.path().join("recovery.m8");
        for length in committed..bytes.len() {
            std::fs::write(&recovery, &bytes[..length]).unwrap();
            let mut store = RecordStore::open(&recovery).unwrap();
            assert_eq!(store.get::<u64>("first").unwrap(), Some(1));
            assert_eq!(store.get::<u64>("last").unwrap(), None);
            assert_eq!(
                std::fs::metadata(&recovery).unwrap().len(),
                committed as u64
            );
            store.put("after_recovery", &3u64).unwrap();
        }
    }

    #[test]
    fn rejects_corruption_without_overwriting_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("records.m8");
        let mut store = RecordStore::open(&path).unwrap();
        store.put("first", &1u64).unwrap();
        drop(store);
        let original = std::fs::read(&path).unwrap();
        for offset in [0, 8, 20, 24] {
            let mut corrupted = original.clone();
            corrupted[offset] ^= 0xff;
            std::fs::write(&path, &corrupted).unwrap();
            assert!(RecordStore::open(&path).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), corrupted);
        }
    }

    #[test]
    fn compresses_repeated_scan_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("records.m8");
        let content = "certificate subject issuer fingerprint path scan result ".repeat(2000);
        let mut store = RecordStore::open(&path).unwrap();
        store.put("scan", &content).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() < content.len() as u64 / 4);
        assert_eq!(store.get::<String>("scan").unwrap(), Some(content));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
