use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::{Velocity, DeltaTime};

use crate::components::{CueBall, NumberedBall};
use crate::resources::BilliardConfig;

/// Rolling friction + velocity integration for billiard balls.
pub fn billiard_velocity_system(
    dt: Res<DeltaTime>,
    config: Res<BilliardConfig>,
    mut query: Query<(&mut Transform, &mut Velocity), Or<(With<CueBall>, With<NumberedBall>)>>,
) {
    let friction = config.rolling_friction;
    let min_vel_sq = config.min_velocity * config.min_velocity;
    let ball_y = config.ball_radius;

    for (mut transform, mut velocity) in &mut query {
        let v = velocity.linear;
        if v.length_squared() < min_vel_sq {
            velocity.linear = glam::Vec3::ZERO;
            continue;
        }

        // Rolling friction: exponential decay
        let decay = (1.0 - friction * dt.0).max(0.0);
        velocity.linear *= decay;

        // Re-check after friction
        if velocity.linear.length_squared() < min_vel_sq {
            velocity.linear = glam::Vec3::ZERO;
            continue;
        }

        // Position integration
        transform.translation += velocity.linear * dt.0;

        // Enforce ball stays on table surface
        transform.translation.y = ball_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anvilkit_ecs::prelude::*;
    use anvilkit_ecs::schedule::AnvilKitSchedule;

    #[test]
    fn test_friction_slows_ball() {
        let mut app = App::new();
        app.insert_resource(DeltaTime(1.0 / 60.0));
        app.insert_resource(BilliardConfig::default());
        app.add_systems(AnvilKitSchedule::Update, billiard_velocity_system);

        let config = BilliardConfig::default();
        let e = app.world.spawn((
            CueBall,
            Transform::from_xyz(0.0, config.ball_radius, 0.0),
            Velocity::linear(glam::Vec3::new(5.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world.get::<Velocity>(e).unwrap();
        assert!(vel.linear.x < 5.0, "Velocity should decrease due to friction");
        assert!(vel.linear.x > 0.0, "Velocity should still be positive");
    }

    #[test]
    fn test_low_velocity_zeroed() {
        let mut app = App::new();
        app.insert_resource(DeltaTime(1.0 / 60.0));
        app.insert_resource(BilliardConfig::default());
        app.add_systems(AnvilKitSchedule::Update, billiard_velocity_system);

        let config = BilliardConfig::default();
        let e = app.world.spawn((
            CueBall,
            Transform::from_xyz(0.0, config.ball_radius, 0.0),
            Velocity::linear(glam::Vec3::new(0.005, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world.get::<Velocity>(e).unwrap();
        assert_eq!(vel.linear, glam::Vec3::ZERO);
    }

    #[test]
    fn test_ball_y_enforced() {
        let mut app = App::new();
        app.insert_resource(DeltaTime(1.0 / 60.0));
        app.insert_resource(BilliardConfig::default());
        app.add_systems(AnvilKitSchedule::Update, billiard_velocity_system);

        let config = BilliardConfig::default();
        let e = app.world.spawn((
            CueBall,
            Transform::from_xyz(0.0, 5.0, 0.0),
            Velocity::linear(glam::Vec3::new(1.0, 0.0, 0.0)),
        )).id();

        app.update();

        let t = app.world.get::<Transform>(e).unwrap();
        assert!((t.translation.y - config.ball_radius).abs() < 0.01);
    }
}
