use macroquad::prelude::*;

pub struct SpireMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
    pub normals: Vec<Vec3>,
}

pub struct PipeMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
    pub normals: Vec<Vec3>,
}

pub fn generate_spire_mesh(height: f32, radius: f32) -> SpireMesh {
    let segments = 6;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    let tip = vec3(0.0, height, 0.0);
    vertices.push(tip);
    normals.push(vec3(0.0, 1.0, 0.0));

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        vertices.push(vec3(x, 0.0, z));
        
        let normal = vec3(x, 0.0, z).normalize();
        normals.push(normal);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push((i + 1) as u16);
        indices.push((next + 1) as u16);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push((i + 1) as u16);
        indices.push((next + 1) as u16);
        indices.push((segments + 1) as u16);
    }

    let base_center = vec3(0.0, 0.0, 0.0);
    vertices.push(base_center);
    normals.push(vec3(0.0, -1.0, 0.0));

    SpireMesh {
        vertices,
        indices,
        normals,
    }
}

pub fn generate_pipe_mesh(pipe_height: f32, pipe_radius: f32) -> PipeMesh {
    let segments = 8;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = angle.cos() * pipe_radius;
        let z = angle.sin() * pipe_radius;
        
        vertices.push(vec3(x, 0.0, z));
        vertices.push(vec3(x, pipe_height, z));
        
        let normal = vec3(x, 0.0, z).normalize();
        normals.push(normal);
        normals.push(normal);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let base_current = (i * 2) as u16;
        let top_current = (i * 2 + 1) as u16;
        let base_next = (next * 2) as u16;
        let top_next = (next * 2 + 1) as u16;

        indices.push(base_current);
        indices.push(top_current);
        indices.push(base_next);

        indices.push(base_next);
        indices.push(top_current);
        indices.push(top_next);
    }

    let top_center_idx = vertices.len() as u16;
    vertices.push(vec3(0.0, pipe_height, 0.0));
    normals.push(vec3(0.0, 1.0, 0.0));

    for i in 0..segments {
        let next = (i + 1) % segments;
        let top_current = (i * 2 + 1) as u16;
        let top_next = (next * 2 + 1) as u16;

        indices.push(top_center_idx);
        indices.push(top_current);
        indices.push(top_next);
    }

    PipeMesh {
        vertices,
        indices,
        normals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_spire_mesh_basic() {
        let mesh = generate_spire_mesh(50.0, 1.0);
        
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }

    #[test]
    fn test_spire_mesh_has_tip() {
        let height = 75.0;
        let mesh = generate_spire_mesh(height, 1.0);
        
        let tip = mesh.vertices[0];
        assert_eq!(tip.y, height);
        assert_eq!(tip.x, 0.0);
        assert_eq!(tip.z, 0.0);
    }

    #[test]
    fn test_spire_mesh_base_vertices() {
        let radius = 2.0;
        let mesh = generate_spire_mesh(50.0, radius);
        
        for i in 1..7 {
            let vertex = mesh.vertices[i];
            assert_eq!(vertex.y, 0.0);
            
            let distance = (vertex.x * vertex.x + vertex.z * vertex.z).sqrt();
            assert!((distance - radius).abs() < 0.001);
        }
    }

    #[test]
    fn test_spire_mesh_indices_divisible_by_three() {
        let mesh = generate_spire_mesh(50.0, 1.0);
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_spire_mesh_indices_in_bounds() {
        let mesh = generate_spire_mesh(50.0, 1.0);
        let vertex_count = mesh.vertices.len() as u16;
        
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn test_generate_pipe_mesh_basic() {
        let mesh = generate_pipe_mesh(3.0, 0.3);
        
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }

    #[test]
    fn test_pipe_mesh_height() {
        let pipe_height = 5.0;
        let mesh = generate_pipe_mesh(pipe_height, 0.5);
        
        let mut has_base = false;
        let mut has_top = false;
        
        for vertex in &mesh.vertices {
            if vertex.y == 0.0 {
                has_base = true;
            }
            if (vertex.y - pipe_height).abs() < 0.001 {
                has_top = true;
            }
        }
        
        assert!(has_base);
        assert!(has_top);
    }

    #[test]
    fn test_pipe_mesh_radius() {
        let pipe_radius = 0.4;
        let mesh = generate_pipe_mesh(3.0, pipe_radius);
        
        for vertex in &mesh.vertices {
            if vertex.y >= 0.0 {
                let distance = (vertex.x * vertex.x + vertex.z * vertex.z).sqrt();
                if distance > 0.001 {
                    assert!((distance - pipe_radius).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn test_pipe_mesh_indices_divisible_by_three() {
        let mesh = generate_pipe_mesh(3.0, 0.3);
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_pipe_mesh_indices_in_bounds() {
        let mesh = generate_pipe_mesh(3.0, 0.3);
        let vertex_count = mesh.vertices.len() as u16;
        
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn test_spire_mesh_different_heights() {
        let mesh1 = generate_spire_mesh(10.0, 1.0);
        let mesh2 = generate_spire_mesh(100.0, 1.0);
        
        assert_eq!(mesh1.vertices.len(), mesh2.vertices.len());
        assert_ne!(mesh1.vertices[0].y, mesh2.vertices[0].y);
    }

    #[test]
    fn test_spire_mesh_different_radii() {
        let mesh1 = generate_spire_mesh(50.0, 0.5);
        let mesh2 = generate_spire_mesh(50.0, 2.0);
        
        assert_eq!(mesh1.vertices.len(), mesh2.vertices.len());
        
        let dist1 = (mesh1.vertices[1].x * mesh1.vertices[1].x + 
                     mesh1.vertices[1].z * mesh1.vertices[1].z).sqrt();
        let dist2 = (mesh2.vertices[1].x * mesh2.vertices[1].x + 
                     mesh2.vertices[1].z * mesh2.vertices[1].z).sqrt();
        
        assert!(dist2 > dist1);
    }

    #[test]
    fn test_pipe_mesh_different_dimensions() {
        let mesh1 = generate_pipe_mesh(2.0, 0.2);
        let mesh2 = generate_pipe_mesh(5.0, 0.5);
        
        assert_eq!(mesh1.vertices.len(), mesh2.vertices.len());
        assert_ne!(mesh1.vertices[1].y, mesh2.vertices[1].y);
    }

    #[test]
    fn test_spire_normals_normalized() {
        let mesh = generate_spire_mesh(50.0, 1.0);
        
        for normal in &mesh.normals {
            let length = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_pipe_normals_normalized() {
        let mesh = generate_pipe_mesh(3.0, 0.3);
        
        for normal in &mesh.normals {
            let length = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        }
    }
}
