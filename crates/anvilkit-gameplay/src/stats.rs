//! # Generic Stat System
//!
//! A flexible stat system with modifier stacking. Each [`Stat`] has a base
//! value and an ordered list of [`Modifier`]s that are collapsed into a single
//! computed value via [`Stat::recompute`].
//!
//! Modifier application order (after sorting by priority, ascending):
//! 1. Start with `base_value`
//! 2. Sum all `Additive` modifiers and add to base
//! 3. Multiply by the product of all `Multiplicative` modifiers
//! 4. If any `Override` modifiers exist, the last one (highest priority) wins
//!
//! ## Example
//!
//! ```rust
//! use anvilkit_gameplay::stats::{Stat, Modifier, ModifierKind};
//!
//! let mut stat = Stat::new(100.0);
//! stat.add_modifier(Modifier::new(ModifierKind::Additive(20.0), 0));
//! stat.recompute();
//! assert_eq!(stat.value(), 120.0);
//! ```

use bevy_ecs::prelude::*;

/// The kind of modification a [`Modifier`] applies to a [`Stat`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierKind {
    /// Added to the base value before multiplication.
    Additive(f32),
    /// Multiplied into the running total after all additive modifiers.
    Multiplicative(f32),
    /// Overwrites the final value. If multiple overrides exist the one with the
    /// highest priority wins (last after sort).
    Override(f32),
}

/// A single modifier that can be attached to a [`Stat`].
#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    /// What kind of modification this applies.
    pub kind: ModifierKind,
    /// Lower priorities are applied first.
    pub priority: i32,
    /// Optional source entity so modifiers can be bulk-removed later.
    pub source: Option<Entity>,
}

impl Modifier {
    /// Create a modifier with no source entity.
    pub fn new(kind: ModifierKind, priority: i32) -> Self {
        Self {
            kind,
            priority,
            source: None,
        }
    }

    /// Create a modifier attributed to a specific source entity.
    pub fn with_source(kind: ModifierKind, priority: i32, source: Entity) -> Self {
        Self {
            kind,
            priority,
            source: Some(source),
        }
    }

    /// Convenience: additive modifier at priority 0 with no source.
    pub fn additive(value: f32) -> Self {
        Self::new(ModifierKind::Additive(value), 0)
    }

    /// Convenience: multiplicative modifier at priority 0 with no source.
    pub fn multiplicative(value: f32) -> Self {
        Self::new(ModifierKind::Multiplicative(value), 0)
    }

    /// Convenience: override modifier at priority 0 with no source.
    pub fn override_val(value: f32) -> Self {
        Self::new(ModifierKind::Override(value), 0)
    }

    /// Builder: set the priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Builder: set the source entity.
    pub fn from_source(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }
}

/// A single numeric stat with modifier support.
///
/// Call [`Stat::recompute`] after adding or removing modifiers to update the
/// cached `computed_value`.
#[derive(Debug, Clone, Component)]
pub struct Stat {
    /// The raw base value before any modifiers.
    pub base_value: f32,
    /// Active modifiers on this stat.
    pub modifiers: Vec<Modifier>,
    /// Cached result of the last [`Stat::recompute`] call.
    pub computed_value: f32,
}

impl Stat {
    /// Create a new stat with the given base value and no modifiers.
    pub fn new(base: f32) -> Self {
        Self {
            base_value: base,
            modifiers: Vec::new(),
            computed_value: base,
        }
    }

