//! Camera visual effects: trauma-based shake, head bob, dynamic FOV.
//!
//! Uses a trauma system (Squirrel Eiserloh, GDC 2016) with Perlin noise
//! for natural-looking camera shake that decays smoothly over time.

/// Inline gradient noise for camera effects.
pub mod noise;
/// Camera transition/blending.
pub mod transition;

use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};
use anvilkit_describe::Describe;

use self::noise::gradient_noise_2d;

/// Camera visual effects component.
///
/// Attach to a camera entity alongside [`CameraController`](crate::controller::CameraController)
/// to enable shake, head bob, and dynamic FOV.
///
/// # Shake (trauma system)
///
/// Instead of sine-wave oscillation, shake uses a **trauma** value in `[0, 1]`
/// that decays linearly over time. Actual displacement = `trauma^power * noise(time)`,
/// producing smooth, non-repetitive motion via Perlin noise.
///
/// Call [`add_trauma`](Self::add_trauma) to trigger shake (e.g., on landing, explosion, hit).
#[derive(Component, Describe)]
/// Camera visual effects: shake, head bob, and dynamic FOV.
pub struct CameraEffects {
    // --- Trauma-based shake ---
    /// Current trauma level `[0.0, 1.0]`. Higher = stronger shake.
    pub trauma: f32,
    /// Trauma decay rate (units per second). Default: 1.5.
    pub trauma_decay: f32,
    /// Power curve exponent. shake = trauma^power. Default: 2.0 (quadratic).
    pub trauma_power: f32,
    /// Maximum positional displacement per axis (camera-local space).
    pub shake_max_offset: Vec3,
    /// Maximum rotational displacement (yaw, pitch) in radians.
    pub shake_max_rotation: Vec2,
    /// Noise sampling speed (higher = faster shake oscillation). Default: 8.0.
    pub shake_noise_speed: f32,
    /// Internal noise time accumulator.
    pub(crate) shake_time: f32,

    // --- Head bob ---
    /// Enable head bob when walking.
    pub head_bob_enabled: bool,
    /// Head bob vertical amplitude.
    pub head_bob_amplitude: f32,
    /// Head bob frequency (steps/sec).
    pub head_bob_frequency: f32,
    /// Internal bob timer.
    pub(crate) head_bob_timer: f32,

    // --- Dynamic FOV ---
    /// Current FOV offset (interpolating toward `fov_target`).
    pub fov_offset: f32,
    /// Target FOV offset (e.g., +10 when sprinting).
    pub fov_target: f32,
    /// FOV interpolation speed.
    pub fov_speed: f32,
}

impl Default for CameraEffects {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            trauma_decay: 1.5,
            trauma_power: 2.0,
            shake_max_offset: Vec3::new(0.15, 0.15, 0.05),
            shake_max_rotation: Vec2::new(0.03, 0.03),
            shake_noise_speed: 8.0,
            shake_time: 0.0,

            head_bob_enabled: false,
            head_bob_amplitude: 0.05,
            head_bob_frequency: 8.0,
            head_bob_timer: 0.0,

            fov_offset: 0.0,
            fov_target: 0.0,
            fov_speed: 5.0,
        }
    }
}

/// Output of [`CameraEffects::tick`]: offsets to apply to the camera.
pub struct EffectsOutput {
    /// Positional offset in camera-local space.
    pub position_offset: Vec3,
    /// Rotational offset (yaw, pitch) in radians.
    pub rotation_offset: Vec2,
    /// FOV offset in degrees.
    pub fov_offset: f32,
}

// Noise seed offsets — separate per axis to prevent correlated motion.
const SEED_X: f32 = 0.0;
const SEED_Y: f32 = 100.0;
const SEED_Z: f32 = 200.0;
const SEED_YAW: f32 = 300.0;
const SEED_PITCH: f32 = 400.0;

impl CameraEffects {
    /// Add trauma (camera shake impulse). Clamps to `[0, 1]`.
    ///
    /// Trauma decays linearly at `trauma_decay` units/second.
    /// Actual shake intensity = `trauma^trauma_power`, so small impacts
    /// feel gentle while large ones feel catastrophic.
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    /// Backward-compatible alias for [`add_trauma`](Self::add_trauma).
    ///
    /// Maps the old `intensity` parameter (which was used directly as displacement)
    /// to the new trauma system. Values are clamped to `[0, 1]`.
    pub fn add_shake(&mut self, intensity: f32) {
        self.add_trauma(intensity);
    }

    /// Tick all effects and return the combined offsets to apply.
    ///
    /// Call once per frame with the frame delta time and whether the player is walking.
    pub fn tick(&mut self, dt: f32, is_walking: bool) -> (Vec3, f32) {
        let output = self.tick_full(dt, is_walking);
        (output.position_offset, output.fov_offset)
    }

