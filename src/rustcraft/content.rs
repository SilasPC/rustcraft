
use crate::util::AABB;
use std::sync::Arc;
use crate::block::Block;
use cgmath::*;
use crate::rustcraft::component::*;

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

pub fn make_blocks() -> Vec<Arc<Block>> {
    vec![
        Block { id: 0, name: "Air", transparent: true, solid: false, no_render: true, texture: (0,0,0), has_gravity: false, drops: None, }, // air
        Block { id: 1, name: "Sand", transparent: false, solid: true, no_render: false, texture: (0,0,0), has_gravity: false, drops: Some(1), }, // stone
        Block { id: 2, name: "Dirt", transparent: false, solid: true, no_render: false, texture: (1,1,1), has_gravity: false, drops: Some(2), }, // dirt
        Block { id: 3, name: "Grass", transparent: false, solid: true, no_render: false, texture: (3,2,1), has_gravity: false, drops: Some(2), }, // grass
        Block { id: 4, name: "Wood", transparent: false, solid: true, no_render: false, texture: (5,4,5), has_gravity: false, drops: Some(4), }, // wood log
        Block { id: 5, name: "Sand", transparent: false, solid: true, no_render: false, texture: (6,6,6), has_gravity: true, drops: Some(5), }, // sand
        Block { id: 6, name: "Leaves", transparent: true, solid: true, no_render: false, texture: (7,7,7), has_gravity: false, drops: None, }, // leaves
    ].into_iter().map(std::sync::Arc::new).collect()
}