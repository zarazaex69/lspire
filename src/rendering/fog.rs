use macroquad::prelude::*;
use crate::rendering::color::grayscale;

#[derive(Clone, Copy, Debug)]
pub struct FogSettings {
    pub density: f32,
    pub color: Color,
    pub start_distance: f32,
    pub end_distance: f32,
}

impl FogSettings {
    pub fn new(density: f32, start_distance: f32, end_distance: f32) -> Self {
        Self {
            density,
            color: grayscale(0.196),
            start_distance,
            end_distance,
        }
    }

    pub fn default() -> Self {
        Self::new(0.5, 20.0, 100.0)
    }

    pub fn calculate_fog_factor(&self, distance: f32) -> f32 {
        if distance <= self.start_distance {
            0.0
        } else if distance >= self.end_distance {
            1.0
        } else {
            let range = self.end_distance - self.start_distance;
            let normalized_distance = (distance - self.start_distance) / range;
            (normalized_distance * self.density).min(1.0)
        }
    }

    pub fn apply_fog_to_color(&self, original_color: Color, distance: f32) -> Color {
        let fog_factor = self.calculate_fog_factor(distance);
        Color::new(
            original_color.r * (1.0 - fog_factor) + self.color.r * fog_factor,
            original_color.g * (1.0 - fog_factor) + self.color.g * fog_factor,
            original_color.b * (1.0 - fog_factor) + self.color.b * fog_factor,
            original_color.a,
        )
    }

    pub fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_distances(&mut self, start: f32, end: f32) {
        self.start_distance = start.max(0.0);
        self.end_distance = end.max(start);
    }
}

impl Default for FogSettings {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fog_factor_before_start() {
        let fog = FogSettings::new(0.5, 20.0, 100.0);
        let factor = fog.calculate_fog_factor(10.0);
        assert_eq!(factor, 0.0);
    }

    #[test]
    fn test_fog_factor_after_end() {
        let fog = FogSettings::new(0.5, 20.0, 100.0);
        let factor = fog.calculate_fog_factor(150.0);
        assert_eq!(factor, 1.0);
    }

    #[test]
    fn test_fog_factor_midpoint() {
        let fog = FogSettings::new(1.0, 20.0, 100.0);
        let factor = fog.calculate_fog_factor(60.0);
        assert!(factor > 0.0 && factor < 1.0);
    }

    #[test]
    fn test_fog_factor_increases_with_distance() {
        let fog = FogSettings::new(1.0, 20.0, 100.0);
        let factor1 = fog.calculate_fog_factor(30.0);
        let factor2 = fog.calculate_fog_factor(70.0);
        assert!(factor2 > factor1);
    }

    #[test]
    fn test_apply_fog_to_color() {
        let fog = FogSettings::new(1.0, 20.0, 100.0);
        let original = grayscale(1.0);
        let fogged = fog.apply_fog_to_color(original, 60.0);
        
        assert!(fogged.r < original.r);
        assert!(fogged.g < original.g);
        assert!(fogged.b < original.b);
    }

    #[test]
    fn test_set_density_clamps() {
        let mut fog = FogSettings::default();
        fog.set_density(1.5);
        assert_eq!(fog.density, 1.0);
        
        fog.set_density(-0.5);
        assert_eq!(fog.density, 0.0);
    }

    #[test]
    fn test_set_distances_validation() {
        let mut fog = FogSettings::default();
        fog.set_distances(50.0, 30.0);
        assert!(fog.end_distance >= fog.start_distance);
    }
}
