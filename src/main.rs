use macroquad::prelude::*;

mod world;
mod rendering;
mod physics;
mod networking;
mod audio;
mod input;
mod ui;

use world::{ChunkManager, WorldState, WeatherState};
use rendering::{InstancedRenderer, grayscale, FogSettings, DrawingSystem, DrawMark};
use physics::{Player, PlayerController};
use input::InputState;
use ui::{hud::StaminaHUD, ShadeSelector};

fn window_conf() -> Conf {
    Conf {
        window_title: "LSPIRE".to_owned(),
        window_width: 1280,
        window_height: 720,
        sample_count: 8,
        ..Default::default()
    }
}

struct GameState {
    player: Player,
    player_controller: PlayerController,
    chunk_manager: ChunkManager,
    renderer: InstancedRenderer,
    camera_yaw: f32,
    camera_pitch: f32,
    stamina_hud: StaminaHUD,
    camera_shake_intensity: f32,
    current_fov: f32,
    target_fov: f32,
    fov_transition_speed: f32,
    fog_settings: FogSettings,
    world_state: WorldState,
    drawing_system: DrawingSystem,
    shade_selector: ShadeSelector,
}

impl GameState {
    fn new(seed: u64) -> Self {
        Self {
            player: Player::new(0, vec3(0.0, 10.0, 0.0)),
            player_controller: PlayerController::new(),
            chunk_manager: ChunkManager::new(seed, 3),
            renderer: InstancedRenderer::new(10000),
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            stamina_hud: StaminaHUD::new(),
            camera_shake_intensity: 0.0,
            current_fov: 70.0f32.to_radians(),
            target_fov: 70.0f32.to_radians(),
            fov_transition_speed: 1.0 / 0.3,
            fog_settings: FogSettings::default(),
            world_state: WorldState::default(),
            drawing_system: DrawingSystem::new(),
            shade_selector: ShadeSelector::new(),
        }
    }

    fn handle_input(&mut self, input: &InputState) {
        let mouse_sensitivity = 0.5;
        self.camera_yaw += input.mouse_delta.x * mouse_sensitivity;
        self.camera_pitch += input.mouse_delta.y * mouse_sensitivity;
        self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);
        
        self.player.rotation = self.camera_yaw;

        if is_key_pressed(KeyCode::Key1) && !self.shade_selector.is_visible() {
            self.world_state.set_weather(WeatherState::Clear);
        }
        if is_key_pressed(KeyCode::Key2) && !self.shade_selector.is_visible() {
            self.world_state.set_weather(WeatherState::LightFog);
        }
        if is_key_pressed(KeyCode::Key3) && !self.shade_selector.is_visible() {
            self.world_state.set_weather(WeatherState::HeavyFog);
        }

        if is_key_pressed(KeyCode::G) {
            self.shade_selector.toggle_visibility();
        }

        if let Some(new_shade) = self.shade_selector.handle_input() {
            self.player.selected_gray_shade = new_shade;
        }

