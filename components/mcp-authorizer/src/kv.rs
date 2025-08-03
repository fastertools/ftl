use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::key_value::{Store, Error as KvError};

/// Wrapper around Spin's key-value store with TTL support
pub struct KvStore {
    store: Store,
}

/// Stored value with expiration metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredValue<T> {
    pub data: T,
    pub expires_at: DateTime<Utc>,
}

/// Key prefixes for different data types
pub mod keys {
    pub const JWKS_CACHE: &str = "jwks";
}

/// Default TTL values (in seconds)
pub mod ttl {
    pub const JWKS_CACHE: u64 = 300; // 5 minutes for JWKS
}

impl KvStore {
    /// Open the default key-value store
    pub fn open_default() -> Result<Self> {
        let store = Store::open_default()
            .context("Failed to open default key-value store")?;
        Ok(Self { store })
    }

    /// Set a value with TTL
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()> {
        let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        let stored = StoredValue {
            data: value,
            expires_at,
        };
        
        let json = serde_json::to_vec(&stored)?;
        self.store.set(key, &json)
            .map_err(|e| anyhow::anyhow!("Failed to store value: {:?}", e))?;
        Ok(())
    }

    /// Get a value if not expired
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.store.get(key) {
            Ok(Some(bytes)) => {
                let stored: StoredValue<T> = serde_json::from_slice(&bytes)?;
                if stored.expires_at > Utc::now() {
                    Ok(Some(stored.data))
                } else {
                    // Value expired, delete it
                    let _ = self.store.delete(key);
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(KvError::AccessDenied) => {
                Err(anyhow::anyhow!("Access denied to key-value store"))
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get value: {:?}", e)),
        }
    }
}

/// Helper to generate cache keys
pub fn cache_key(prefix: &str, key: &str) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(&hash[..8]); // Use first 8 bytes for shorter keys
    
    format!("{}:{}", prefix, hash_hex)
}