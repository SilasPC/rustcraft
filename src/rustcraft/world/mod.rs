mod gen;
mod data;
mod raycast;
mod generation;
mod voxel_data;
pub mod updates;

use crate::world::updates::Updates;
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

pub struct ChunkData<'cnt> {
    pub chunk: Box<Chunk<'cnt>>,
    /// Self is not treated as a neighbour, so count should be in 0..=26
    pub loaded_neighbours: usize,
}

impl<'cnt> ChunkData<'cnt> {
    pub fn all_neighbours_loaded(&self) -> bool {
        self.loaded_neighbours == 26
    }
}

pub struct VoxelData<'cnt> {
    pub chunks: HashMap<ChunkPos, ChunkData<'cnt>>,
    pub changed_chunks: HashSet<ChunkPos>,
}

pub struct WorldData<'cnt> {
    pub force_loaded: HashSet<ChunkPos>,
    pub block_updates: Updates,
    pub entities: EntityData,
    pub blocks: VoxelData<'cnt>,
    pub seed: String,
    pub air: &'cnt BlockData,
    pub noise: Box<dyn TerrainGenerator>,
    pub ticks: u64,
    pub to_load: VecDeque<Loading>,
}

impl<'cnt> WorldData<'cnt> {
    
    pub fn new(seed: &str, air: &'cnt BlockData) -> Self {
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
        let block_updates = Updates::default();
        let force_loaded = HashSet::default();
        WorldData { force_loaded, block_updates, entities, to_load: VecDeque::new(), seed: seed.to_owned(), blocks, noise, air, ticks: 0 }
    }

}