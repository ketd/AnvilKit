//! Duration-based status effects with stacking policies.
//!
//! A [`StatusEffect`] represents a named, timed buff/debuff that can be applied
//! to an entity.  Multiple effects are held in a [`StatusEffectList`] component,
//! which handles ticking, expiry, and stacking.

use bevy_ecs::prelude::*;

/// Determines how a new status effect interacts with an existing one of the
/// same name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPolicy {
    /// Replace the existing effect (reset remaining time).
    Replace,
    /// Extend the existing effect's remaining time by the new effect's duration.
    Extend,
    /// Increment stacks (up to `max_stacks`), resetting remaining time.
    Stack,
}

/// A single status effect instance.
#[derive(Debug, Clone, Component)]
pub struct StatusEffect {
    /// Human-readable name used as an identifier.
    pub name: String,
    /// The full duration of one application.
    pub duration: f32,
    /// Seconds remaining before this effect expires.
    pub remaining: f32,
    /// Current number of stacks.
    pub stacks: u32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// How this effect handles re-application.
    pub stack_policy: StackPolicy,
}

impl StatusEffect {
    /// Creates a new status effect with 1 stack, max 1 stack, and
    /// [`StackPolicy::Replace`].
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            remaining: duration,
            stacks: 1,
            max_stacks: 1,
            stack_policy: StackPolicy::Replace,
        }
    }

    /// Builder helper — set the stack policy.
    pub fn with_policy(mut self, policy: StackPolicy) -> Self {
        self.stack_policy = policy;
        self
    }

    /// Builder helper — set the maximum number of stacks.
    pub fn with_max_stacks(mut self, max: u32) -> Self {
        self.max_stacks = max;
        self
    }

    /// Returns `true` when the effect has expired (`remaining <= 0`).
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Advance the effect by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
    }

    /// Apply another effect of the same name according to the stack policy.
    ///
    /// - **Replace** — reset `remaining` to the new effect's `duration`.
    /// - **Extend** — add the new effect's `duration` to `remaining`.
    /// - **Stack** — increment `stacks` (capped at `max_stacks`) and reset
    ///   `remaining`.
    pub fn apply(&mut self, other: &StatusEffect) {
        match self.stack_policy {
            StackPolicy::Replace => {
                self.remaining = other.duration;
            }
            StackPolicy::Extend => {
                self.remaining += other.duration;
            }
            StackPolicy::Stack => {
                if self.stacks < self.max_stacks {
                    self.stacks += 1;
                }
                self.remaining = other.duration;
            }
        }
    }
}

/// A collection of [`StatusEffect`]s attached to an entity.
#[derive(Component, Debug, Clone, Default)]
pub struct StatusEffectList {
    /// The active effects.
    pub effects: Vec<StatusEffect>,
}

impl StatusEffectList {
    /// Add a status effect.  If an effect with the same name already exists,
    /// the existing effect's [`StatusEffect::apply`] is called instead of
    /// inserting a duplicate.
    pub fn add(&mut self, effect: StatusEffect) {
        if let Some(existing) = self.effects.iter_mut().find(|e| e.name == effect.name) {
            existing.apply(&effect);
        } else {
            self.effects.push(effect);
        }
    }

    /// Tick all effects by `dt` and remove any that have expired.
    pub fn tick(&mut self, dt: f32) {
        for effect in &mut self.effects {
            effect.tick(dt);
        }
        self.effects.retain(|e| !e.is_expired());
    }

    /// Returns `true` if an effect with the given `name` is active (not expired).
    pub fn has(&self, name: &str) -> bool {
        self.effects.iter().any(|e| e.name == name)
    }

    /// Returns a reference to the effect with the given `name`, if present.
    pub fn get(&self, name: &str) -> Option<&StatusEffect> {
        self.effects.iter().find(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_effect_starts_with_full_duration() {
        let eff = StatusEffect::new("burn", 5.0);
        assert_eq!(eff.remaining, 5.0);
        assert_eq!(eff.stacks, 1);
        assert!(!eff.is_expired());
    }

    #[test]
    fn tick_expires_effect() {
        let mut eff = StatusEffect::new("poison", 2.0);
        eff.tick(1.0);
        assert!(!eff.is_expired());
        eff.tick(1.5);
        assert!(eff.is_expired());
    }

    #[test]
    fn replace_policy_resets_remaining() {
        let mut eff = StatusEffect::new("slow", 5.0);
        eff.tick(3.0); // remaining = 2.0

        let other = StatusEffect::new("slow", 5.0);
        eff.apply(&other);
        assert!((eff.remaining - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn extend_policy_adds_duration() {
        let mut eff = StatusEffect::new("regen", 4.0).with_policy(StackPolicy::Extend);
        eff.tick(1.0); // remaining = 3.0

        let other = StatusEffect::new("regen", 4.0);
        eff.apply(&other);
        assert!((eff.remaining - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stack_policy_increments_and_caps() {
        let mut eff = StatusEffect::new("might", 10.0)
            .with_policy(StackPolicy::Stack)
            .with_max_stacks(3);
        assert_eq!(eff.stacks, 1);

        let other = StatusEffect::new("might", 10.0);
        eff.apply(&other);
        assert_eq!(eff.stacks, 2);
        eff.apply(&other);
        assert_eq!(eff.stacks, 3);
        eff.apply(&other); // should not exceed max
        assert_eq!(eff.stacks, 3);
    }

    #[test]
    fn list_add_merges_same_name() {
        let mut list = StatusEffectList::default();
        list.add(StatusEffect::new("burn", 5.0).with_policy(StackPolicy::Extend));
        list.add(StatusEffect::new("burn", 5.0));
        assert_eq!(list.effects.len(), 1);
        assert!((list.effects[0].remaining - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn list_tick_removes_expired() {
        let mut list = StatusEffectList::default();
        list.add(StatusEffect::new("short", 1.0));
        list.add(StatusEffect::new("long", 10.0));
        assert_eq!(list.effects.len(), 2);

        list.tick(2.0);
        assert_eq!(list.effects.len(), 1);
        assert!(list.has("long"));
        assert!(!list.has("short"));
    }

    #[test]
    fn list_get_returns_effect() {
        let mut list = StatusEffectList::default();
        list.add(StatusEffect::new("shield", 8.0));
        let shield = list.get("shield");
        assert!(shield.is_some());
        assert_eq!(shield.unwrap().duration, 8.0);
        assert!(list.get("nonexistent").is_none());
    }
}
