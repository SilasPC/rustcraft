
use crate::TextureAtlas;
use crate::prelude::*;

pub struct ItemRegistry {
    pub items: HashMap<String, ItemLike>,
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> &ItemLike {
        self.items.get(id).unwrap()
    }
}

