
use super::super::item::ItemStack;
use super::*;

pub struct Item {
    pub item: ItemStack,
    pub age: usize,
}

impl Item {

    pub fn system_tick_age_items(data: &mut crate::Data) {
        let mut despawn = vec![];
        for (ent, item) in data.ecs.query_mut::<&mut Item>() {
            item.age += 1;
            if item.age > 20 * 10 { // seconds
                despawn.push(ent);
            }
        }
        for ent in despawn {
            let _ = data.ecs.despawn(ent);
        }
    }

}