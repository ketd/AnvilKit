//! # AnvilKit Data Tables & i18n
//!
//! Data-driven configuration and localization.
//!
//! - [`DataTable`] — typed key-value table loaded from RON/JSON
//! - [`Locale`] — translation lookup with fallback

pub mod data_table;
pub mod locale;
pub mod plugin;

pub use data_table::DataTable;
pub use locale::Locale;
pub use plugin::DataTablePlugin;

pub mod prelude {
    pub use crate::data_table::DataTable;
    pub use crate::locale::Locale;
    pub use crate::plugin::DataTablePlugin;
}
