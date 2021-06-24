
use cgmath::*;
use std::rc::Rc;
use super::render::*;
use crate::texture::Texture;

pub struct ContainerGUI(pub Vec<FlatGUI>);

impl GUI for ContainerGUI {
    fn render(&mut self, r: &mut Renderer) {
        for gui in &mut self.0 {
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
    fn render(&mut self, r: &mut Renderer) {
        self.texture.bind();
        // TODO: broken?
        if !r.is_mouse_over(self.texture.size(), self.anchor, self.scale) {
            r.render(self.pos, self.texture.size(), self.anchor, self.scale);
        }
    }
}
