
use crate::item::ItemData;
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
use crate::coords::*;

pub fn make_player() -> (impl hecs::DynamicBundle, AABB) {
    let pos = Position::from(Vector3 {x:50., y: 55., z: 50.}.as_coord());
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

pub fn make_registry(texture_atlas: Arc<TextureAtlas>) -> Arc<Registry> {
    let mut x: SerialItemRegistry = toml::from_str(&std::fs::read_to_string("assets/items.toml").unwrap()).unwrap();
    x.block.sort_by_key(|a| a.id);
    x.item.sort_by_key(|a| a.id);
    assert!(x.block.last().unwrap().id < x.item.first().unwrap().id);
    let blocks: Vec<_> = x.block.into_iter().map(Block::new_registered_as_shared).collect();
    let items: Vec<_> = x.item.into_iter().map(Item::new_registered_as_shared).collect();
    for block in &blocks {
        // by incrementing the count to at least 2, these can never be mutated via Arc::make_mut
        // remember to decrement on Register Drop
        //! unsafe { block.inc_arc_count() } CAUSES ILLEGAL INSTRUCTION???
    }
    let items_offset = blocks.len();
    Arc::new(Registry {
        item_vao: gen_item_vao(&items, &texture_atlas),
        iso_block_vao: gen_block_vao(&blocks, &texture_atlas),
        blocks,
        items,
        items_offset,
        texture_atlas,
    })
}

impl Drop for Registry {
    fn drop(&mut self) {
        for block in &self.blocks {
            //! unsafe { block.dec_arc_count() }
        }
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
        ItemStack::of(reg.get(9), 4)
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
    block: Vec<BlockData>,
    item: Vec<ItemData>,
}