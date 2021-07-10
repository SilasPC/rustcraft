
use crate::TextureAtlas;
use crate::prelude::*;

pub struct Registry {
    pub blocks: Vec<Block>,
    pub items: Vec<Item>,
    pub items_offset: usize,
    pub texture_atlas: Arc<TextureAtlas>,
    pub iso_block_vao: VAO,
    pub item_vao: VAO,
}

impl Registry {
    pub fn get(&self, idx: usize) -> ItemLike {
        if idx >= self.items_offset {
            self.items[idx-self.items_offset].clone().into()
        } else {
            self.blocks[idx].clone().into()
        }
    }
    pub fn item(&self, idx: usize) -> &Item {
        &self.items[idx+self.items_offset]
    }
}

impl std::ops::Index<usize> for Registry {
    type Output = Block;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}
