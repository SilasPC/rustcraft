
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

    pub fn system_tick_age_items(data: &mut crate::Data) {
        let mut despawn = vec![];
        for (ent, item) in data.ecs.query_mut::<&mut ItemCmp>() {
            item.age += 1;
            if item.age > 20 * LIVE_SECS {
                despawn.push(ent);
            }
        }
        for ent in despawn {
            let _ = data.ecs.despawn(ent);
        }
    }

}