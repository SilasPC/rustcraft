
use crate::engine::vao::VAO;
use std::rc::Rc;
use super::font::*;

pub struct Text {
    pub font: Rc<Font>,
    pub text: String,
    pub vao: VAO,
}

impl Text {
    
    pub fn new(text: String, font: Rc<Font>) -> Self {
        let size = font.atlas.size();
        let size = (size.0 as f32, size.1 as f32);
        let (verts, uvs) = Self::build(text.as_ref(), font.as_ref(), size);
        let vao = VAO::textured(&verts, &uvs);
        Self {
            vao,
            font,
            text,
        }
    }

    pub fn set_text(&mut self, text: String) {
        let size = self.font.atlas.size();
        let size = (size.0 as f32, size.1 as f32);
        let (verts, uvs) = Self::build(text.as_ref(), self.font.as_ref(), size);
        self.vao.update(&verts, &uvs);
        self.text = text;
    }

    fn build(text: &str, font: &Font, size: (f32,f32)) -> (Vec<f32>,Vec<f32>) {
        let mut verts = vec![];
        let mut uvs = vec![];
        let mut x = 0.;
        let mut y = 0.;
        let (w, h) = size;
        for c in text.chars().filter(|c| c.is_ascii()) {
            if c == '\n' {
                y -= 38. / 256.;
                x = 0.;
            }
            let c = font.ascii[c as usize];
            {
                let x = x + c.xo / w;
                let w = c.w / w;
                let h = c.h / h;
                verts.extend_from_slice(&[
                    x, y, 0.,
                    x+w, y, 0.,
                    x, y+h, 0.,
                    x, y+h, 0.,
                    x+w, y, 0.,
                    x+w, y+h, 0.,
                ]);
                let x = c.x / size.0;
                let y = c.y / size.1;
                uvs.extend_from_slice(&[ // ys are swapped
                    x, y+h,
                    x+w, y+h,
                    x, y,
                    x, y,
                    x+w, y+h,
                    x+w, y,
                ]);
            }
            x += c.adv / w;
        }
        (verts,uvs)
    }

}
