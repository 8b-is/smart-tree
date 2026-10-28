//! Encrypted token storage for Google OAuth2 credentials
//!
//! Tokens are encrypted at rest with AES-256-GCM using a machine-local key.
//! Key material is stored in ~/.st/google/.keymat with strict 0o600 permissions.
//!
//! "Your data is safe with me. I live here with you." - Liquid

use anyhow::{Context, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use std::fs;
use std::path::PathBuf;

const KEY_SIZE: usize = 32; // AES-256

/// Encrypted file-based token store
pub struct TokenStore {
    store_dir: PathBuf,
    key: LessSafeKey,
}

impl TokenStore {
    /// Create or load a token store.
    /// Key material is generated on first use and stored in `store_dir/.keymat`.
    pub fn new(store_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&store_dir)
            .with_context(|| format!("Failed to create store dir: {}", store_dir.display()))?;

        let keymat_path = store_dir.join(".keymat");
        let key_bytes = if keymat_path.exists() {
            fs::read(&keymat_path).context("Failed to read key material")?
        } else {
            let rng = SystemRandom::new();
            let mut key_bytes = vec![0u8; KEY_SIZE];
            rng.fill(&mut key_bytes)
                .map_err(|_| anyhow::anyhow!("Failed to generate random key"))?;

            fs::write(&keymat_path, &key_bytes).context("Failed to write key material")?;

            // Set strict permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&keymat_path, fs::Permissions::from_mode(0o600))
                    .context("Failed to set key file permissions")?;
            }

            key_bytes
        };

        if key_bytes.len() != KEY_SIZE {
            anyhow::bail!(
                "Invalid key material size: expected {}, got {}",
                KEY_SIZE,
                key_bytes.len()
            );
        }

        let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
        let key = LessSafeKey::new(unbound);

        Ok(Self { store_dir, key })
    }

    /// Encrypt and save token data
    pub fn save(&self, name: &str, data: &[u8]) -> Result<()> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        // Format: nonce || ciphertext+tag
        let mut output = Vec::with_capacity(NONCE_LEN + in_out.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&in_out);

        let file_path = self.store_dir.join(format!("{name}.enc"));
        fs::write(&file_path, &output)
            .with_context(|| format!("Failed to write encrypted file: {}", file_path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// Load and decrypt token data
    pub fn load(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let file_path = self.store_dir.join(format!("{name}.enc"));
        if !file_path.exists() {
            return Ok(None);
        }

        let raw = fs::read(&file_path)
            .with_context(|| format!("Failed to read encrypted file: {}", file_path.display()))?;

        if raw.len() < NONCE_LEN {
            anyhow::bail!("Encrypted file too short");
        }

        let (nonce_bytes, ciphertext) = raw.split_at(NONCE_LEN);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid nonce"))?,
        );

        let mut in_out = ciphertext.to_vec();
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Decryption failed - token file may be corrupted"))?;

        Ok(Some(plaintext.to_vec()))
    }

    /// Remove stored tokens
    pub fn clear(&self, name: &str) -> Result<()> {
        let file_path = self.store_dir.join(format!("{name}.enc"));
        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("Failed to remove: {}", file_path.display()))?;
        }
        Ok(())
    }

    /// Check if tokens exist for the given name
    pub fn exists(&self, name: &str) -> bool {
        self.store_dir.join(format!("{name}.enc")).exists()
    }

    /// Save auth config (unencrypted, just metadata)
    pub fn save_config(&self, config: &super::models::AuthConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        let config_path = self.store_dir.join("config.json");
        fs::write(&config_path, json)?;
        Ok(())
    }

    /// Load auth config
    pub fn load_config(&self) -> Result<Option<super::models::AuthConfig>> {
        let config_path = self.store_dir.join("config.json");
        if !config_path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&config_path)?;
        Ok(Some(serde_json::from_str(&json)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::new(dir.path().to_path_buf()).unwrap();

        let data = b"my_secret_oauth_token_12345";
        store.save("test_tokens", data).unwrap();

        let loaded = store.load("test_tokens").unwrap().unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_clear_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::new(dir.path().to_path_buf()).unwrap();

        store.save("to_delete", b"secret").unwrap();
        assert!(store.exists("to_delete"));

        store.clear("to_delete").unwrap();
        assert!(!store.exists("to_delete"));
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::new(dir.path().to_path_buf()).unwrap();

        assert!(store.load("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_key_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // Create store and save
        let store1 = TokenStore::new(path.clone()).unwrap();
        store1.save("persist_test", b"hello_world").unwrap();

        // Create new store instance (should load same key)
        let store2 = TokenStore::new(path).unwrap();
        let loaded = store2.load("persist_test").unwrap().unwrap();
        assert_eq!(loaded, b"hello_world");
    }
}
