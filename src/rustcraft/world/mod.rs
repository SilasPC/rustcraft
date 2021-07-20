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

pub struct VoxelData {
    pub chunks: HashMap<ChunkPos, Box<Chunk>>,
}

pub struct WorldData {
    pub entities: EntityData,
    pub blocks: VoxelData,
    pub seed: String,
    pub air: Block,
    pub noise: TerrainGen,
    pub ticks: u64,
    pub to_load: VecDeque<Loading>,
    pub changed_chunks: HashSet<ChunkPos>,
    pub to_update: Vec<BlockPos>,
}

impl WorldData {
    
    pub fn new(seed: &str, air: Block) -> Self {
        let noise = crate::perlin::PerlinNoise::new(seed.to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed.to_owned(), 1, 1.);
        let palettes = [
            ["stone","dirt","grass"],
            ["stone","sand","sand"]
        ];
        let noise = TerrainGen {
            noise,
            noise_basic,
            palettes
        };
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
            chunks: HashMap::new()
        };
        WorldData { entities, to_update: vec![], changed_chunks: HashSet::new(), to_load: VecDeque::new(), seed: seed.to_owned(), blocks, noise, air, ticks: 0 }
    }

}