        if is_mouse_button_pressed(MouseButton::Left) && !self.shade_selector.is_visible() {
            self.handle_drawing();
        }
    }

    fn handle_drawing(&mut self) {
        let camera_offset = vec3(0.0, 1.6, 0.0);
        let camera_pos = self.player.position + camera_offset;

        let (sin_yaw, cos_yaw) = self.camera_yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.camera_pitch.sin_cos();
        let ray_direction = vec3(
            sin_yaw * cos_pitch,
            sin_pitch,
            cos_yaw * cos_pitch,
        ).normalize();

        if let Some(hit) = self.drawing_system.raycast_surface(camera_pos, ray_direction, 10.0) {
            let mark = DrawMark::new(
                hit.uv,
                self.player.selected_gray_shade,
                0.05,
            );
            self.drawing_system.add_mark(hit.surface_id, mark);
        }
    }

    fn update(&mut self, input: &InputState, dt: f32) {
        self.player_controller.update(&mut self.player, input, dt);
        self.chunk_manager.update_loaded_chunks(self.player.position);
        self.world_state.update(dt);
        
        let fog_density = self.world_state.get_fog_density();
        self.fog_settings.set_density(fog_density);
        
        self.update_camera_effects(dt);
    }

    fn update_camera_effects(&mut self, dt: f32) {
        let horizontal_speed = vec2(self.player.velocity.x, self.player.velocity.z).length();
        
        let base_speed = self.player_controller.move_speed;
        let max_sprint_speed = base_speed * self.player_controller.sprint_multiplier;
        
        let speed_ratio = (horizontal_speed / max_sprint_speed).min(1.0);
        
        self.camera_shake_intensity = if self.player.is_sprinting && self.player.is_grounded {
            speed_ratio * 0.02
        } else {
            0.0
        };
        
        self.target_fov = if self.player.is_sprinting && self.player.is_grounded {
            75.0f32.to_radians()
        } else {
            70.0f32.to_radians()
        };
        
        let fov_diff = self.target_fov - self.current_fov;
        let fov_change = fov_diff * self.fov_transition_speed * dt;
        self.current_fov += fov_change;
        
        if fov_diff.abs() < 0.01 {
            self.current_fov = self.target_fov;
        }
    }

    fn render(&mut self, dt: f32) {
        let camera_offset = vec3(0.0, 1.6, 0.0);
        let camera_pos = self.player.position + camera_offset;

        let shake_offset = if self.camera_shake_intensity > 0.0 {
            let time = get_time() as f32;
            vec3(
                (time * 20.0).sin() * self.camera_shake_intensity,
                (time * 25.0).cos() * self.camera_shake_intensity,
                (time * 22.0).sin() * self.camera_shake_intensity,
            )
        } else {
            Vec3::ZERO
        };

        let final_camera_pos = camera_pos + shake_offset;

        let (sin_yaw, cos_yaw) = self.camera_yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.camera_pitch.sin_cos();
        let camera_target = final_camera_pos + vec3(
            sin_yaw * cos_pitch,
            sin_pitch,
            cos_yaw * cos_pitch,
        );

        let camera = Camera3D {
            position: final_camera_pos,
            target: camera_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: self.current_fov,
            projection: Projection::Perspective,
            ..Default::default()
        };

        set_camera(&camera);

        let ambient_light = self.world_state.get_ambient_light();
        
        draw_grid(20, 1.0, 
            grayscale(0.5 * ambient_light), 
            grayscale(0.3 * ambient_light)
        );

        let camera_pos_2d = vec2(final_camera_pos.x, final_camera_pos.z);
        
        let cube1_pos = vec3(0.0, 0.5, 0.0);
        let distance1 = vec2(cube1_pos.x, cube1_pos.z).distance(camera_pos_2d);
        let cube1_color = self.fog_settings.apply_fog_to_color(
            grayscale(0.5 * ambient_light), 
            distance1
        );
        draw_cube(cube1_pos, vec3(1.0, 1.0, 1.0), None, cube1_color);
        
        let cube2_pos = vec3(5.0, 2.0, 0.0);
        let distance2 = vec2(cube2_pos.x, cube2_pos.z).distance(camera_pos_2d);
        let cube2_color = self.fog_settings.apply_fog_to_color(
            grayscale(0.7 * ambient_light), 
            distance2
        );
        draw_cube(cube2_pos, vec3(1.0, 4.0, 1.0), None, cube2_color);
        
        let cube3_pos = vec3(-5.0, 1.5, 5.0);
        let distance3 = vec2(cube3_pos.x, cube3_pos.z).distance(camera_pos_2d);
        let cube3_color = self.fog_settings.apply_fog_to_color(
            grayscale(0.3 * ambient_light), 
            distance3
        );
        draw_cube(cube3_pos, vec3(1.0, 3.0, 1.0), None, cube3_color);

        self.renderer.render_all_with_culling(&camera);

        set_default_camera();

        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            20.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Pos: {:.1}, {:.1}, {:.1}", 
                self.player.position.x, 
                self.player.position.y, 
                self.player.position.z
            ),
            10.0,
            40.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Grounded: {}", self.player.is_grounded),
            10.0,
            60.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Stamina: {:.1}%", self.player.stamina),
            10.0,
            80.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Sprinting: {}", self.player.is_sprinting),
            10.0,
            100.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Time: {:.2} | Light: {:.2}", 
                self.world_state.time_of_day,
                ambient_light
            ),
            10.0,
            120.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            &format!("Weather: {:?} | Fog: {:.3}", 
                self.world_state.weather,
                self.world_state.get_fog_density()
            ),
            10.0,
            140.0,
            20.0,
            grayscale(1.0),
        );
        draw_text(
            "1: Clear | 2: Light Fog | 3: Heavy Fog",
            10.0,
            160.0,
            16.0,
            grayscale(0.7),
        );
        draw_text(
            &format!("Selected Shade: {} | Press G to toggle shade selector", 
                self.player.selected_gray_shade
            ),
            10.0,
            180.0,
            16.0,
            grayscale(0.7),
        );
        draw_text(
            "Left Click to draw on surfaces",
            10.0,
            200.0,
            16.0,
            grayscale(0.7),
        );

        self.stamina_hud.draw(self.player.stamina, dt);
        self.shade_selector.draw(self.player.selected_gray_shade);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed = 12345u64;
    let mut game_state = GameState::new(seed);
    let mut input_state = InputState::new();

    set_cursor_grab(true);
    show_mouse(false);

    loop {
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        input_state.update();

        game_state.handle_input(&input_state);

        game_state.update(&input_state, dt);

        clear_background(grayscale(0.196));

        game_state.render(dt);

        next_frame().await;
    }
}
