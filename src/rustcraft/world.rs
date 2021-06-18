
use crate::block::Block;
use crate::chunk::*;
use cgmath::*;

pub struct WorldData {
    pub chunks: Vec<Vec<Chunk>>,
}

impl WorldData {
    pub fn new(block_map: &Vec<Block>, atlas: &crate::texture::TextureAtlas) -> Self {
        let mut chunks = vec![
            vec![],
            vec![],
            vec![],
        ];
        for x in 0..3isize {
            for z in 0..3isize {
                chunks[x as usize].push(
                    Chunk::new(Vector3 { x, y: 0, z }, block_map, atlas)
                );
            }
        }
        WorldData { chunks }
    }
}