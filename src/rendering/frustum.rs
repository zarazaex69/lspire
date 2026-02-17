use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let normalized = normal.normalize();
        let distance = normalized.dot(point);
        Self {
            normal: normalized,
            distance,
        }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_camera(camera: &Camera3D) -> Self {
        Self::from_camera_with_aspect(camera, screen_width() / screen_height())
    }

    pub fn from_camera_with_aspect(camera: &Camera3D, aspect: f32) -> Self {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward).normalize();

        let half_v_side = (camera.fovy / 2.0).tan();
        let half_h_side = half_v_side * aspect;

        let far_distance = 1000.0;
        let near_distance = 0.1;

        let front_mult_far = far_distance * forward;
        let front_mult_near = near_distance * forward;

        let near_center = camera.position + front_mult_near;
        let far_center = camera.position + front_mult_far;

        let near_plane = Plane::from_point_normal(near_center, forward);
        let far_plane = Plane::from_point_normal(far_center, -forward);

        let far_top = far_center + up * half_v_side * far_distance;
        let far_bottom = far_center - up * half_v_side * far_distance;
        let far_right = far_center + right * half_h_side * far_distance;
        let far_left = far_center - right * half_h_side * far_distance;

        let top_normal = (far_top - camera.position).cross(right).normalize();
        let bottom_normal = right.cross(far_bottom - camera.position).normalize();
        let right_normal = up.cross(far_right - camera.position).normalize();
        let left_normal = (far_left - camera.position).cross(up).normalize();

        let top_plane = Plane::from_point_normal(camera.position, top_normal);
        let bottom_plane = Plane::from_point_normal(camera.position, bottom_normal);
        let right_plane = Plane::from_point_normal(camera.position, right_normal);
        let left_plane = Plane::from_point_normal(camera.position, left_normal);

        Self {
            planes: [
                near_plane,
                far_plane,
                right_plane,
                left_plane,
                top_plane,
                bottom_plane,
            ],
        }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }

    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let p = vec3(
                if plane.normal.x > 0.0 { max.x } else { min.x },
                if plane.normal.y > 0.0 { max.y } else { min.y },
                if plane.normal.z > 0.0 { max.z } else { min.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance_to_point() {
        let plane = Plane::from_point_normal(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
        
        assert!((plane.distance_to_point(vec3(0.0, 5.0, 0.0)) - 5.0).abs() < 0.001);
        assert!((plane.distance_to_point(vec3(0.0, -5.0, 0.0)) + 5.0).abs() < 0.001);
        assert!(plane.distance_to_point(vec3(0.0, 0.0, 0.0)).abs() < 0.001);
    }

    #[test]
    fn test_frustum_contains_point_inside() {
        let camera = Camera3D {
            position: vec3(0.0, 0.0, 0.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 60.0f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };

        let frustum = Frustum::from_camera_with_aspect(&camera, 16.0 / 9.0);
        
        assert!(frustum.contains_point(vec3(0.0, 0.0, -5.0)));
    }

    #[test]
    fn test_frustum_contains_point_behind() {
        let camera = Camera3D {
            position: vec3(0.0, 0.0, 0.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 60.0f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };

        let frustum = Frustum::from_camera_with_aspect(&camera, 16.0 / 9.0);
        
        assert!(!frustum.contains_point(vec3(0.0, 0.0, 5.0)));
    }

    #[test]
    fn test_frustum_contains_sphere() {
        let camera = Camera3D {
            position: vec3(0.0, 0.0, 0.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 60.0f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };

        let frustum = Frustum::from_camera_with_aspect(&camera, 16.0 / 9.0);
        
        assert!(frustum.contains_sphere(vec3(0.0, 0.0, -5.0), 1.0));
        
        assert!(!frustum.contains_sphere(vec3(0.0, 0.0, 10.0), 1.0));
    }

    #[test]
    fn test_frustum_contains_aabb() {
        let camera = Camera3D {
            position: vec3(0.0, 0.0, 0.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 60.0f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };

        let frustum = Frustum::from_camera_with_aspect(&camera, 16.0 / 9.0);
        
        assert!(frustum.contains_aabb(
            vec3(-1.0, -1.0, -6.0),
            vec3(1.0, 1.0, -4.0)
        ));
        
        assert!(!frustum.contains_aabb(
            vec3(-1.0, -1.0, 4.0),
            vec3(1.0, 1.0, 6.0)
        ));
    }
}
