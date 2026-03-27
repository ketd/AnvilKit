use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_render::plugin::CameraComponent;
use glam::Vec3;

use crate::controller::{CameraController, CameraMode};
use crate::effects::CameraEffects;

/// Resource for accumulated mouse delta per frame.
///
/// Deprecated: the engine now forwards raw mouse motion into [`InputState::mouse_delta()`]
/// automatically. Use `InputState::mouse_delta()` instead.
#[deprecated(note = "Use InputState::mouse_delta() instead — engine forwards DeviceEvent::MouseMotion automatically")]
pub struct MouseDelta {
    /// Horizontal mouse movement in pixels since last frame.
    pub dx: f32,
    /// Vertical mouse movement in pixels since last frame.
    pub dy: f32,
}

#[allow(deprecated)]
impl std::fmt::Debug for MouseDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseDelta").field("dx", &self.dx).field("dy", &self.dy).finish()
    }
}

#[allow(deprecated)]
impl Default for MouseDelta {
    fn default() -> Self { Self { dx: 0.0, dy: 0.0 } }
}

#[allow(deprecated)]
impl Resource for MouseDelta {}

/// Core camera controller system.
///
/// Reads mouse delta from [`InputState::mouse_delta()`], which is accumulated from
/// `DeviceEvent::MouseMotion` by the engine's event loop.
pub fn camera_controller_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut query: Query<(
        &mut CameraController,
        &mut Transform,
        &mut CameraComponent,
        Option<&mut CameraEffects>,
    )>,
) {
    let mouse_delta = input.mouse_delta();
    for (mut ctrl, mut transform, mut cam, effects) in query.iter_mut() {
        // --- Apply mouse look ---
        ctrl.yaw += mouse_delta.x * ctrl.mouse_sensitivity;
        ctrl.pitch = (ctrl.pitch + mouse_delta.y * ctrl.mouse_sensitivity)
            .clamp(ctrl.pitch_limits.0, ctrl.pitch_limits.1);

        // --- Smoothing ---
        if ctrl.smoothing > 0.0 {
            let factor = (1.0 - ctrl.smoothing).powf(dt.0 * 60.0);
            ctrl.smooth_yaw += (ctrl.yaw - ctrl.smooth_yaw) * (1.0 - factor);
            ctrl.smooth_pitch += (ctrl.pitch - ctrl.smooth_pitch) * (1.0 - factor);
        } else {
            ctrl.smooth_yaw = ctrl.yaw;
            ctrl.smooth_pitch = ctrl.pitch;
        }

        // Pre-compute values to avoid borrow conflicts with mode match
        let rotation = ctrl.rotation();
        let eff_yaw = ctrl.effective_yaw();
        let eff_pitch = ctrl.effective_pitch();
        let zoom_speed = ctrl.zoom_speed;
        let smoothing = ctrl.smoothing;
        let move_speed = ctrl.move_speed;
        let forward_xz = ctrl.forward_xz();
        let right_xz = ctrl.right_xz();
        let smooth_pos = ctrl.smooth_position;

        // Determine mode-specific behavior
        enum Action {
            SetRotation(glam::Quat),
            ThirdPerson {
                look_target: Vec3,
                new_distance: f32,
                new_smooth_pos: Vec3,
            },
            Free {
                dir: Vec3,
            },
        }

        let action = match &ctrl.mode {
            CameraMode::FirstPerson => {
                Action::SetRotation(rotation)
            }
            CameraMode::ThirdPerson {
                target,
                distance,
                min_distance,
                max_distance,
            } => {
                let mut dist = *distance;
                let scroll = input.scroll_delta();
                if scroll.abs() > 0.01 {
                    dist = (dist - scroll * zoom_speed).clamp(*min_distance, *max_distance);
                }

                let look_rotation = glam::Quat::from_rotation_y(eff_yaw)
                    * glam::Quat::from_rotation_x(eff_pitch);
                let offset = look_rotation * Vec3::new(0.0, 0.0, -dist);
                let look_target = *target + Vec3::new(0.0, 1.6, 0.0);
                let desired_pos = look_target + offset;

                let new_smooth_pos = if smoothing > 0.0 {
                    let factor = (1.0 - smoothing).powf(dt.0 * 60.0);
                    smooth_pos + (desired_pos - smooth_pos) * (1.0 - factor)
                } else {
                    desired_pos
                };

                Action::ThirdPerson {
                    look_target,
                    new_distance: dist,
                    new_smooth_pos,
                }
            }
            CameraMode::Free => {
                let mut dir = Vec3::ZERO;
                if input.is_key_pressed(KeyCode::W) { dir += forward_xz; }
                if input.is_key_pressed(KeyCode::S) { dir -= forward_xz; }
                if input.is_key_pressed(KeyCode::A) { dir -= right_xz; }
                if input.is_key_pressed(KeyCode::D) { dir += right_xz; }
                if input.is_key_pressed(KeyCode::Space) { dir.y += 1.0; }
                if input.is_key_pressed(KeyCode::LShift) { dir.y -= 1.0; }
                if dir.length_squared() > 0.0 { dir = dir.normalize(); }
                Action::Free { dir }
            }
        };

        // Apply action (now ctrl is no longer borrowed by match)
        match action {
            Action::SetRotation(rot) => {
                transform.rotation = rot;
            }
            Action::ThirdPerson { new_smooth_pos, look_target, new_distance, .. } => {
                ctrl.smooth_position = new_smooth_pos;
                transform.translation = new_smooth_pos;

                // Update distance in mode
                if let CameraMode::ThirdPerson { ref mut distance, .. } = &mut ctrl.mode {
                    *distance = new_distance;
                }

                // Look at target (right-handed: right = forward × up, up = right × forward)
                let look_dir = (look_target - transform.translation).normalize_or_zero();
                if look_dir.length_squared() > 0.5 {
                    let forward = look_dir;
                    // Handle degenerate case: forward nearly parallel to world up
                    let world_up = if forward.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
                    let right = forward.cross(world_up).normalize_or_zero();
                    let up = right.cross(forward).normalize_or_zero();
                    transform.rotation =
                        glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward));
                }
            }
            Action::Free { dir } => {
                transform.rotation = rotation;
                let speed = move_speed * dt.0;
                transform.translation += dir * speed;
            }
        }

        // --- Camera effects ---
        if let Some(mut fx) = effects {
            let has_move_input = input.is_key_pressed(KeyCode::W)
                || input.is_key_pressed(KeyCode::S)
                || input.is_key_pressed(KeyCode::A)
                || input.is_key_pressed(KeyCode::D);

            let (pos_offset, fov_offset) = fx.tick(dt.0, has_move_input);

            // Apply position offset in camera-local space
            let local_offset = transform.rotation * pos_offset;
            transform.translation += local_offset;

            // Apply FOV offset from the camera's configured base FOV
            let base_fov = ctrl.base_fov;
            cam.fov = base_fov + fov_offset;
        }
    }
}
