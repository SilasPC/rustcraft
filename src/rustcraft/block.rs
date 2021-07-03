
use crate::item::Item;
use std::rc::Rc;
use crate::vao::VAO;
use crate::TextureAtlas;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Behavior {
    pub on_use: Option<fn(&mut Arc<Block>)>,
    pub on_hit: Option<fn(&mut Arc<Block>)>,
    pub on_place: Option<fn(&mut Arc<Block>)>,
    pub on_update: Option<fn(&mut Arc<Block>)>,
    pub on_break: Option<fn(&mut Arc<Block>)>,
}

impl std::hash::Hash for Behavior {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {}
}

impl Eq for Behavior {}
impl PartialEq for Behavior {
    fn eq(&self, rhs: &Self) -> bool {true}
}

impl std::fmt::Debug for Behavior {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Behavior").finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Deserialize)]
pub struct Block {
    pub id: usize,
    pub name: String,
    #[serde(default = "yes")]
    pub solid: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub no_render: bool,
    pub texture: (usize,usize,usize),
    #[serde(default)]
    pub has_gravity: bool,
    #[serde(default)]
    pub drops: Option<usize>,
    #[serde(skip)]
    pub behavior: Option<Box<Behavior>>,
}

const fn yes() -> bool {true}
