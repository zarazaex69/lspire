use bevy::prelude::*;
use bevy::window::PresentMode;

mod audio;
mod camera;
mod debug;
mod lobby;
mod menu;
mod network;
mod physics;
mod player;
mod remote_player;
mod skybox;
mod world;

use audio::AudioPlugin;
use camera::CameraPlugin;
use debug::DebugPlugin;
use lobby::LobbyPlugin;
use menu::MenuPlugin;
use network::NetworkPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use remote_player::RemotePlayerPlugin;
use skybox::SkyboxPlugin;
use world::WorldPlugin;

#[bevy_main]
fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "lspire".to_string(),
            present_mode: PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
    .add_plugins(MenuPlugin)
    .add_plugins(LobbyPlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins((WorldPlugin, PlayerPlugin, RemotePlayerPlugin, PhysicsPlugin, CameraPlugin, DebugPlugin, SkyboxPlugin, AudioPlugin))
    .run();
}
