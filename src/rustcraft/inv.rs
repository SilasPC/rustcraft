
use crate::pgui::GUI;
use crate::util::fdiv;
use crate::gui::render::Cursor;
use crate::player::inventory::PlayerInventory;
use crate::gui::render::GUIRenderer;
use crate::prelude::*;

pub struct InventoryRenderer {
    pub atlas: Arc<TextureAtlas>,
    pub gui: GUIRenderer,
    pub iren: ItemGUIRenderer,
    pub highlight: Arc<Texture>,
}

impl InventoryRenderer {

    pub fn render(&mut self, pgui: &GUI, slots: &[Option<ItemStack>], m: PixelPos, picked_item: &Option<ItemStack>) {
        self.gui.start();
        unsafe {
            gl::Enable(gl::BLEND);
        }
        self.render_centered(&pgui.inventory, slots, m);
        if let Some(item) = picked_item.as_ref().map(|s| &s.item) {
            self.render_floating_item(item, m);
        }
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    fn render_floating_item(&mut self, item: &ItemLike, m: PixelPos) {
        self.gui.start();
        self.gui.square.bind();
        self.atlas.bind();
        self.gui.set_pixels(m.0.0, m.0.1);
        self.gui.set_uniforms(16, 16);
        self.iren.draw(item);
    }

    fn render_centered(&mut self, i: &InventoryGUI, slots: &[Option<ItemStack>], m: PixelPos) {
        self.gui.square.bind();

        let (w,h) = (900, 700);
        let (hw, hh) = (450, 350);

        let (iw, ih) = i.texture.size();
        let (iw, ih) = (iw as i32, ih as i32);

        i.texture.bind();
        self.gui.set_pixels(hw, hh);
        self.gui.move_pixels(-iw / 2, -ih / 2);
        self.gui.set_uniforms(iw, ih);
        self.gui.square.draw();

        let (cx, cy) = self.gui.cursor.pos.into();
        
        let mp = (
            (m.0.0 - cx) / self.gui.pixel_scale,
            (m.0.1 - cy) / self.gui.pixel_scale,
        );

        // slot highlighting
        if let Some(slot) = (i.slot_at)(mp.into()) {
            let p = i.slots[slot].0;
            self.highlight.bind();
            self.gui.cursor.pos = (
                cx + p.0 * self.gui.pixel_scale,
                cy + p.1 * self.gui.pixel_scale,
            ).into();
            self.gui.set_uniforms(16, 16);
            self.gui.square.draw();
        }
        
        // items
        self.atlas.bind();
        for (i, p) in i.slots.iter().enumerate() {
            let p = p.0;
            if let Some(item) = &slots[i] {
                self.gui.cursor.pos = (
                    cx + p.0 * self.gui.pixel_scale,
                    cy + p.1 * self.gui.pixel_scale,
                ).into();
                self.gui.set_uniforms(16, 16);
                self.iren.draw(&item.item);
            }
        }

    }

}

#[derive(Clone)]
pub struct InventoryGUI {
    pub texture: Arc<Texture>,
    pub slots: Vec<PixelPos>,
    pub slot_at: fn(p: PixelPos) -> Option<usize>,
}
