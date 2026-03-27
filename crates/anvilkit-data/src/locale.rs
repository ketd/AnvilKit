//! Localization — translate keys to localized strings.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Locale resource — holds translations for the current language.
#[derive(Debug, Clone, Resource)]
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
}
