//! Generic stat system with modifiers.

use bevy_ecs::prelude::*;

/// Modifier kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierKind {
    Additive(f32),
    Multiplicative(f32),
    Override(f32),
}

/// A modifier applied to a stat.
#[derive(Debug, Clone)]
pub struct Modifier {
    pub kind: ModifierKind,
    pub priority: i32,
    pub source: Option<Entity>,
}

impl Modifier {
    pub fn additive(value: f32) -> Self {
        Self { kind: ModifierKind::Additive(value), priority: 0, source: None }
    }
    pub fn multiplicative(value: f32) -> Self {
        Self { kind: ModifierKind::Multiplicative(value), priority: 0, source: None }
    }
    pub fn override_val(value: f32) -> Self {
        Self { kind: ModifierKind::Override(value), priority: 0, source: None }
    }
    pub fn with_priority(mut self, priority: i32) -> Self { self.priority = priority; self }
    pub fn with_source(mut self, source: Entity) -> Self { self.source = Some(source); self }
}

/// A stat with base value and modifiers.
#[derive(Debug, Clone, Component)]
pub struct Stat {
    pub base_value: f32,
    pub modifiers: Vec<Modifier>,
    computed_value: f32,
}

impl Stat {
    pub fn new(base: f32) -> Self {
        Self { base_value: base, modifiers: Vec::new(), computed_value: base }
    }

    pub fn value(&self) -> f32 { self.computed_value }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
        self.recompute();
    }

    pub fn remove_modifiers_from(&mut self, source: Entity) {
        self.modifiers.retain(|m| m.source != Some(source));
        self.recompute();
    }

    pub fn recompute(&mut self) {
        self.modifiers.sort_by_key(|m| m.priority);

        let mut value = self.base_value;
        let mut add_sum = 0.0f32;
        let mut mul_product = 1.0f32;
        let mut override_val: Option<f32> = None;

        for m in &self.modifiers {
            match m.kind {
                ModifierKind::Additive(v) => add_sum += v,
                ModifierKind::Multiplicative(v) => mul_product *= v,
                ModifierKind::Override(v) => override_val = Some(v),
            }
        }

        if let Some(ov) = override_val {
            value = ov;
        } else {
            value = (value + add_sum) * mul_product;
        }

        self.computed_value = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_new() {
        let s = Stat::new(100.0);
        assert_eq!(s.value(), 100.0);
    }

    #[test]
    fn test_additive() {
        let mut s = Stat::new(100.0);
        s.add_modifier(Modifier::additive(20.0));
        assert_eq!(s.value(), 120.0);
    }

    #[test]
    fn test_multiplicative() {
        let mut s = Stat::new(100.0);
        s.add_modifier(Modifier::multiplicative(1.5));
        assert_eq!(s.value(), 150.0);
    }

    #[test]
    fn test_combined() {
        let mut s = Stat::new(100.0);
        s.add_modifier(Modifier::additive(50.0));
        s.add_modifier(Modifier::multiplicative(2.0));
        assert_eq!(s.value(), 300.0); // (100 + 50) * 2
    }

    #[test]
    fn test_override() {
        let mut s = Stat::new(100.0);
        s.add_modifier(Modifier::additive(50.0));
        s.add_modifier(Modifier::override_val(42.0));
        assert_eq!(s.value(), 42.0);
    }

    #[test]
    fn test_remove_by_source() {
        let mut world = bevy_ecs::world::World::new();
        let src = world.spawn_empty().id();

        let mut s = Stat::new(100.0);
        s.add_modifier(Modifier::additive(30.0).with_source(src));
        s.add_modifier(Modifier::additive(20.0));
        assert_eq!(s.value(), 150.0);

        s.remove_modifiers_from(src);
        assert_eq!(s.value(), 120.0);
    }
}
