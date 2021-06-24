
use crate::vao::VAO;
use crate::program::Program;
use cgmath::*;

pub trait GUI {
    /// Typical implementation will bind a texture, then call renderer.render(...)
    fn render(&mut self, renderer: &mut Renderer);
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

    pub fn render(&mut self, gui: &mut impl GUI, screen_size: (u32,u32), mouse_pos: (i32, i32), delta: f32) {
        let screen_size = (screen_size.0 as f32, screen_size.1 as f32);
        let mouse_pos = (mouse_pos.0 as f32 / screen_size.0, mouse_pos.1 as f32 / screen_size.1);
        let mouse_pos = (mouse_pos.0 * 2. - 1., mouse_pos.1 * 2. - 1.);
        let aspect_ratio = screen_size.0 / screen_size.1;
        self.program.enable();
        self.program.load_vec2(0, &Vector2 {x: 0., y: 0.});
        self.program.load_vec2(1, &Vector2 {x: 1./aspect_ratio, y: 0.});
        self.vao.bind();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let mut renderer = Renderer {
            renderer: self,
            screen_size,
            delta,
            mouse_pos
        };
        gui.render(&mut renderer);
        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

}

pub struct Renderer<'a> {
    renderer: &'a mut GUIRenderer,
    screen_size: (f32, f32),
    delta: f32,
    mouse_pos: (f32, f32)
}

impl<'a> Renderer<'a> {

    pub fn is_mouse_over(&self, image_size: (f32, f32), anchor: Anchor, scale: Scale) -> bool {
        let [scale, offset] = calc(self.screen_size, image_size, anchor, scale);
        let pos = self.mouse_pos;

        // println!("mouse: {:?}, scale: {:?}, offset: {:?}",pos,scale,offset);

        pos.0 >= offset.x && pos.1 >= offset.y && pos.0 <= offset.x + scale.x && pos.1 <= offset.y + scale.y

    }

    pub fn render(&self, position: Vector2<f32>, image_size: (f32, f32), anchor: Anchor, scale: Scale) {
        let aspect_ratio = self.screen_size.0 / self.screen_size.1;
        let [scale, offset] = calc(self.screen_size, image_size, anchor, scale);
        self.renderer.program.load_vec2(0, &(position + offset));
        self.renderer.program.load_vec2(1, &scale);
        self.renderer.vao.draw();
    }

    pub fn delta(&self) -> f32 {self.delta}

}

fn calc(screen_size: (f32,f32), image_size: (f32,f32), anchor: Anchor, scale: Scale) -> [Vector2<f32>; 2] {
    let ar = screen_size.0 / screen_size.1;
    let iar = image_size.0 / image_size.1;
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
        Scale::Pixels(s) => {
            let s = s as f32;
            Vector2 {
                x: 2. * s * image_size.0 / screen_size.0,
                y: 2. * s * image_size.1 / screen_size.1,
            }
        }
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
    Pixels(u32),
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
