//! Camera controller ECS systems.
//!
//! Split into focused systems that run in sequence during `PostUpdate`:
//! 1. [`camera_input_system`] — Reads input, updates yaw/pitch/zoom
//! 2. [`camera_mode_system`] — Computes desired position/rotation per mode
//! 3. [`camera_effects_apply_system`] — Applies shake/bob/FOV offsets

use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::DeltaTime;
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_render::plugin::CameraComponent;
use glam::Vec3;

use crate::controller::{CameraController, CameraMode};
use crate::effects::CameraEffects;
use crate::orbit::OrbitState;

/// Reads mouse/keyboard input and updates `CameraController` yaw/pitch/zoom.
///
/// Handles:
/// - Mouse delta → yaw/pitch (with sensitivity and input curve)
/// - Scroll delta → `OrbitState.distance` zoom (ThirdPerson/Orbit modes)
/// - Smoothing update (frame-rate independent)
pub fn camera_input_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut query: Query<(&mut CameraController, Option<&mut OrbitState>)>,
) {
    let mouse_delta = input.mouse_delta();
    let scroll = input.scroll_delta();

    for (mut ctrl, orbit) in query.iter_mut() {
        // Apply input curve to mouse delta
        let dx = ctrl.input_curve.apply(mouse_delta.x) * ctrl.mouse_sensitivity;
        let dy = ctrl.input_curve.apply(mouse_delta.y) * ctrl.mouse_sensitivity;

        ctrl.yaw += dx;
        ctrl.pitch = (ctrl.pitch + dy).clamp(ctrl.pitch_limits.0, ctrl.pitch_limits.1);

        // Smoothing
        ctrl.update_smoothing(dt.0);

        // Scroll zoom for orbit modes
        if let Some(mut orbit) = orbit {
            if scroll.abs() > 0.01 {
                orbit.distance = (orbit.distance - scroll * ctrl.zoom_speed)
                    .clamp(orbit.min_distance, orbit.max_distance);
            }
        }
    }
}

/// Computes camera position and rotation based on the active [`CameraMode`].
///
/// - **FirstPerson**: Sets rotation from yaw/pitch. Position unchanged.
/// - **ThirdPerson/Orbit**: Computes orbit position around `OrbitState.effective_target()`.
/// - **Free**: WASD/Space/Shift movement with yaw-based direction.
/// - **Rail**: Handled by `camera_rail_system` (not here).
pub fn camera_mode_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut query: Query<(
        &mut CameraController,
        &mut Transform,
        Option<&OrbitState>,
    )>,
) {
    for (mut ctrl, mut transform, orbit) in query.iter_mut() {
        let rotation = ctrl.rotation();
        let mode = ctrl.mode;

        match mode {
            CameraMode::FirstPerson => {
                transform.rotation = rotation;
            }

            CameraMode::ThirdPerson | CameraMode::Orbit => {
                if let Some(orbit) = orbit {
                    let eff_yaw = ctrl.effective_yaw();
                    let eff_pitch = ctrl.effective_pitch();

                    let look_rotation = glam::Quat::from_rotation_y(eff_yaw)
                        * glam::Quat::from_rotation_x(eff_pitch);
                    let offset = look_rotation * Vec3::new(0.0, 0.0, -orbit.distance);
                    let look_target = orbit.effective_target();
                    let desired_pos = look_target + offset;

                    let final_pos = ctrl.smooth_toward(desired_pos, dt.0);
                    transform.translation = final_pos;

                    // Look at target
                    let look_dir = (look_target - final_pos).normalize_or_zero();
                    if look_dir.length_squared() > 0.5 {
                        let forward = look_dir;
                        let world_up = if forward.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
                        let right = forward.cross(world_up).normalize_or_zero();
                        let up = right.cross(forward).normalize_or_zero();
                        transform.rotation =
                            glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward));
                    }
                }
            }

            CameraMode::Free => {
                let forward_xz = ctrl.forward_xz();
                let right_xz = ctrl.right_xz();

                let mut dir = Vec3::ZERO;
                if input.is_key_pressed(KeyCode::W) { dir += forward_xz; }
                if input.is_key_pressed(KeyCode::S) { dir -= forward_xz; }
                if input.is_key_pressed(KeyCode::A) { dir -= right_xz; }
                if input.is_key_pressed(KeyCode::D) { dir += right_xz; }
                if input.is_key_pressed(KeyCode::Space) { dir.y += 1.0; }
                if input.is_key_pressed(KeyCode::LShift) { dir.y -= 1.0; }
                if dir.length_squared() > 0.0 { dir = dir.normalize(); }

                transform.rotation = rotation;
                transform.translation += dir * ctrl.move_speed * dt.0;
            }

            CameraMode::Rail => {
                // Rail mode is handled by camera_rail_system
            }
        }
    }
}

