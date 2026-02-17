use macroquad::prelude::*;
use crate::rendering::grayscale;

pub struct ShadeSelector {
    shades: Vec<u8>,
    visible: bool,
    selected_index: usize,
}

impl ShadeSelector {
    pub fn new() -> Self {
        Self {
            shades: vec![0, 36, 73, 109, 146, 182, 219, 255],
            visible: false,
            selected_index: 4,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn handle_input(&mut self) -> Option<u8> {
        if !self.visible {
            return None;
        }

        let mut selection_changed = false;

        if is_key_pressed(KeyCode::Key1) {
            self.selected_index = 0;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key2) {
            self.selected_index = 1;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key3) {
            self.selected_index = 2;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key4) {
            self.selected_index = 3;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key5) {
            self.selected_index = 4;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key6) {
            self.selected_index = 5;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key7) {
            self.selected_index = 6;
            selection_changed = true;
        } else if is_key_pressed(KeyCode::Key8) {
            self.selected_index = 7;
            selection_changed = true;
        }

        if selection_changed {
            Some(self.shades[self.selected_index])
        } else {
            None
        }
    }

    pub fn draw(&self, current_shade: u8) {
        if !self.visible {
            return;
        }

        let screen_width = screen_width();
        let screen_height = screen_height();
        
        let panel_width = 400.0;
        let panel_height = 120.0;
        let panel_x = (screen_width - panel_width) / 2.0;
        let panel_y = screen_height - panel_height - 100.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            grayscale(0.2),
        );

        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            2.0,
            grayscale(0.8),
        );

        draw_text(
            "Select Shade (1-8)",
            panel_x + 10.0,
            panel_y + 25.0,
            20.0,
            grayscale(1.0),
        );

        let swatch_size = 40.0;
        let swatch_spacing = 45.0;
        let start_x = panel_x + 15.0;
        let start_y = panel_y + 45.0;

        for (i, &shade) in self.shades.iter().enumerate() {
            let x = start_x + i as f32 * swatch_spacing;
            let y = start_y;

            let is_selected = shade == current_shade;
            
            draw_rectangle(
                x,
                y,
                swatch_size,
                swatch_size,
                grayscale(shade as f32 / 255.0),
            );

            let border_color = if is_selected {
                grayscale(1.0)
            } else {
                grayscale(0.5)
            };
            let border_width = if is_selected { 3.0 } else { 1.0 };

            draw_rectangle_lines(
                x,
                y,
                swatch_size,
                swatch_size,
                border_width,
                border_color,
            );

            draw_text(
                &format!("{}", i + 1),
                x + swatch_size / 2.0 - 5.0,
                y - 5.0,
                16.0,
                grayscale(0.8),
            );
        }
    }

    pub fn get_selected_shade(&self) -> u8 {
        self.shades[self.selected_index]
    }
}

impl Default for ShadeSelector {
    fn default() -> Self {
        Self::new()
    }
}
