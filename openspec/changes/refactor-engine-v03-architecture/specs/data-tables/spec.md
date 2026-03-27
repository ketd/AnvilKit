## ADDED Requirements

### Requirement: Data Table System
The system SHALL provide `DataTable<K, V>` as a typed, immutable lookup table loaded from RON or JSON files.

Data tables SHALL support loading from the asset pipeline and hot-reloading when the source file changes.

The system SHALL provide `DataTablePlugin` that registers loaded tables as ECS resources.

#### Scenario: Item definition table
- **WHEN** `items.ron` contains `{ "sword": { name: "Iron Sword", damage: 10 }, "shield": { name: "Wooden Shield", armor: 5 } }`
- **THEN** `data_table.get("sword").unwrap().damage` returns 10

#### Scenario: Hot reload
- **WHEN** an item's stats are modified in the RON file while the game is running (with hot-reload enabled)
- **THEN** the DataTable resource is updated on the next frame

### Requirement: Localization System
The system SHALL provide a `Locale` resource holding the current language code and a loaded translation map.

The system SHALL provide `t!(key)` macro or `locale.translate(key)` method that returns the translated string for the current locale, falling back to the key itself if no translation exists.

Translation files SHALL be loaded from `assets/i18n/{locale}.ron` (e.g., `en.ron`, `zh.ron`).

#### Scenario: Language switch
- **WHEN** `Locale` is changed from "en" to "zh"
- **THEN** all subsequent `t!("greeting")` calls return the Chinese translation

#### Scenario: Missing translation fallback
- **WHEN** `t!("unknown_key")` is called and no translation exists
- **THEN** the key string "unknown_key" is returned as-is
