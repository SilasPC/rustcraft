
pub trait BlockBehaivour {
    fn did_place() {}
    fn block_update() {}
    fn did_break() {}
}

pub struct Block {
    pub id: usize,
    pub solid: bool,
    pub transparent: bool,
    pub no_render: bool,
    pub texture: (usize,usize,usize),
    pub has_gravity: bool,
}
