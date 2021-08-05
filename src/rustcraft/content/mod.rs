pub mod inventory;
pub mod base;

use crate::util::fdiv;
use crate::loader::Loader;
use crate::crafting::CraftingRegistry;
use crate::rustcraft::component::{Physics,Position,PlayerData,View};
use crate::prelude::*;

pub fn make_player() -> (impl hecs::DynamicBundle, util::AABB) {
    let pos = Position::new(Vector3 {x:50., y: 55., z: 50.}.as_coord(), (0.8,1.9,0.8).into());
    let aabb = pos.get_aabb();
    let view = View::from(Vector3 {
        x: 0.5,
        y: 1.8,
        z: 0.5,
    });
    ((pos, Physics::new(), view, PlayerData::new()), aabb)
}

#[derive(serde::Deserialize)]
struct SerialItemRegistry {
    block: Vec<BlockData>,
    item: Vec<ItemData>,
}

impl SerialItemRegistry {
    pub fn from_path(path: &str) -> Self {
        toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }
    pub fn into_map(self) -> HashMap<String, ItemLike> {
        
        /* x.block.sort_by_cached_key(|a| a.id.clone());
        x.item.sort_by_cached_key(|a| a.id.clone()); */

        /* x.block[3].behavior = Some(Box::new(Behavior {
            on_rnd_tick: Some(grass_update),
            .. Default::default()
        })); */
        /* x.block[8].behavior = Some(Box::new(Behavior {
            on_update: Some(fire_update),
            .. Default::default()
        })); */
        
        self.block.into_iter()
            .map(Block::new_registered_as_shared)
            .map(ItemLike::from)
            .map(|il| (il.id().to_owned(), il))
            .chain(
                self.item.into_iter()
                    .map(Item::new_registered_as_shared)
                    .map(ItemLike::from)
                    .map(|il| (il.id().to_owned(), il))
            ).collect()
    }
}

#[derive(serde::Deserialize)]
struct SavedRecipies {
    shaped: Vec<SavedRecipe>,
}

#[derive(serde::Deserialize)]
struct SavedRecipe {
    input: Vec<String>,
    output: String,
    #[serde(default = "one")]
    count: usize
}

fn chest_use(pos: BlockPos, data: &mut WorldData) {
    // TODO somehow trigger inventory to open
}

const fn one() -> usize {1}

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

pub struct Content {
    pub items: ItemRegistry,
    pub crafting: CraftingRegistry,
    pub entities: EntityRegistry,
    pub components: ComponentRegistry,
    pub behaviors: BehaviorRegistry,
}

#[derive(Default)]
pub struct BehaviorRegistry {
    pub behaviors: HashMap<String, BehaviorFn>,
}

pub trait ContentMod {
    fn name(&mut self) -> &str;
    fn register_components(&mut self, cnt: &mut ContentBuilder) {}
    fn register_entities(&mut self, cnt: &mut ContentBuilder) {}
    fn register_behaviors(&mut self, cnt: &mut ContentBuilder) {}
    fn register_items(&mut self, cnt: &mut ContentBuilder) {}
    fn register_recipies(&mut self, cnt: &mut ContentBuilder) {}
}