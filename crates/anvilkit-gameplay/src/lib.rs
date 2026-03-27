//! # AnvilKit Gameplay Systems
//!
//! Common gameplay patterns as optional, feature-gated modules.
//!
//! ## Features
//!
//! - `stats` — Generic stat system with modifiers (additive, multiplicative, override)
//! - `inventory` — Slot-based and stackable item inventory
//! - `cooldown` — Cooldown timers for abilities and actions
//! - `status-effect` — Duration-based status effects with stacking policies
//! - `entity-pool` — Object pool for entity recycling

#[cfg(feature = "stats")]
pub mod stats;

#[cfg(feature = "stats")]
pub mod health;

#[cfg(feature = "inventory")]
pub mod inventory;

#[cfg(feature = "cooldown")]
pub mod cooldown;

#[cfg(feature = "status-effect")]
pub mod status_effect;

#[cfg(feature = "entity-pool")]
pub mod entity_pool;

/// Prelude for convenient imports.
pub mod prelude {
    #[cfg(feature = "stats")]
    pub use crate::stats::*;

    #[cfg(feature = "stats")]
    pub use crate::health::*;

    #[cfg(feature = "inventory")]
    pub use crate::inventory::*;

    #[cfg(feature = "cooldown")]
    pub use crate::cooldown::*;

    #[cfg(feature = "status-effect")]
    pub use crate::status_effect::*;

    #[cfg(feature = "entity-pool")]
    pub use crate::entity_pool::*;
}
