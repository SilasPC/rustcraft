
use crate::engine::texture::Texture;
use crate::engine::gui::{gui::*, render::*};
use cgmath::*;

pub struct PlayerGUI {
    pub toolbar: FlatGUI,
    pub selector: FlatGUI,
    pub crosshairs: FlatGUI,
    pub hearts: ContainerGUI,
    pub scroll_pos: i32,
}

impl PlayerGUI {

    pub fn new() -> Self {

        let toolbar = Texture::from_path("assets/item_bar.png");
        let toolbar = FlatGUI {
            texture: toolbar.into(),
            anchor: Anchor::Bottom,
            pos: -Vector2::unit_y(),
            scale: Scale::FixedWidth(1.3)
        };
        // width == 1.3 => height == 1.3/9.

        let selector = Texture::from_path("assets/item_selected.png");
        let selector = FlatGUI {
            texture: selector.into(),
            anchor: Anchor::Bottom.add(Anchor::Offset(-4.,0.)),
            pos: -Vector2::unit_y(),
            scale: Scale::FixedWidth(1.3/9.)
        };

        let crosshairs = Texture::from_path("assets/crosshairs.png");
        let crosshairs = FlatGUI {
            texture: crosshairs.into(),
            anchor: Anchor::Center,
            pos: Vector2 { x: 0., y: 0. },
            scale: Scale::FixedHeight(0.1),
        };

        let heart = std::rc::Rc::from(Texture::from_path("assets/heart.png"));
        let mut hearts = vec![];
        for i in 0..10 {
            hearts.push(FlatGUI {
                texture: std::rc::Rc::clone(&heart),
                anchor: Anchor::BottomLeft.add(Anchor::Offset(1.1 * i as f32, 0.)),
                pos: Vector2 {x: -1.3/2., y: -0.8 },
                scale: Scale::FixedWidth(1.3/9. / 2.)
            })
        }
        let hearts = ContainerGUI(hearts);

        Self {
            scroll_pos: 0i32,
            toolbar,
            crosshairs,
            selector,
            hearts
        }

    }

    pub fn scroll(&mut self, scroll: i32) {
        self.scroll_pos = ((self.scroll_pos + scroll) % 9 + 9) % 9
    }

}

impl GUI for PlayerGUI {
    fn render(&mut self, renderer: &mut Renderer) {
        self.selector.anchor = Anchor::Bottom.add(Anchor::Offset(self.scroll_pos as f32 - 4.,0.));

        self.toolbar.render(renderer);
        self.selector.render(renderer);
        self.hearts.render(renderer);
        self.crosshairs.render(renderer);
    }
}