
use std::sync::Arc;
use super::texture::*;
use super::text::font::*;


pub struct Loader {
    textures: Vec<Arc<Texture>>,
    atlases: Vec<Arc<TextureAtlas>>,
    fonts: Vec<Arc<Font>>,
}

impl Loader {

    pub fn new() -> Self {
        Self {
            textures: vec![],
            atlases: vec![],
            fonts: vec![],
        }
    }

    pub fn load_texture(&mut self, path: &str) -> Arc<Texture> {
        let text = Arc::new(Texture::from_path(path));
        self.textures.push(text.clone());
        text
    }
    
    pub fn load_texture_atlas(&mut self, path: &str, size: usize) -> Arc<TextureAtlas> {
        let text = Arc::new(
            TextureAtlas::new(
                Texture::from_path(path),
                size
            )
        );
        self.atlases.push(text.clone());
        text
    }

    pub fn load_font(&mut self, atlas_file: &str, fnt_file: &str) -> Arc<Font> {
        let font = Arc::new(Font::from_font_files(atlas_file, fnt_file));
        self.fonts.push(font.clone());
        font
    }

}