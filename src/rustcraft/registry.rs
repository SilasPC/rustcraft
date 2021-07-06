
use crate::item::ItemLike;
use crate::vao::VAO;
use crate::TextureAtlas;
use std::rc::Rc;
use crate::item::Item;
use std::sync::Arc;
use crate::block::Block;

pub struct Registry {
    pub blocks: Vec<Block>,
    pub items: Vec<Arc<Item>>,
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
    pub fn item(&self, idx: usize) -> &Arc<Item> {
        &self.items[idx+self.items_offset]
    }
}

impl std::ops::Index<usize> for Registry {
    type Output = Block;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}
