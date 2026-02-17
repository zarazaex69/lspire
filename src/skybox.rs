use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_skybox);
    }
}

fn setup_skybox(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let skybox_texture = generate_sky_texture();
    let texture_handle = images.add(skybox_texture);

    let skybox_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let sphere_mesh = meshes.add(Sphere::new(500.0).mesh().ico(5).unwrap());

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(skybox_material),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(-1.0, 1.0, 1.0)),
    ));
}

fn generate_sky_texture() -> Image {
    let size = 512;
    let perlin = Perlin::new(42);
    
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 / size as f64;
            let ny = y as f64 / size as f64;

            let horizon = 0.5;
            let sky_gradient = (ny - horizon).max(0.0) / (1.0 - horizon);
            let ground_gradient = (horizon - ny).max(0.0) / horizon;

            let noise_value = perlin.get([nx * 4.0, ny * 4.0]) * 0.5 + 0.5;
            let cloud_factor = (noise_value * 0.3) as f32;

            let (r, g, b) = if ny > horizon {
                let base_r = 0.4 + sky_gradient as f32 * 0.2;
                let base_g = 0.6 + sky_gradient as f32 * 0.2;
                let base_b = 0.9 + sky_gradient as f32 * 0.1;
                
                (
                    (base_r + cloud_factor).min(1.0),
                    (base_g + cloud_factor).min(1.0),
                    (base_b + cloud_factor * 0.5).min(1.0),
                )
            } else {
                let base_r = 0.35 - ground_gradient as f32 * 0.1;
                let base_g = 0.48 - ground_gradient as f32 * 0.1;
                let base_b = 0.66 - ground_gradient as f32 * 0.1;
                
                (base_r, base_g, base_b)
            };

            data.push((r * 255.0) as u8);
            data.push((g * 255.0) as u8);
            data.push((b * 255.0) as u8);
            data.push(255);
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    )
}
