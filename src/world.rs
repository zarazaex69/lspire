use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_lighting, spawn_checkerboard_floor));
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    commands.spawn(DirectionalLight {
        illuminance: 10000.0,
        shadows_enabled: true,
        ..default()
    });
}

fn spawn_checkerboard_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tile_size = 2.0;
    let grid_size = 20;

    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        ..default()
    });

    let black_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        ..default()
    });

    let cube_mesh = meshes.add(Cuboid::new(tile_size, 0.2, tile_size));

    for x in -grid_size..grid_size {
        for z in -grid_size..grid_size {
            let is_white = (x + z) % 2 == 0;
            let material = if is_white {
                white_material.clone()
            } else {
                black_material.clone()
            };

            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(x as f32 * tile_size, -0.1, z as f32 * tile_size),
                RigidBody::Fixed,
                Collider::cuboid(tile_size / 2.0, 0.1, tile_size / 2.0),
            ));
        }
    }
}
