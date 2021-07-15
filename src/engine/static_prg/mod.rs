
use crate::prelude::*;

pub struct StaticProgram {
    program: Program,
}

impl StaticProgram {
    
    pub fn new() -> Self {
        Self {
            program: Program::load(
                include_str!("./vert.glsl"),
                include_str!("./frag.glsl"),
                vec!["transform", "view", "project", "uvScale", "uvOffset", "lightScale"]
            )
        }
    }

    pub fn load_light(&mut self, val: f32) {
        self.program.load_f32(5, val)
    }
    pub fn load_view(&mut self, view: &Matrix4<f32>, project: &Matrix4<f32>) {
        self.program.load_mat4(1, view);
        self.program.load_mat4(2, project);
    }
    pub fn load_transform(&mut self, transform: &Matrix4<f32>) {
        self.program.load_mat4(0, transform);
    }
    pub fn load_uv(&mut self, scale: (f32, f32), offset: (f32, f32)) {
        self.program.load_vec2(3, &scale.into());
        self.program.load_vec2(4, &offset.into());
    }

    pub fn enable(&mut self) {
        self.program.enable();
    }

}

