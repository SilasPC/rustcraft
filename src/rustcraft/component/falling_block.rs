
use crate::block::Block;
use std::sync::Arc;
use crate::game_loop::set_block;
use super::*;

pub struct FallingBlock {
    pub block: Arc<Block>
}

impl FallingBlock {
    pub fn of(block: Arc<Block>) -> Self { Self { block } }
    pub fn system_collide_land(data: &mut crate::Data) {
        let mut to_destroy = vec![];
        for (ent, (pos, phys, this)) in data.ecs.query_mut::<(&mut Position, &mut Physics, &FallingBlock)>() {
            if phys.is_grounded() {
                // spawn item if fail:
                set_block(&mut data.world, &data.ent_tree, &pos.pos, &this.block, true);
                data.ent_tree.remove(ent);
                to_destroy.push(ent);
            }
        }
        for ent in to_destroy {
            let _ = data.ecs.despawn(ent);
        }
    }
}