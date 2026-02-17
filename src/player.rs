use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, handle_speed_control));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity {
    pub value: Vec3,
}

#[derive(Component)]
pub struct PlayerSpeed {
    pub current: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for PlayerSpeed {
    fn default() -> Self {
        Self {
            current: 8.0,
            min: 2.0,
            max: 30.0,
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Velocity {
            value: Vec3::ZERO,
        },
        PlayerSpeed::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Visibility::Hidden,
    ));
}

fn handle_speed_control(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut PlayerSpeed, With<Player>>,
) {
    let Ok(mut speed) = query.get_single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        let delta = event.y * 0.5;
        speed.current = (speed.current + delta).clamp(speed.min, speed.max);
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &PlayerSpeed), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok((mut transform, mut velocity, speed)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let jump_force = 12.0;

    let forward = camera_transform.forward();
    let right = camera_transform.right();

    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right_flat;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right_flat;
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    transform.translation += direction * speed.current * time.delta_secs();

    let is_grounded = transform.translation.y <= 1.0;

    if keyboard.pressed(KeyCode::Space) && is_grounded {
        velocity.value.y = jump_force;
    }
}
