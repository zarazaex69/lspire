use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::player::{Player, PlayerSpeed, PlayerMovement};
use crate::menu::GameState;
use crate::network::{NetworkState, NetworkMode};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(OnEnter(GameState::InGame), setup_debug_ui)
            .add_systems(Update, (toggle_debug_ui, update_debug_info).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
struct DebugText;

#[derive(Resource)]
struct DebugVisible(bool);

fn setup_debug_ui(mut commands: Commands) {
    commands.insert_resource(DebugVisible(false));

    commands.spawn((
        DebugText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn toggle_debug_ui(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_visible: ResMut<DebugVisible>,
    mut query: Query<&mut Visibility, With<DebugText>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_visible.0 = !debug_visible.0;
        
        if let Ok(mut visibility) = query.get_single_mut() {
            *visibility = if debug_visible.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn update_debug_info(
    diagnostics: Res<DiagnosticsStore>,
    debug_visible: Res<DebugVisible>,
    net_state: Res<NetworkState>,
    player_query: Query<(&Transform, &PlayerSpeed, &PlayerMovement), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut text_query: Query<&mut Text, With<DebugText>>,
    time: Res<Time>,
) {
    if !debug_visible.0 {
        return;
    }

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|ft| ft.smoothed())
        .unwrap_or(0.0);

    let mut debug_info = format!(
        "FPS: {:.1}\nFrame Time: {:.2}ms\n",
        fps,
        frame_time
    );
    
    if net_state.mode == NetworkMode::Client {
        debug_info.push_str(&format!("Ping: {:.0}ms\n", net_state.ping_ms));
    } else if net_state.mode == NetworkMode::Server {
        debug_info.push_str("Mode: Server\n");
    } else {
        debug_info.push_str("Mode: None\n");
    }
    
    debug_info.push_str("\n");

    if let Ok((player_transform, player_speed, player_movement)) = player_query.get_single() {
        let pos = player_transform.translation;
        debug_info.push_str(&format!(
            "Position:\n  X: {:.2}\n  Y: {:.2}\n  Z: {:.2}\n\n",
            pos.x, pos.y, pos.z
        ));
        debug_info.push_str(&format!(
            "Speed: {:.1} / {:.1}\n",
            player_speed.current, player_speed.max
        ));
        debug_info.push_str(&format!(
            "Drift: {:.1}%\n\n",
            player_movement.drift_factor * 100.0
        ));
    }

    if let Ok(camera_transform) = camera_query.get_single() {
        let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        debug_info.push_str(&format!(
            "Camera:\n  Yaw: {:.2}\n  Pitch: {:.2}\n\n",
            yaw.to_degrees(),
            pitch.to_degrees()
        ));
    }

    debug_info.push_str(&format!("Time: {:.2}s", time.elapsed_secs()));

    **text = debug_info;
}
