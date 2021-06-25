
pub trait BlockBehaivour {
    fn did_place() {}
    fn block_update() {}
    fn did_break() {}
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub solid: bool,
    pub transparent: bool,
    pub no_render: bool,
    pub texture: (usize,usize,usize),
    pub has_gravity: bool,
    pub drops: Option<usize>,
}

impl Eq for Block {}
impl PartialEq for Block {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}
