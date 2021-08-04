mod gen;
mod data;
mod raycast;
mod generation;
mod voxel_data;

pub use generation::*;
pub use data::*;
pub use raycast::*;

use crate::prelude::*;

#[derive(Debug)]
pub enum Loading {
    Filling(i32, ChunkPos),
    Detailing(i32, ChunkPos),
    Meshing(i32, ChunkPos),
}

pub struct EntityData {
    pub ecs: hecs::World,
    pub tree: BVH<hecs::Entity, hecs::Entity>,
    pub player: hecs::Entity,
}

pub struct ChunkData {
    pub chunk: Box<Chunk>,
    /// Self is treated as a neighbour, so count should be in 0..=27
    pub loaded_neighbours: usize,
}

impl ChunkData {
    pub fn all_neighbours_loaded(&self) -> bool {
        self.loaded_neighbours == 27
    }
}

pub struct VoxelData {
    pub chunks: HashMap<ChunkPos, ChunkData>,
    pub changed_chunks: HashSet<ChunkPos>,
}

pub struct WorldData {
    pub entities: EntityData,
    pub blocks: VoxelData,
    pub seed: String,
    pub air: Block,
    pub noise: Box<dyn TerrainGenerator>,
    pub ticks: u64,
    pub to_load: VecDeque<Loading>,
    pub to_update: Vec<BlockPos>,
}

impl WorldData {
    
    pub fn new(seed: &str, air: Block) -> Self {
        let noise = IslandGenerator::new_dyn(seed);
        let mut ecs = hecs::World::new();
        let mut tree = BVH::new();
        let player = {
            let (cam, aabb) = make_player();
            let cam = ecs.spawn(cam);
            tree.insert(cam, cam, &aabb);
            cam
        };
        let entities = EntityData {
            ecs,
            tree,
            player
        };
        let blocks = VoxelData {
            chunks: HashMap::new(),
            changed_chunks: HashSet::new(),
        };
        WorldData { entities, to_update: vec![], to_load: VecDeque::new(), seed: seed.to_owned(), blocks, noise, air, ticks: 0 }
    }

}