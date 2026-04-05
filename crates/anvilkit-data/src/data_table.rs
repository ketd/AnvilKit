//! Data table — typed key-value store loaded from RON or JSON.

use bevy_ecs::prelude::*;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::hash::Hash;

/// A typed data table mapping keys to values.
///
/// Loaded from RON files at startup or runtime.
/// Stored as an ECS Resource for global access.
#[derive(Debug, Clone, Resource)]
pub struct DataTable<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> {
    entries: HashMap<K, V>,
    name: String,
}

impl<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> DataTable<K, V> {
    /// Create an empty data table.
    pub fn new(name: impl Into<String>) -> Self {
        Self { entries: HashMap::new(), name: name.into() }
    }

    /// Get a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, value);
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter()
    }

    /// Table name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<K, V> DataTable<K, V>
where
    K: Eq + Hash + Send + Sync + DeserializeOwned + 'static,
    V: Send + Sync + DeserializeOwned + 'static,
{
    /// Load from a RON string.
    pub fn from_ron(name: impl Into<String>, ron_str: &str) -> Result<Self, String> {
        let entries: HashMap<K, V> = ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse RON: {}", e))?;
        Ok(Self { entries, name: name.into() })
    }

    /// Load from a JSON string.
    pub fn from_json(name: impl Into<String>, json_str: &str) -> Result<Self, String> {
        let entries: HashMap<K, V> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(Self { entries, name: name.into() })
    }

    /// Load from a RON file on disk.
    pub fn from_ron_file(name: impl Into<String>, path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read file {}: {}", path.as_ref().display(), e))?;
        Self::from_ron(name, &content)
    }

    /// Load from a JSON file on disk.
    pub fn from_json_file(name: impl Into<String>, path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read file {}: {}", path.as_ref().display(), e))?;
        Self::from_json(name, &content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let table: DataTable<String, i32> = DataTable::new("test");
        assert!(table.is_empty());
        assert_eq!(table.name(), "test");
    }

    #[test]
    fn test_insert_get() {
        let mut table = DataTable::new("items");
        table.insert("sword".to_string(), 10);
        table.insert("shield".to_string(), 5);
        assert_eq!(table.get(&"sword".to_string()), Some(&10));
        assert_eq!(table.get(&"shield".to_string()), Some(&5));
        assert_eq!(table.get(&"bow".to_string()), None);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn test_from_ron() {
        let ron = r#"{"hp": 100, "mp": 50, "str": 12}"#;
        let table: DataTable<String, i32> = DataTable::from_ron("stats", ron).unwrap();
        assert_eq!(table.get(&"hp".to_string()), Some(&100));
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn test_from_ron_file() {
        // Write a temp RON file
        let dir = std::env::temp_dir().join("anvilkit_data_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_table.ron");
        std::fs::write(&path, r#"{"a": 1, "b": 2}"#).unwrap();

        let table: DataTable<String, i32> = DataTable::from_ron_file("test", &path).unwrap();
        assert_eq!(table.get(&"a".to_string()), Some(&1));
        assert_eq!(table.len(), 2);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_from_ron_file_not_found() {
        let result: Result<DataTable<String, i32>, _> = DataTable::from_ron_file("test", "/nonexistent/path.ron");
        assert!(result.is_err());
    }

    #[test]
    fn test_iter() {
        let mut table = DataTable::new("t");
        table.insert(1, "a".to_string());
        table.insert(2, "b".to_string());
        let count = table.iter().count();
        assert_eq!(count, 2);
    }
}
