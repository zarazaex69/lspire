use macroquad::prelude::*;
use super::frustum::Frustum;

#[derive(Clone, Debug)]
pub struct InstanceData {
    pub transform: Mat4,
    pub color: Color,
    pub bounding_radius: f32,
}

pub struct InstancedRenderer {
    spire_instances: Vec<InstanceData>,
    pipe_instances: Vec<InstanceData>,
    max_instances: usize,
}

impl InstancedRenderer {
    pub fn new(max_instances: usize) -> Self {
        Self {
            spire_instances: Vec::with_capacity(max_instances),
            pipe_instances: Vec::with_capacity(max_instances),
            max_instances,
        }
    }

    pub fn add_instance(&mut self, instance: InstanceData, is_pipe: bool) {
        if is_pipe {
            if self.pipe_instances.len() < self.max_instances {
                self.pipe_instances.push(instance);
            }
        } else {
            if self.spire_instances.len() < self.max_instances {
                self.spire_instances.push(instance);
            }
        }
    }

    pub fn render_all(&self) {
        for instance in &self.spire_instances {
            self.render_instance(instance);
        }
        
        for instance in &self.pipe_instances {
            self.render_instance(instance);
        }
    }

    pub fn render_all_with_culling(&self, camera: &Camera3D) {
        let frustum = Frustum::from_camera(camera);
        
        for instance in &self.spire_instances {
            let position = vec3(
                instance.transform.w_axis.x,
                instance.transform.w_axis.y,
                instance.transform.w_axis.z
            );
            
            if frustum.contains_sphere(position, instance.bounding_radius) {
                self.render_instance(instance);
            }
        }
        
        for instance in &self.pipe_instances {
            let position = vec3(
                instance.transform.w_axis.x,
                instance.transform.w_axis.y,
                instance.transform.w_axis.z
            );
            
            if frustum.contains_sphere(position, instance.bounding_radius) {
                self.render_instance(instance);
            }
        }
    }

    fn render_instance(&self, instance: &InstanceData) {
        let transform = instance.transform;
        let position = vec3(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);
        
        draw_cube(position, vec3(1.0, 1.0, 1.0), None, instance.color);
    }

    pub fn clear(&mut self) {
        self.spire_instances.clear();
        self.pipe_instances.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::color::grayscale;

    #[test]
    fn test_new_renderer_empty() {
        let renderer = InstancedRenderer::new(100);
        assert_eq!(renderer.spire_instances.len(), 0);
        assert_eq!(renderer.pipe_instances.len(), 0);
    }

    #[test]
    fn test_add_spire_instance() {
        let mut renderer = InstancedRenderer::new(100);
        let instance = InstanceData {
            transform: Mat4::IDENTITY,
            color: grayscale(0.5),
            bounding_radius: 1.0,
        };
        
        renderer.add_instance(instance, false);
        assert_eq!(renderer.spire_instances.len(), 1);
        assert_eq!(renderer.pipe_instances.len(), 0);
    }

    #[test]
    fn test_add_pipe_instance() {
        let mut renderer = InstancedRenderer::new(100);
        let instance = InstanceData {
            transform: Mat4::IDENTITY,
            color: grayscale(0.3),
            bounding_radius: 0.5,
        };
        
        renderer.add_instance(instance, true);
        assert_eq!(renderer.spire_instances.len(), 0);
        assert_eq!(renderer.pipe_instances.len(), 1);
    }

    #[test]
    fn test_add_multiple_instances() {
        let mut renderer = InstancedRenderer::new(100);
        
        for i in 0..10 {
            let instance = InstanceData {
                transform: Mat4::from_translation(vec3(i as f32, 0.0, 0.0)),
                color: grayscale(0.5),
                bounding_radius: 1.0,
            };
            renderer.add_instance(instance, false);
        }
        
        for i in 0..5 {
            let instance = InstanceData {
                transform: Mat4::from_translation(vec3(0.0, i as f32, 0.0)),
                color: grayscale(0.3),
                bounding_radius: 0.5,
            };
            renderer.add_instance(instance, true);
        }
        
        assert_eq!(renderer.spire_instances.len(), 10);
        assert_eq!(renderer.pipe_instances.len(), 5);
    }

    #[test]
    fn test_max_instances_limit() {
        let mut renderer = InstancedRenderer::new(5);
        
        for i in 0..10 {
            let instance = InstanceData {
                transform: Mat4::from_translation(vec3(i as f32, 0.0, 0.0)),
                color: grayscale(0.5),
                bounding_radius: 1.0,
            };
            renderer.add_instance(instance, false);
        }
        
        assert_eq!(renderer.spire_instances.len(), 5);
    }

    #[test]
    fn test_clear_instances() {
        let mut renderer = InstancedRenderer::new(100);
        
        for i in 0..10 {
            let instance = InstanceData {
                transform: Mat4::IDENTITY,
                color: grayscale(0.5),
                bounding_radius: 1.0,
            };
            renderer.add_instance(instance, i % 2 == 0);
        }
        
        assert!(renderer.spire_instances.len() > 0 || renderer.pipe_instances.len() > 0);
        
        renderer.clear();
        
        assert_eq!(renderer.spire_instances.len(), 0);
        assert_eq!(renderer.pipe_instances.len(), 0);
    }

    #[test]
    fn test_instance_data_transform() {
        let position = vec3(10.0, 20.0, 30.0);
        let transform = Mat4::from_translation(position);
        let instance = InstanceData {
            transform,
            color: grayscale(0.5),
            bounding_radius: 1.0,
        };
        
        assert_eq!(instance.transform.w_axis.x, position.x);
        assert_eq!(instance.transform.w_axis.y, position.y);
        assert_eq!(instance.transform.w_axis.z, position.z);
    }

    #[test]
    fn test_instance_data_color() {
        let color = grayscale(0.7);
        let instance = InstanceData {
            transform: Mat4::IDENTITY,
            color,
            bounding_radius: 1.0,
        };
        
        assert_eq!(instance.color.r, 0.7);
        assert_eq!(instance.color.g, 0.7);
        assert_eq!(instance.color.b, 0.7);
        assert_eq!(instance.color.a, 1.0);
    }

    #[test]
    fn test_separate_spire_and_pipe_limits() {
        let mut renderer = InstancedRenderer::new(3);
        
        for i in 0..5 {
            let instance = InstanceData {
                transform: Mat4::from_translation(vec3(i as f32, 0.0, 0.0)),
                color: grayscale(0.5),
                bounding_radius: 1.0,
            };
            renderer.add_instance(instance, false);
        }
        
        for i in 0..5 {
            let instance = InstanceData {
                transform: Mat4::from_translation(vec3(0.0, i as f32, 0.0)),
                color: grayscale(0.3),
                bounding_radius: 0.5,
            };
            renderer.add_instance(instance, true);
        }
        
        assert_eq!(renderer.spire_instances.len(), 3);
        assert_eq!(renderer.pipe_instances.len(), 3);
    }

    #[test]
    fn test_instance_data_bounding_radius() {
        let instance = InstanceData {
            transform: Mat4::IDENTITY,
            color: grayscale(0.5),
            bounding_radius: 5.0,
        };
        
        assert_eq!(instance.bounding_radius, 5.0);
    }
}
