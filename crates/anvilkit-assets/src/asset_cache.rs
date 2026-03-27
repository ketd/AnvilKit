//! # 资产缓存
//!
//! 基于内容哈希的编译资产缓存，使用 LRU 淘汰策略。
//!
//! ## 设计
//!
//! - `AssetCache`: 内容寻址缓存，将源数据的哈希映射到编译后的产物
//! - `AssetCacheConfig`: 缓存配置（最大容量、缓存目录、启用开关）
//! - 使用 `DefaultHasher` (SipHash) 进行内容哈希
//! - LRU 淘汰：当缓存满时，优先淘汰最久未使用的条目
//!
//! ## 示例
//!
//! ```rust
//! use anvilkit_assets::asset_cache::{AssetCache, AssetCacheConfig};
//! use std::path::PathBuf;
//!
//! let config = AssetCacheConfig {
//!     max_size_bytes: 1024 * 1024, // 1 MB
//!     cache_dir: PathBuf::from(".cache/assets"),
//!     enabled: true,
//! };
//! let mut cache = AssetCache::new(config);
//!
//! let data = b"hello world";
//! let hash = AssetCache::content_hash(data);
//! cache.put(hash, "test.txt".as_ref(), b"compiled output").unwrap();
//! assert_eq!(cache.get(hash), Some(b"compiled output".to_vec()));
//! ```

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Configuration for the asset cache.
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::asset_cache::AssetCacheConfig;
/// use std::path::PathBuf;
///
/// let config = AssetCacheConfig::default();
/// assert_eq!(config.max_size_bytes, 512 * 1024 * 1024);
/// assert!(config.enabled);
/// ```
pub struct AssetCacheConfig {
    /// Maximum cache size in bytes. Default: 512 MB.
    pub max_size_bytes: usize,
    /// Directory for cache files (reserved for future disk persistence). Default: ".cache/assets".
    pub cache_dir: PathBuf,
    /// Whether the cache is enabled. Default: true.
    pub enabled: bool,
}

impl Default for AssetCacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 512 * 1024 * 1024, // 512 MB
            cache_dir: PathBuf::from(".cache/assets"),
            enabled: true,
        }
    }
}

/// Content-addressed cache for compiled/processed assets.
///
/// Uses `DefaultHasher` (SipHash) for content hashing and LRU eviction when full.
/// The cache is purely in-memory; the `cache_dir` and file paths in `CacheEntry`
/// are stored for future disk persistence but no I/O is performed.
///
/// # 示例
///
/// ```rust
/// use anvilkit_assets::asset_cache::{AssetCache, AssetCacheConfig};
/// use std::path::PathBuf;
///
/// let mut cache = AssetCache::new(AssetCacheConfig::default());
/// assert!(cache.is_empty());
///
/// let data = b"source content";
/// let hash = AssetCache::content_hash(data);
/// let compiled = b"compiled artifact";
/// cache.put(hash, "model.glb".as_ref(), compiled).unwrap();
///
/// assert_eq!(cache.len(), 1);
/// assert_eq!(cache.size(), compiled.len());
/// assert_eq!(cache.get(hash), Some(compiled.to_vec()));
/// ```
pub struct AssetCache {
    config: AssetCacheConfig,
    /// Maps content hash → CacheEntry (in-memory index).
    entries: HashMap<u64, CacheEntry>,
    /// In-memory store of compiled data, keyed by content hash.
    data_store: HashMap<u64, Vec<u8>>,
    /// LRU order: most recently used at back, least recently used at front.
    lru_order: Vec<u64>,
    /// Current total size in bytes.
    current_size: usize,
}

/// A single cached entry's metadata.
#[allow(dead_code)] // Fields reserved for future disk persistence / debugging.
struct CacheEntry {
    /// Content hash of the source data.
    hash: u64,
    /// Size of the compiled artifact in bytes.
    size: usize,
    /// Logical path within cache_dir (for future disk persistence).
    path: PathBuf,
    /// Original asset path (for debugging).
    source_path: PathBuf,
}

