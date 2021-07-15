
use crate::prelude::*;
use crate::engine::program::Program;

pub struct LineProgram {
    program: Program,
    cube: Arc<VAO>
}

impl LineProgram {
    pub fn new(cube: Arc<VAO>) -> Self {
        let program = Program::load(VERT, FRAG, vec!["transform", "view", "project", "color"]);
        Self { program, cube }
    }
    pub fn enable(&mut self) {
        self.program.enable();
    }
    pub fn load_view(&mut self, view: &Matrix4<f32>, project: &Matrix4<f32>) {
        self.program.load_mat4(1, view);
        self.program.load_mat4(2, project);
    }
    pub fn load_transform(&mut self, transform: &Matrix4<f32>) {
        self.program.load_mat4(0, transform);
    }
    pub fn load_color(&mut self, c: &Vector4<f32>) {
        self.program.load_vec4(3, c);
    }
    pub fn bind(&mut self) {
        self.cube.bind();
    }
    pub fn draw(&mut self) {
        self.cube.draw();
    }
    pub fn bind_and_draw(&mut self) {
        self.cube.bind();
        self.cube.draw();
    }
}

pub fn box_vao() -> VAO {
    VAO::lines(&[
        0., 0., 0.,
        1., 0., 0.,
        0., 0., 0.,
        0., 1., 0.,
        0., 0., 0.,
        0., 0., 1.,
        1., 1., 1.,
        0., 1., 1.,
        1., 1., 1.,
        1., 0., 1.,
        1., 1., 1.,
        1., 1., 0.,
        0., 1., 0.,
        1., 1., 0.,
        0., 1., 0.,
        0., 1., 1.,
        1., 0., 1.,
        0., 0., 1.,
        1., 0., 1.,
        1., 0., 0.,
        1., 0., 0.,
        1., 1., 0.,
        0., 0., 1.,
        0., 1., 1.,
    ])
}

const VERT: &'static str = r#"
#version 400 core

uniform mat4 transform;
uniform mat4 view;
uniform mat4 project;

layout (location = 0) in vec3 vert;

void main()
{
    gl_Position = project * view * transform * vec4(vert, 1.0);
}
"#;

const FRAG: &'static str = r#"
#version 400 core

uniform vec4 color;

out vec4 Color;

void main()
{
    Color = color;
}
"#;