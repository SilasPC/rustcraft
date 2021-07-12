
use crate::TextureAtlas;
use crate::prelude::*;

pub struct Registry {
    pub items: HashMap<String, ItemLike>,
    pub texture_atlas: Arc<TextureAtlas>,
}

impl Registry {
    pub fn get(&self, id: &str) -> &ItemLike {
        self.items.get(id).unwrap()
    }
}

