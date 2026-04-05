//! Localization — translate keys to localized strings.

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use anvilkit_describe::Describe;

/// Locale resource — holds translations for the current language.
#[derive(Debug, Clone, Resource, Describe)]
/// Localization resource with key-to-string translation map.
pub struct Locale {
    language: String,
    translations: HashMap<String, String>,
}

impl Locale {
    /// Create a new locale for the given language.
    pub fn new(language: impl Into<String>) -> Self {
        Self { language: language.into(), translations: HashMap::new() }
    }

    /// Current language identifier (e.g., "en", "zh", "ja").
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Set the language.
    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }

    /// Add a translation entry.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.translations.insert(key.into(), value.into());
    }

    /// Load translations from a RON string (HashMap<String, String>).
    pub fn load_ron(&mut self, ron_str: &str) -> Result<usize, String> {
        let entries: HashMap<String, String> = ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse locale RON: {}", e))?;
        let count = entries.len();
        self.translations.extend(entries);
        Ok(count)
    }

    /// Translate a key. Falls back to the key itself if not found.
    pub fn translate<'a>(&'a self, key: &'a str) -> &'a str {
        self.translations.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    /// Translate with alias `t()`.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translate(key)
    }

    /// Check if a key has a translation.
    pub fn has_key(&self, key: &str) -> bool {
        self.translations.contains_key(key)
    }

    /// Number of translation entries.
    pub fn len(&self) -> usize {
        self.translations.len()
    }

    /// Whether translations are empty.
    pub fn is_empty(&self) -> bool {
        self.translations.is_empty()
    }

    /// Translate with parameter substitution.
    ///
    /// Replaces `{key}` placeholders in the translated string with provided values.
    ///
    /// ```rust
    /// use anvilkit_data::locale::Locale;
    /// let mut loc = Locale::new("en");
    /// loc.insert("greeting", "Hello, {name}! You have {count} items.");
    /// let result = loc.t_fmt("greeting", &[("name", "Alice"), ("count", "3")]);
    /// assert_eq!(result, "Hello, Alice! You have 3 items.");
    /// ```
    pub fn t_fmt(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut result = self.translate(key).to_string();
        for (k, v) in params {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    /// Load translations from a RON file on disk.
    pub fn load_ron_file(&mut self, path: impl AsRef<std::path::Path>) -> Result<usize, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read locale file {}: {}", path.as_ref().display(), e))?;
        self.load_ron(&content)
    }

    /// Switch language: clear all translations, set language, and load from a RON file.
    pub fn switch_language(
        &mut self,
        language: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<usize, String> {
        self.translations.clear();
        self.language = language.into();
        self.load_ron_file(path)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_found() {
        let mut loc = Locale::new("en");
        loc.insert("greeting", "Hello!");
        assert_eq!(loc.translate("greeting"), "Hello!");
    }

    #[test]
    fn test_translate_fallback() {
        let loc = Locale::new("en");
        // Missing key returns the key itself
        assert_eq!(loc.translate("missing.key"), "missing.key");
    }

    #[test]
    fn test_load_ron() {
        let ron = r#"{"ui.ok": "OK", "ui.cancel": "Cancel"}"#;
        let mut loc = Locale::new("en");
        let count = loc.load_ron(ron).unwrap();
        assert_eq!(count, 2);
        assert_eq!(loc.t("ui.ok"), "OK");
        assert_eq!(loc.t("ui.cancel"), "Cancel");
    }

    #[test]
    fn test_language() {
        let mut loc = Locale::new("zh");
        assert_eq!(loc.language(), "zh");
        loc.set_language("ja");
        assert_eq!(loc.language(), "ja");
    }

    #[test]
    fn test_has_key() {
        let mut loc = Locale::new("en");
        loc.insert("exists", "yes");
        assert!(loc.has_key("exists"));
        assert!(!loc.has_key("nope"));
    }

    #[test]
    fn test_t_fmt_substitution() {
        let mut loc = Locale::new("en");
        loc.insert("msg", "Hello, {name}! Score: {score}");
        let result = loc.t_fmt("msg", &[("name", "Alice"), ("score", "100")]);
        assert_eq!(result, "Hello, Alice! Score: 100");
    }

    #[test]
    fn test_t_fmt_no_params() {
        let mut loc = Locale::new("en");
        loc.insert("plain", "No placeholders here");
        let result = loc.t_fmt("plain", &[]);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_t_fmt_missing_key_returns_key_with_params() {
        let loc = Locale::new("en");
        let result = loc.t_fmt("missing.key", &[("x", "y")]);
        assert_eq!(result, "missing.key");
    }

    #[test]
    fn test_load_ron_file() {
        let dir = std::env::temp_dir().join("anvilkit_locale_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_locale.ron");
        std::fs::write(&path, r#"{"hello": "world", "foo": "bar"}"#).unwrap();

        let mut loc = Locale::new("en");
        let count = loc.load_ron_file(&path).unwrap();
        assert_eq!(count, 2);
        assert_eq!(loc.t("hello"), "world");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_switch_language() {
        let dir = std::env::temp_dir().join("anvilkit_locale_switch_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("zh.ron");
        std::fs::write(&path, r#"{"greeting": "你好"}"#).unwrap();

        let mut loc = Locale::new("en");
        loc.insert("greeting", "Hello");
        assert_eq!(loc.t("greeting"), "Hello");

        loc.switch_language("zh", &path).unwrap();
        assert_eq!(loc.language(), "zh");
        assert_eq!(loc.t("greeting"), "你好");

        std::fs::remove_file(&path).ok();
    }
}
