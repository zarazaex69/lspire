use macroquad::prelude::*;

pub struct StaminaHUD {
    displayed_stamina: f32,
    warning_flash_timer: f32,
}

impl StaminaHUD {
    pub fn new() -> Self {
        Self {
            displayed_stamina: 100.0,
            warning_flash_timer: 0.0,
        }
    }

    pub fn draw(&mut self, current_stamina: f32, dt: f32) {
        let lerp_speed = 5.0;
        self.displayed_stamina += (current_stamina - self.displayed_stamina) * lerp_speed * dt;

        let screen_width = screen_width();
        let screen_height = screen_height();

        let bar_width = 300.0;
        let bar_height = 20.0;
        let bar_x = (screen_width - bar_width) / 2.0;
        let bar_y = screen_height - 60.0;

        let bg_color = Color::from_rgba(40, 40, 40, 200);
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, bg_color);

        let fill_width = (self.displayed_stamina / 100.0) * bar_width;

        let is_warning = current_stamina < 30.0;
        let bar_color = if is_warning {
            self.warning_flash_timer += dt;
            let flash_frequency = 3.0;
            let flash_value = (self.warning_flash_timer * flash_frequency).sin() * 0.5 + 0.5;
            let intensity = (100.0 + flash_value * 100.0) as u8;
            Color::from_rgba(intensity, intensity, intensity, 255)
        } else {
            Color::from_rgba(200, 200, 200, 255)
        };

        draw_rectangle(bar_x, bar_y, fill_width, bar_height, bar_color);

        let border_color = Color::from_rgba(150, 150, 150, 255);
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, border_color);
    }
}
