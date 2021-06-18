
use cgmath::*;
use std::rc::Rc;
use super::render::*;
use crate::texture::Texture;

pub struct ContainerGUI(pub Vec<FlatGUI>);

impl GUI for ContainerGUI {
    fn render(&self, r: &mut Renderer) {
        for gui in &self.0 {
            gui.render(r);
        }
    }
}

pub struct FlatGUI {
    pub texture: Rc<Texture>,
    pub anchor: Anchor,
    pub scale: Scale,
    pub pos: Vector2<f32>,
}

impl GUI for FlatGUI {
    fn render(&self, r: &mut Renderer) {
        self.texture.bind();
        r.render(self.pos, self.texture.aspect_ratio(), self.anchor, self.scale);
    }
}