
use crate::engine::program::Program;
use crate::engine::vao::VAO;

pub struct LineProgram {
    pub program: Program
}

impl LineProgram {
    pub fn new() -> Self {
        let program = Program::load(VERT, FRAG, vec!["transform", "view", "project", "color"]);
        Self {program}
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