
use std::rc::Rc;
use super::texture::*;
use super::text::font::*;


pub struct Loader {
    textures: Vec<Rc<Texture>>,
    atlases: Vec<Rc<TextureAtlas>>,
    fonts: Vec<Rc<Font>>,
}

impl Loader {

    pub fn new() -> Self {
        Self {
            textures: vec![],
            atlases: vec![],
            fonts: vec![],
        }
    }

    pub fn load_texture(&mut self, path: &str) -> Rc<Texture> {
        let text = Rc::new(Texture::from_path(path));
        self.textures.push(text.clone());
        text
    }
    
    pub fn load_texture_atlas(&mut self, path: &str, size: usize) -> Rc<TextureAtlas> {
        let text = Rc::new(
            TextureAtlas::new(
                Texture::from_path(path),
                size
            )
        );
        self.atlases.push(text.clone());
        text
    }

    pub fn load_font(&mut self, atlas_file: &str, fnt_file: &str) -> Rc<Font> {
        let font = Rc::new(Font::from_font_files(atlas_file, fnt_file));
        self.fonts.push(font.clone());
        font
    }

}