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

#[derive(Component)]
pub struct PlayerMovement {
    pub velocity: Vec3,
    pub drift_factor: f32,
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
        PlayerMovement {
            velocity: Vec3::ZERO,
            drift_factor: 0.0,
        },
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
    mut player_query: Query<(&mut Transform, &mut Velocity, &PlayerSpeed, &mut PlayerMovement), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok((mut transform, mut velocity, speed, mut movement)) = player_query.get_single_mut() else {
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

    let mut input_direction = Vec3::ZERO;
    let has_input = keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::KeyS)
        || keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD);

    if keyboard.pressed(KeyCode::KeyW) {
        input_direction += forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_direction -= forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_direction -= right_flat;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_direction += right_flat;
    }

    if input_direction.length() > 0.0 {
        input_direction = input_direction.normalize();
    }

    let target_velocity = if has_input {
        input_direction * speed.current
    } else {
        Vec3::ZERO
    };

    let speed_ratio = speed.current / speed.max;
    let drift_threshold = 0.4;
    
    let (acceleration_lerp, drift_amount) = if has_input {
        if speed_ratio > drift_threshold {
            let drift = ((speed_ratio - drift_threshold) / (1.0 - drift_threshold)).powf(1.5);
            movement.drift_factor = drift;
            (0.03 + drift * 0.07, drift)
        } else {
            movement.drift_factor = 0.0;
            (0.15, 0.0)
        }
    } else {
        movement.drift_factor = 0.0;
        (0.08, 0.0)
    };

    movement.velocity = movement.velocity.lerp(target_velocity, acceleration_lerp);

    transform.translation += movement.velocity * time.delta_secs();

    let is_grounded = transform.translation.y <= 1.0;

    if keyboard.pressed(KeyCode::Space) && is_grounded {
        velocity.value.y = jump_force;
    }
}
