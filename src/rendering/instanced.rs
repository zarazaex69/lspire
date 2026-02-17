use macroquad::prelude::*;

pub struct InstancedRenderer {
    max_instances: usize,
}

impl InstancedRenderer {
    pub fn new(max_instances: usize) -> Self {
        Self { max_instances }
    }

    pub fn render(&self) {
    }

    pub fn clear(&mut self) {
    }
}
