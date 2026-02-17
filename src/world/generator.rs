use macroquad::prelude::*;
use super::ChunkPos;

#[derive(Debug, Clone)]
pub struct Spire {
    pub position: Vec3,
    pub height: f32,
    pub radius: f32,
    pub has_pipe: bool,
}

pub struct WorldGenerator {
    seed: u64,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate_chunk_data(&self, _chunk_pos: ChunkPos) -> Vec<Spire> {
        Vec::new()
    }

    fn calculate_spire_height(&self, _x: f32, _z: f32) -> f32 {
        50.0
    }

    fn should_place_pipe(&self, _x: f32, _z: f32) -> bool {
        false
    }
}
