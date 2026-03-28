//! Spring arm component — collision avoidance for third-person cameras.
//!
//! Inspired by Unreal Engine's `USpringArmComponent` and Godot's `SpringArm3D`.
//! Performs a ray/sphere sweep from the orbit target toward the desired camera
//! position. When the sweep hits geometry, the camera is pulled closer to avoid
//! clipping through walls.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::OrbitState;

/// Spring arm component for camera collision avoidance.
///
/// Attach to the same entity as [`OrbitState`] to enable automatic distance
/// adjustment when obstacles are between the orbit target and the camera.
///
/// The system performs a ray cast from the orbit target along the camera's
/// orbit direction. If the ray hits something within the orbit distance,
/// the camera is pulled to `hit_distance - margin` to avoid clipping.
#[derive(Component)]
pub struct SpringArm {
    /// Sphere radius for the sweep test (camera's effective collision radius).
    pub probe_radius: f32,
    /// Safety margin subtracted from hit distance.
    pub margin: f32,
    /// How quickly the arm retracts toward collision point (units/sec).
    pub retract_speed: f32,
    /// How quickly the arm extends back to full length after collision clears (units/sec).
    pub extend_speed: f32,
    /// Current actual distance (may be shorter than `OrbitState.distance` due to collision).
    pub(crate) current_distance: f32,
    /// Whether the arm is currently retracted due to collision.
    pub(crate) is_colliding: bool,
}

impl Default for SpringArm {
    fn default() -> Self {
        Self {
            probe_radius: 0.3,
            margin: 0.2,
            retract_speed: 30.0,
            extend_speed: 5.0,
            current_distance: 0.0,
            is_colliding: false,
        }
    }
}

impl SpringArm {
    /// Create a spring arm with the given probe radius.
    pub fn new(probe_radius: f32) -> Self {
        Self {
            probe_radius,
            ..Default::default()
        }
    }

    /// Builder: set margin.
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Builder: set retract/extend speeds.
    pub fn with_speeds(mut self, retract: f32, extend: f32) -> Self {
        self.retract_speed = retract;
        self.extend_speed = extend;
        self
    }

    /// Get the current effective distance (accounting for collision).
    pub fn effective_distance(&self) -> f32 {
        self.current_distance
    }

    /// Check if the arm is currently retracted due to collision.
    pub fn is_colliding(&self) -> bool {
        self.is_colliding
    }
}

/// Spring arm collision system.
///
/// For each entity with `SpringArm` + `OrbitState` + `Transform`, performs
/// a simple ray cast from the orbit target along the camera direction and
/// adjusts the camera's position if geometry is in the way.
///
/// Uses `AabbCollider` entities for collision testing.
pub fn camera_spring_arm_system(
    dt: Res<anvilkit_ecs::physics::DeltaTime>,
    mut cameras: Query<(
        &mut SpringArm,
        &OrbitState,
        &mut anvilkit_core::math::Transform,
    )>,
    colliders: Query<(
        &anvilkit_core::math::Transform,
        &anvilkit_ecs::physics::AabbCollider,
    ), Without<SpringArm>>,
) {
    for (mut arm, orbit, mut cam_transform) in cameras.iter_mut() {
        let look_target = orbit.effective_target();
        let cam_pos = cam_transform.translation;
        let to_camera = cam_pos - look_target;
        let direction = to_camera.normalize_or_zero();

        if direction.length_squared() < 0.5 {
            arm.current_distance = orbit.distance;
            arm.is_colliding = false;
            continue;
        }

        let desired_dist = orbit.distance;
        let mut nearest_hit = desired_dist;

        // Test against all AABB colliders
        for (collider_transform, aabb) in colliders.iter() {
            let center = collider_transform.translation;
            let half = aabb.half_extents;

            // Quick sphere-AABB distance check
            if let Some(t) = ray_aabb_intersection(
                look_target,
                direction,
                center - half,
                center + half,
            ) {
                if t > 0.0 && t < nearest_hit {
                    nearest_hit = (t - arm.margin).max(arm.probe_radius);
                }
            }
        }

        // Smoothly adjust distance
        let target_dist = nearest_hit;
        arm.is_colliding = target_dist < desired_dist - 0.01;

        if arm.current_distance < 0.01 {
            arm.current_distance = desired_dist;
        }

        if arm.is_colliding {
            // Retract quickly
            let speed = arm.retract_speed * dt.0;
            arm.current_distance += (target_dist - arm.current_distance) * speed.min(1.0);
        } else {
            // Extend slowly back to full distance
            let speed = arm.extend_speed * dt.0;
            arm.current_distance += (desired_dist - arm.current_distance) * speed.min(1.0);
        }

        // Update camera position
        cam_transform.translation = look_target + direction * arm.current_distance;
    }
}

/// Simple ray-AABB intersection test.
/// Returns the nearest `t` value (distance along ray) or `None` if no intersection.
fn ray_aabb_intersection(
    origin: Vec3,
    direction: Vec3,
    aabb_min: Vec3,
    aabb_max: Vec3,
) -> Option<f32> {
    let inv_dir = Vec3::new(
        if direction.x.abs() > f32::EPSILON { 1.0 / direction.x } else { f32::MAX },
        if direction.y.abs() > f32::EPSILON { 1.0 / direction.y } else { f32::MAX },
        if direction.z.abs() > f32::EPSILON { 1.0 / direction.z } else { f32::MAX },
    );

    let t1 = (aabb_min.x - origin.x) * inv_dir.x;
    let t2 = (aabb_max.x - origin.x) * inv_dir.x;
    let t3 = (aabb_min.y - origin.y) * inv_dir.y;
    let t4 = (aabb_max.y - origin.y) * inv_dir.y;
    let t5 = (aabb_min.z - origin.z) * inv_dir.z;
    let t6 = (aabb_max.z - origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        None
    } else {
        Some(if tmin > 0.0 { tmin } else { tmax })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let arm = SpringArm::default();
        assert_eq!(arm.probe_radius, 0.3);
        assert_eq!(arm.margin, 0.2);
        assert!(!arm.is_colliding());
    }

    #[test]
    fn test_builders() {
        let arm = SpringArm::new(0.5)
            .with_margin(0.3)
            .with_speeds(20.0, 3.0);
        assert_eq!(arm.probe_radius, 0.5);
        assert_eq!(arm.margin, 0.3);
        assert_eq!(arm.retract_speed, 20.0);
        assert_eq!(arm.extend_speed, 3.0);
    }

    #[test]
    fn test_ray_aabb_hit() {
        let origin = Vec3::new(0.0, 0.0, -5.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let t = ray_aabb_intersection(origin, direction, aabb_min, aabb_max);
        assert!(t.is_some());
        let t = t.unwrap();
        assert!((t - 4.0).abs() < 0.01, "t={t}");
    }

    #[test]
    fn test_ray_aabb_miss() {
        let origin = Vec3::new(0.0, 5.0, -5.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let t = ray_aabb_intersection(origin, direction, aabb_min, aabb_max);
        assert!(t.is_none());
    }

    #[test]
    fn test_ray_aabb_behind() {
        let origin = Vec3::new(0.0, 0.0, 5.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let t = ray_aabb_intersection(origin, direction, aabb_min, aabb_max);
        assert!(t.is_none());
    }
}
