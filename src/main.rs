use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "lspire".to_owned(),
        window_width: 1920,
        window_height: 1080,
        sample_count: 4,
        ..Default::default()
    }
}

struct FlyCamera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl FlyCamera {
    fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn update(&mut self, delta: f32) {
        let speed = 5.0 * delta;
        let mouse_sensitivity = 0.3;

        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let forward = vec3(sin_yaw, 0.0, cos_yaw);
        let right = vec3(cos_yaw, 0.0, -sin_yaw);

        if is_key_down(KeyCode::W) {
            self.position += forward * speed;
        }
        if is_key_down(KeyCode::S) {
            self.position -= forward * speed;
        }
        if is_key_down(KeyCode::A) {
            self.position += right * speed;
        }
        if is_key_down(KeyCode::D) {
            self.position -= right * speed;
        }
        if is_key_down(KeyCode::Space) {
            self.position.y += speed;
        }
        if is_key_down(KeyCode::LeftShift) {
            self.position.y -= speed;
        }

        let mouse_delta = mouse_delta_position();
        self.yaw += mouse_delta.x * mouse_sensitivity;
        self.pitch += mouse_delta.y * mouse_sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    fn get_target(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        
        self.position + vec3(
            sin_yaw * cos_pitch,
            sin_pitch,
            cos_yaw * cos_pitch,
        )
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut camera = FlyCamera::new(vec3(0.0, 5.0, -10.0));
    
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        let delta = get_frame_time();
        
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        camera.update(delta);

        clear_background(DARKGRAY);

        set_camera(&Camera3D {
            position: camera.position,
            target: camera.get_target(),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            projection: Projection::Perspective,
            ..Default::default()
        });

        draw_grid(20, 1.0, GRAY, LIGHTGRAY);
        
        draw_cube(vec3(0.0, 0.5, 0.0), vec3(1.0, 1.0, 1.0), None, GRAY);
        draw_cube(vec3(3.0, 1.0, 2.0), vec3(1.0, 2.0, 1.0), None, DARKGRAY);
        draw_cube(vec3(-2.0, 0.75, -3.0), vec3(1.0, 1.5, 1.0), None, LIGHTGRAY);

        set_default_camera();

        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Pos: {:.1}, {:.1}, {:.1}", camera.position.x, camera.position.y, camera.position.z),
            10.0,
            40.0,
            20.0,
            WHITE,
        );

        next_frame().await;
    }
}
