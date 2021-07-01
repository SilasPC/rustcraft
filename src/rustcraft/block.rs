
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

impl Eq for Behavior {}
impl PartialEq for Behavior {
    fn eq(&self, rhs: &Self) -> bool {true}
}

impl std::fmt::Debug for Behavior {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Behavior").finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub id: usize,
    pub name: &'static str,
    pub solid: bool,
    pub transparent: bool,
    pub no_render: bool,
    pub texture: (usize,usize,usize),
    pub has_gravity: bool,
    pub drops: Option<usize>,
    pub behavior: Option<Box<Behavior>>,
}

