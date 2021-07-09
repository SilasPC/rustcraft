
use crate::item::Item;
use std::rc::Rc;
use crate::vao::VAO;
use crate::TextureAtlas;
use std::sync::Arc;

pub type BehaviorFn = fn(&mut Block);

#[derive(Clone, Default)]
pub struct Behavior {
    pub on_use: Option<BehaviorFn>,
    pub on_hit: Option<BehaviorFn>,
    pub on_place: Option<BehaviorFn>,
    pub on_update: Option<BehaviorFn>,
    pub on_break: Option<BehaviorFn>,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Block(Arc<(BlockData,bool)>);

impl std::ops::Deref for Block {
    type Target = BlockData;
    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl AsRef<BlockData> for Block {
    fn as_ref(&self) -> &BlockData {
        &self.0.0
    }
}

impl Block {
    pub fn new_registered_as_shared(data: BlockData) -> Self {
        Self(Arc::new((data,true)))
    }
    pub fn new_not_shared(data: BlockData) -> Self {
        Self(Arc::new((data,false)))
    }
    pub fn mutate(&mut self) -> &mut BlockData {
        let mt = Arc::make_mut(&mut self.0);
        mt.1 = false;
        &mut mt.0
    }
    pub fn is_shared(&self) -> bool {self.0.1}
    pub fn ptr_eq(&self, rhs: &Self) -> bool {Arc::ptr_eq(&self.0, &rhs.0)}
    pub fn render_eq(&self, rhs: &Self) -> bool {self.0.0.render_eq(&rhs.0.0)}
    pub unsafe fn inc_arc_count(&self) {
        Arc::increment_strong_count(&self.0)
    }
    pub unsafe fn dec_arc_count(&self) {
        Arc::decrement_strong_count(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BlockData {
    pub id: usize,
    pub name: String,
    #[serde(default = "yes")]
    pub solid: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub replacable: bool,
    #[serde(default)]
    pub light: u8,
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

impl BlockData {
    pub fn render_eq(&self, rhs: &Self) -> bool {
        self.texture == rhs.texture
        && self.transparent == rhs.transparent
        && self.no_render == rhs.no_render
    }
}

const fn yes() -> bool {true}
