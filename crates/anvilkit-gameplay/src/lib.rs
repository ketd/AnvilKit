//! # AnvilKit Gameplay Systems
//!
//! Common gameplay patterns as optional, feature-gated modules.
//!
//! ## Features
//!
//! - `stats` — Health component and damage/heal events
//! - `inventory` — Slot-based and stackable item inventory

#[cfg(feature = "stats")]
pub mod health;

#[cfg(feature = "inventory")]
pub mod inventory;

/// Prelude for convenient imports.
pub mod prelude {
    #[cfg(feature = "stats")]
    pub use crate::health::*;

    #[cfg(feature = "inventory")]
    pub use crate::inventory::*;
}
