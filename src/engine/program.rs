
use gl;
use std;
use std::ffi::CString;
use crate::util::make_cstr;
use gl::types::GLint as int;
use gl::types::GLuint as uint;
use cgmath::*;

#[derive(Debug)]
pub struct Program {
    id: uint,
    frag: uint,
    vert: uint,
    uniforms: Vec<int>,
}

impl Program {

    pub fn load(vert: &str, frag: &str, uniforms: Vec<&str>) -> Self {

        let vert = compile_shader(vert, gl::VERTEX_SHADER);
        let frag = compile_shader(frag, gl::FRAGMENT_SHADER);

        let id = unsafe {gl::CreateProgram()};

        unsafe {
            gl::AttachShader(id, vert);
            gl::AttachShader(id, frag);
            gl::LinkProgram(id);
        };

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {

            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = make_cstr(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            panic!("{}", error.to_string_lossy());

        }

        let uniforms = uniforms.into_iter()
            .map(CString::new)
            .map(Result::unwrap)
            .map(|u| unsafe {gl::GetUniformLocation(id, u.as_ptr())})
            .collect();

        unsafe {
            gl::DetachShader(id, vert);
            gl::DetachShader(id, frag);
        }

        Self { id, vert, frag, uniforms }

    }

    pub fn enable(&self) {unsafe {gl::UseProgram(self.id);}}
    pub fn disable(&self) {unsafe {gl::UseProgram(0);}}

    pub fn id(&self) -> gl::types::GLuint { self.id }

    pub fn load_f32(&self, index: usize, data: f32) {
        unsafe {
            gl::Uniform1f(self.uniforms[index], data);
        }
    }
    pub fn load_vec2(&self, index: usize, data: &Vector2<f32>) {
        unsafe {
            gl::Uniform2f(self.uniforms[index], data.x, data.y);
        }
    }
    pub fn load_vec3(&self, index: usize, data: &Vector3<f32>) {
        unsafe {
            gl::Uniform3f(self.uniforms[index], data.x, data.y, data.z);
        }
    }
    pub fn load_vec4(&self, index: usize, data: &Vector4<f32>) {
        unsafe {
            gl::Uniform4f(self.uniforms[index], data.x, data.y, data.z, data.w);
        }
    }
    pub fn load_mat4(&self, index: usize, data: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms[index], 1, gl::FALSE, data.as_ptr());
        }
    }

}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
            gl::DeleteShader(self.frag);
            gl::DeleteShader(self.vert);
        }
    }
}

fn compile_shader(
    src: &str,
    kind: gl::types::GLenum
) -> gl::types::GLuint {

    let source = CString::new(src).unwrap();

    let id;
    unsafe {
        id = gl::CreateShader(kind);
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {

        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = make_cstr(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }

        panic!("\nFailed to compile shader '{}':\n{}", src, error.to_string_lossy());

    }

    id

}
