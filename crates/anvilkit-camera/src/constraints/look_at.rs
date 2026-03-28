//! Soft look-at constraint for camera orientation.
//!
//! Provides damped look-at behavior with an optional screen-space dead zone.
//! The camera only rotates toward the target when the target exits the dead zone,
//! and rotation speed is controlled by damping.

use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

/// Soft look-at target component.
///
/// When attached to a camera entity and `enabled` is `true`, the system will
/// gradually rotate the camera to face `target`, with damping and an optional
/// dead zone to prevent micro-corrections.
#[derive(Component)]
pub struct LookAtTarget {
    /// The world-space position to look at.
    pub target: Vec3,
    /// Screen-space dead zone (normalized `[0, 1]`).
    /// Within this zone, no rotation correction is applied.
    /// `(0.1, 0.1)` = 10% of screen in each direction from center.
    pub dead_zone: Vec2,
    /// Damping speed (higher = faster tracking).
    /// Uses frame-rate independent formula: `1 - e^(-speed * dt)`.
    pub damping: f32,
    /// Whether the constraint is active.
    pub enabled: bool,
}

impl Default for LookAtTarget {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            dead_zone: Vec2::ZERO,
            damping: 10.0,
            enabled: true,
        }
    }
}

impl LookAtTarget {
    /// Create a look-at target.
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Builder: set dead zone (normalized screen space).
    pub fn with_dead_zone(mut self, dead_zone: Vec2) -> Self {
        self.dead_zone = dead_zone;
        self
    }

    /// Builder: set damping speed.
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }
}

/// Applies the soft look-at constraint to camera transforms.
///
/// For each camera with an enabled `LookAtTarget`, computes the direction
/// to the target and smoothly rotates the camera toward it using damped slerp.
pub fn camera_look_at_system(
    dt: Res<anvilkit_ecs::physics::DeltaTime>,
    mut query: Query<(
        &LookAtTarget,
        &mut anvilkit_core::math::Transform,
    )>,
) {
    for (look_at, mut transform) in query.iter_mut() {
        if !look_at.enabled {
            continue;
        }

        let to_target = look_at.target - transform.translation;
        if to_target.length_squared() < 0.001 {
            continue;
        }

        let desired_dir = to_target.normalize();

        // Compute the desired rotation (look-at)
        let forward = desired_dir;
        let world_up = if forward.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let right = forward.cross(world_up).normalize_or_zero();
        if right.length_squared() < 0.5 {
            continue;
        }
        let up = right.cross(forward).normalize_or_zero();
        let desired_rot = glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward));

        // Dead zone check: compute angle between current forward and desired
        let current_forward = transform.rotation * Vec3::Z;
        let angle = current_forward.dot(desired_dir).clamp(-1.0, 1.0).acos();

        // Convert dead zone to approximate angle threshold
        let dz_angle = (look_at.dead_zone.x.max(look_at.dead_zone.y) * std::f32::consts::FRAC_PI_2)
            .max(0.0);

        if angle < dz_angle {
            continue;
        }

        // Damped slerp toward desired rotation
        let factor = 1.0 - (-look_at.damping * dt.0).exp();
        transform.rotation = transform.rotation.slerp(desired_rot, factor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let la = LookAtTarget::default();
        assert!(la.enabled);
        assert_eq!(la.damping, 10.0);
        assert_eq!(la.dead_zone, Vec2::ZERO);
    }

    #[test]
    fn test_new() {
        let la = LookAtTarget::new(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(la.target, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_builders() {
        let la = LookAtTarget::new(Vec3::ZERO)
            .with_dead_zone(Vec2::new(0.1, 0.1))
            .with_damping(5.0);
        assert_eq!(la.dead_zone, Vec2::new(0.1, 0.1));
        assert_eq!(la.damping, 5.0);
    }
}
