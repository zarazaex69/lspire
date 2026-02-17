use macroquad::prelude::*;

pub struct InputState {
    pub move_forward: bool,
    pub move_back: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub draw: bool,
    pub mouse_delta: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            move_forward: false,
            move_back: false,
            move_left: false,
            move_right: false,
            jump: false,
            draw: false,
            mouse_delta: Vec2::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.move_forward = is_key_down(KeyCode::W);
        self.move_back = is_key_down(KeyCode::S);
        self.move_left = is_key_down(KeyCode::A);
        self.move_right = is_key_down(KeyCode::D);
        self.jump = is_key_pressed(KeyCode::Space);
        self.draw = is_mouse_button_down(MouseButton::Left);
        self.mouse_delta = mouse_delta_position();
    }
}
