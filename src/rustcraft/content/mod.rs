pub mod inventory;
pub mod base;
pub mod builder;

use crate::loader::Loader;
use crate::crafting::CraftingRegistry;
use crate::prelude::*;
use component::{Physics, Position, PlayerData, View};

pub fn make_player() -> ((Position, Physics, View, PlayerData), util::AABB) {
    let pos = Position::new(Vector3 {x:50., y: 55., z: 50.}.as_coord(), (0.8,1.9,0.8).into());
    let aabb = pos.get_aabb();
    let view = View::from(Vector3 {
        x: 0.5,
        y: 1.8,
        z: 0.5,
    });
    let mut phys = Physics::new();
    phys.gravity = false;
    phys.freecam = false;
    ((pos, phys, view, PlayerData::new()), aabb)
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

pub struct Content {
    pub blocks: HashMap<String, BlockData>,
    pub items: ItemRegistry,
    pub crafting: CraftingRegistry,
    // pub entities: EntityRegistry,
    //pub components: ComponentRegistry,
    pub behaviors: BehaviorRegistry,
}

#[derive(Default)]
pub struct BehaviorRegistry {
    pub behaviors: HashMap<String, BehaviorFn>,
}
