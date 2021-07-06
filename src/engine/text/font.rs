
use std::sync::Arc;
use cgmath::Vector2;
use crate::engine::texture::Texture;
use std::rc::Rc;
use super::text::Text;
use crate::engine::program::Program;

pub struct TextRenderer {
    program: Program,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            program: Program::load(
                include_str!("vert.glsl"),
                include_str!("frag.glsl"),
                vec!["scale","position"]
            )
        }
    }
    pub fn render(&self, text: &Text, x: f32, y: f32, size: (u32, u32)) {
        let ar = size.0 as f32 / size.1 as f32;
        let (w,h) = (2. / size.0 as f32, 2. / size.1 as f32);
        let s = 0.3;
        self.program.enable();
        self.program.load_vec2(0, &Vector2 {x: s * ar, y: s});
        self.program.load_vec2(1, &Vector2 {x, y});
        text.vao.bind();
        text.font.atlas.bind();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        text.vao.draw();
        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}

pub struct Font {
    pub atlas: Texture,
    pub ascii: Vec<Char>,
}

impl Font {

    pub fn build_text(self: &Arc<Self>, text: String) -> Text {
        Text::new(text, self.clone())
    }

    pub fn from_font_files(bitmap: &str, fontmap: &str) -> Self {
        let atlas = Texture::from_path(bitmap);
        let ascii = read_fnt_file(fontmap);
        Self {
            atlas,
            ascii
        }
    }

}

#[derive(Default,Clone,Copy,Debug)]
pub struct Char {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub xo: f32,
    pub yo: f32,
    pub adv: f32,
}

fn read_fnt_file(file: &str) -> Vec<Char> {
    let rgx = regex::Regex::new(r"id=(\d+)\s+x=(\d+)\s+y=(\d+)\s+width=(\d+)\s+height=(\d+)\s+xoffset=(-?\d+)\s+yoffset=(-?\d+)\s+xadvance=(\d+)").unwrap();
    let file = std::fs::read_to_string(file).unwrap();
    let mut chars = vec![Char::default(); 256];
    for line in file.lines().filter(|l| l.starts_with("char ")) {
        let caps = rgx.captures(line).unwrap();
        chars[caps[1].parse::<usize>().unwrap()] = Char {
            x: caps[2].parse().unwrap(),
            y: caps[3].parse().unwrap(),
            w: caps[4].parse().unwrap(),
            h: caps[5].parse().unwrap(),
            xo: caps[6].parse().unwrap(),
            yo: caps[7].parse().unwrap(),
            adv: caps[8].parse().unwrap(),
        }
    }
    chars
}