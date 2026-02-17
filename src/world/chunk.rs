use macroquad::prelude::*;
use std::collections::HashMap;
use super::Spire;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub struct Chunk {
    pub position: ChunkPos,
    pub spires: Vec<Spire>,
    pub is_loaded: bool,
}

pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
    load_radius: u32,
    seed: u64,
}

impl ChunkManager {
    pub fn new(seed: u64, load_radius: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            load_radius,
            seed,
        }
    }

    pub fn update_loaded_chunks(&mut self, _player_pos: Vec3) {
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn generate_chunk(&mut self, _pos: ChunkPos) {
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
    }
}
