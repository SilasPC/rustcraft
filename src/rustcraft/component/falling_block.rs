
use super::*;

pub struct FallingBlock {
    pub block: Block
}

impl FallingBlock {

    pub fn of(block: Block) -> Self { Self { block } }

    pub fn system_collide_land(data: &mut WorldData) {
        let mut to_destroy = vec![];
        let mut to_spawn = vec![];
        for (ent, (pos, phys, this)) in data.entities.ecs.query_mut::<(&mut Position, &mut Physics, &FallingBlock)>() {
            if phys.is_grounded() {
                data.entities.tree.remove(ent);
                to_destroy.push(ent);
                if !data.blocks.replace_at(&pos.pos, &this.block) {
                    let pos = Position::new(pos.pos.align_center(), (0.3,0.3,0.3).into());
                    let aabb = pos.get_aabb();
                    to_spawn.push(((
                        pos,
                        Physics::new(),
                        ItemCmp::from(ItemStack::of(this.block.clone(), 1))
                    ), aabb));
                }
            }
        }
        for (cmps, aabb) in to_spawn {
            let ent = data.entities.ecs.spawn(cmps);
            data.entities.tree.insert(ent, ent, &aabb);
        }
        for ent in to_destroy {
            let _ = data.entities.ecs.despawn(ent);
        }
    }

    /// Behavior function causing the block to fall if suspended in mid air
    pub fn behaviour_on_update(pos: BlockPos, data: &mut WorldData) {
        let block = data.blocks.block_at(&pos).unwrap();
        let below = pos.shifted(Face::YNeg);
        if let Some(below) = data.blocks.block_at(&below) {
            let below = below.as_ref();
            if !below.solid {
                let block = block.clone();
                data.blocks.set_block_at(&pos, &data.air);
                let pos_comp = Position::new(pos.as_world(), (1.,1.,1.).into());
                let phys = Physics::new();
                let aabb = pos_comp.get_aabb();
                let falling_block = data.entities.ecs.spawn((
                    pos_comp, phys, FallingBlock::of(block)
                ));
                data.entities.tree.insert(falling_block, falling_block, &aabb);
                data.block_updates.add_area(pos);
            }
        }
    }

}