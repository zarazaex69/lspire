use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DrawMark {
    pub position: Vec2,
    pub shade: u8,
    pub size: f32,
}

impl DrawMark {
    pub fn new(position: Vec2, shade: u8, size: f32) -> Self {
        Self {
            position,
            shade,
            size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawingData {
    pub surface_id: u32,
    pub marks: Vec<DrawMark>,
}

impl DrawingData {
    pub fn new(surface_id: u32) -> Self {
        Self {
            surface_id,
            marks: Vec::new(),
        }
    }

    pub fn add_mark(&mut self, mark: DrawMark) {
        self.marks.push(mark);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub surface_id: u32,
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct DrawingSystem {
    drawings: HashMap<u32, DrawingData>,
    texture_cache: HashMap<u32, Texture2D>,
    texture_resolution: u32,
}

impl DrawingSystem {
    pub fn new() -> Self {
        Self {
            drawings: HashMap::new(),
            texture_cache: HashMap::new(),
            texture_resolution: 512,
        }
    }

    pub fn add_mark(&mut self, surface_id: u32, mark: DrawMark) {
        self.drawings
            .entry(surface_id)
            .or_insert_with(|| DrawingData::new(surface_id))
            .add_mark(mark);
        
        self.texture_cache.remove(&surface_id);
    }

    pub fn get_drawing_data(&self, surface_id: u32) -> Option<&DrawingData> {
        self.drawings.get(&surface_id)
    }

    pub fn get_texture(&mut self, surface_id: u32) -> Option<&Texture2D> {
        if !self.texture_cache.contains_key(&surface_id) {
            if let Some(drawing_data) = self.drawings.get(&surface_id) {
                let texture = self.generate_texture(drawing_data);
                self.texture_cache.insert(surface_id, texture);
            }
        }
        self.texture_cache.get(&surface_id)
    }

    fn generate_texture(&self, drawing_data: &DrawingData) -> Texture2D {
        let res = self.texture_resolution as u16;
        let mut image = Image::gen_image_color(res, res, Color::new(0.5, 0.5, 0.5, 1.0));

        for mark in &drawing_data.marks {
            let x = (mark.position.x * res as f32) as i32;
            let y = (mark.position.y * res as f32) as i32;
            let radius = (mark.size * res as f32) as i32;
            
            let shade_f32 = mark.shade as f32 / 255.0;
            let color = Color::new(shade_f32, shade_f32, shade_f32, 1.0);

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let px = x + dx;
                        let py = y + dy;
                        if px >= 0 && px < res as i32 && py >= 0 && py < res as i32 {
                            image.set_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }

        Texture2D::from_image(&image)
    }

    pub fn raycast_surface(&self, ray_origin: Vec3, ray_direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = max_distance;

        let ground_plane_y = 0.0;
        if ray_direction.y.abs() > 0.001 {
            let t = (ground_plane_y - ray_origin.y) / ray_direction.y;
            if t > 0.0 && t < closest_distance {
                let hit_pos = ray_origin + ray_direction * t;
                let distance = (hit_pos - ray_origin).length();
                
                if distance < closest_distance {
                    let surface_id = Self::compute_surface_id(hit_pos, vec3(0.0, 1.0, 0.0));
                    let uv = vec2(
                        (hit_pos.x % 1.0 + 1.0) % 1.0,
                        (hit_pos.z % 1.0 + 1.0) % 1.0,
                    );
                    
                    closest_hit = Some(RaycastHit {
                        surface_id,
                        position: hit_pos,
                        normal: vec3(0.0, 1.0, 0.0),
                        uv,
                    });
                    closest_distance = distance;
                }
            }
        }

        closest_hit
    }

    fn compute_surface_id(position: Vec3, normal: Vec3) -> u32 {
        let grid_x = (position.x / 10.0).floor() as i32;
        let grid_y = (position.y / 10.0).floor() as i32;
        let grid_z = (position.z / 10.0).floor() as i32;
        
        let normal_key = if normal.y.abs() > 0.9 {
            0
        } else if normal.x.abs() > 0.9 {
            1
        } else {
            2
        };

        let hash = ((grid_x as u32).wrapping_mul(73856093))
            ^ ((grid_y as u32).wrapping_mul(19349663))
            ^ ((grid_z as u32).wrapping_mul(83492791))
            ^ (normal_key * 6542989);
        
        hash
    }

    pub fn clear_surface(&mut self, surface_id: u32) {
        self.drawings.remove(&surface_id);
        self.texture_cache.remove(&surface_id);
    }

    pub fn clear_all(&mut self) {
        self.drawings.clear();
        self.texture_cache.clear();
    }
}

impl Default for DrawingSystem {
    fn default() -> Self {
        Self::new()
    }
}
