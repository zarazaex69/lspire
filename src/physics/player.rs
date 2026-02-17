use macroquad::prelude::*;

pub struct Player {
    pub id: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: f32,
    pub is_grounded: bool,
    pub selected_gray_shade: u8,
}

impl Player {
    pub fn new(id: u8, position: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            rotation: 0.0,
            is_grounded: false,
            selected_gray_shade: 128,
        }
    }
}

pub struct PlayerController {
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 8.0,
            gravity: 20.0,
        }
    }

    pub fn update(&self, _player: &mut Player, _dt: f32) {
    }

    pub fn apply_gravity(&self, player: &mut Player, dt: f32) {
        player.velocity.y -= self.gravity * dt;
    }
}
