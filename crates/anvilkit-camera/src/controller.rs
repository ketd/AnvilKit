use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_describe::Describe;

use crate::input_curve::InputCurve;

/// Camera control mode.
///
/// Defines the behavioral mode for a camera entity. Each mode determines how
/// the camera responds to input and computes its transform.
///
/// Modes that require orbit data (`ThirdPerson`, `Orbit`) need an
/// [`OrbitState`](crate::orbit::OrbitState) component on the same entity.
/// `ThirdPerson` additionally needs a [`CameraRig`](crate::orbit::rig::CameraRig)
/// to automatically follow a target entity.
///
/// `Rail` mode requires a [`CameraRail`](crate::constraints::rail::CameraRail) component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Describe)]
/// Camera behavioral mode.
pub enum CameraMode {
    /// First-person: mouse directly controls yaw/pitch.
    FirstPerson,
    /// Third-person follow: orbit around a followed target.
    /// Requires `OrbitState` + `CameraRig` on the same entity.
    ThirdPerson,
    /// Orbit: orbit around a fixed point (inspection/editor).
    /// Requires `OrbitState` on the same entity.
    Orbit,
    /// Free fly: 6DOF editor/debug camera.
    Free,
    /// Rail: camera follows a predefined path.
    /// Requires `CameraRail` on the same entity.
    Rail,
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::FirstPerson
    }
}

/// Camera controller component, attached to camera entities.
///
/// Controls how the camera responds to mouse/gamepad input and moves through
/// the world. The actual behavior depends on the [`CameraMode`].
#[derive(Component, Describe)]
/// Camera controller with mode, sensitivity, and smoothing.
pub struct CameraController {
    /// Current camera control mode.
    pub mode: CameraMode,
    /// Horizontal rotation angle in radians.
    #[describe(hint = "Yaw angle in radians")]
    pub yaw: f32,
    /// Vertical rotation angle in radians.
    #[describe(hint = "Pitch angle in radians")]
    pub pitch: f32,
    /// Minimum and maximum pitch values in radians (clamping range).
    pub pitch_limits: (f32, f32),
    /// Mouse look sensitivity multiplier.
    #[describe(hint = "Mouse look sensitivity", range = "0.0001..0.1", default = "0.003")]
    pub mouse_sensitivity: f32,
    /// Movement speed in units per second (Free mode).
    #[describe(hint = "Free-fly movement speed", range = "0.1..200.0", default = "10.0")]
    pub move_speed: f32,
    /// Zoom speed multiplier for scroll-based zooming.
    #[describe(hint = "Scroll zoom speed", range = "0.1..10.0", default = "1.0")]
    pub zoom_speed: f32,
    /// Base FOV in degrees (used as the reference for effects offsets).
    #[describe(hint = "Base vertical FOV in degrees", range = "30.0..120.0", default = "70.0")]
    pub base_fov: f32,

    /// Smoothing speed for interpolation.
    /// `0.0` = instant (no smoothing), higher values = smoother but laggier.
    /// Uses frame-rate independent formula: `1 - e^(-speed * dt)`.
    #[describe(hint = "Camera smoothing speed (0=instant)", range = "0.0..50.0", default = "0.0")]
    pub smoothing_speed: f32,

    /// Input response curve for mouse look.
    pub input_curve: InputCurve,

    // Internal smooth state
    pub(crate) smooth_yaw: f32,
    pub(crate) smooth_pitch: f32,
    pub(crate) smooth_position: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: CameraMode::FirstPerson,
            yaw: 0.0,
            pitch: 0.0,
            pitch_limits: (-1.48, 1.48), // ~85 degrees
            mouse_sensitivity: 0.003,
            move_speed: 10.0,
            zoom_speed: 2.0,
            base_fov: 70.0,
            smoothing_speed: 0.0,
            input_curve: InputCurve::linear(),
            smooth_yaw: 0.0,
            smooth_pitch: 0.0,
            smooth_position: Vec3::ZERO,
        }
    }
}

impl CameraController {
    /// Get the effective yaw (smoothed if smoothing_speed > 0).
    pub fn effective_yaw(&self) -> f32 {
        if self.smoothing_speed > 0.0 {
            self.smooth_yaw
        } else {
            self.yaw
        }
    }

    /// Get the effective pitch (smoothed if smoothing_speed > 0).
    pub fn effective_pitch(&self) -> f32 {
        if self.smoothing_speed > 0.0 {
            self.smooth_pitch
        } else {
            self.pitch
        }
    }

    /// Compute rotation quaternion from effective yaw/pitch.
    pub fn rotation(&self) -> glam::Quat {
        glam::Quat::from_rotation_y(self.effective_yaw())
            * glam::Quat::from_rotation_x(self.effective_pitch())
    }

    /// Compute forward direction (yaw-only, for movement).
    pub fn forward_xz(&self) -> Vec3 {
        glam::Quat::from_rotation_y(self.effective_yaw()) * Vec3::Z
    }

