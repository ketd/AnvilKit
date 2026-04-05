//! # AnvilKit Describe — Self-Describing API Types
//!
//! Foundation for AI-agent introspection in AnvilKit. Every public Component
//! and Resource type implements [`Describe`], providing machine-readable schema
//! information that agents can query to understand the engine's API surface.
//!
//! ## Usage
//!
//! ```rust
//! use anvilkit_describe::{Describe, ComponentSchema};
//!
//! #[derive(Describe)]
//! /// Bloom post-processing settings.
//! struct BloomSettings {
//!     /// Whether bloom is enabled.
//!     #[describe(hint = "Toggle bloom effect")]
//!     pub enabled: bool,
//!
//!     /// HDR brightness threshold.
//!     #[describe(range = "0.0..5.0", default = "1.0", hint = "Higher = less bloom")]
//!     pub threshold: f32,
//!
//!     /// Bloom intensity multiplier.
//!     #[describe(range = "0.0..2.0", default = "0.3")]
//!     pub intensity: f32,
//! }
//!
//! let schema = BloomSettings::schema();
//! assert_eq!(schema.name, "BloomSettings");
//! assert_eq!(schema.fields.len(), 3);
//! assert_eq!(schema.fields[1].name, "threshold");
//! assert_eq!(schema.fields[1].range_min, "0.0");
//! assert_eq!(schema.fields[1].range_max, "5.0");
//!
//! // JSON output for agent consumption
//! let json = schema.to_json();
//! assert!(json.contains("threshold"));
//! ```

use serde::Serialize;

// Re-export the derive macro
pub use anvilkit_describe_derive::Describe;

/// Trait for self-describing types. Implement via `#[derive(Describe)]`.
///
/// Returns a [`ComponentSchema`] containing the type's name, description,
/// and per-field metadata (type, range, default, hint).
pub trait Describe {
    /// Returns the machine-readable schema for this type.
    fn schema() -> ComponentSchema;
}

/// Schema describing a Component or Resource type.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentSchema {
    /// Type name (e.g., "BloomSettings").
    pub name: &'static str,
    /// Type-level documentation.
    pub description: &'static str,
    /// Per-field schemas.
    pub fields: Vec<FieldSchema>,
}

impl ComponentSchema {
    /// Serialize to JSON string for agent consumption.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Serialize to compact JSON (no whitespace).
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Schema describing a single field within a Component or Resource.
#[derive(Debug, Clone, Serialize)]
pub struct FieldSchema {
    /// Field name (e.g., "threshold").
    pub name: &'static str,
    /// Rust type name (e.g., "f32", "Vec3").
    pub type_name: &'static str,
    /// Field documentation extracted from `///` comments.
    pub description: &'static str,
    /// Agent-readable hint for this field.
    pub hint: &'static str,
    /// Default value as a string (empty if not specified).
    pub default: &'static str,
    /// Minimum value of valid range (empty if unbounded).
    pub range_min: &'static str,
    /// Maximum value of valid range (empty if unbounded).
    pub range_max: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as anvilkit_describe;

    #[derive(Describe)]
    /// A test component for bloom settings.
    struct TestBloom {
        /// Whether bloom is enabled.
        #[describe(hint = "Toggle bloom")]
        pub enabled: bool,

        /// HDR brightness threshold.
        #[describe(range = "0.0..5.0", default = "1.0", hint = "Brightness cutoff")]
        pub threshold: f32,

        /// Bloom intensity.
        #[describe(range = "0.0..2.0", default = "0.3")]
        pub intensity: f32,
    }

    #[test]
    fn test_derive_basic() {
        let schema = TestBloom::schema();
        assert_eq!(schema.name, "TestBloom");
        assert!(schema.description.contains("test component"));
        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_field_metadata() {
        let schema = TestBloom::schema();
        let threshold = &schema.fields[1];
        assert_eq!(threshold.name, "threshold");
        assert_eq!(threshold.type_name, "f32");
        assert_eq!(threshold.hint, "Brightness cutoff");
        assert_eq!(threshold.range_min, "0.0");
        assert_eq!(threshold.range_max, "5.0");
        assert_eq!(threshold.default, "1.0");
    }

    #[test]
    fn test_doc_extraction() {
        let schema = TestBloom::schema();
        assert!(schema.fields[0].description.contains("Whether bloom is enabled"));
    }

    #[test]
    fn test_json_output() {
        let schema = TestBloom::schema();
        let json = schema.to_json();
        assert!(json.contains("\"name\": \"TestBloom\""));
        assert!(json.contains("\"threshold\""));
        assert!(json.contains("\"range_min\": \"0.0\""));
    }

    #[test]
    fn test_json_compact() {
        let schema = TestBloom::schema();
        let json = schema.to_json_compact();
        assert!(!json.contains('\n'));
        assert!(json.contains("TestBloom"));
    }

    #[derive(Describe)]
    /// An empty struct.
    struct EmptyComponent;

    #[test]
    fn test_unit_struct() {
        let schema = EmptyComponent::schema();
        assert_eq!(schema.name, "EmptyComponent");
        assert_eq!(schema.fields.len(), 0);
    }

    #[derive(Describe)]
    /// A simple enum.
    enum TestMode {
        /// First person view.
        FirstPerson,
        /// Third person view.
        ThirdPerson,
        /// Orbit camera.
        Orbit,
    }

    #[test]
    fn test_enum_describe() {
        let schema = TestMode::schema();
        assert_eq!(schema.name, "TestMode");
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].name, "FirstPerson");
        assert_eq!(schema.fields[0].type_name, "variant");
        assert!(schema.fields[0].description.contains("First person"));
    }
}
