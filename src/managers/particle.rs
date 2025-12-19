use crate::components::{Block, Camera, Particle};
use crate::constants::{
    BLOCK_SIZE, BOUNCE_DAMPENING_X, FRICTION_ON_GROUND, GRAVITY, MAX_LIFESPAN_ON_GROUND_SEC,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use macroquad::prelude::*; // For get_time()

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

    pub fn update(&mut self, collidable_blocks: &[&Block], camera: &Camera) {
        // Keep collidable_blocks for now
        for particle in &mut self.active_particles {
            if !particle.alive {
                continue;
            }

            particle.vy += GRAVITY;
            particle.x += particle.vx;

            for block in collidable_blocks {
                if check_collision(particle, block) {
                    if particle.vx > 0.0 {
                        particle.x = block.x - 0.1;
                    } else {
                        particle.x = block.x + BLOCK_SIZE + 0.1;
                    }
                    particle.vx *= BOUNCE_DAMPENING_X;
                    break;
                }
            }

            particle.y += particle.vy;
            let mut is_on_ground = false;

            for block in collidable_blocks {
                if check_collision(particle, block) {
                    if particle.vy > 0.0 {
                        particle.y = block.y - 0.1;
                        particle.vy = 0.0;
                        particle.vx *= FRICTION_ON_GROUND;
                        is_on_ground = true;
                    } else if particle.vy < 0.0 {
                        particle.y = block.y + BLOCK_SIZE + 0.1;
                        particle.vy = 0.0;
                    }
                    break;
                }
            }

            if is_on_ground {
                let now = get_time();
                if particle.time_landed.is_none() {
                    particle.time_landed = Some(now);
                } else if now - particle.time_landed.unwrap() > MAX_LIFESPAN_ON_GROUND_SEC {
                    particle.alive = false;
                }
            } else {
                particle.time_landed = None;
            }

            let margin = BLOCK_SIZE * 10.0;
            if particle.x < camera.x - margin
                || particle.x > camera.x + SCREEN_WIDTH + margin
                || particle.y < camera.y - margin
                || particle.y > camera.y + SCREEN_HEIGHT + margin
            {
                particle.alive = false;
            }
        }
        self.active_particles.retain(|p| p.alive);
    }
}

fn check_collision(particle: &Particle, block: &Block) -> bool {
    particle.x >= block.x
        && particle.x < block.x + BLOCK_SIZE
        && particle.y >= block.y
        && particle.y < block.y + BLOCK_SIZE
}
