use bevy_ecs::prelude::*;
use glam::Vec3;

/// Camera control mode.
pub enum CameraMode {
    /// First-person: mouse directly controls yaw/pitch.
    FirstPerson,
    /// Third-person follow: orbit camera around a target point.
    ThirdPerson {
        /// World-space position the camera orbits around.
        target: Vec3,
        /// Current distance from the target.
        distance: f32,
        /// Minimum allowed orbit distance.
        min_distance: f32,
        /// Maximum allowed orbit distance.
        max_distance: f32,
    },
    /// Free fly: editor/debug camera.
    Free,
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::FirstPerson
    }
}

/// Camera controller component, attached to camera entities.
#[derive(Component)]
pub struct CameraController {
    /// Current camera control mode (first-person, third-person, or free).
    pub mode: CameraMode,
    /// Horizontal rotation angle in radians.
    pub yaw: f32,
    /// Vertical rotation angle in radians.
    pub pitch: f32,
    /// Minimum and maximum pitch values in radians (clamping range).
    pub pitch_limits: (f32, f32),
    /// Mouse look sensitivity multiplier.
    pub mouse_sensitivity: f32,
    /// Movement speed in units per second.
    pub move_speed: f32,
    /// Zoom speed multiplier for scroll-based zooming.
    pub zoom_speed: f32,
    /// Smoothing factor for camera interpolation (0.0 = no smoothing).
    pub smoothing: f32,
    /// Base FOV in degrees (used as the reference for effects offsets)
    pub base_fov: f32,
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
            smoothing: 0.0,
            base_fov: 70.0,
            smooth_yaw: 0.0,
            smooth_pitch: 0.0,
            smooth_position: Vec3::ZERO,
        }
    }
}

impl CameraController {
    /// Get the effective yaw (smoothed if smoothing > 0).
    pub fn effective_yaw(&self) -> f32 {
        if self.smoothing > 0.0 {
            self.smooth_yaw
        } else {
            self.yaw
        }
    }

    /// Get the effective pitch (smoothed if smoothing > 0).
    pub fn effective_pitch(&self) -> f32 {
        if self.smoothing > 0.0 {
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
    pub fn toggle_perspective(&mut self, player_pos: Vec3) {
        match &self.mode {
            CameraMode::FirstPerson => {
                self.mode = CameraMode::ThirdPerson {
                    target: player_pos,
                    distance: 5.0,
                    min_distance: 2.0,
                    max_distance: 20.0,
                };
                self.smoothing = 0.15;
            }
            CameraMode::ThirdPerson { .. } => {
                self.mode = CameraMode::FirstPerson;
                self.smoothing = 0.0;
            }
            CameraMode::Free => {
                self.mode = CameraMode::FirstPerson;
                self.smoothing = 0.0;
            }
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
        assert!(matches!(ctrl.mode, CameraMode::FirstPerson));
        ctrl.toggle_perspective(Vec3::ZERO);
        assert!(matches!(ctrl.mode, CameraMode::ThirdPerson { .. }));
        ctrl.toggle_perspective(Vec3::ZERO);
        assert!(matches!(ctrl.mode, CameraMode::FirstPerson));
    }

    #[test]
    fn test_pitch_limits() {
        let ctrl = CameraController::default();
        assert!(ctrl.pitch_limits.0 < 0.0);
        assert!(ctrl.pitch_limits.1 > 0.0);
    }
}
