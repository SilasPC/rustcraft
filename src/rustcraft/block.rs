
use crate::TextureAtlas;
use std::sync::Arc;

pub struct BlockRegistry {
    pub blocks: Vec<Arc<Block>>,
    pub texture_atlas: TextureAtlas,
}

impl BlockRegistry {
    pub fn new(blocks: Vec<Arc<Block>>, texture_atlas: TextureAtlas) -> Self {
        Self {
            blocks,
            texture_atlas,
        }
    }
}

impl std::ops::Index<usize> for BlockRegistry {
    type Output = Arc<Block>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

pub trait BlockBehaivour {
    fn did_place() {}
    fn block_update() {}
    fn did_break() {}
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub name: &'static str,
    pub solid: bool,
    pub transparent: bool,
    pub no_render: bool,
    pub texture: (usize,usize,usize),
    pub has_gravity: bool,
    pub drops: Option<usize>,
}

impl Eq for Block {}
impl PartialEq for Block {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}
