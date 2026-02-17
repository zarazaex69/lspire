use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};
use super::ChunkPos;

const CHUNK_SIZE: i32 = 16;
const SPIRE_SPACING: f32 = 4.0;
const MIN_SPIRE_HEIGHT: f32 = 10.0;
const MAX_SPIRE_HEIGHT: f32 = 100.0;
const NOISE_SCALE: f64 = 0.05;
const PIPE_THRESHOLD: f64 = 0.3;

#[derive(Debug, Clone)]
pub struct Spire {
    pub position: Vec3,
    pub height: f32,
    pub radius: f32,
    pub has_pipe: bool,
}

pub struct WorldGenerator {
    seed: u64,
    noise: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            noise: Perlin::new(seed as u32),
        }
    }

    pub fn generate_chunk_data(&self, chunk_pos: ChunkPos) -> Vec<Spire> {
        let mut spires = Vec::new();
        
        let chunk_world_x = chunk_pos.x * CHUNK_SIZE;
        let chunk_world_z = chunk_pos.z * CHUNK_SIZE;
        
        for local_x in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                if local_x % 4 == 0 && local_z % 4 == 0 {
                    let world_x = chunk_world_x + local_x;
                    let world_z = chunk_world_z + local_z;
                    
                    let x_f = world_x as f32;
                    let z_f = world_z as f32;
                    
                    let height = self.calculate_spire_height(x_f, z_f);
                    let has_pipe = self.should_place_pipe(x_f, z_f);
                    
                    spires.push(Spire {
                        position: vec3(x_f, 0.0, z_f),
                        height,
                        radius: 1.0,
                        has_pipe,
                    });
                }
            }
        }
        
        spires
    }

    fn calculate_spire_height(&self, x: f32, z: f32) -> f32 {
        let noise_value = self.noise.get([x as f64 * NOISE_SCALE, z as f64 * NOISE_SCALE]);
        
        let normalized = (noise_value + 1.0) / 2.0;
        
        let height = MIN_SPIRE_HEIGHT + normalized as f32 * (MAX_SPIRE_HEIGHT - MIN_SPIRE_HEIGHT);
        
        height.clamp(MIN_SPIRE_HEIGHT, MAX_SPIRE_HEIGHT)
    }

    fn should_place_pipe(&self, x: f32, z: f32) -> bool {
        let offset_x = (x as f64 + 1000.0) * NOISE_SCALE;
        let offset_z = (z as f64 + 2000.0) * NOISE_SCALE;
        
        let noise_value = self.noise.get([offset_x, offset_z]);
        
        noise_value > PIPE_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_generator_creation() {
        let gen = WorldGenerator::new(12345);
        assert_eq!(gen.seed, 12345);
    }

    #[test]
    fn test_generate_chunk_data_not_empty() {
        let gen = WorldGenerator::new(42);
        let pos = ChunkPos { x: 0, z: 0 };
        let spires = gen.generate_chunk_data(pos);
        
        assert!(!spires.is_empty());
    }

    #[test]
    fn test_spire_height_bounds() {
        let gen = WorldGenerator::new(999);
        
        for x in -10..10 {
            for z in -10..10 {
                let height = gen.calculate_spire_height(x as f32, z as f32);
                assert!(height >= MIN_SPIRE_HEIGHT, "Height {} is below minimum", height);
                assert!(height <= MAX_SPIRE_HEIGHT, "Height {} is above maximum", height);
            }
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let gen = WorldGenerator::new(12345);
        let pos = ChunkPos { x: 5, z: -3 };
        
        let spires1 = gen.generate_chunk_data(pos);
        let spires2 = gen.generate_chunk_data(pos);
        
        assert_eq!(spires1.len(), spires2.len());
        
        for (s1, s2) in spires1.iter().zip(spires2.iter()) {
            assert_eq!(s1.position, s2.position);
            assert_eq!(s1.height, s2.height);
            assert_eq!(s1.has_pipe, s2.has_pipe);
        }
    }

    #[test]
    fn test_different_seeds_produce_different_results() {
        let gen1 = WorldGenerator::new(111);
        let gen2 = WorldGenerator::new(222);
        let pos = ChunkPos { x: 0, z: 0 };
        
        let spires1 = gen1.generate_chunk_data(pos);
        let spires2 = gen2.generate_chunk_data(pos);
        
        assert_eq!(spires1.len(), spires2.len());
        
        let mut has_difference = false;
        for (s1, s2) in spires1.iter().zip(spires2.iter()) {
            if s1.height != s2.height || s1.has_pipe != s2.has_pipe {
                has_difference = true;
                break;
            }
        }
        
        assert!(has_difference, "Different seeds should produce different results");
    }

    #[test]
    fn test_pipe_placement_deterministic() {
        let gen = WorldGenerator::new(777);
        
        let has_pipe1 = gen.should_place_pipe(10.0, 20.0);
        let has_pipe2 = gen.should_place_pipe(10.0, 20.0);
        
        assert_eq!(has_pipe1, has_pipe2);
    }

    #[test]
    fn test_spire_spacing_in_chunk() {
        let gen = WorldGenerator::new(555);
        let pos = ChunkPos { x: 0, z: 0 };
        let spires = gen.generate_chunk_data(pos);
        
        for spire in &spires {
            let x = spire.position.x as i32;
            let z = spire.position.z as i32;
            
            assert_eq!(x % 4, 0, "Spire x position should be multiple of 4");
            assert_eq!(z % 4, 0, "Spire z position should be multiple of 4");
        }
    }

    #[test]
    fn test_chunk_boundary_generation() {
        let gen = WorldGenerator::new(333);
        
        let pos1 = ChunkPos { x: 0, z: 0 };
        let pos2 = ChunkPos { x: 1, z: 0 };
        
        let spires1 = gen.generate_chunk_data(pos1);
        let spires2 = gen.generate_chunk_data(pos2);
        
        assert!(!spires1.is_empty());
        assert!(!spires2.is_empty());
        
        for s1 in &spires1 {
            for s2 in &spires2 {
                assert_ne!(s1.position, s2.position, "Adjacent chunks should not have overlapping spires");
            }
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_property_5_deterministic_generation(
            seed in any::<u64>(),
            chunk_x in -100i32..100,
            chunk_z in -100i32..100
        ) {
            let gen = WorldGenerator::new(seed);
            let pos = ChunkPos { x: chunk_x, z: chunk_z };
            
            let result1 = gen.generate_chunk_data(pos);
            let result2 = gen.generate_chunk_data(pos);
            
            prop_assert_eq!(result1.len(), result2.len(), 
                "Chunk should have same number of spires on regeneration");
            
            for (spire1, spire2) in result1.iter().zip(result2.iter()) {
                prop_assert_eq!(spire1.position, spire2.position,
                    "Spire positions should be identical for same seed and chunk");
                prop_assert_eq!(spire1.height, spire2.height,
                    "Spire heights should be identical for same seed and chunk");
                prop_assert_eq!(spire1.radius, spire2.radius,
                    "Spire radii should be identical for same seed and chunk");
                prop_assert_eq!(spire1.has_pipe, spire2.has_pipe,
                    "Pipe placement should be identical for same seed and chunk");
            }
        }

        #[test]
        fn test_property_7_spire_height_bounds(
            seed in any::<u64>(),
            chunk_x in -100i32..100,
            chunk_z in -100i32..100
        ) {
            let gen = WorldGenerator::new(seed);
            let pos = ChunkPos { x: chunk_x, z: chunk_z };
            
            let spires = gen.generate_chunk_data(pos);
            
            for spire in spires.iter() {
                prop_assert!(
                    spire.height >= MIN_SPIRE_HEIGHT,
                    "Spire height {} is below minimum {}",
                    spire.height,
                    MIN_SPIRE_HEIGHT
                );
                prop_assert!(
                    spire.height <= MAX_SPIRE_HEIGHT,
                    "Spire height {} is above maximum {}",
                    spire.height,
                    MAX_SPIRE_HEIGHT
                );
            }
        }
    }
}
