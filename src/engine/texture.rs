
use gl::types::GLuint as uint;
use image::*;

pub struct TextureAtlas {
    texture: Texture,
    size: usize
}

impl TextureAtlas {
    pub fn new(texture: Texture, size: usize) -> Self {
        Self {
            texture,
            size
        }
    }
    pub fn get_uv(&self, index: usize) -> (f32,f32) {
        let u = (index % self.size) as f32 / self.size as f32;
        let v = (index / self.size) as f32 / self.size as f32;
        (u,v)
    }
    pub fn uv_dif(&self) -> f32 {1./self.size as f32}
    pub fn id(&self) -> uint {self.texture.id}
    pub fn texture(&self) -> &Texture {&self.texture}
}


pub struct Texture {
    id: uint,
    size: (f32, f32),
}

impl Texture {

    pub fn from_path(path: &str) -> Self {
        use image::io::Reader as ImgReader;
        let img = ImgReader::open(path)
            .map_err(|_| format!("Failed to open texture file {}",path))
            .unwrap()
            .decode().unwrap();
        let img = img.into_rgba8();
        let size = img.dimensions();
        Self::from_rgba(&img, size)
    }

    // https://docs.rs/piston2d-opengl_graphics/0.78.0/src/opengl_graphics/texture.rs.html#181-224
    fn from_rgba(img: &RgbaImage, size: (u32,u32)) -> Self {
        let size = (size.0 as f32, size.1 as f32);
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as *const _,
            );
        }
        Texture { id, size }
    }

    pub fn id(&self) -> uint {self.id}
    pub fn size(&self) -> (f32, f32) {self.size}

    pub fn aspect_ratio(&self) -> f32 {
        let (x,y) = self.size;
        x as f32 / y as f32
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as f32);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as f32);
        }
    }

}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}