    /// Append a modifier. You must call [`Stat::recompute`] afterwards for the
    /// change to take effect.
    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
    }

    /// Remove every modifier whose `source` matches the given entity.
    pub fn remove_modifiers_from(&mut self, source: Entity) {
        self.modifiers.retain(|m| m.source != Some(source));
    }

    /// Recompute the cached value by applying all modifiers in priority order.
    ///
    /// Application order:
    /// 1. Sort modifiers by priority (ascending).
    /// 2. Start with `base_value`.
    /// 3. Sum all [`ModifierKind::Additive`] values and add to base.
    /// 4. Multiply by the product of all [`ModifierKind::Multiplicative`] values.
    /// 5. If any [`ModifierKind::Override`] exists, the last one (highest
    ///    priority) replaces the result entirely.
    pub fn recompute(&mut self) {
        self.modifiers.sort_by_key(|m| m.priority);

        let mut additive_sum: f32 = 0.0;
        let mut multiplicative_product: f32 = 1.0;
        let mut last_override: Option<f32> = None;

        for modifier in &self.modifiers {
            match modifier.kind {
                ModifierKind::Additive(v) => additive_sum += v,
                ModifierKind::Multiplicative(v) => multiplicative_product *= v,
                ModifierKind::Override(v) => last_override = Some(v),
            }
        }

        let value = (self.base_value + additive_sum) * multiplicative_product;

        self.computed_value = if let Some(ov) = last_override {
            ov
        } else {
            value
        };
    }

    /// Return the last computed value.
    pub fn value(&self) -> f32 {
        self.computed_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stat_has_base_as_computed() {
        let stat = Stat::new(50.0);
        assert_eq!(stat.value(), 50.0);
        assert_eq!(stat.base_value, 50.0);
        assert!(stat.modifiers.is_empty());
    }

    #[test]
    fn additive_modifier_increases_value() {
        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::new(ModifierKind::Additive(25.0), 0));
        stat.add_modifier(Modifier::new(ModifierKind::Additive(-10.0), 0));
        stat.recompute();
        assert!((stat.value() - 115.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplicative_modifier_scales_value() {
        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::new(ModifierKind::Multiplicative(1.5), 0));
        stat.recompute();
        assert!((stat.value() - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn additive_then_multiplicative() {
        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::new(ModifierKind::Additive(50.0), 0));
        stat.add_modifier(Modifier::new(ModifierKind::Multiplicative(2.0), 1));
        stat.recompute();
        // (100 + 50) * 2 = 300
        assert!((stat.value() - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn override_replaces_computed_value() {
        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::new(ModifierKind::Additive(9999.0), 0));
        stat.add_modifier(Modifier::new(ModifierKind::Override(42.0), 10));
        stat.recompute();
        assert!((stat.value() - 42.0).abs() < f32::EPSILON);
    }

    #[test]
    fn last_override_wins_by_priority() {
        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::new(ModifierKind::Override(10.0), 0));
        stat.add_modifier(Modifier::new(ModifierKind::Override(99.0), 5));
        stat.recompute();
        // priority 5 > priority 0, so 99 wins (last after sort)
        assert!((stat.value() - 99.0).abs() < f32::EPSILON);
    }

    #[test]
    fn remove_modifiers_from_source() {
        let mut world = bevy_ecs::world::World::new();
        let source_a = world.spawn_empty().id();
        let source_b = world.spawn_empty().id();

        let mut stat = Stat::new(100.0);
        stat.add_modifier(Modifier::with_source(ModifierKind::Additive(10.0), 0, source_a));
        stat.add_modifier(Modifier::with_source(ModifierKind::Additive(20.0), 0, source_b));
        stat.add_modifier(Modifier::with_source(ModifierKind::Additive(30.0), 0, source_a));

        stat.remove_modifiers_from(source_a);
        stat.recompute();

        // Only source_b's +20 remains
        assert!((stat.value() - 120.0).abs() < f32::EPSILON);
        assert_eq!(stat.modifiers.len(), 1);
    }

    #[test]
    fn priority_ordering_determines_application() {
        let mut stat = Stat::new(10.0);
        // Insert in reverse priority order to prove sorting works
        stat.add_modifier(Modifier::new(ModifierKind::Multiplicative(2.0), 10));
        stat.add_modifier(Modifier::new(ModifierKind::Additive(5.0), 1));
        stat.recompute();
        // After sort: additive(prio 1) then multiplicative(prio 10)
        // (10 + 5) * 2 = 30
        assert!((stat.value() - 30.0).abs() < f32::EPSILON);

        // Now add an override at the lowest priority — it still wins because
        // override semantics always take precedence regardless of position.
        stat.add_modifier(Modifier::new(ModifierKind::Override(7.0), -100));
        stat.recompute();
        assert!((stat.value() - 7.0).abs() < f32::EPSILON);
    }
}
