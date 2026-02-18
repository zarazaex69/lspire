use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::player::Player;
use crate::physics::GameSystemSet;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (
                setup_cursor_grab,
                toggle_cursor_grab,
                first_person_camera,
            ).in_set(GameSystemSet::Camera));
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub target_pitch: f32,
    pub target_yaw: f32,
    pub sensitivity: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            target_pitch: 0.0,
            target_yaw: 0.0,
            sensitivity: 0.002,
        }
    }
}

#[derive(Resource)]
struct CursorGrabbed(bool);

fn spawn_camera(mut commands: Commands) {
    commands.insert_resource(CursorGrabbed(false));
    
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(0.0, 1.6, 0.0),
        FirstPersonCamera::default(),
        DistanceFog {
            color: Color::srgb(0.35, 0.48, 0.66),
            falloff: FogFalloff::Linear {
                start: 20.0,
                end: 60.0,
            },
            ..default()
        },
    ));
}

fn setup_cursor_grab(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cursor_grabbed: ResMut<CursorGrabbed>,
) {
    if cursor_grabbed.0 {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            cursor_grabbed.0 = true;
        }
    }
}

fn toggle_cursor_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cursor_grabbed: ResMut<CursorGrabbed>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            match window.cursor_options.grab_mode {
                CursorGrabMode::Locked => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    cursor_grabbed.0 = false;
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                    cursor_grabbed.0 = true;
                }
            }
        }
    }
}

fn first_person_camera(
    player_query: Query<(&Transform, &crate::player::PlayerMovement), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut FirstPersonCamera), (With<Camera3d>, Without<Player>)>,
    mut motion_events: EventReader<bevy::input::mouse::MouseMotion>,
    time: Res<Time>,
) {
    let Ok((player_transform, player_movement)) = player_query.get_single() else {
        return;
    };

    let Ok((mut camera_transform, mut fps_camera)) = camera_query.get_single_mut() else {
        return;
    };

    let mut delta_yaw = 0.0;
    let mut delta_pitch = 0.0;

    for event in motion_events.read() {
        delta_yaw -= event.delta.x * fps_camera.sensitivity;
        delta_pitch -= event.delta.y * fps_camera.sensitivity;
    }

    fps_camera.target_yaw += delta_yaw;
    fps_camera.target_pitch = (fps_camera.target_pitch + delta_pitch).clamp(-1.54, 1.54);

    let smoothing = if player_movement.drift_factor > 0.1 {
        5.0 + player_movement.drift_factor * 10.0
    } else {
        100.0
    };

    let delta_time = time.delta_secs().min(0.1);
    let lerp_factor = (smoothing * delta_time).min(1.0);

    fps_camera.yaw += (fps_camera.target_yaw - fps_camera.yaw) * lerp_factor;
    fps_camera.pitch += (fps_camera.target_pitch - fps_camera.pitch) * lerp_factor;

    if !fps_camera.yaw.is_finite() {
        fps_camera.yaw = 0.0;
        fps_camera.target_yaw = 0.0;
    }
    if !fps_camera.pitch.is_finite() {
        fps_camera.pitch = 0.0;
        fps_camera.target_pitch = 0.0;
    }

    let eye_height = 1.6;
    camera_transform.translation = player_transform.translation + Vec3::new(0.0, eye_height, 0.0);

    camera_transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        fps_camera.yaw,
        fps_camera.pitch,
        0.0,
    );
}
