
use crate::vao::VAO;
use crate::program::Program;
use cgmath::*;

pub struct Cursor {
    pub pos: Vector2<i32>,
}

impl Cursor {

    pub fn new() -> Self {
        Self {
            pos: Vector2 {x:0, y:0}
        }
    }

    pub fn move_pixels(&mut self, x: i32, y: i32) {
        self.pos.x += x;
        self.pos.y += y;
    }

}

pub struct GUIRenderer {
    pub screen_size: (i32, i32),
    pub pixel_scale: i32,
    pub cursor: Cursor,
    pub square: VAO,
    pub program: Program,
}

impl GUIRenderer {

    pub fn new(screen_size: (i32, i32)) -> Self {
        let verts = vec![
            0., 0., 0.,
            1., 0., 0.,
            0., 1., 0.,
            0., 1., 0.,
            1., 0., 0.,
            1., 1., 0.,
        ];
        let uvs = vec![ // y is swapped
            0., 1.,
            1., 1.,
            0., 0.,
            0., 0.,
            1., 1.,
            1., 0.,
        ];

        let square = VAO::textured(&verts, &uvs);

        let program = Program::load(
            include_str!("vert.glsl"),
            include_str!("frag.glsl"),
            vec!["position", "scale"]
        );

        Self {
            pixel_scale: 3,
            screen_size,
            cursor: Cursor::new(),
            square,
            program,
        }
        
    }

    pub fn start(&self) {
        self.program.enable();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn stop(&self) {
        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn move_pixels(&mut self, x: i32, y: i32) {
        self.cursor.move_pixels(x * self.pixel_scale, y * self.pixel_scale);
    }

    pub fn set_pixels(&mut self, x: i32, y: i32) {
        self.cursor.pos.x = x;
        self.cursor.pos.y = y;
    }

    pub fn set_uniforms(&self, img_width: i32, img_height: i32) {
        let (pw, ph) = (2. / self.screen_size.0 as f32, 2. / self.screen_size.1 as f32);
        self.program.load_vec2(0, &Vector2 {
            x: pw * self.cursor.pos.x as f32 - 1.,
            y: ph * self.cursor.pos.y as f32 - 1.,
        });
        self.program.load_vec2(1, &Vector2 {
            x: (self.pixel_scale * img_width) as f32 * pw,
            y: (self.pixel_scale * img_height) as f32 * ph,
        });
    }

}

/* pub struct ScaleOffset {
    pub scale: Vector2<f32>,
    pub offset: Vector2<f32>,
}

pub fn calc_scale_offset(screen_size: (f32,f32), image_size: (f32,f32), anchor: Anchor) -> ScaleOffset {
    let ar = screen_size.0 / screen_size.1;
    let iar = image_size.0 / image_size.1;
    let scale = {
        let s = 3.; // pixel scale
        Vector2 {
            x: 2. * s * image_size.0 / screen_size.0,
            y: 2. * s * image_size.1 / screen_size.1,
        }
    };

    let mut offset = anchor.to_vec();
    offset.x *= scale.x;
    offset.y *= scale.y;

    ScaleOffset { scale, offset }

} */

/* #[derive(Clone,Copy)]
pub enum Anchor {
    Offset(f32,f32),
    Center,
    Top,
    Bottom,
    Right,
    Left,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl Anchor {
    pub fn to_vec(self) -> Vector2<f32> {
        match self {
            Self::Center => Vector2 { x: -0.5, y: -0.5 },
            Self::Bottom => Vector2 { x: -0.5, y: 0. },
            Self::Top => Vector2 { x: 0.5, y: 0. },
            Self::Right => Vector2 { x: -1., y: -0.5 },
            Self::Left => Vector2 { x: 0., y: -0.5 },
            Self::TopRight => Vector2 { x: -1., y: -1. },
            Self::TopLeft => Vector2 { x: 0., y: -1. },
            Self::BottomRight => Vector2 { x: -1., y: 0. },
            Self::BottomLeft => Vector2 { x: 0., y: 0. },
            Self::Offset(x,y) => Vector2 { x, y },
        }
    }
    pub fn add(self, rhs: Self) -> Self {
        let Vector2 {x, y} = self.to_vec() + rhs.to_vec();
        Self::Offset(x,y)
    }
} */
