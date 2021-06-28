
use cgmath::*;
use std::rc::Rc;
use super::render::*;
use crate::texture::Texture;

pub struct ContainerGUI(pub Vec<FlatGUI>);

pub struct FlatGUI {
    pub texture: Rc<Texture>,
    pub pos: Vector2<f32>,
}