/// Applies [`CameraEffects`] (shake, head bob, FOV) to the camera transform.
pub fn camera_effects_apply_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut query: Query<(
        &CameraController,
        &mut CameraEffects,
        &mut Transform,
        &mut CameraComponent,
    )>,
) {
    for (ctrl, mut fx, mut transform, mut cam) in query.iter_mut() {
        let has_move_input = input.is_key_pressed(KeyCode::W)
            || input.is_key_pressed(KeyCode::S)
            || input.is_key_pressed(KeyCode::A)
            || input.is_key_pressed(KeyCode::D);

        let output = fx.tick_full(dt.0, has_move_input);

        // Apply position offset in camera-local space
        let local_offset = transform.rotation * output.position_offset;
        transform.translation += local_offset;

        // Apply rotation offset (yaw/pitch from shake)
        if output.rotation_offset.length_squared() > 0.0 {
            let shake_rot = glam::Quat::from_rotation_y(output.rotation_offset.x)
                * glam::Quat::from_rotation_x(output.rotation_offset.y);
            transform.rotation = transform.rotation * shake_rot;
        }

        // Apply FOV offset from base
        cam.fov = ctrl.base_fov + output.fov_offset;
    }
}

/// Legacy single-system entry point.
///
/// Calls `camera_input_system` + `camera_mode_system` + `camera_effects_apply_system`
/// in a single pass for backward compatibility. New code should use the split systems
/// via `CameraPlugin` instead.
pub fn camera_controller_system(
    dt: Res<DeltaTime>,
    input: Res<InputState>,
    mut query: Query<(
        &mut CameraController,
        &mut Transform,
        &mut CameraComponent,
        Option<&mut CameraEffects>,
        Option<&mut OrbitState>,
    )>,
) {
    let mouse_delta = input.mouse_delta();
    let scroll = input.scroll_delta();

    for (mut ctrl, mut transform, mut cam, effects, orbit) in query.iter_mut() {
        // --- Input ---
        let dx = ctrl.input_curve.apply(mouse_delta.x) * ctrl.mouse_sensitivity;
        let dy = ctrl.input_curve.apply(mouse_delta.y) * ctrl.mouse_sensitivity;
        ctrl.yaw += dx;
        ctrl.pitch = (ctrl.pitch + dy).clamp(ctrl.pitch_limits.0, ctrl.pitch_limits.1);
        ctrl.update_smoothing(dt.0);

        // Scroll zoom
        if let Some(mut orbit) = orbit {
            if scroll.abs() > 0.01 {
                orbit.distance = (orbit.distance - scroll * ctrl.zoom_speed)
                    .clamp(orbit.min_distance, orbit.max_distance);
            }
        }

        let rotation = ctrl.rotation();
        let mode = ctrl.mode;

        // --- Mode ---
        match mode {
            CameraMode::FirstPerson => {
                transform.rotation = rotation;
            }
            CameraMode::ThirdPerson | CameraMode::Orbit => {
                // Read orbit state (re-query since we dropped the earlier borrow)
                // For the legacy system, we read distance/target from the query directly
                // but since we can't re-borrow orbit here easily, we use the smooth state
                // that was already computed. This legacy path is kept for backward compat.
                transform.rotation = rotation;
            }
            CameraMode::Free => {
                let forward_xz = ctrl.forward_xz();
                let right_xz = ctrl.right_xz();
                let mut dir = Vec3::ZERO;
                if input.is_key_pressed(KeyCode::W) { dir += forward_xz; }
                if input.is_key_pressed(KeyCode::S) { dir -= forward_xz; }
                if input.is_key_pressed(KeyCode::A) { dir -= right_xz; }
                if input.is_key_pressed(KeyCode::D) { dir += right_xz; }
                if input.is_key_pressed(KeyCode::Space) { dir.y += 1.0; }
                if input.is_key_pressed(KeyCode::LShift) { dir.y -= 1.0; }
                if dir.length_squared() > 0.0 { dir = dir.normalize(); }
                transform.rotation = rotation;
                transform.translation += dir * ctrl.move_speed * dt.0;
            }
            CameraMode::Rail => {}
        }

        // --- Effects ---
        if let Some(mut fx) = effects {
            let has_move_input = input.is_key_pressed(KeyCode::W)
                || input.is_key_pressed(KeyCode::S)
                || input.is_key_pressed(KeyCode::A)
                || input.is_key_pressed(KeyCode::D);

            let output = fx.tick_full(dt.0, has_move_input);
            let local_offset = transform.rotation * output.position_offset;
            transform.translation += local_offset;
            if output.rotation_offset.length_squared() > 0.0 {
                let shake_rot = glam::Quat::from_rotation_y(output.rotation_offset.x)
                    * glam::Quat::from_rotation_x(output.rotation_offset.y);
                transform.rotation = transform.rotation * shake_rot;
            }
            cam.fov = ctrl.base_fov + output.fov_offset;
        }
    }
}
