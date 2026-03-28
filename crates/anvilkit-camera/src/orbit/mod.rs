/// Camera rig for entity following.
pub mod rig;
/// Spring arm collision avoidance.
pub mod spring_arm;

use bevy_ecs::prelude::*;
use glam::Vec3;

/// Orbit state component — stores orbit distance, limits, and target position.
///
/// Attach to the same entity as [`CameraController`](crate::controller::CameraController)
/// when using [`CameraMode::ThirdPerson`](crate::controller::CameraMode::ThirdPerson)
/// or [`CameraMode::Orbit`](crate::controller::CameraMode::Orbit).
///
/// This replaces the old pattern of embedding distance/target data inside the
/// `CameraMode` enum variants, enabling cleaner system queries and state persistence
/// across mode switches.
#[derive(Component)]
pub struct OrbitState {
    /// World-space position the camera orbits around.
    pub target: Vec3,
    /// Current distance from the target.
    pub distance: f32,
    /// Minimum allowed orbit distance.
    pub min_distance: f32,
    /// Maximum allowed orbit distance.
    pub max_distance: f32,
    /// Offset applied to target position before computing orbit.
    /// E.g., `Vec3::new(0.0, 1.6, 0.0)` for eye height in third-person.
    pub target_offset: Vec3,
}

impl Default for OrbitState {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 5.0,
            min_distance: 1.0,
            max_distance: 50.0,
            target_offset: Vec3::ZERO,
        }
    }
}

impl OrbitState {
    /// Create a new orbit state with the given target and distance.
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            ..Default::default()
        }
    }

    /// Builder: set distance limits.
    pub fn with_distance_limits(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min;
        self.max_distance = max;
        self
    }

    /// Builder: set target offset (e.g. eye height).
    pub fn with_target_offset(mut self, offset: Vec3) -> Self {
        self.target_offset = offset;
        self
    }

    /// Clamp distance to the configured limits.
    pub fn clamp_distance(&mut self) {
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }

    /// Get the effective target position (target + offset).
    pub fn effective_target(&self) -> Vec3 {
        self.target + self.target_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let state = OrbitState::default();
        assert_eq!(state.target, Vec3::ZERO);
        assert_eq!(state.distance, 5.0);
        assert_eq!(state.min_distance, 1.0);
        assert_eq!(state.max_distance, 50.0);
        assert_eq!(state.target_offset, Vec3::ZERO);
    }

    #[test]
    fn test_new() {
        let target = Vec3::new(1.0, 2.0, 3.0);
        let state = OrbitState::new(target, 10.0);
        assert_eq!(state.target, target);
        assert_eq!(state.distance, 10.0);
        assert_eq!(state.min_distance, 1.0);
        assert_eq!(state.max_distance, 50.0);
    }

    #[test]
    fn test_builders() {
        let state = OrbitState::new(Vec3::ZERO, 5.0)
            .with_distance_limits(2.0, 20.0)
            .with_target_offset(Vec3::new(0.0, 1.6, 0.0));
        assert_eq!(state.min_distance, 2.0);
        assert_eq!(state.max_distance, 20.0);
        assert_eq!(state.target_offset, Vec3::new(0.0, 1.6, 0.0));
    }

    #[test]
    fn test_clamp_distance_below_min() {
        let mut state = OrbitState::new(Vec3::ZERO, 0.5)
            .with_distance_limits(1.0, 10.0);
        state.clamp_distance();
        assert_eq!(state.distance, 1.0);
    }

    #[test]
    fn test_clamp_distance_above_max() {
        let mut state = OrbitState::new(Vec3::ZERO, 20.0)
            .with_distance_limits(1.0, 10.0);
        state.clamp_distance();
        assert_eq!(state.distance, 10.0);
    }

    #[test]
    fn test_clamp_distance_in_range() {
        let mut state = OrbitState::new(Vec3::ZERO, 5.0)
            .with_distance_limits(1.0, 10.0);
        state.clamp_distance();
        assert_eq!(state.distance, 5.0);
    }

    #[test]
    fn test_effective_target() {
        let state = OrbitState::new(Vec3::new(1.0, 0.0, 3.0), 5.0)
            .with_target_offset(Vec3::new(0.0, 1.6, 0.0));
        assert_eq!(state.effective_target(), Vec3::new(1.0, 1.6, 3.0));
    }
}
