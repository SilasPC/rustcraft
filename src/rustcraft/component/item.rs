
use super::*;

const LIVE_SECS: usize = 10;

pub struct ItemCmp {
    pub item: ItemStack,
    pub age: usize,
}

impl From<ItemStack> for ItemCmp {
    fn from(item: ItemStack) -> Self {
        Self {
            item,
            age: 0
        }
    }
}

impl ItemCmp {

    pub fn system_tick_age_items(data: &mut crate::WorldData) {

        let mut despawn = vec![];
        let mut picked_up = vec![];
        let mut update = HashMap::new();
        if let Ok(pos) = data.entities.ecs.query_one_mut::<&Position>(data.entities.player).cloned() {
            let mut aabb = pos.get_aabb();
            aabb.extend_radius(2.);
            for ent in data.entities.tree.query(&aabb) {
                if let Ok((ipos, item)) = data.entities.ecs.query_one_mut::<(&Position,&ItemCmp)>(*ent) {
                    if item.age > 3 && ipos.pos.distance(pos.pos.0) < 1.5+0.4 {
                        picked_up.push((ent, Some(item.item.clone())));
                    }
                }
            }
        }
        if let Ok(pdata) = data.entities.ecs.query_one_mut::<&mut PlayerData>(data.entities.player) {
            for (ent, mut stack) in picked_up {
                pdata.inventory.merge(&mut stack);
                if stack.is_none() {
                    despawn.push(*ent);
                } else {
                    update.insert(ent, stack.unwrap());
                }
            }
        }
        for (ent, item) in data.entities.ecs.query_mut::<&mut ItemCmp>() {
            if let Some(stack) = update.remove(&ent) {
                item.item = stack;
            }
            item.age += 1;
            if item.age > 20 * LIVE_SECS {
                despawn.push(ent);
            }
        }
        for ent in despawn {
            let _ = data.entities.ecs.despawn(ent);
        }
    }

}