    /// Compute right direction (yaw-only, for movement).
    pub fn right_xz(&self) -> Vec3 {
        glam::Quat::from_rotation_y(self.effective_yaw()) * Vec3::X
    }

    /// Toggle between FirstPerson and ThirdPerson modes.
    ///
    /// When switching to ThirdPerson, enables smoothing. When switching back,
    /// disables smoothing. Returns the new mode for callers that need to
    /// insert/remove `OrbitState` and `CameraRig` components.
    pub fn toggle_perspective(&mut self) -> CameraMode {
        match self.mode {
            CameraMode::FirstPerson => {
                self.mode = CameraMode::ThirdPerson;
                self.smoothing_speed = 8.0;
            }
            _ => {
                self.mode = CameraMode::FirstPerson;
                self.smoothing_speed = 0.0;
            }
        }
        self.mode
    }

    /// Apply frame-rate independent smoothing to yaw/pitch.
    pub(crate) fn update_smoothing(&mut self, dt: f32) {
        if self.smoothing_speed > 0.0 {
            let factor = 1.0 - (-self.smoothing_speed * dt).exp();
            self.smooth_yaw += (self.yaw - self.smooth_yaw) * factor;
            self.smooth_pitch += (self.pitch - self.smooth_pitch) * factor;
        } else {
            self.smooth_yaw = self.yaw;
            self.smooth_pitch = self.pitch;
        }
    }

    /// Apply frame-rate independent smoothing to position.
    pub(crate) fn smooth_toward(&mut self, target: Vec3, dt: f32) -> Vec3 {
        if self.smoothing_speed > 0.0 {
            let factor = 1.0 - (-self.smoothing_speed * dt).exp();
            self.smooth_position += (target - self.smooth_position) * factor;
            self.smooth_position
        } else {
            self.smooth_position = target;
            target
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_controller() {
        let ctrl = CameraController::default();
        assert_eq!(ctrl.yaw, 0.0);
        assert_eq!(ctrl.pitch, 0.0);
        assert_eq!(ctrl.base_fov, 70.0);
        assert!(matches!(ctrl.mode, CameraMode::FirstPerson));
    }

    #[test]
    fn test_rotation_identity_at_zero() {
        let ctrl = CameraController::default();
        let rot = ctrl.rotation();
        let diff = rot.dot(glam::Quat::IDENTITY);
        assert!((diff - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_forward_xz_at_zero_yaw() {
        let ctrl = CameraController::default();
        let fwd = ctrl.forward_xz();
        assert!(fwd.z.abs() > 0.99);
    }

    #[test]
    fn test_toggle_perspective() {
        let mut ctrl = CameraController::default();
        assert_eq!(ctrl.mode, CameraMode::FirstPerson);
        let new_mode = ctrl.toggle_perspective();
        assert_eq!(new_mode, CameraMode::ThirdPerson);
        assert_eq!(ctrl.mode, CameraMode::ThirdPerson);
        let new_mode = ctrl.toggle_perspective();
        assert_eq!(new_mode, CameraMode::FirstPerson);
    }

    #[test]
    fn test_pitch_limits() {
        let ctrl = CameraController::default();
        assert!(ctrl.pitch_limits.0 < 0.0);
        assert!(ctrl.pitch_limits.1 > 0.0);
    }

    #[test]
    fn test_smoothing_frame_rate_independence() {
        // Run at 30fps for 1 second
        let mut ctrl_30 = CameraController::default();
        ctrl_30.smoothing_speed = 5.0;
        ctrl_30.yaw = 1.0;
        for _ in 0..30 {
            ctrl_30.update_smoothing(1.0 / 30.0);
        }

        // Run at 60fps for 1 second
        let mut ctrl_60 = CameraController::default();
        ctrl_60.smoothing_speed = 5.0;
        ctrl_60.yaw = 1.0;
        for _ in 0..60 {
            ctrl_60.update_smoothing(1.0 / 60.0);
        }

        // Both should converge to roughly the same value
        let diff = (ctrl_30.smooth_yaw - ctrl_60.smooth_yaw).abs();
        assert!(
            diff < 0.01,
            "30fps={}, 60fps={}, diff={}",
            ctrl_30.smooth_yaw,
            ctrl_60.smooth_yaw,
            diff
        );
    }

    #[test]
    fn test_smooth_toward() {
        let mut ctrl = CameraController::default();
        ctrl.smoothing_speed = 10.0;
        let target = Vec3::new(10.0, 5.0, 0.0);
        for _ in 0..120 {
            ctrl.smooth_toward(target, 1.0 / 60.0);
        }
        let dist = (ctrl.smooth_position - target).length();
        assert!(dist < 0.01, "should converge to target, dist={dist}");
    }

    #[test]
    fn test_camera_mode_eq() {
        assert_eq!(CameraMode::FirstPerson, CameraMode::FirstPerson);
        assert_ne!(CameraMode::FirstPerson, CameraMode::Free);
    }
}
