use macroquad::prelude::*;

pub fn grayscale(value: f32) -> Color {
    let clamped = value.clamp(0.0, 1.0);
    Color::new(clamped, clamped, clamped, 1.0)
}

pub fn grayscale_with_alpha(value: f32, alpha: f32) -> Color {
    let clamped = value.clamp(0.0, 1.0);
    let clamped_alpha = alpha.clamp(0.0, 1.0);
    Color::new(clamped, clamped, clamped, clamped_alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_basic() {
        let color = grayscale(0.5);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_grayscale_black() {
        let color = grayscale(0.0);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_grayscale_white() {
        let color = grayscale(1.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_grayscale_clamps_above_one() {
        let color = grayscale(1.5);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
    }

    #[test]
    fn test_grayscale_clamps_below_zero() {
        let color = grayscale(-0.5);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_grayscale_with_alpha_basic() {
        let color = grayscale_with_alpha(0.7, 0.8);
        assert_eq!(color.r, 0.7);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_grayscale_with_alpha_clamps_value() {
        let color = grayscale_with_alpha(2.0, 0.5);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
    }

    #[test]
    fn test_grayscale_with_alpha_clamps_alpha() {
        let color = grayscale_with_alpha(0.5, 2.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_grayscale_rgb_equality() {
        let color = grayscale(0.42);
        assert_eq!(color.r, color.g);
        assert_eq!(color.g, color.b);
    }

    #[test]
    fn test_grayscale_with_alpha_rgb_equality() {
        let color = grayscale_with_alpha(0.33, 0.66);
        assert_eq!(color.r, color.g);
        assert_eq!(color.g, color.b);
    }

    #[test]
    fn test_multiple_grayscale_values() {
        let values = [0.0, 0.25, 0.5, 0.75, 1.0];
        for &value in &values {
            let color = grayscale(value);
            assert_eq!(color.r, value);
            assert_eq!(color.g, value);
            assert_eq!(color.b, value);
        }
    }
}
