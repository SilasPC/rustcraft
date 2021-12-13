
use crate::util::fdiv;
use crate::loader::Loader;
use crate::crafting::CraftingRegistry;
use crate::rustcraft::component::{Physics,Position,PlayerData,View};
use crate::prelude::*;

pub trait ContentMod {
    fn name(&mut self) -> &str;
    fn register_components(&mut self, cnt: &mut ContentBuilder) {}
    fn register_entities(&mut self, cnt: &mut ContentBuilder) {}
    fn register_behaviors(&mut self, cnt: &mut ContentBuilder) {}
    fn register_items(&mut self, cnt: &mut ContentBuilder) {}
    fn register_recipies(&mut self, cnt: &mut ContentBuilder) {}
}

pub struct ContentBuilder {
    pub items: HashMap<String, ItemLike>,
    pub crafting: CraftingRegistry,
    pub entities: EntityRegistry,
    pub components: ComponentRegistry,
    pub behaviors: BehaviorRegistry,
}

impl ContentBuilder {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            crafting: CraftingRegistry::new(),
            entities: EntityRegistry::new(),
            components: ComponentRegistry::new(),
            behaviors: BehaviorRegistry::default(),
        }
    }
    pub fn load_mod(&mut self, cmod: &mut dyn ContentMod) {
        cmod.register_components(self);
        cmod.register_entities(self);
        cmod.register_behaviors(self);
        cmod.register_items(self);
        cmod.register_recipies(self);
    }
    pub fn finish(self, texture_atlas: Arc<TextureAtlas>) -> Content {
        Content {
            blocks: self.items
                .iter()
                .filter_map(|(k, v)| v.as_block().map(|b| (k, b)))
                .map(|(k, v)| (k.clone(), v.0.0.clone()))
                .collect(),
            items: ItemRegistry {
                texture_atlas,
                items: self.items,
            },
            crafting: self.crafting,
            entities: self.entities,
            components: self.components,
            behaviors: self.behaviors,
        }
    }
}