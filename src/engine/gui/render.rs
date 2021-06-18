
use crate::vao::VAO;
use crate::program::Program;
use cgmath::*;

pub trait GUI {
    fn render(&self, renderer: &mut Renderer);
}

pub struct GUIRenderer {
    vao: VAO,
    program: Program,
}

impl GUIRenderer {

    pub fn new() -> Self {
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

        let vao = VAO::textured(&verts, &uvs);

        let program = Program::load(
            include_str!("vert.glsl"),
            include_str!("frag.glsl"),
            vec!["position", "scale"]
        );

        Self {
            vao,
            program,
        }
        
    }

    pub fn render(&mut self, gui: &impl GUI, aspect_ratio: f32) {
        self.program.enable();
        self.program.load_vec2(0, &Vector2 {x: 0., y: 0.});
        self.program.load_vec2(1, &Vector2 {x: 1./aspect_ratio, y: 0.});
        self.vao.bind();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let mut renderer = Renderer(self, aspect_ratio);
        gui.render(&mut renderer);
        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

}

pub struct Renderer<'a>(&'a mut GUIRenderer, f32);

impl<'a> Renderer<'a> {

    pub fn render(&self, position: Vector2<f32>, image_aspect_ratio: f32, anchor: Anchor, scale: Scale) {
        let [mut scale, offset] = calc(self.1, image_aspect_ratio, anchor, scale);
        self.0.program.load_vec2(0, &(position + offset));
        self.0.program.load_vec2(1, &scale);
        self.0.vao.draw();
    }

}

fn calc(ar: f32, iar: f32, anchor: Anchor, scale: Scale) -> [Vector2<f32>; 2] {

    let scale = match scale {
        Scale::FixedWidth(w) => {
            Vector2 {
                x: w,
                y: w * ar / iar,
            }
        },
        Scale::FixedHeight(h) => {
            Vector2 {
                x: h * iar / ar,
                y: h,
            }
        },
    };

    let mut offset = anchor.to_vec();
    offset.x *= scale.x;
    offset.y *= scale.y;

    [scale, offset]

}

#[derive(Clone,Copy)]
pub enum Scale {
    FixedWidth(f32),
    FixedHeight(f32),
}

#[derive(Clone,Copy)]
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
}