impl AssetCache {
    /// Create a new asset cache with the given configuration.
    pub fn new(config: AssetCacheConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
            data_store: HashMap::new(),
            lru_order: Vec::new(),
            current_size: 0,
        }
    }

    /// Compute a content hash for the given bytes.
    ///
    /// Uses `DefaultHasher` (SipHash), a fast non-cryptographic hash
    /// from the standard library.
    ///
    /// # 示例
    ///
    /// ```rust
    /// use anvilkit_assets::asset_cache::AssetCache;
    ///
    /// let hash1 = AssetCache::content_hash(b"hello");
    /// let hash2 = AssetCache::content_hash(b"hello");
    /// let hash3 = AssetCache::content_hash(b"world");
    /// assert_eq!(hash1, hash2);
    /// assert_ne!(hash1, hash3);
    /// ```
    pub fn content_hash(data: &[u8]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Look up a cached compiled artifact by its source content hash.
    ///
    /// Returns `None` if not cached or if the cache is disabled.
    /// Updates LRU order on hit.
    pub fn get(&mut self, content_hash: u64) -> Option<Vec<u8>> {
        if !self.config.enabled {
            return None;
        }

        if self.entries.contains_key(&content_hash) {
            self.touch_lru(content_hash);
            self.data_store.get(&content_hash).cloned()
        } else {
            None
        }
    }

    /// Store a compiled artifact in the cache.
    ///
    /// Evicts LRU entries if the cache would exceed `max_size_bytes`.
    /// Returns an error if the single artifact is larger than the max cache size.
    pub fn put(
        &mut self,
        content_hash: u64,
        source_path: &Path,
        compiled_data: &[u8],
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let data_size = compiled_data.len();

        // Single artifact exceeds entire cache capacity
        if data_size > self.config.max_size_bytes {
            return Err(format!(
                "artifact size ({} bytes) exceeds max cache size ({} bytes)",
                data_size, self.config.max_size_bytes,
            ));
        }

        // If this hash already exists, remove the old entry first
        if self.entries.contains_key(&content_hash) {
            self.remove(content_hash);
        }

        // Evict until we have room
        self.evict(data_size);

        // Build cache-relative path from the hash
        let cache_path = self.config.cache_dir.join(format!("{:016x}.bin", content_hash));

        let entry = CacheEntry {
            hash: content_hash,
            size: data_size,
            path: cache_path,
            source_path: source_path.to_path_buf(),
        };

        self.entries.insert(content_hash, entry);
        self.data_store.insert(content_hash, compiled_data.to_vec());
        self.lru_order.push(content_hash);
        self.current_size += data_size;

        Ok(())
    }

    /// Remove a specific entry from the cache.
    ///
    /// Returns `true` if the entry was present and removed.
    pub fn remove(&mut self, content_hash: u64) -> bool {
        if let Some(entry) = self.entries.remove(&content_hash) {
            self.current_size -= entry.size;
            self.data_store.remove(&content_hash);
            self.lru_order.retain(|&h| h != content_hash);
            true
        } else {
            false
        }
    }

    /// Clear the entire cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.data_store.clear();
        self.lru_order.clear();
        self.current_size = 0;
    }

    /// Current cache size in bytes.
    pub fn size(&self) -> usize {
        self.current_size
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns `true` if the cache contains an entry for the given content hash.
    pub fn contains(&self, content_hash: u64) -> bool {
        self.entries.contains_key(&content_hash)
    }

    /// Returns a reference to the cache configuration.
    pub fn config(&self) -> &AssetCacheConfig {
        &self.config
    }

    /// Evict least recently used entries until `current_size + needed_bytes <= max_size_bytes`.
    fn evict(&mut self, needed_bytes: usize) {
        while self.current_size + needed_bytes > self.config.max_size_bytes {
            if let Some(oldest_hash) = self.lru_order.first().copied() {
                self.remove(oldest_hash);
            } else {
                // Nothing left to evict
                break;
            }
        }
    }

    /// Move the given hash to the back of the LRU list (most recently used).
    fn touch_lru(&mut self, content_hash: u64) {
        self.lru_order.retain(|&h| h != content_hash);
        self.lru_order.push(content_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(max_size: usize) -> AssetCacheConfig {
        AssetCacheConfig {
            max_size_bytes: max_size,
            cache_dir: PathBuf::from(".cache/test"),
            enabled: true,
        }
    }

    #[test]
    fn test_default_config() {
        let config = AssetCacheConfig::default();
        assert_eq!(config.max_size_bytes, 512 * 1024 * 1024);
        assert_eq!(config.cache_dir, PathBuf::from(".cache/assets"));
        assert!(config.enabled);
    }

    #[test]
    fn test_new_cache_is_empty() {
        let cache = AssetCache::new(test_config(1024));
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_content_hash_deterministic() {
        let data = b"hello world";
        let h1 = AssetCache::content_hash(data);
        let h2 = AssetCache::content_hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_different_data() {
        let h1 = AssetCache::content_hash(b"hello");
        let h2 = AssetCache::content_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_empty() {
        let h1 = AssetCache::content_hash(b"");
        let h2 = AssetCache::content_hash(b"");
        assert_eq!(h1, h2);
        // Empty data still produces a hash
        let h3 = AssetCache::content_hash(b"non-empty");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = AssetCache::new(test_config(4096));
        let source = b"source content";
        let compiled = b"compiled output";
        let hash = AssetCache::content_hash(source);

        cache.put(hash, Path::new("test.glb"), compiled).unwrap();

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.size(), compiled.len());
        assert!(cache.contains(hash));

        let result = cache.get(hash);
        assert_eq!(result, Some(compiled.to_vec()));
    }

    #[test]
    fn test_get_miss() {
        let mut cache = AssetCache::new(test_config(4096));
        let hash = AssetCache::content_hash(b"nonexistent");
        assert_eq!(cache.get(hash), None);
    }

    #[test]
    fn test_put_overwrites_existing() {
        let mut cache = AssetCache::new(test_config(4096));
        let hash = AssetCache::content_hash(b"source");

        cache.put(hash, Path::new("a.glb"), b"first").unwrap();
        assert_eq!(cache.size(), 5); // "first".len()

        cache.put(hash, Path::new("a.glb"), b"second version").unwrap();
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.size(), 14); // "second version".len()
        assert_eq!(cache.get(hash), Some(b"second version".to_vec()));
    }

    #[test]
    fn test_remove() {
        let mut cache = AssetCache::new(test_config(4096));
        let hash = AssetCache::content_hash(b"src");

        cache.put(hash, Path::new("x.bin"), b"data").unwrap();
        assert_eq!(cache.len(), 1);

        assert!(cache.remove(hash));
        assert!(cache.is_empty());
        assert_eq!(cache.size(), 0);
        assert!(!cache.contains(hash));

        // Removing again returns false
        assert!(!cache.remove(hash));
    }

    #[test]
    fn test_clear() {
        let mut cache = AssetCache::new(test_config(4096));
        for i in 0..5u8 {
            let hash = AssetCache::content_hash(&[i]);
            cache.put(hash, Path::new("x.bin"), &[i; 100]).unwrap();
        }
        assert_eq!(cache.len(), 5);
        assert_eq!(cache.size(), 500);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_lru_eviction_basic() {
        // Cache can hold 200 bytes; each entry is 100 bytes
        let mut cache = AssetCache::new(test_config(200));

        let h1 = AssetCache::content_hash(b"a");
        let h2 = AssetCache::content_hash(b"b");
        let h3 = AssetCache::content_hash(b"c");

        cache.put(h1, Path::new("a"), &[0u8; 100]).unwrap();
        cache.put(h2, Path::new("b"), &[1u8; 100]).unwrap();

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.size(), 200);

        // Adding a third should evict the first (LRU)
        cache.put(h3, Path::new("c"), &[2u8; 100]).unwrap();

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.size(), 200);
        assert!(!cache.contains(h1)); // evicted
        assert!(cache.contains(h2));
        assert!(cache.contains(h3));
    }

    #[test]
    fn test_lru_touch_on_get() {
        // Cache can hold 200 bytes; each entry is 100 bytes
        let mut cache = AssetCache::new(test_config(200));

        let h1 = AssetCache::content_hash(b"a");
        let h2 = AssetCache::content_hash(b"b");
        let h3 = AssetCache::content_hash(b"c");

        cache.put(h1, Path::new("a"), &[0u8; 100]).unwrap();
        cache.put(h2, Path::new("b"), &[1u8; 100]).unwrap();

        // Touch h1 so it becomes most recently used
        let _ = cache.get(h1);

        // Adding h3 should evict h2 (now the LRU), not h1
        cache.put(h3, Path::new("c"), &[2u8; 100]).unwrap();

        assert!(cache.contains(h1));  // was touched, kept
        assert!(!cache.contains(h2)); // evicted
        assert!(cache.contains(h3));  // just added
    }

    #[test]
    fn test_eviction_multiple_entries() {
        // Cache holds 300 bytes; small entries are 100 bytes each
        let mut cache = AssetCache::new(test_config(300));

        let h1 = AssetCache::content_hash(b"1");
        let h2 = AssetCache::content_hash(b"2");
        let h3 = AssetCache::content_hash(b"3");

        cache.put(h1, Path::new("1"), &[0u8; 100]).unwrap();
        cache.put(h2, Path::new("2"), &[0u8; 100]).unwrap();
        cache.put(h3, Path::new("3"), &[0u8; 100]).unwrap();

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.size(), 300);

        // Insert a 250-byte entry; must evict at least 2 of the 3
        let h_big = AssetCache::content_hash(b"big");
        cache.put(h_big, Path::new("big"), &[0u8; 250]).unwrap();

        // h1 and h2 should be evicted (oldest first), h3 may or may not
        assert!(cache.contains(h_big));
        assert!(!cache.contains(h1));
        assert!(!cache.contains(h2));
        // h3 evicted too since 100 + 250 > 300
        assert!(!cache.contains(h3));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.size(), 250);
    }

    #[test]
    fn test_put_too_large() {
        let mut cache = AssetCache::new(test_config(100));
        let hash = AssetCache::content_hash(b"big");

        let result = cache.put(hash, Path::new("big.bin"), &[0u8; 200]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds max cache size"));
        assert!(cache.is_empty());
    }

    #[test]
    fn test_disabled_cache() {
        let config = AssetCacheConfig {
            max_size_bytes: 4096,
            cache_dir: PathBuf::from(".cache/test"),
            enabled: false,
        };
        let mut cache = AssetCache::new(config);

        let hash = AssetCache::content_hash(b"source");

        // Put succeeds but stores nothing
        cache.put(hash, Path::new("x"), b"data").unwrap();
        assert!(cache.is_empty());

        // Get returns None
        assert_eq!(cache.get(hash), None);
    }

    #[test]
    fn test_config_accessor() {
        let config = test_config(2048);
        let cache = AssetCache::new(config);
        assert_eq!(cache.config().max_size_bytes, 2048);
        assert!(cache.config().enabled);
    }

    #[test]
    fn test_many_entries_stress() {
        let mut cache = AssetCache::new(test_config(1000));

        // Insert 100 entries of 50 bytes each; cache only holds 1000/50 = 20
        for i in 0u32..100 {
            let hash = AssetCache::content_hash(&i.to_le_bytes());
            cache
                .put(hash, Path::new("stress.bin"), &[i as u8; 50])
                .unwrap();
        }

        // Should have exactly 20 entries (the most recent 20)
        assert_eq!(cache.len(), 20);
        assert_eq!(cache.size(), 1000);

        // The last 20 entries (i = 80..100) should be present
        for i in 80u32..100 {
            let hash = AssetCache::content_hash(&i.to_le_bytes());
            assert!(cache.contains(hash), "entry {} should be present", i);
        }

        // Earlier entries should be evicted
        for i in 0u32..80 {
            let hash = AssetCache::content_hash(&i.to_le_bytes());
            assert!(!cache.contains(hash), "entry {} should be evicted", i);
        }
    }

    #[test]
    fn test_put_exact_max_size() {
        let mut cache = AssetCache::new(test_config(100));
        let hash = AssetCache::content_hash(b"exact");
        cache.put(hash, Path::new("exact.bin"), &[0u8; 100]).unwrap();
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.size(), 100);
    }

    #[test]
    fn test_zero_size_entry() {
        let mut cache = AssetCache::new(test_config(1024));
        let hash = AssetCache::content_hash(b"empty-compiled");
        cache.put(hash, Path::new("empty.bin"), b"").unwrap();
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get(hash), Some(vec![]));
    }
}
