use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::{UdpSocket, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NetworkState>()
            .init_resource::<ServerList>()
            .init_resource::<PlayerRegistry>()
            .add_event::<NetworkEvent>()
            .add_systems(Update, (
                handle_network_events,
                update_server_discovery,
                sync_players,
                send_ping,
            ));
    }
}

#[derive(Resource)]
pub struct NetworkState {
    pub mode: NetworkMode,
    pub socket: Option<Arc<UdpSocket>>,
    pub server_addr: Option<SocketAddr>,
    pub local_player_id: u32,
    pub last_discovery: Instant,
    pub ping_ms: f32,
    pub last_ping_sent: Instant,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            mode: NetworkMode::None,
            socket: None,
            server_addr: None,
            local_player_id: 0,
            last_discovery: Instant::now(),
            ping_ms: 0.0,
            last_ping_sent: Instant::now(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum NetworkMode {
    #[default]
    None,
    Server,
    Client,
}

#[derive(Resource, Default)]
pub struct ServerList {
    pub servers: HashMap<SocketAddr, ServerInfo>,
}

#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub player_count: u8,
    pub max_players: u8,
    pub last_seen: Instant,
}

#[derive(Resource, Default)]
pub struct PlayerRegistry {
    pub players: HashMap<u32, PlayerData>,
    pub client_addresses: HashMap<u32, SocketAddr>,
}

#[derive(Clone, Debug)]
pub struct PlayerData {
    pub id: u32,
    pub position: Vec3,
    pub rotation: Quat,
    pub entity: Option<Entity>,
}

#[derive(Event)]
pub enum NetworkEvent {
    ConnectedToServer(SocketAddr),
    PlayerJoined(u32),
    PlayerLeft(u32),
    PlayerMoved(u32, Vec3, Quat),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    ServerAnnounce {
        name: String,
        player_count: u8,
        max_players: u8,
    },
    DiscoveryRequest,
    JoinRequest {
        player_name: String,
    },
    JoinAccept {
        player_id: u32,
        existing_players: Vec<(u32, Vec3, Quat)>,
    },
    PlayerSpawn {
        player_id: u32,
        position: Vec3,
        rotation: Quat,
    },
    PlayerUpdate {
        player_id: u32,
        position: Vec3,
        rotation: Quat,
    },
    PlayerDisconnect {
        player_id: u32,
    },
    Ping {
        timestamp: u128,
    },
    Pong {
        timestamp: u128,
    },
}

impl NetworkState {
    pub fn create_server() -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:7878")?;
        socket.set_nonblocking(true)?;
        socket.set_broadcast(true)?;
        
        let state = NetworkState {
            mode: NetworkMode::Server,
            socket: Some(Arc::new(socket)),
            server_addr: None,
            local_player_id: 0,
            last_discovery: Instant::now(),
            ping_ms: 0.0,
            last_ping_sent: Instant::now(),
        };
        
        Ok(state)
    }
    
    pub fn start_discovery() -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:7879")?;
        socket.set_nonblocking(true)?;
        socket.set_broadcast(true)?;
        
        let msg = NetworkMessage::DiscoveryRequest;
        let data = bincode::serialize(&msg).unwrap();
        socket.send_to(&data, "255.255.255.255:7878")?;
        
        Ok(NetworkState {
            mode: NetworkMode::None,
            socket: Some(Arc::new(socket)),
            server_addr: None,
            local_player_id: 0,
            last_discovery: Instant::now(),
            ping_ms: 0.0,
            last_ping_sent: Instant::now(),
        })
    }
    
    pub fn connect_to_server(&mut self, server_addr: SocketAddr) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        socket.connect(server_addr)?;
        
        let msg = NetworkMessage::JoinRequest {
            player_name: "Player".to_string(),
        };
        let data = bincode::serialize(&msg).unwrap();
        socket.send(&data)?;
        
        self.socket = Some(Arc::new(socket));
        self.server_addr = Some(server_addr);
        self.mode = NetworkMode::Client;
        
        Ok(())
    }
    
    pub fn send_message(&self, msg: &NetworkMessage) -> Result<(), std::io::Error> {
        if let Some(socket) = &self.socket {
            let data = bincode::serialize(msg).unwrap();
            match self.mode {
                NetworkMode::Server => {
                    socket.send_to(&data, "255.255.255.255:7879")?;
                }
                NetworkMode::Client => {
                    socket.send(&data)?;
                }
                NetworkMode::None => {}
            }
        }
        Ok(())
    }
}

