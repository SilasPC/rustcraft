
use crate::content::inventory::PlayerGUI;
use crate::prelude::*;
use inventory::*;

pub struct InventoryRenderer {
    pub atlas: Arc<TextureAtlas>,
    pub gui: GUIRenderer,
    pub iren: ItemGUIRenderer,
    pub highlight: Arc<Texture>,
}

impl InventoryRenderer {

    pub fn render(&mut self, pgui: &PlayerGUI, d: &impl InventoryData, m: PixelPos, picked_item: &Option<ItemStack>, show_inventory: bool) {
        self.gui.start();
        unsafe {
            gl::Enable(gl::BLEND);
        }
        self.render_bottom(pgui, d, m);
        if show_inventory {
            self.render_centered(&pgui.inventory, d, m);
            if let Some(item) = picked_item.as_ref().map(|s| &s.item) {
                self.render_floating_item(item, m);
            }
        }
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    pub fn corner_cursor(&mut self, i: &impl InventoryShell, m: PixelPos) -> Option<usize> {
        let (w,h) = (900, 700);
        let (hw, hh) = (450, 350);

        let t = i.texture();
        let (iw, ih) = t.size();
        let (iw, ih) = (iw as i32, ih as i32);

        t.bind();
        self.gui.set_pixels(hw, hh);
        self.gui.move_pixels(-iw / 2, -ih / 2);
        self.gui.set_uniforms(iw, ih);
        self.gui.square.draw();

        let (cx, cy) = self.gui.cursor.pos.into();
        let mp = (
            (m.0.0 - cx) / self.gui.pixel_scale,
            (m.0.1 - cy) / self.gui.pixel_scale,
        );

        i.slot_at(mp.into())
    }

    pub fn render_floating_item(&mut self, item: &ItemLike, m: PixelPos) {
        self.gui.start();
        self.gui.square.bind();
        self.atlas.bind();
        self.gui.set_pixels(m.0.0, m.0.1);
        self.gui.move_pixels(-8, -8);
        self.gui.set_uniforms(16, 16);
        self.iren.draw(item);
    }

    pub fn render_bottom(&mut self, pgui: &PlayerGUI, d: &impl InventoryData, m: PixelPos) {
        self.gui.square.bind();

        let (w,h) = (900, 700);
        let (hw, hh) = (450, 350);

        let (iw, ih) = pgui.hotbar.texture.size();
        let (iw, ih) = (iw as i32, ih as i32);

        pgui.hotbar.texture.bind();
        self.gui.set_pixels(hw, 0);
        self.gui.move_pixels(-iw / 2, 0);
        self.gui.set_uniforms(iw, ih);
        self.gui.square.draw();

        let (cx, cy) = self.gui.cursor.pos.into();
        
        let mp = (
            (m.0.0 - cx) / self.gui.pixel_scale,
            (m.0.1 - cy) / self.gui.pixel_scale,
        );

        self.render_priv(&pgui.hotbar, d, None, (cx, cy));

        pgui.selector.bind();
        self.gui.set_pixels(hw, 0);
        self.gui.move_pixels(-iw / 2, 0);
        self.gui.move_pixels(2 + pgui.selected_slot * 20, 2);
        self.gui.set_uniforms(20, 20);
        self.gui.square.bind();
        self.gui.square.draw();

    }

    pub fn render_centered(&mut self, i: &impl InventoryShell, d: &impl InventoryData, m: PixelPos) {
        self.gui.square.bind();

        let (w,h) = (900, 700);
        let (hw, hh) = (450, 350);

        let t = i.texture();
        let (iw, ih) = t.size();
        let (iw, ih) = (iw as i32, ih as i32);

        t.bind();
        self.gui.set_pixels(hw, hh);
        self.gui.move_pixels(-iw / 2, -ih / 2);
        self.gui.set_uniforms(iw, ih);
        self.gui.square.draw();

        let (cx, cy) = self.gui.cursor.pos.into();
        
        let mp = (
            (m.0.0 - cx) / self.gui.pixel_scale,
            (m.0.1 - cy) / self.gui.pixel_scale,
        );

        self.render_priv(i, d, mp.into(), (cx, cy));

    }

    pub fn render_priv(&mut self, i: &impl InventoryShell, d: &impl InventoryData, mp: Option<(i32, i32)>, c: (i32, i32)) {

        let (cx, cy) = self.gui.cursor.pos.into();

        let slots = i.slots();

        // slot highlighting
        if let Some(mp) = mp {
            if let Some(slot) = i.slot_at(mp.into()) {
                let p = slots[slot].0;
                self.highlight.bind();
                self.gui.cursor.pos = (
                    cx + p.0 * self.gui.pixel_scale,
                    cy + p.1 * self.gui.pixel_scale,
                ).into();
                self.gui.set_uniforms(16, 16);
                self.gui.square.draw();
            }
        }
        
        // items
        self.atlas.bind();
        for (i, p) in slots.iter().enumerate() {
            let p = p.0;
            if let Some(item) = d.slot(i) {
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