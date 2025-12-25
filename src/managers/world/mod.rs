use crate::components::MacroGrid;
use noise::{Perlin, Seedable};
use std::collections::{HashMap, HashSet};

pub mod access;
pub mod generation;
pub mod liquids;
pub mod modifications;
pub mod update;
pub mod view;

pub struct WorldManager {
    pub macrogrids: HashMap<(i32, i32), MacroGrid>,
    pub generated_chunk_coords: HashSet<(i32, i32)>,
    pub visited_chunks: HashSet<(i32, i32)>,
    pub pending_modifications: HashMap<(i32, i32), crate::managers::persistence::ChunkSaveData>,
    pub active_liquids: HashSet<(i32, i32)>,
    pub liquid_tick_counter: u64,
    pub world_seed_main: u32,
    pub world_seed_ore: u32,
    pub(crate) noise_main: Perlin,
    pub(crate) noise_ore: Perlin,
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldManager {
    pub fn new() -> Self {
        let mut rng = ::rand::rng();
        let seed_main = ::rand::Rng::random::<u32>(&mut rng);
        let seed_ore = ::rand::Rng::random::<u32>(&mut rng);

        let noise_main = Perlin::new(seed_main).set_seed(seed_main);
        let noise_ore = Perlin::new(seed_ore).set_seed(seed_ore);

        Self {
            macrogrids: HashMap::new(),
            generated_chunk_coords: HashSet::new(),
            visited_chunks: HashSet::new(),
            pending_modifications: HashMap::new(),
            active_liquids: HashSet::new(),
            liquid_tick_counter: 0,
            world_seed_main: seed_main,
            world_seed_ore: seed_ore,
            noise_main,
            noise_ore,
        }
    }

    pub fn seed(&mut self, main: u32, ore: u32) {
        self.world_seed_main = main;
        self.world_seed_ore = ore;
        self.noise_main = Perlin::new(main).set_seed(main);
        self.noise_ore = Perlin::new(ore).set_seed(ore);
        self.macrogrids.clear();
        self.generated_chunk_coords.clear();
        self.visited_chunks.clear();
        self.pending_modifications.clear();
        self.active_liquids.clear();
        self.liquid_tick_counter = 0;
    }

    pub fn reset(&mut self) {
        self.macrogrids.clear();
        self.generated_chunk_coords.clear();
        self.visited_chunks.clear();
        self.pending_modifications.clear();
        self.active_liquids.clear();
        self.liquid_tick_counter = 0;
    }
}
