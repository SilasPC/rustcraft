
use crate::prelude::*;

#[derive(Default)]
pub struct EntityRegistry {
    pub entities: HashMap<String, EntityTemplate>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct EntityTemplate {
    pub cmps: Vec<Box<dyn ComponentBuilder>>,
}

impl EntityTemplate {
    pub fn build_all_into(&self, b: &mut hecs::EntityBuilder) {
        for cmp in &self.cmps {
            cmp.build_into(b)
        }
    }
}

/* fn summon(pos: Position, t: &EntityTemplate, ecs: &mut hecs::World) {
    let mut b = hecs::EntityBuilder::new();
    for c in &t.cmps {
        b.add(c);
    }
    let aabb = pos.get_aabb();
    b.add(pos);
    let ent = ecs.spawn(b.build());
    // tree.insert(ent, ent, aabb);
    
} */