fn handle_network_events(
    mut net_state: ResMut<NetworkState>,
    mut server_list: ResMut<ServerList>,
    mut player_registry: ResMut<PlayerRegistry>,
    mut events: EventWriter<NetworkEvent>,
) {
    let socket = match &net_state.socket {
        Some(s) => s.clone(),
        None => return,
    };
    
    let mut buf = [0u8; 65535];
    let mut pending_updates = Vec::new();
    
    while let Ok((size, addr)) = socket.recv_from(&mut buf) {
        if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&buf[..size]) {
            pending_updates.push((msg, addr));
        }
    }
    
    for (msg, addr) in pending_updates {
        match msg {
            NetworkMessage::ServerAnnounce { name, player_count, max_players } => {
                server_list.servers.insert(addr, ServerInfo {
                    name,
                    player_count,
                    max_players,
                    last_seen: Instant::now(),
                });
            }
            NetworkMessage::DiscoveryRequest => {
                if net_state.mode == NetworkMode::Server {
                    let response = NetworkMessage::ServerAnnounce {
                        name: "LAN Server".to_string(),
                        player_count: player_registry.players.len() as u8,
                        max_players: 8,
                    };
                    let _ = net_state.send_message(&response);
                }
            }
            NetworkMessage::JoinRequest { .. } => {
                if net_state.mode == NetworkMode::Server {
                    let new_id = player_registry.players.len() as u32 + 1;
                    
                    let existing: Vec<_> = player_registry.players.values()
                        .map(|p| (p.id, p.position, p.rotation))
                        .collect();
                    
                    let accept = NetworkMessage::JoinAccept {
                        player_id: new_id,
                        existing_players: existing,
                    };
                    
                    let data = bincode::serialize(&accept).unwrap();
                    let _ = socket.send_to(&data, addr);
                    
                    player_registry.client_addresses.insert(new_id, addr);
                    
                    player_registry.players.insert(new_id, PlayerData {
                        id: new_id,
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                        entity: None,
                    });
                    
                    let spawn_msg = NetworkMessage::PlayerSpawn {
                        player_id: new_id,
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                    };
                    let spawn_data = bincode::serialize(&spawn_msg).unwrap();
                    for (id, client_addr) in player_registry.client_addresses.iter() {
                        if *id != new_id {
                            let _ = socket.send_to(&spawn_data, client_addr);
                        }
                    }
                    
                    events.send(NetworkEvent::PlayerJoined(new_id));
                }
            }
            NetworkMessage::JoinAccept { player_id, existing_players } => {
                net_state.local_player_id = player_id;
                
                for (id, pos, rot) in existing_players {
                    if id != player_id {
                        player_registry.players.insert(id, PlayerData {
                            id,
                            position: pos,
                            rotation: rot,
                            entity: None,
                        });
                        events.send(NetworkEvent::PlayerJoined(id));
                    }
                }
                
                events.send(NetworkEvent::ConnectedToServer(addr));
            }
            NetworkMessage::PlayerSpawn { player_id, position, rotation } => {
                if player_id != net_state.local_player_id {
                    player_registry.players.insert(player_id, PlayerData {
                        id: player_id,
                        position,
                        rotation,
                        entity: None,
                    });
                    events.send(NetworkEvent::PlayerJoined(player_id));
                }
            }
            NetworkMessage::PlayerUpdate { player_id, position, rotation } => {
                if net_state.mode == NetworkMode::Server {
                    if let Some(player) = player_registry.players.get_mut(&player_id) {
                        player.position = position;
                        player.rotation = rotation;
                    }
                    
                    let update_msg = NetworkMessage::PlayerUpdate {
                        player_id,
                        position,
                        rotation,
                    };
                    let data = bincode::serialize(&update_msg).unwrap();
                    
                    for (id, client_addr) in player_registry.client_addresses.iter() {
                        if *id != player_id {
                            let _ = socket.send_to(&data, client_addr);
                        }
                    }
                } else if player_id != net_state.local_player_id {
                    if let Some(player) = player_registry.players.get_mut(&player_id) {
                        player.position = position;
                        player.rotation = rotation;
                    } else {
                        player_registry.players.insert(player_id, PlayerData {
                            id: player_id,
                            position,
                            rotation,
                            entity: None,
                        });
                    }
                    events.send(NetworkEvent::PlayerMoved(player_id, position, rotation));
                }
            }
            NetworkMessage::PlayerDisconnect { player_id } => {
                player_registry.players.remove(&player_id);
                events.send(NetworkEvent::PlayerLeft(player_id));
            }
            NetworkMessage::Ping { timestamp } => {
                if net_state.mode == NetworkMode::Server {
                    let pong = NetworkMessage::Pong { timestamp };
                    let data = bincode::serialize(&pong).unwrap();
                    let _ = socket.send_to(&data, addr);
                }
            }
            NetworkMessage::Pong { timestamp } => {
                if net_state.mode == NetworkMode::Client {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    net_state.ping_ms = (now - timestamp) as f32;
                }
            }
            _ => {}
        }
    }
}

fn update_server_discovery(
    mut net_state: ResMut<NetworkState>,
    mut server_list: ResMut<ServerList>,
) {
    if net_state.mode == NetworkMode::Server {
        if net_state.last_discovery.elapsed() > Duration::from_secs(2) {
            let msg = NetworkMessage::ServerAnnounce {
                name: "LAN Server".to_string(),
                player_count: 0,
                max_players: 8,
            };
            let _ = net_state.send_message(&msg);
            net_state.last_discovery = Instant::now();
        }
    }
    
    server_list.servers.retain(|_, info| {
        info.last_seen.elapsed() < Duration::from_secs(5)
    });
}

fn sync_players(
    net_state: Res<NetworkState>,
    player_registry: Res<PlayerRegistry>,
    player_query: Query<(&Transform, Entity), With<crate::player::Player>>,
) {
    if net_state.mode == NetworkMode::None {
        return;
    }
    
    let socket = match &net_state.socket {
        Some(s) => s,
        None => return,
    };
    
    for (transform, _) in player_query.iter() {
        let msg = NetworkMessage::PlayerUpdate {
            player_id: net_state.local_player_id,
            position: transform.translation,
            rotation: transform.rotation,
        };
        
        if net_state.mode == NetworkMode::Server {
            let data = bincode::serialize(&msg).unwrap();
            for (id, client_addr) in player_registry.client_addresses.iter() {
                if *id != net_state.local_player_id {
                    let _ = socket.send_to(&data, client_addr);
                }
            }
        } else {
            let _ = net_state.send_message(&msg);
        }
    }
}

fn send_ping(mut net_state: ResMut<NetworkState>) {
    if net_state.mode != NetworkMode::Client {
        return;
    }
    
    if net_state.last_ping_sent.elapsed() < Duration::from_millis(500) {
        return;
    }
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let msg = NetworkMessage::Ping { timestamp };
    let _ = net_state.send_message(&msg);
    net_state.last_ping_sent = Instant::now();
}
