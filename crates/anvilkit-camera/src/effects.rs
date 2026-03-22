use bevy_ecs::prelude::*;
use glam::Vec3;

/// Camera visual effects: shake, head bob, dynamic FOV.
#[derive(Component)]
pub struct CameraEffects {
    /// FOV dynamic offset (e.g. +10 when sprinting).
    pub fov_offset: f32,
    /// Target FOV offset (interpolated toward).
    pub fov_target: f32,
    /// FOV interpolation speed.
    pub fov_speed: f32,

    /// Current shake intensity.
    pub shake_intensity: f32,
    /// Shake decay rate.
    pub shake_decay: f32,
    /// Shake frequency (oscillations/sec).
    pub shake_frequency: f32,
    /// Internal shake timer.
    shake_timer: f32,

    /// Enable head bob when walking.
    pub head_bob_enabled: bool,
    /// Head bob vertical amplitude.
    pub head_bob_amplitude: f32,
    /// Head bob frequency (steps/sec).
    pub head_bob_frequency: f32,
    /// Internal bob timer.
    pub head_bob_timer: f32,
}

impl Default for CameraEffects {
    fn default() -> Self {
        Self {
            fov_offset: 0.0,
            fov_target: 0.0,
            fov_speed: 5.0,
            shake_intensity: 0.0,
            shake_decay: 5.0,
            shake_frequency: 15.0,
            shake_timer: 0.0,
            head_bob_enabled: false,
            head_bob_amplitude: 0.05,
            head_bob_frequency: 8.0,
            head_bob_timer: 0.0,
        }
    }
}

impl CameraEffects {
    /// Add a one-shot shake impulse.
    pub fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = self.shake_intensity.max(intensity);
    }

    /// Tick effects and return (position_offset, fov_offset).
    pub fn tick(&mut self, dt: f32, is_walking: bool) -> (Vec3, f32) {
        let mut pos_offset = Vec3::ZERO;

        // --- Shake ---
        if self.shake_intensity > 0.001 {
            self.shake_timer += dt;
            let t = self.shake_timer * self.shake_frequency * std::f32::consts::TAU;
            let sx = t.sin() * self.shake_intensity * 0.5;
            let sy = (t * 1.3).cos() * self.shake_intensity;
            pos_offset += Vec3::new(sx, sy, 0.0);
            self.shake_intensity = (self.shake_intensity - self.shake_decay * dt).max(0.0);
        } else {
            self.shake_intensity = 0.0;
            self.shake_timer = 0.0;
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
            self.head_bob_timer *= 0.9_f32.powf(dt * 60.0);
        }

        // --- FOV ---
        self.fov_offset += (self.fov_target - self.fov_offset) * (self.fov_speed * dt).min(1.0);

        (pos_offset, self.fov_offset)
    }
}
