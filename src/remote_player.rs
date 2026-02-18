use bevy::prelude::*;
use crate::network::{NetworkEvent, PlayerRegistry};

pub struct RemotePlayerPlugin;

impl Plugin for RemotePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_remote_players,
            update_remote_players,
            despawn_remote_players,
        ));
    }
}

#[derive(Component)]
pub struct RemotePlayer {
    pub id: u32,
}

fn spawn_remote_players(
    mut commands: Commands,
    mut events: EventReader<NetworkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_registry: ResMut<PlayerRegistry>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::PlayerJoined(id) | NetworkEvent::PlayerMoved(id, _, _) => {
                if let Some(player_data) = player_registry.players.get_mut(id) {
                    if player_data.entity.is_none() {
                        let entity = commands.spawn((
                            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.6))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb(0.3, 0.5, 0.8),
                                ..default()
                            })),
                            Transform::from_translation(player_data.position)
                                .with_rotation(player_data.rotation),
                            RemotePlayer { id: *id },
                        )).id();
                        
                        player_data.entity = Some(entity);
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_remote_players(
    player_registry: Res<PlayerRegistry>,
    mut query: Query<(&RemotePlayer, &mut Transform)>,
) {
    for (remote, mut transform) in query.iter_mut() {
        if let Some(player_data) = player_registry.players.get(&remote.id) {
            transform.translation = transform.translation.lerp(player_data.position, 0.3);
            transform.rotation = transform.rotation.slerp(player_data.rotation, 0.3);
        }
    }
}

fn despawn_remote_players(
    mut commands: Commands,
    mut events: EventReader<NetworkEvent>,
    query: Query<(Entity, &RemotePlayer)>,
) {
    for event in events.read() {
        if let NetworkEvent::PlayerLeft(id) = event {
            for (entity, remote) in query.iter() {
                if remote.id == *id {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
