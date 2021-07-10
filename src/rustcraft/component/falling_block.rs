
use super::*;

pub struct FallingBlock {
    pub block: Block
}

impl FallingBlock {
    pub fn of(block: Block) -> Self { Self { block } }
    pub fn system_collide_land(data: &mut crate::Data) {
        let mut to_destroy = vec![];
        for (ent, (pos, phys, this)) in data.ecs.query_mut::<(&mut Position, &mut Physics, &FallingBlock)>() {
            if phys.is_grounded() {
                // spawn item if fail:
                data.world.set_block_at(&pos.pos, &this.block);
                data.ent_tree.remove(ent);
                to_destroy.push(ent);
            }
        }
        for ent in to_destroy {
            let _ = data.ecs.despawn(ent);
        }
    }
}