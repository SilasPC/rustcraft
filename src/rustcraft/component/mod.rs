
mod physics;
mod position;
mod viewable;
mod ai;
mod falling_block;
mod item;
mod player;
mod model;
mod path_finding;

pub use path_finding::*;
pub use crate::prelude::*;
pub use model::*;
pub use falling_block::*;
pub use physics::*;
pub use position::*;
pub use viewable::*;
pub use ai::*;
pub use item::*;
pub use player::*;

#[derive(Default)]
pub struct ComponentRegistry {
    pub components: HashMap<String, Box<dyn ComponentBuilder>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {Self::default()}
    pub fn register<T: ComponentBuilder>(&mut self, name: String, cmp: T) {
        self.components.insert(name, (box cmp) as Box<dyn ComponentBuilder>);
    }
    pub fn get(&self, name: &str) -> Box<dyn ComponentBuilder> {
        self.components.get(name).unwrap().dyn_clone()
    }
    /* pub fn from() {
        let v = toml::Value::String("".to_owned());
        ?
    } */
}

pub trait ComponentBuilder: 'static {
    fn dyn_clone(&self) -> Box<dyn ComponentBuilder>;
    fn build_into(&self, b: &mut hecs::EntityBuilder);
    fn from_toml(&self, val: toml::Value, b: &mut hecs::EntityBuilder);
}

impl<'de, T: hecs::Component + Clone + serde::Deserialize<'de>> ComponentBuilder for T {
    fn dyn_clone(&self) -> Box<dyn ComponentBuilder> {
        box self.clone() as Box<dyn ComponentBuilder>
    }
    fn build_into(&self, b: &mut hecs::EntityBuilder) {
        b.add(self.clone());
    }
    fn from_toml(&self, val: toml::Value, b: &mut hecs::EntityBuilder) {
        b.add(val.try_into::<T>().unwrap());
    }
}