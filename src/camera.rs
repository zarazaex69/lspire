use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, setup_cursor_grab))
            .add_systems(Update, (toggle_cursor_grab, first_person_camera));
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.002,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.35, 0.48, 0.66)),
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

fn setup_cursor_grab(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn toggle_cursor_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            match window.cursor_options.grab_mode {
                CursorGrabMode::Locked => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
            }
        }
    }
}

fn first_person_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut FirstPersonCamera), (With<Camera3d>, Without<Player>)>,
    mut motion_events: EventReader<bevy::input::mouse::MouseMotion>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let Ok((mut camera_transform, mut fps_camera)) = camera_query.get_single_mut() else {
        return;
    };

    for event in motion_events.read() {
        fps_camera.yaw -= event.delta.x * fps_camera.sensitivity;
        fps_camera.pitch -= event.delta.y * fps_camera.sensitivity;
        fps_camera.pitch = fps_camera.pitch.clamp(-1.54, 1.54);
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
