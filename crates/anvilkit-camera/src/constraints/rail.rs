//! Camera rail/dolly component — camera follows a predefined path.
//!
//! Define a path with control points, then attach `CameraRail` to a camera
//! entity with `CameraMode::Rail`. The system advances the camera along
//! the path based on `speed` or manual `t` control.

use bevy_ecs::prelude::*;
use glam::Vec3;

/// Interpolation mode for the rail path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RailInterpolation {
    /// Linear interpolation between control points.
    Linear,
    /// Catmull-Rom spline for smooth curves.
    CatmullRom {
        /// Tension parameter. 0.5 = standard, 0.0 = loose, 1.0 = tight.
        tension: f32,
    },
}

impl Default for RailInterpolation {
    fn default() -> Self {
        Self::CatmullRom { tension: 0.5 }
    }
}

/// Camera rail component for dolly/path-following cameras.
///
/// Attach to a camera entity with [`CameraMode::Rail`](crate::controller::CameraMode::Rail).
/// The system moves the camera along the path defined by `points`.
#[derive(Component)]
pub struct CameraRail {
    /// Control points defining the rail path (world space).
    pub points: Vec<Vec3>,
    /// Current progress along the rail `[0.0, 1.0]`.
    pub t: f32,
    /// Speed of progress (units of `t` per second). `0.0` = manual control only.
    pub speed: f32,
    /// Interpolation mode.
    pub interpolation: RailInterpolation,
    /// Whether to loop back to start when reaching the end.
    pub looping: bool,
}

impl Default for CameraRail {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            t: 0.0,
            speed: 0.0,
            interpolation: RailInterpolation::default(),
            looping: false,
        }
    }
}

impl CameraRail {
    /// Create a rail with the given control points.
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            points,
            ..Default::default()
        }
    }

    /// Builder: set speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Builder: set interpolation mode.
    pub fn with_interpolation(mut self, interpolation: RailInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    /// Builder: enable looping.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Evaluate the path position at the current `t`.
    pub fn evaluate(&self) -> Vec3 {
        self.evaluate_at(self.t)
    }

    /// Evaluate the path position at a given `t` in `[0, 1]`.
    pub fn evaluate_at(&self, t: f32) -> Vec3 {
        let n = self.points.len();
        if n == 0 {
            return Vec3::ZERO;
        }
        if n == 1 {
            return self.points[0];
        }

        let t = t.clamp(0.0, 1.0);
        let segment_count = n - 1;
        let scaled = t * segment_count as f32;
        let segment = (scaled as usize).min(segment_count - 1);
        let local_t = scaled - segment as f32;

        match self.interpolation {
            RailInterpolation::Linear => {
                self.points[segment].lerp(self.points[segment + 1], local_t)
            }
            RailInterpolation::CatmullRom { tension } => {
                let p0 = if segment > 0 {
                    self.points[segment - 1]
                } else {
                    self.points[0]
                };
                let p1 = self.points[segment];
                let p2 = self.points[segment + 1];
                let p3 = if segment + 2 < n {
                    self.points[segment + 2]
                } else {
                    self.points[n - 1]
                };
                catmull_rom(p0, p1, p2, p3, local_t, tension)
            }
        }
    }

    /// Compute the tangent (direction of motion) at a given `t`.
    pub fn tangent_at(&self, t: f32) -> Vec3 {
        let epsilon = 0.001;
        let a = self.evaluate_at((t - epsilon).max(0.0));
        let b = self.evaluate_at((t + epsilon).min(1.0));
        (b - a).normalize_or_zero()
    }
}

/// Catmull-Rom spline interpolation between p1 and p2.
fn catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32, tension: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    let s = tension;

    let m1 = s * (p2 - p0);
    let m2 = s * (p3 - p1);

    let a = 2.0 * t3 - 3.0 * t2 + 1.0;
    let b = t3 - 2.0 * t2 + t;
    let c = -2.0 * t3 + 3.0 * t2;
    let d = t3 - t2;

    p1 * a + m1 * b + p2 * c + m2 * d
}

/// Rail system — advances `CameraRail.t` and sets the camera transform.
pub fn camera_rail_system(
    dt: Res<anvilkit_ecs::physics::DeltaTime>,
    mut query: Query<(
        &mut CameraRail,
        &crate::controller::CameraController,
        &mut anvilkit_core::math::Transform,
    )>,
) {
    for (mut rail, ctrl, mut transform) in query.iter_mut() {
        if ctrl.mode != crate::controller::CameraMode::Rail {
            continue;
        }
        if rail.points.len() < 2 {
            continue;
        }

        // Advance t
        if rail.speed > 0.0 {
            rail.t += rail.speed * dt.0;
            if rail.looping {
                rail.t = rail.t.rem_euclid(1.0);
            } else {
                rail.t = rail.t.clamp(0.0, 1.0);
            }
        }

        // Set position
        transform.translation = rail.evaluate();

        // Orient along tangent
        let tangent = rail.tangent_at(rail.t);
        if tangent.length_squared() > 0.5 {
            let forward = tangent;
            let world_up = if forward.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
            let right = forward.cross(world_up).normalize_or_zero();
            let up = right.cross(forward).normalize_or_zero();
            transform.rotation = glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_rail() {
        let rail = CameraRail::default();
        assert_eq!(rail.evaluate(), Vec3::ZERO);
    }

    #[test]
    fn test_single_point() {
        let rail = CameraRail::new(vec![Vec3::ONE]);
        assert_eq!(rail.evaluate(), Vec3::ONE);
    }

    #[test]
    fn test_linear_midpoint() {
        let rail = CameraRail::new(vec![Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)])
            .with_interpolation(RailInterpolation::Linear);
        let pos = rail.evaluate_at(0.5);
        assert!((pos - Vec3::new(5.0, 0.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_linear_endpoints() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let rail = CameraRail::new(vec![a, b])
            .with_interpolation(RailInterpolation::Linear);
        assert!((rail.evaluate_at(0.0) - a).length() < 0.01);
        assert!((rail.evaluate_at(1.0) - b).length() < 0.01);
    }

    #[test]
    fn test_catmull_rom_passes_through_points() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 1.0, 0.0),
        ];
        let rail = CameraRail::new(points.clone());
        // At segment boundaries, should pass through control points
        let at_0 = rail.evaluate_at(0.0);
        assert!((at_0 - points[0]).length() < 0.01, "at_0 = {at_0}");
        let at_1 = rail.evaluate_at(1.0);
        assert!((at_1 - points[3]).length() < 0.01, "at_1 = {at_1}");
    }

    #[test]
    fn test_tangent() {
        let rail = CameraRail::new(vec![Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)])
            .with_interpolation(RailInterpolation::Linear);
        let tangent = rail.tangent_at(0.5);
        assert!((tangent - Vec3::new(1.0, 0.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_looping() {
        let mut rail = CameraRail::new(vec![Vec3::ZERO, Vec3::X])
            .with_speed(1.0)
            .with_looping(true);
        rail.t = 1.5;
        // After advancing, should wrap
        let wrapped = rail.t.rem_euclid(1.0);
        assert!((wrapped - 0.5).abs() < 0.01);
    }
}
