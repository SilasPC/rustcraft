
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
}