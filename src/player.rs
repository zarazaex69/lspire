use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::physics::GameSystemSet;
use crate::menu::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(Update, (
                handle_speed_control,
                player_movement,
                check_death,
            ).in_set(GameSystemSet::Input).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct SpawnPoint(pub Vec3);

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
    pub is_braking: bool,
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
    let spawn_position = Vec3::new(0.0, 2.0, 0.0);
    
    commands.spawn((
        Player,
        SpawnPoint(spawn_position),
        PlayerSpeed::default(),
        PlayerMovement {
            velocity: Vec3::ZERO,
            drift_factor: 0.0,
            is_braking: false,
        },
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.3),
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        GravityScale(1.0),
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Transform::from_xyz(spawn_position.x, spawn_position.y, spawn_position.z),
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
    mut player_query: Query<(Entity, &mut Velocity, &PlayerSpeed, &mut PlayerMovement, &Transform), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    rapier_context: ReadRapierContext,
) {
    let rapier_context = rapier_context.single();
    
    let Ok((player_entity, mut velocity, speed, mut movement, transform)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let jump_force = 6.0;

    if !camera_transform.rotation.is_finite() {
        return;
    }

    let forward_vec = camera_transform.rotation * Vec3::NEG_Z;
    let right_vec = camera_transform.rotation * Vec3::X;

    let forward_flat = Vec3::new(forward_vec.x, 0.0, forward_vec.z);
    let right_flat = Vec3::new(right_vec.x, 0.0, right_vec.z);

    let forward_flat = if forward_flat.length_squared() > 0.0001 {
        forward_flat.normalize()
    } else {
        Vec3::NEG_Z
    };

    let right_flat = if right_flat.length_squared() > 0.0001 {
        right_flat.normalize()
    } else {
        Vec3::X
    };

    let is_braking = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    movement.is_braking = is_braking;

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

    if input_direction.length_squared() > 0.0001 {
        input_direction = input_direction.normalize();
    }

    let target_velocity = if is_braking {
        Vec3::ZERO
    } else if has_input {
        input_direction * speed.current
    } else {
        Vec3::ZERO
    };

    let speed_ratio = speed.current / speed.max;
    let drift_threshold = 0.4;
    
    let acceleration_lerp = if is_braking {
        let current_speed = movement.velocity.length();
        let speed_normalized = (current_speed / speed.max).clamp(0.0, 1.0);
        let brake_strength = 0.25 + speed_normalized * 0.15;
        brake_strength
    } else if has_input {
        if speed_ratio > drift_threshold {
            let drift = ((speed_ratio - drift_threshold) / (1.0 - drift_threshold)).powf(1.5);
            movement.drift_factor = drift;
            0.03 + drift * 0.07
        } else {
            movement.drift_factor = 0.0;
            0.15
        }
    } else {
        movement.drift_factor = 0.0;
        0.08
    };

    movement.velocity = movement.velocity.lerp(target_velocity, acceleration_lerp);

    velocity.linvel.x = movement.velocity.x;
    velocity.linvel.z = movement.velocity.z;

    let capsule_half_height = 0.5;
    let capsule_radius = 0.3;
    let ray_pos = transform.translation - Vec3::Y * capsule_half_height;
    let ray_dir = Vec3::NEG_Y;
    let max_toi = capsule_radius + 0.1;
    let filter = QueryFilter::default().exclude_rigid_body(player_entity);

    let is_grounded = rapier_context
        .cast_ray(ray_pos, ray_dir, max_toi, true, filter)
        .is_some();

    if keyboard.just_pressed(KeyCode::Space) && is_grounded {
        velocity.linvel.y = jump_force;
    }
}

fn check_death(
    mut query: Query<(&mut Transform, &mut Velocity, &mut PlayerMovement, &SpawnPoint), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut movement, spawn_point)) = query.get_single_mut() else {
        return;
    };

    let death_y = -20.0;
    
    if transform.translation.y < death_y {
        transform.translation = spawn_point.0;
        velocity.linvel = Vec3::ZERO;
        velocity.angvel = Vec3::ZERO;
        movement.velocity = Vec3::ZERO;
        movement.drift_factor = 0.0;
        movement.is_braking = false;
    }
}
