use bevy_ecs::prelude::*;
use glam::Vec3;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::Velocity;

use crate::components::{CueBall, NumberedBall};
use crate::resources::{BilliardConfig, GameState, BallTracker};

/// Detect balls entering pockets.
pub fn pocket_detection_system(
    config: Res<BilliardConfig>,
    mut game_state: ResMut<GameState>,
    mut tracker: ResMut<BallTracker>,
    mut cue_query: Query<(&mut Transform, &mut Velocity), With<CueBall>>,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity, &mut NumberedBall), Without<CueBall>>,
) {
    let pocket_r = config.pocket_radius;
    let pockets = &config.pocket_positions;

    // Check cue ball
    for (mut t, mut v) in &mut cue_query {
        for pocket in pockets {
            let dx = t.translation.x - pocket.x;
            let dz = t.translation.z - pocket.z;
            if dx * dx + dz * dz < pocket_r * pocket_r {
                // Scratch: cue ball potted
                t.translation = Vec3::new(0.0, -10.0, 0.0);
                v.linear = Vec3::ZERO;
                game_state.is_scratch = true;
                tracker.on_table[0] = false;
            }
        }
    }

    // Check numbered balls
    for (_entity, mut t, mut v, mut ball) in &mut ball_query {
        if ball.potted {
            continue;
        }
        for pocket in pockets {
            let dx = t.translation.x - pocket.x;
            let dz = t.translation.z - pocket.z;
            if dx * dx + dz * dz < pocket_r * pocket_r {
                // Ball potted
                t.translation = Vec3::new(0.0, -10.0, 0.0);
                v.linear = Vec3::ZERO;
                ball.potted = true;
                tracker.on_table[ball.number as usize] = false;
                game_state.potted_this_turn.push(ball.number);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anvilkit_ecs::prelude::*;
    use anvilkit_ecs::schedule::AnvilKitSchedule;

    #[test]
    fn test_ball_potted() {
        let mut app = App::new();
        let config = BilliardConfig::default();
        let r = config.ball_radius;
        // Use the first pocket position from config
        let pocket = config.pocket_positions[0];
        app.insert_resource(config);
        app.insert_resource(GameState::default());
        app.insert_resource(BallTracker::default());
        app.add_systems(AnvilKitSchedule::Update, pocket_detection_system);

        // Place cue ball safely away
        app.world.spawn((
            CueBall,
            Transform::from_xyz(0.0, r, 0.0),
            Velocity::zero(),
        ));
        // Place ball 1 right on the pocket
        app.world.spawn((
            NumberedBall { number: 1, potted: false },
            Transform::from_xyz(pocket.x, r, pocket.z),
            Velocity::zero(),
        ));

        app.update();

        let tracker = app.world.get_resource::<BallTracker>().unwrap();
        assert!(!tracker.on_table[1], "Ball 1 should be potted");
    }

    #[test]
    fn test_scratch_detection() {
        let mut app = App::new();
        let config = BilliardConfig::default();
        let r = config.ball_radius;
        let pocket = config.pocket_positions[0];
        app.insert_resource(config);
        app.insert_resource(GameState::default());
        app.insert_resource(BallTracker::default());
        app.add_systems(AnvilKitSchedule::Update, pocket_detection_system);

        // Place cue ball on a pocket
        app.world.spawn((
            CueBall,
            Transform::from_xyz(pocket.x, r, pocket.z),
            Velocity::zero(),
        ));

        app.update();

        let gs = app.world.get_resource::<GameState>().unwrap();
        assert!(gs.is_scratch, "Should detect scratch");
    }
}
