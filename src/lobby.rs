use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use crate::menu::GameState;
use crate::network::{NetworkState, ServerList, NetworkEvent};

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Lobby), setup_lobby)
            .add_systems(Update, (
                lobby_button_system,
                lobby_action,
                update_server_list_ui,
                handle_connection_events,
            ).run_if(in_state(GameState::Lobby)))
            .add_systems(OnExit(GameState::Lobby), cleanup_lobby);
    }
}

#[derive(Component)]
struct LobbyUI;

#[derive(Component)]
enum LobbyButton {
    CreateServer,
    Refresh,
    Back,
    JoinServer(std::net::SocketAddr),
}

#[derive(Component)]
struct ServerListContainer;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.9);
const HOVERED_BUTTON: Color = Color::srgba(0.25, 0.25, 0.25, 0.95);
const PRESSED_BUTTON: Color = Color::srgba(0.35, 0.75, 0.35, 0.95);

fn setup_lobby(
    mut commands: Commands,
    mut net_state: ResMut<NetworkState>,
    mut windows: Query<&mut Window>,
) {
    for mut window in windows.iter_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    if let Ok(state) = NetworkState::start_discovery() {
        *net_state = state;
    }

    commands.spawn((
        Camera2d,
        LobbyUI,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
            LobbyUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("MULTIPLAYER LOBBY"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Px(700.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    ServerListContainer,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Searching for servers..."),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                });

            parent.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(20.0),
                ..default()
            }).with_children(|parent| {
                spawn_button(parent, "Create Server", LobbyButton::CreateServer);
                spawn_button(parent, "Refresh", LobbyButton::Refresh);
                spawn_button(parent, "Back", LobbyButton::Back);
            });
        });
}

fn spawn_button(parent: &mut ChildBuilder, text: &str, button_type: LobbyButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            button_type,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn lobby_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn lobby_action(
    interaction_query: Query<(&Interaction, &LobbyButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut net_state: ResMut<NetworkState>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button {
                LobbyButton::CreateServer => {
                    if let Ok(state) = NetworkState::create_server() {
                        *net_state = state;
                        next_state.set(GameState::InGame);
                    }
                }
                LobbyButton::Refresh => {
                    if let Ok(state) = NetworkState::start_discovery() {
                        *net_state = state;
                    }
                }
                LobbyButton::Back => {
                    next_state.set(GameState::Menu);
                }
                LobbyButton::JoinServer(addr) => {
                    if net_state.connect_to_server(*addr).is_ok() {
                    }
                }
            }
        }
    }
}

fn update_server_list_ui(
    mut commands: Commands,
    server_list: Res<ServerList>,
    container_query: Query<Entity, With<ServerListContainer>>,
    children_query: Query<&Children>,
) {
    if !server_list.is_changed() {
        return;
    }

    for container in container_query.iter() {
        if let Ok(children) = children_query.get(container) {
            for &child in children.iter() {
                commands.entity(child).despawn_recursive();
            }
        }

        commands.entity(container).with_children(|parent| {
            if server_list.servers.is_empty() {
                parent.spawn((
                    Text::new("No servers found. Create one or refresh."),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            } else {
                for (addr, info) in server_list.servers.iter() {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(70.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                padding: UiRect::all(Val::Px(15.0)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            LobbyButton::JoinServer(*addr),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(format!("{} - {}/{} players", info.name, info.player_count, info.max_players)),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            parent.spawn((
                                Text::new(format!("IP: {}", addr)),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                            ));
                        });
                }
            }
        });
    }
}

fn handle_connection_events(
    mut events: EventReader<NetworkEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::ConnectedToServer(_) => {
                next_state.set(GameState::InGame);
            }
            _ => {}
        }
    }
}

fn cleanup_lobby(
    mut commands: Commands,
    lobby_query: Query<Entity, With<LobbyUI>>,
) {
    for entity in &lobby_query {
        commands.entity(entity).despawn_recursive();
    }
}
