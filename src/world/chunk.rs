use macroquad::prelude::*;
use std::collections::HashMap;
use super::Spire;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
}

pub struct Chunk {
    pub position: ChunkPos,
    pub spires: Vec<Spire>,
    pub mesh_data: Option<MeshData>,
    pub is_loaded: bool,
}

pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
    load_radius: u32,
    seed: u64,
    generator: super::WorldGenerator,
}

impl ChunkManager {
    pub fn new(seed: u64, load_radius: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            load_radius,
            seed,
            generator: super::WorldGenerator::new(seed),
        }
    }

    pub fn update_loaded_chunks(&mut self, player_pos: Vec3) {
        let player_chunk_x = (player_pos.x / 16.0).floor() as i32;
        let player_chunk_z = (player_pos.z / 16.0).floor() as i32;

        let radius = self.load_radius as i32;

        let mut chunks_to_load = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let chunk_pos = ChunkPos {
                    x: player_chunk_x + dx,
                    z: player_chunk_z + dz,
                };

                if !self.chunks.contains_key(&chunk_pos) {
                    chunks_to_load.push(chunk_pos);
                }
            }
        }

        for pos in chunks_to_load {
            self.generate_chunk(pos);
        }

        let mut chunks_to_unload = Vec::new();
        for (pos, _) in self.chunks.iter() {
            let dx = (pos.x - player_chunk_x).abs();
            let dz = (pos.z - player_chunk_z).abs();

            if dx > radius || dz > radius {
                chunks_to_unload.push(*pos);
            }
        }

        for pos in chunks_to_unload {
            self.unload_chunk(pos);
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn generate_chunk(&mut self, pos: ChunkPos) {
        let spires = self.generator.generate_chunk_data(pos);

        let chunk = Chunk {
            position: pos,
            spires,
            mesh_data: None,
            is_loaded: true,
        };

        self.chunks.insert(pos, chunk);
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_pos_creation() {
        let pos = ChunkPos { x: 5, z: -3 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.z, -3);
    }

    #[test]
    fn test_chunk_pos_equality() {
        let pos1 = ChunkPos { x: 1, z: 2 };
        let pos2 = ChunkPos { x: 1, z: 2 };
        let pos3 = ChunkPos { x: 2, z: 1 };
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_chunk_pos_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        let pos1 = ChunkPos { x: 0, z: 0 };
        let pos2 = ChunkPos { x: 0, z: 0 };
        
        set.insert(pos1);
        set.insert(pos2);
        
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_chunk_creation() {
        let pos = ChunkPos { x: 0, z: 0 };
        let chunk = Chunk {
            position: pos,
            spires: Vec::new(),
            mesh_data: None,
            is_loaded: true,
        };
        
        assert_eq!(chunk.position, pos);
        assert_eq!(chunk.spires.len(), 0);
        assert!(chunk.mesh_data.is_none());
        assert!(chunk.is_loaded);
    }

    #[test]
    fn test_chunk_with_spires() {
        let pos = ChunkPos { x: 1, z: 1 };
        let spire = Spire {
            position: vec3(0.0, 0.0, 0.0),
            height: 50.0,
            radius: 1.0,
            has_pipe: true,
        };
        
        let chunk = Chunk {
            position: pos,
            spires: vec![spire.clone()],
            mesh_data: None,
            is_loaded: true,
        };
        
        assert_eq!(chunk.spires.len(), 1);
        assert_eq!(chunk.spires[0].height, 50.0);
    }

    #[test]
    fn test_chunk_with_mesh_data() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mesh = MeshData {
            vertices: vec![vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)],
            indices: vec![0, 1],
        };
        
        let chunk = Chunk {
            position: pos,
            spires: Vec::new(),
            mesh_data: Some(mesh),
            is_loaded: true,
        };
        
        assert!(chunk.mesh_data.is_some());
        let mesh_ref = chunk.mesh_data.as_ref().unwrap();
        assert_eq!(mesh_ref.vertices.len(), 2);
        assert_eq!(mesh_ref.indices.len(), 2);
    }

    #[test]
    fn test_chunk_manager_creation() {
        let manager = ChunkManager::new(12345, 3);
        assert_eq!(manager.seed, 12345);
        assert_eq!(manager.load_radius, 3);
        assert_eq!(manager.chunks.len(), 0);
    }

    #[test]
    fn test_chunk_manager_get_nonexistent() {
        let manager = ChunkManager::new(12345, 3);
        let pos = ChunkPos { x: 0, z: 0 };
        assert!(manager.get_chunk(pos).is_none());
    }

    #[test]
    fn test_generate_chunk() {
        let mut manager = ChunkManager::new(42, 3);
        let pos = ChunkPos { x: 0, z: 0 };
        
        manager.generate_chunk(pos);
        
        let chunk = manager.get_chunk(pos);
        assert!(chunk.is_some());
        
        let chunk = chunk.unwrap();
        assert_eq!(chunk.position, pos);
        assert!(chunk.is_loaded);
        assert!(!chunk.spires.is_empty());
    }

    #[test]
    fn test_unload_chunk() {
        let mut manager = ChunkManager::new(42, 3);
        let pos = ChunkPos { x: 0, z: 0 };
        
        manager.generate_chunk(pos);
        assert!(manager.get_chunk(pos).is_some());
        
        manager.unload_chunk(pos);
        assert!(manager.get_chunk(pos).is_none());
    }

    #[test]
    fn test_update_loaded_chunks_loads_nearby() {
        let mut manager = ChunkManager::new(123, 1);
        let player_pos = vec3(8.0, 0.0, 8.0);
        
        manager.update_loaded_chunks(player_pos);
        
        let center = ChunkPos { x: 0, z: 0 };
        assert!(manager.get_chunk(center).is_some());
        
        let neighbors = [
            ChunkPos { x: -1, z: 0 },
            ChunkPos { x: 1, z: 0 },
            ChunkPos { x: 0, z: -1 },
            ChunkPos { x: 0, z: 1 },
        ];
        
        for pos in neighbors {
            assert!(manager.get_chunk(pos).is_some(), "Neighbor chunk {:?} should be loaded", pos);
        }
    }

    #[test]
    fn test_update_loaded_chunks_unloads_distant() {
        let mut manager = ChunkManager::new(456, 1);
        
        let far_pos = ChunkPos { x: 10, z: 10 };
        manager.generate_chunk(far_pos);
        assert!(manager.get_chunk(far_pos).is_some());
        
        let player_pos = vec3(0.0, 0.0, 0.0);
        manager.update_loaded_chunks(player_pos);
        
        assert!(manager.get_chunk(far_pos).is_none(), "Far chunk should be unloaded");
    }

    #[test]
    fn test_update_loaded_chunks_respects_radius() {
        let mut manager = ChunkManager::new(789, 2);
        let player_pos = vec3(0.0, 0.0, 0.0);
        
        manager.update_loaded_chunks(player_pos);
        
        let within_radius = ChunkPos { x: 2, z: 2 };
        assert!(manager.get_chunk(within_radius).is_some());
        
        let outside_radius = ChunkPos { x: 3, z: 3 };
        assert!(manager.get_chunk(outside_radius).is_none());
    }

    #[test]
    fn test_chunk_generation_deterministic() {
        let mut manager1 = ChunkManager::new(999, 3);
        let mut manager2 = ChunkManager::new(999, 3);
        let pos = ChunkPos { x: 5, z: -3 };
        
        manager1.generate_chunk(pos);
        manager2.generate_chunk(pos);
        
        let chunk1 = manager1.get_chunk(pos).unwrap();
        let chunk2 = manager2.get_chunk(pos).unwrap();
        
        assert_eq!(chunk1.spires.len(), chunk2.spires.len());
        
        for (s1, s2) in chunk1.spires.iter().zip(chunk2.spires.iter()) {
            assert_eq!(s1.position, s2.position);
            assert_eq!(s1.height, s2.height);
            assert_eq!(s1.has_pipe, s2.has_pipe);
        }
    }
}
