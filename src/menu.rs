use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::CursorGrabMode;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, (
                button_system,
                menu_action,
                rotate_menu_camera,
            ).run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Lobby,
    InGame,
}

#[derive(Component)]
struct MenuUI;

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
enum MenuButton {
    Multiplayer,
    Quit,
}

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.9);
const HOVERED_BUTTON: Color = Color::srgba(0.25, 0.25, 0.25, 0.95);
const PRESSED_BUTTON: Color = Color::srgba(0.35, 0.75, 0.35, 0.95);

fn setup_menu(mut commands: Commands, mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 8.0, 15.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.35, 0.48, 0.66),
            falloff: FogFalloff::Linear {
                start: 20.0,
                end: 60.0,
            },
            ..default()
        },
        MenuCamera,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        MenuUI,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            MenuUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LSPIRE"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
            ));

            spawn_button(parent, "Multiplayer", MenuButton::Multiplayer);
            spawn_button(parent, "Quit", MenuButton::Quit);
        });
}

fn spawn_button(parent: &mut ChildBuilder, text: &str, button_type: MenuButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Px(10.0)),
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
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn button_system(
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

fn menu_action(
    interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, menu_button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button {
                MenuButton::Multiplayer => {
                    next_state.set(GameState::Lobby);
                }
                MenuButton::Quit => {
                    exit.send(AppExit::Success);
                }
            }
        }
    }
}

fn rotate_menu_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MenuCamera>>,
) {
    for mut transform in &mut camera_query {
        let radius = 20.0;
        let height = 10.0;
        let speed = 0.15;
        
        let angle = time.elapsed_secs() * speed;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        
        transform.translation = Vec3::new(x, height, z);
        transform.look_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y);
    }
}

fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MenuUI>>,
    camera_query: Query<Entity, With<MenuCamera>>,
) {
    for entity in &menu_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn_recursive();
    }
}
