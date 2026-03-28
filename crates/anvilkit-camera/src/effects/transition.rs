//! Camera transition component — smooth blending between camera states.
//!
//! When a camera mode switch occurs, the transition system captures a snapshot
//! of the current camera state and smoothly blends toward the new state over
//! a configurable duration with easing.

use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

/// Easing function type for camera transitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingType {
    /// Linear interpolation (constant speed).
    Linear,
    /// Smooth step: 3t^2 - 2t^3.
    SmoothStep,
    /// Smoother step: 6t^5 - 15t^4 + 10t^3.
    SmootherStep,
    /// Ease-in-out cubic.
    EaseInOutCubic,
    /// Ease-out quartic (fast start, slow end).
    EaseOutQuart,
}

impl EasingType {
    /// Evaluate the easing function at `t` in `[0, 1]`.
    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
            Self::SmootherStep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
        }
    }
}

impl Default for EasingType {
    fn default() -> Self {
        Self::SmoothStep
    }
}

/// Camera transition component for smooth mode/position blending.
///
/// When `active` is `true`, the system interpolates between the captured
/// source state and the camera's current desired state.
///
/// To trigger a transition, call [`start`](Self::start) with the camera's
/// current position, rotation, and FOV before changing the mode.
#[derive(Component)]
pub struct CameraTransition {
    /// Whether a transition is currently active.
    pub(crate) active: bool,
    /// Snapshot: source position.
    pub(crate) from_position: Vec3,
    /// Snapshot: source rotation.
    pub(crate) from_rotation: Quat,
    /// Snapshot: source FOV.
    pub(crate) from_fov: f32,
    /// Transition duration in seconds.
    pub duration: f32,
    /// Elapsed time.
    pub(crate) elapsed: f32,
    /// Easing function.
    pub easing: EasingType,
}

impl Default for CameraTransition {
    fn default() -> Self {
        Self {
            active: false,
            from_position: Vec3::ZERO,
            from_rotation: Quat::IDENTITY,
            from_fov: 70.0,
            duration: 0.5,
            elapsed: 0.0,
            easing: EasingType::SmoothStep,
        }
    }
}

impl CameraTransition {
    /// Create a transition with the given duration and easing.
    pub fn new(duration: f32, easing: EasingType) -> Self {
        Self {
            duration,
            easing,
            ..Default::default()
        }
    }

    /// Start a transition from the given camera state.
    ///
    /// Call this *before* switching the camera mode. The system will blend
    /// from this state toward whatever the new mode produces.
    pub fn start(&mut self, position: Vec3, rotation: Quat, fov: f32) {
        self.active = true;
        self.from_position = position;
        self.from_rotation = rotation;
        self.from_fov = fov;
        self.elapsed = 0.0;
    }

    /// Whether a transition is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current blend factor `[0, 1]` with easing applied.
    pub fn blend_factor(&self) -> f32 {
        if !self.active || self.duration <= 0.0 {
            return 1.0;
        }
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.easing.eval(t)
    }
}

/// Transition blending system.
///
/// When a `CameraTransition` is active, blends the camera's `Transform` and
/// `CameraComponent` between the snapshot and the current desired state.
pub fn camera_transition_system(
    dt: Res<anvilkit_ecs::physics::DeltaTime>,
    mut query: Query<(
        &mut CameraTransition,
        &mut anvilkit_core::math::Transform,
        &mut anvilkit_render::plugin::CameraComponent,
    )>,
) {
    for (mut transition, mut transform, mut cam) in query.iter_mut() {
        if !transition.active {
            continue;
        }

        transition.elapsed += dt.0;

        let t = transition.blend_factor();

        // Blend position
        let current_pos = transform.translation;
        transform.translation = transition.from_position.lerp(current_pos, t);

        // Blend rotation (slerp)
        let current_rot = transform.rotation;
        transform.rotation = transition.from_rotation.slerp(current_rot, t);

        // Blend FOV
        let current_fov = cam.fov;
        cam.fov = transition.from_fov + (current_fov - transition.from_fov) * t;

        // Complete transition
        if transition.elapsed >= transition.duration {
            transition.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_endpoints() {
        for easing in [
            EasingType::Linear,
            EasingType::SmoothStep,
            EasingType::SmootherStep,
            EasingType::EaseInOutCubic,
            EasingType::EaseOutQuart,
        ] {
            assert!(
                (easing.eval(0.0)).abs() < f32::EPSILON,
                "{easing:?} at 0.0"
            );
            assert!(
                (easing.eval(1.0) - 1.0).abs() < f32::EPSILON,
                "{easing:?} at 1.0"
            );
        }
    }

    #[test]
    fn test_easing_monotonic() {
        for easing in [
            EasingType::Linear,
            EasingType::SmoothStep,
            EasingType::SmootherStep,
            EasingType::EaseOutQuart,
        ] {
            let mut prev = 0.0;
            for i in 0..=100 {
                let t = i as f32 / 100.0;
                let v = easing.eval(t);
                assert!(v >= prev - f32::EPSILON, "{easing:?} not monotonic at t={t}");
                prev = v;
            }
        }
    }

    #[test]
    fn test_transition_start() {
        let mut t = CameraTransition::default();
        assert!(!t.is_active());
        t.start(Vec3::ONE, Quat::IDENTITY, 60.0);
        assert!(t.is_active());
        assert_eq!(t.from_position, Vec3::ONE);
    }

    #[test]
    fn test_transition_blend_factor_at_zero() {
        let mut t = CameraTransition::new(1.0, EasingType::Linear);
        t.start(Vec3::ZERO, Quat::IDENTITY, 70.0);
        assert!((t.blend_factor() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transition_blend_factor_at_end() {
        let mut t = CameraTransition::new(1.0, EasingType::Linear);
        t.start(Vec3::ZERO, Quat::IDENTITY, 70.0);
        t.elapsed = 1.0;
        assert!((t.blend_factor() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transition_inactive_returns_one() {
        let t = CameraTransition::default();
        assert!((t.blend_factor() - 1.0).abs() < f32::EPSILON);
    }
}
