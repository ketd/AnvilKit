use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_core::math::Transform;
use anvilkit_core::math::Velocity;

use crate::components::{CueBall, NumberedBall};
use crate::resources::BilliardConfig;

/// Ball-ball elastic collision (N² pairwise).
pub fn ball_collision_system(
    config: Res<BilliardConfig>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity), Or<(With<CueBall>, With<NumberedBall>)>>,
) {
    let r = config.ball_radius;
    let two_r = 2.0 * r;
    let restitution = config.ball_restitution;

    // Collect positions/velocities into a vec so we can iterate pairwise.
    let mut data: Vec<(Entity, Vec3, Vec3)> = query
        .iter()
        .map(|(e, t, v)| (e, t.translation, v.linear))
        .collect();

    let n = data.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let diff = data[j].1 - data[i].1;
            // Only check XZ distance (balls are on the same Y plane)
            let dx = diff.x;
            let dz = diff.z;
            let dist_sq = dx * dx + dz * dz;
            let min_dist = two_r;

            if dist_sq < min_dist * min_dist && dist_sq > 1e-8 {
                let dist = dist_sq.sqrt();
                let normal = Vec3::new(dx / dist, 0.0, dz / dist);

                // Separate overlapping balls
                let overlap = min_dist - dist;
                let sep = normal * (overlap * 0.5);
                data[i].1 -= sep;
                data[j].1 += sep;

                // Relative velocity along collision normal
                let rel_vel = data[i].2 - data[j].2;
                let rel_speed = rel_vel.dot(normal);

                // Only resolve if balls are approaching
                if rel_speed > 0.0 {
                    let impulse = normal * rel_speed * (1.0 + restitution) / 2.0;
                    data[i].2 -= impulse;
                    data[j].2 += impulse;
                }
            }
        }
    }

    // Write back
    for (entity, pos, vel) in &data {
        if let Ok((_, mut t, mut v)) = query.get_mut(*entity) {
            t.translation = *pos;
            v.linear = *vel;
        }
    }
}

/// Ball-cushion collision (4 planar boundaries).
pub fn cushion_collision_system(
    config: Res<BilliardConfig>,
    mut query: Query<(&mut Transform, &mut Velocity), Or<(With<CueBall>, With<NumberedBall>)>>,
) {
    let r = config.ball_radius;
    let hw = config.table_half_width;
    let hd = config.table_half_depth;
    let restitution = config.cushion_restitution;

    // Inner boundary = table half extents minus ball radius
    let bx = hw - r;
    let bz = hd - r;

    for (mut transform, mut velocity) in &mut query {
        let pos = &mut transform.translation;

        // +X cushion
        if pos.x > bx {
            pos.x = bx;
            velocity.linear.x = -velocity.linear.x.abs() * restitution;
        }
        // -X cushion
        if pos.x < -bx {
            pos.x = -bx;
            velocity.linear.x = velocity.linear.x.abs() * restitution;
        }
        // +Z cushion
        if pos.z > bz {
            pos.z = bz;
            velocity.linear.z = -velocity.linear.z.abs() * restitution;
        }
        // -Z cushion
        if pos.z < -bz {
            pos.z = -bz;
            velocity.linear.z = velocity.linear.z.abs() * restitution;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anvilkit::prelude::*;

    #[test]
    fn test_ball_ball_collision() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        let config = BilliardConfig::default();
        let r = config.ball_radius;
        app.insert_resource(config);
        app.add_systems(AnvilKitSchedule::Update, ball_collision_system);

        // Two balls overlapping (distance < 2*r), approaching each other
        let a = app.world_mut().spawn((
            CueBall,
            Transform::from_xyz(0.0, r, 0.0),
            Velocity::linear(Vec3::new(5.0, 0.0, 0.0)),
        )).id();
        let b = app.world_mut().spawn((
            NumberedBall { number: 1, potted: false },
            Transform::from_xyz(r * 1.5, r, 0.0), // within 2*r
            Velocity::zero(),
        )).id();

        app.update();

        let va = app.world().get::<Velocity>(a).unwrap();
        let vb = app.world().get::<Velocity>(b).unwrap();
        assert!(va.linear.x < 5.0, "Ball A should slow down");
        assert!(vb.linear.x > 0.0, "Ball B should speed up");
    }

    #[test]
    fn test_cushion_bounce() {
        let mut app = App::new();
        app.add_plugins(AnvilKitEcsPlugin);
        let config = BilliardConfig::default();
        let r = config.ball_radius;
        let hw = config.table_half_width;
        app.insert_resource(config);
        app.add_systems(AnvilKitSchedule::Update, cushion_collision_system);

        // Ball right at the +X boundary
        let e = app.world_mut().spawn((
            CueBall,
            Transform::from_xyz(hw, r, 0.0), // past boundary (hw - r)
            Velocity::linear(Vec3::new(5.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world().get::<Velocity>(e).unwrap();
        assert!(vel.linear.x < 0.0, "Ball should bounce back from +X cushion");
    }
}
