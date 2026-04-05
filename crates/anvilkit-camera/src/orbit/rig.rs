//! Camera rig component — automatic entity following with offset and damping.

use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_describe::Describe;

/// Camera rig component that automatically follows a target entity.
///
/// When attached to the same entity as a [`CameraController`](crate::controller::CameraController)
/// with [`CameraMode::ThirdPerson`](crate::controller::CameraMode::ThirdPerson), the
/// [`camera_rig_system`] reads the target entity's `Transform` each frame and updates
/// the [`OrbitState.target`](crate::orbit::OrbitState) accordingly.
///
/// This replaces the old pattern of manually writing `target = player_pos` in game code.
#[derive(Component, Describe)]
/// Camera rig that follows a target entity with offset and damping.
pub struct CameraRig {
    /// Entity to follow. The system reads this entity's `Transform` each frame.
    pub target_entity: Entity,
    /// Offset from target entity's position (world space).
    /// E.g., `Vec3::new(0.0, 1.6, 0.0)` for eye height.
    #[describe(hint = "World-space offset from target (e.g., eye height)")]
    pub offset: Vec3,
    /// Follow damping speed. Higher = faster follow.
    /// `0.0` = instant follow (no lag).
    /// Uses frame-rate independent formula: `1 - e^(-speed * dt)`.
    #[describe(hint = "Follow damping (0=instant)", range = "0.0..50.0", default = "0.0")]
    pub follow_damping: f32,
    /// Internal smoothed target position.
    pub(crate) smoothed_target: Vec3,
}

impl CameraRig {
    /// Create a new rig following the given entity.
    pub fn new(target_entity: Entity) -> Self {
        Self {
            target_entity,
            offset: Vec3::ZERO,
            follow_damping: 0.0,
            smoothed_target: Vec3::ZERO,
        }
    }

    /// Builder: set offset (e.g., eye height).
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Builder: set follow damping speed.
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.follow_damping = damping;
        self
    }
}

/// System that updates [`OrbitState.target`](crate::orbit::OrbitState) from
/// the [`CameraRig`]'s target entity `Transform`.
///
/// Runs in `PostUpdate`, before `camera_input_system`.
pub fn camera_rig_system(
    dt: Res<anvilkit_core::time::DeltaTime>,
    transforms: Query<&anvilkit_core::math::Transform, Without<CameraRig>>,
    mut rigs: Query<(&mut CameraRig, &mut super::OrbitState)>,
) {
    for (mut rig, mut orbit) in rigs.iter_mut() {
        let Ok(target_transform) = transforms.get(rig.target_entity) else {
            continue;
        };

        let desired = target_transform.translation + rig.offset;

        if rig.follow_damping > 0.0 {
            let factor = 1.0 - (-rig.follow_damping * dt.0).exp();
            let delta = (desired - rig.smoothed_target) * factor;
            rig.smoothed_target += delta;
        } else {
            rig.smoothed_target = desired;
        }

        orbit.target = rig.smoothed_target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let entity = Entity::from_raw(42);
        let rig = CameraRig::new(entity);
        assert_eq!(rig.target_entity, entity);
        assert_eq!(rig.offset, Vec3::ZERO);
        assert_eq!(rig.follow_damping, 0.0);
    }

    #[test]
    fn test_builders() {
        let entity = Entity::from_raw(1);
        let rig = CameraRig::new(entity)
            .with_offset(Vec3::new(0.0, 1.6, 0.0))
            .with_damping(10.0);
        assert_eq!(rig.offset, Vec3::new(0.0, 1.6, 0.0));
        assert_eq!(rig.follow_damping, 10.0);
    }
}
