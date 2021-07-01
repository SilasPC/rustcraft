
use crate::item::ItemLike;
use crate::vao::VAO;
use crate::TextureAtlas;
use std::rc::Rc;
use crate::item::Item;
use std::sync::Arc;
use crate::block::Block;

pub struct Registry {
    pub blocks: Vec<Arc<Block>>,
    pub items: Vec<Arc<Item>>,
    pub texture_atlas: Rc<TextureAtlas>,
    pub iso_block_vao: VAO,
    pub item_vao: VAO,
}

impl Registry {
    pub fn get(&self, idx: usize) -> ItemLike {
        if idx >= self.blocks.len() {
            self.items[idx-self.blocks.len()].clone().into()
        } else {
            self.blocks[idx].clone().into()
        }
    }
    pub fn item(&self, idx: usize) -> &Arc<Item> {
        &self.items[idx+self.blocks.len()]
    }
}

impl std::ops::Index<usize> for Registry {
    type Output = Arc<Block>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}