    /// Tick all effects and return full output including rotation offsets.
    pub fn tick_full(&mut self, dt: f32, is_walking: bool) -> EffectsOutput {
        let mut pos_offset = Vec3::ZERO;
        let mut rot_offset = Vec2::ZERO;

        // --- Trauma shake ---
        if self.trauma > 0.001 {
            self.shake_time += dt * self.shake_noise_speed;
            let shake = self.trauma.powf(self.trauma_power);

            // Sample Perlin noise at different seeds for each axis
            let t = self.shake_time;
            let nx = gradient_noise_2d(t + SEED_X, SEED_X);
            let ny = gradient_noise_2d(t + SEED_Y, SEED_Y);
            let nz = gradient_noise_2d(t + SEED_Z, SEED_Z);
            let nyaw = gradient_noise_2d(t + SEED_YAW, SEED_YAW);
            let npitch = gradient_noise_2d(t + SEED_PITCH, SEED_PITCH);

            pos_offset.x += shake * self.shake_max_offset.x * nx;
            pos_offset.y += shake * self.shake_max_offset.y * ny;
            pos_offset.z += shake * self.shake_max_offset.z * nz;
            rot_offset.x += shake * self.shake_max_rotation.x * nyaw;
            rot_offset.y += shake * self.shake_max_rotation.y * npitch;

            // Decay trauma linearly
            self.trauma = (self.trauma - self.trauma_decay * dt).max(0.0);
        } else {
            self.trauma = 0.0;
            self.shake_time = 0.0;
        }

        // --- Head bob ---
        if self.head_bob_enabled && is_walking {
            self.head_bob_timer += dt * self.head_bob_frequency;
            let bob_y = (self.head_bob_timer * std::f32::consts::TAU).sin()
                * self.head_bob_amplitude;
            let bob_x = (self.head_bob_timer * std::f32::consts::PI).cos()
                * self.head_bob_amplitude * 0.5;
            pos_offset += Vec3::new(bob_x, bob_y, 0.0);
        } else {
            // Smoothly return to center
            self.head_bob_timer *= (-4.0_f32 * dt).exp();
        }

        // --- FOV ---
        self.fov_offset += (self.fov_target - self.fov_offset) * (self.fov_speed * dt).min(1.0);

        EffectsOutput {
            position_offset: pos_offset,
            rotation_offset: rot_offset,
            fov_offset: self.fov_offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_shake() {
        let mut fx = CameraEffects::default();
        let (pos, fov) = fx.tick(1.0 / 60.0, false);
        assert_eq!(pos, Vec3::ZERO);
        assert_eq!(fov, 0.0);
    }

    #[test]
    fn test_add_trauma_clamps() {
        let mut fx = CameraEffects::default();
        fx.add_trauma(0.5);
        assert!((fx.trauma - 0.5).abs() < f32::EPSILON);
        fx.add_trauma(0.8);
        assert!((fx.trauma - 1.0).abs() < f32::EPSILON); // clamped
    }

    #[test]
    fn test_add_shake_alias() {
        let mut fx = CameraEffects::default();
        fx.add_shake(0.3);
        assert!((fx.trauma - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trauma_decays() {
        let mut fx = CameraEffects::default();
        fx.add_trauma(1.0);
        for _ in 0..120 {
            fx.tick(1.0 / 60.0, false);
        }
        assert!(fx.trauma < 0.01, "trauma should decay to near-zero, got {}", fx.trauma);
    }

    #[test]
    fn test_shake_produces_displacement() {
        let mut fx = CameraEffects::default();
        fx.add_trauma(1.0);
        let (pos, _) = fx.tick(1.0 / 60.0, false);
        let magnitude = pos.length();
        assert!(magnitude > 0.0, "shake with trauma=1 should produce displacement");
    }

    #[test]
    fn test_shake_varies_over_time() {
        let mut fx = CameraEffects::default();
        fx.trauma = 1.0;
        fx.trauma_decay = 0.0; // no decay to keep shake constant
        let (pos1, _) = fx.tick(1.0 / 60.0, false);
        let (pos2, _) = fx.tick(1.0 / 60.0, false);
        // Positions should differ between frames (noise varies)
        assert_ne!(pos1, pos2, "shake should vary between frames");
    }

    #[test]
    fn test_head_bob_when_walking() {
        let mut fx = CameraEffects::default();
        fx.head_bob_enabled = true;
        // Run several frames to accumulate timer
        for _ in 0..10 {
            fx.tick(1.0 / 60.0, true);
        }
        let (pos, _) = fx.tick(1.0 / 60.0, true);
        assert!(pos.length() > 0.0, "head bob should produce displacement when walking");
    }

    #[test]
    fn test_head_bob_off_when_disabled() {
        let mut fx = CameraEffects::default();
        fx.head_bob_enabled = false;
        let (pos, _) = fx.tick(1.0 / 60.0, true);
        assert_eq!(pos, Vec3::ZERO);
    }

    #[test]
    fn test_fov_interpolation() {
        let mut fx = CameraEffects::default();
        fx.fov_target = 10.0;
        for _ in 0..120 {
            fx.tick(1.0 / 60.0, false);
        }
        assert!(
            (fx.fov_offset - 10.0).abs() < 0.5,
            "FOV should converge toward target, got {}",
            fx.fov_offset
        );
    }

    #[test]
    fn test_tick_full_includes_rotation() {
        let mut fx = CameraEffects::default();
        fx.add_trauma(1.0);
        let output = fx.tick_full(1.0 / 60.0, false);
        let rot_magnitude = output.rotation_offset.length();
        assert!(rot_magnitude > 0.0, "trauma shake should produce rotation offset");
    }

    #[test]
    fn test_zero_trauma_resets_timer() {
        let mut fx = CameraEffects::default();
        fx.add_trauma(0.1);
        fx.trauma_decay = 100.0; // fast decay
        fx.tick(1.0 / 60.0, false);
        fx.tick(1.0 / 60.0, false);
        assert_eq!(fx.shake_time, 0.0, "timer should reset when trauma reaches zero");
    }
}
