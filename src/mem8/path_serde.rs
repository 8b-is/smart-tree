//! Preserve native filesystem paths in binary storage; JSON uses display paths.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};

pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        return serializer.serialize_str(&path.to_string_lossy());
    }
    if let Some(text) = path.to_str() {
        return (0u8, text.as_bytes()).serialize(serializer);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        (1u8, path.as_os_str().as_bytes()).serialize(serializer)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let bytes: Vec<_> = path
            .as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect();
        (2u8, bytes).serialize(serializer)
    }
    #[cfg(not(any(unix, windows)))]
    Err(serde::ser::Error::custom(
        "Unsupported native path encoding",
    ))
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
    if deserializer.is_human_readable() {
        return String::deserialize(deserializer).map(PathBuf::from);
    }
    let (encoding, bytes): (u8, Vec<u8>) = Deserialize::deserialize(deserializer)?;
    if encoding == 0 {
        return String::from_utf8(bytes)
            .map(PathBuf::from)
            .map_err(serde::de::Error::custom);
    }
    #[cfg(unix)]
    if encoding == 1 {
        use std::os::unix::ffi::OsStringExt;
        return Ok(std::ffi::OsString::from_vec(bytes).into());
    }
    #[cfg(windows)]
    if encoding == 2 && bytes.len() % 2 == 0 {
        use std::os::windows::ffi::OsStringExt;
        let wide: Vec<_> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        return Ok(std::ffi::OsString::from_wide(&wide).into());
    }
    Err(serde::de::Error::custom(
        "Unsupported native path encoding on this platform",
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredPath(#[serde(with = "crate::mem8::path_serde")] pub PathBuf);

pub mod map {
    use super::*;
    use std::collections::HashMap;

    pub fn serialize<S: Serializer, V: Serialize>(
        map: &HashMap<PathBuf, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let entries: Vec<_> = map
            .iter()
            .map(|(path, value)| (StoredPath(path.clone()), value))
            .collect();
        entries.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
        deserializer: D,
    ) -> Result<HashMap<PathBuf, V>, D::Error> {
        let entries: Vec<(StoredPath, V)> = Deserialize::deserialize(deserializer)?;
        Ok(entries
            .into_iter()
            .map(|(path, value)| (path.0, value))
            .collect())
    }
}

pub mod option {
    use super::*;

    pub fn serialize<S: Serializer>(
        path: &Option<PathBuf>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        path.as_ref()
            .map(|path| StoredPath(path.clone()))
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<PathBuf>, D::Error> {
        Option::<StoredPath>::deserialize(deserializer).map(|path| path.map(|path| path.0))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::mem8::record_store::RecordStore;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn byte_paths_roundtrip_even_on_filesystems_that_reject_the_names() {
        let temp = tempfile::tempdir().unwrap();
        let first = PathBuf::from(std::ffi::OsString::from_vec(b"/document-\xff.md".to_vec()));
        let second = PathBuf::from(std::ffi::OsString::from_vec(b"/document-\xfe.md".to_vec()));
        let path = temp.path().join("paths.m8");
        let mut store = RecordStore::open_memory(&path).unwrap();
        store
            .put(
                "paths",
                &vec![StoredPath(first.clone()), StoredPath(second.clone())],
            )
            .unwrap();
        drop(store);
        let mut store = RecordStore::open_memory(&path).unwrap();
        let restored = store.get::<Vec<StoredPath>>("paths").unwrap().unwrap();
        assert_eq!(restored[0].0, first);
        assert_eq!(restored[1].0, second);
        assert_ne!(restored[0].0, restored[1].0);
    }
}
