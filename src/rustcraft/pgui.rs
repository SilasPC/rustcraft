
use std::rc::Rc;
use crate::item::ItemStack;
use std::sync::Arc;
use crate::block::Block;
use crate::player::inventory::PlayerInventory;
use crate::engine::texture::Texture;
use crate::engine::gui::{gui::*, render::*};
use cgmath::*;

pub struct GUI {
    pub toolbar: Texture,
    pub selector: Texture,
    pub crosshairs: Texture,
    pub inventory: Texture,
    pub selected_slot: i32,
}

impl GUI {

    pub fn new() -> Self {

        let toolbar = Texture::from_path("assets/item_bar.png").into();
        let selector = Texture::from_path("assets/item_selected.png").into();
        let crosshairs = Texture::from_path("assets/crosshairs.png").into();
        let inventory = Texture::from_path("assets/inventory.png").into();

        /* let heart = std::rc::Rc::from(Texture::from_path("assets/heart.png"));
        let mut hearts = vec![];
        for i in 0..10 {
            hearts.push(FlatGUI {
                texture: std::rc::Rc::clone(&heart),
                pos: Vector2 {x: -1.3/2., y: -0.8 },
            })
        }
        let hearts = ContainerGUI(hearts); */

        Self {
            selected_slot: 0i32,
            toolbar,
            crosshairs,
            selector,
            inventory
        }

    }

    pub fn scroll(&mut self, scroll: i32) {
        self.selected_slot = ((self.selected_slot + scroll) % 9 + 9) % 9
    }

    pub fn determine_hovered_slot(&self, mouse: (i32, i32)) -> Option<u32> {
        let pos = (0i32,0);
        let (w, h) = (900, 700);
        let (hw, hh) = (450, 350);
        // hotbar
        let lr = (hw-3*90+6, 6);
        let x = (mouse.0 - lr.0) / 3;
        let y = ((h - mouse.1) - lr.1) / 3;
        if x % 20 < 16 && y < 16 {
            let x = x / 20;
            if x >= 0 && x <= 8 {
                return Some(x as u32)
            }
        }
        // inventory
        let lr = (hw-3*90+6, hh-3*70+6);
        let x = (mouse.0 - lr.0) / 3;
        let y = ((h - mouse.1) - lr.1) / 3;
        if x % 20 >= 16 || y % 20 >= 16 {
            return None
        }
        if x < 0 || y < 0 {
            return None
        }
        let x = x / 20;
        let y = y / 20;
        if x < 0 || x > 8 || y < 0 || y > 2 {
            return None
        }
        Some((x + 9 * (3 - y)) as u32)
    }

    pub fn render(
        &self,
        r: &mut GUIRenderer,
        vao: &crate::engine::vao::VAO,
        atlas: &crate::engine::texture::TextureAtlas,
        hotbar: &PlayerInventory,
        show_inventory: bool,
        mouse_pos: (i32, i32)
    ) {
        r.start();
        r.square.bind();

        let (w,h) = (900, 700);
        let (hw, hh) = (450, 350);

        // toolbar 180x20
        self.toolbar.bind();
        r.set_pixels(hw, 0);
        r.move_pixels(-90, 0);
        r.set_uniforms(180, 20);
        r.square.draw();
        
        // selector 20x20
        self.selector.bind();
        r.move_pixels(20 * self.selected_slot, 0);
        r.set_uniforms(20, 20);
        r.square.draw();

        // inventory 180 x 140
        if show_inventory {
            self.inventory.bind();
            r.set_pixels(hw, hh);
            r.move_pixels(-90, -70);
            r.set_uniforms(180, 140);
            r.square.draw();
        }

        // items
        atlas.texture().bind();
        vao.bind();
        r.set_pixels(hw, 0); // hotbar
        r.move_pixels(-90, 0);
        r.move_pixels(2, 2);
        for s in &hotbar.hotbar {

            if let Some(s) = s {
                r.set_uniforms(16, 16);
                vao.draw_18(s.item.id as i32);
            }

            r.move_pixels(20, 0);
        }
        if show_inventory {
            r.set_pixels(hw, hh); // inventory
            r.move_pixels(-90, -70);
            r.move_pixels(2, 2);
            r.move_pixels(0, 2*20);
            for (i, s) in hotbar.inventory.iter().enumerate() {
                if i % 9 == 0 && i > 0 {
                    r.move_pixels(-9*20, -20);
                }
    
                if let Some(s) = s {
                    r.set_uniforms(16, 16);
                    vao.draw_18(s.item.id as i32);
                }
    
                r.move_pixels(20, 0);
            }
        }
        
        r.stop();
    }

}
