use bevy::prelude::*;
use crate::player::{Player, Velocity};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_gravity, apply_velocity));
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<&mut Velocity, With<Player>>) {
    let gravity = -25.0;

    for mut velocity in &mut query {
        velocity.value.y += gravity * time.delta_secs();
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for (mut transform, mut velocity) in &mut query {
        transform.translation += velocity.value * time.delta_secs();

        let ground_level = 1.0;
        if transform.translation.y < ground_level {
            transform.translation.y = ground_level;
            velocity.value.y = 0.0;
        }
    }
}
