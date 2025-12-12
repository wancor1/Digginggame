use crate::components::{Block, Particle};

pub struct ParticleManager {
    pub active_particles: Vec<Particle>,
}

impl ParticleManager {
    pub fn new() -> Self {
        Self {
            active_particles: Vec::new(),
        }
    }

    pub fn add_particles(&mut self, new_particles: Vec<Particle>) {
        self.active_particles.extend(new_particles);
    }

    pub fn update(&mut self, collidable_blocks: &[&Block]) {
        for particle in &mut self.active_particles {
            particle.update(collidable_blocks);
        }
        self.active_particles.retain(|p| p.alive);
    }

    pub fn draw(&self) {
        for particle in &self.active_particles {
            particle.draw();
        }
    }
}
