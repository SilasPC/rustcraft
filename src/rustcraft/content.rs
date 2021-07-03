
use std::collections::HashMap;
use crate::item::ItemStack;
use crate::crafting::CraftingRegistry;
use crate::registry::Registry;
use crate::util::*;
use crate::block::*;
use super::item::Item;
use std::rc::Rc;
use crate::TextureAtlas;
use std::sync::Arc;
use cgmath::*;
use crate::rustcraft::component::{Physics,Position,PlayerData,View};

pub fn make_player() -> (impl hecs::DynamicBundle, AABB) {
    let pos = Position::from(Vector3 {x:50., y: 55., z: 50.});
    let mut phys = Physics::new(Vector3 {
        x: 0.8,
        y: 1.9,
        z: 0.8,
    });
    //cam_phys.set_flying(true);
    let aabb = phys.get_aabb(&pos);
    let view = View::from(Vector3 {
        x: 0.5,
        y: 1.8,
        z: 0.5,
    });
    ((pos, phys, view, PlayerData::new()), aabb)
}

pub fn make_registry(texture_atlas: Rc<TextureAtlas>) -> Registry {
    let mut x: SerialItemRegistry = toml::from_str(&std::fs::read_to_string("assets/items.toml").unwrap()).unwrap();
    x.block.sort_by_key(|a| a.id);
    x.item.sort_by_key(|a| a.id);
    assert!(x.block.last().unwrap().id < x.item.first().unwrap().id);
    let blocks: Vec<_> = x.block.into_iter().map(std::sync::Arc::new).collect();
    let items = x.item.into_iter().map(std::sync::Arc::new).collect();
    let items_offset = blocks.len();
    Registry {
        item_vao: gen_item_vao(&items, &texture_atlas),
        iso_block_vao: gen_block_vao(&blocks, &texture_atlas),
        blocks,
        items,
        items_offset,
        texture_atlas,
    }
}

pub fn make_crafting_registry(reg: &Registry) -> CraftingRegistry {
    let mut cr = CraftingRegistry::new();
    cr.register(
        true, &[ // planks => sticks
            reg.get(7).into(), None, None,
            None, None, None,
            None, None, None,
        ],
        ItemStack::of(reg.get(8), 4)
    );
    cr.register(
        true, &[ // logs => planks
            reg.get(4).into(), None, None,
            None, None, None,
            None, None, None,
        ],
        ItemStack::of(reg.get(7), 4)
    );
    cr
}

#[derive(serde::Deserialize)]
struct SerialItemRegistry {
    block: Vec<Block>,
    item: Vec<Item>,
}