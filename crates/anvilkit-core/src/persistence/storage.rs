//! # 大规模数据 KV 存储
//!
//! 基于文件系统的 key-value 存储后端，用于 chunk 数据等大规模结构化数据。
//! 每个 key 对应一个文件，支持前缀枚举和批量写入。

use std::path::{Path, PathBuf};
use crate::error::AnvilKitError;

/// 文件系统 KV 存储
///
/// 每个 key 映射到 `base_dir/key` 文件。Key 中的 `/` 创建子目录。
///
/// # 示例
///
/// ```rust,no_run
/// use anvilkit_core::persistence::WorldStorage;
///
/// let storage = WorldStorage::open("saves/world1").unwrap();
/// storage.put("chunk/3/-2", b"binary chunk data").unwrap();
/// let data = storage.get("chunk/3/-2").unwrap();
/// ```
pub struct WorldStorage {
    base_dir: PathBuf,
}

impl WorldStorage {
    /// 打开或创建一个存储目录。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AnvilKitError> {
        let base_dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| AnvilKitError::generic(format!("Failed to create storage dir: {}", e)))?;
        Ok(Self { base_dir })
    }

    /// 读取 key 对应的值。Key 不存在返回 None。
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.key_path(key);
        std::fs::read(&path).ok()
    }

    /// 写入 key-value 对。自动创建中间目录。
    pub fn put(&self, key: &str, value: &[u8]) -> Result<(), AnvilKitError> {
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AnvilKitError::generic(format!("Failed to create dir for key '{}': {}", key, e)))?;
        }
        // 写入临时文件再 rename，确保原子性
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, value)
            .map_err(|e| AnvilKitError::generic(format!("Failed to write key '{}': {}", key, e)))?;
        std::fs::rename(&tmp_path, &path)
            .map_err(|e| AnvilKitError::generic(format!("Failed to rename for key '{}': {}", key, e)))?;
        Ok(())
    }

    /// 删除 key。
    pub fn delete(&self, key: &str) -> Result<(), AnvilKitError> {
        let path = self.key_path(key);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| AnvilKitError::generic(format!("Failed to delete key '{}': {}", key, e)))?;
        }
        Ok(())
    }

    /// 枚举所有以 prefix 开头的 key（不加载值）。
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        let prefix_path = self.base_dir.join(prefix);
        let search_dir = if prefix_path.is_dir() {
            prefix_path
        } else {
            prefix_path.parent().unwrap_or(&self.base_dir).to_path_buf()
        };

        let mut keys = Vec::new();
        self.collect_keys_recursive(&search_dir, prefix, &mut keys);
        keys
    }

    /// 批量写入（非事务性，但保证每个 key 的原子性）。
    pub fn batch_put(&self, entries: &[(&str, &[u8])]) -> Result<(), AnvilKitError> {
        for (key, value) in entries {
            self.put(key, value)?;
        }
        Ok(())
    }

    /// 存储根目录路径。
    pub fn path(&self) -> &Path {
        &self.base_dir
    }

    fn key_path(&self, key: &str) -> PathBuf {
        self.base_dir.join(key)
    }

    fn collect_keys_recursive(&self, dir: &Path, prefix: &str, keys: &mut Vec<String>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.collect_keys_recursive(&path, prefix, keys);
            } else if path.extension().map_or(true, |e| e != "tmp") {
                if let Ok(rel) = path.strip_prefix(&self.base_dir) {
                    let key = rel.to_string_lossy().replace('\\', "/");
                    if key.starts_with(prefix) {
                        keys.push(key);
                    }
                }
            }
        }
    }
}
