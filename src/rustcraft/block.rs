
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

impl std::fmt::Debug for Behavior {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Behavior").finish()
    }
}

pub struct BlockRegistry {
    pub blocks: Vec<Arc<Block>>,
    pub texture_atlas: Rc<TextureAtlas>,
    pub iso_block_vao: VAO,
}

impl std::ops::Index<usize> for BlockRegistry {
    type Output = Arc<Block>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

#[derive(Clone, Debug)]
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

impl Eq for Block {}
impl PartialEq for Block {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}
