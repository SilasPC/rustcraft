
use crate::item::ItemStack;
use std::sync::Arc;
use crate::block::Block;
use crate::player::inventory::PlayerInventory;
use crate::engine::texture::Texture;
use crate::engine::gui::{gui::*, render::*};
use cgmath::*;

pub struct PlayerGUI {
    pub toolbar: FlatGUI,
    pub selector: FlatGUI,
    pub crosshairs: FlatGUI,
    pub hearts: ContainerGUI,
    pub selected_slot: i32,
}

impl PlayerGUI {

    pub fn new() -> Self {

        let toolbar = Texture::from_path("assets/item_bar.png");
        let toolbar = FlatGUI {
            texture: toolbar.into(),
            anchor: Anchor::Bottom,
            pos: -Vector2::unit_y(),
        };

        let selector = Texture::from_path("assets/item_selected.png");
        let selector = FlatGUI {
            texture: selector.into(),
            anchor: Anchor::Bottom.add(Anchor::Offset(-4.,0.)),
            pos: -Vector2::unit_y(),
        };

        let crosshairs = Texture::from_path("assets/crosshairs.png");
        let crosshairs = FlatGUI {
            texture: crosshairs.into(),
            anchor: Anchor::Center,
            pos: Vector2 { x: 0., y: 0. },
        };

        let heart = std::rc::Rc::from(Texture::from_path("assets/heart.png"));
        let mut hearts = vec![];
        for i in 0..10 {
            hearts.push(FlatGUI {
                texture: std::rc::Rc::clone(&heart),
                anchor: Anchor::BottomLeft.add(Anchor::Offset(1.1 * i as f32, 0.)),
                pos: Vector2 {x: -1.3/2., y: -0.8 },
            })
        }
        let hearts = ContainerGUI(hearts);

        Self {
            selected_slot: 0i32,
            toolbar,
            crosshairs,
            selector,
            hearts
        }

    }

    pub fn scroll(&mut self, scroll: i32) {
        self.selected_slot = ((self.selected_slot + scroll) % 9 + 9) % 9
    }

    pub fn render(
        &self,
        cursor: &mut crate::engine::gui::render::Cursor,
        guirend: &GUIRenderer,
        vao: &crate::engine::vao::VAO,
        atlas: &crate::engine::texture::TextureAtlas,
        hotbar: &PlayerInventory
    ) {
        guirend.program.enable();
        guirend.square.bind();
        guirend.start();
        
        // toolbar 180x20
        cursor.move_pixels(3 * -90, 0);
        let orig_pos = cursor.pos;
        
        let tb = &self.toolbar;
        tb.texture.bind();
        guirend.set_uniform(&cursor.pos, &(3. * cursor.img_size_to_scale(180, 20)));
        guirend.square.draw();
        
        self.selector.texture.bind();
        cursor.move_pixels(20 * 3 * self.selected_slot, 0);
        guirend.set_uniform(&cursor.pos, &(3. * cursor.img_size_to_scale(20, 20)));
        guirend.square.draw();

        cursor.pos = orig_pos;

        atlas.texture().bind();
        vao.bind();
        
        // 16 pixels of space inside toolbar, border 2 pixels, 3 pixelscale
        cursor.move_pixels(6, 6);
        for s in &hotbar.hotbar {

            if let Some(s) = s {
                guirend.set_uniform(&cursor.pos, &cursor.img_size_to_scale(3*16, 3*16));
                vao.draw_18(s.item.id as i32);
            }

            cursor.move_pixels(3*4+3*16, 0);
        }
        
        guirend.stop();
    }

}
