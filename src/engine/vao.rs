
use gl::types::{GLuint as uint, GLint as int, GLenum};

pub enum RenderKind {
    Triangles,
    Lines,
}

pub struct VAO {
    id: uint,
    verts: uint,
    uvs: uint,
    vertex_count: i32,
    kind: RenderKind,
}

impl VAO {

    pub fn lines(verts: &[f32]) -> Self {
        
        unsafe {
            
            let mut id = 0;
    
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            
            let mut verts_id = 0;
            gl::GenBuffers(1, &mut verts_id);
            configure_float_vbo(verts_id, 0, 3);
            transfer_to_array_buffer(verts_id, verts);
    
            gl::BindVertexArray(0);
    
            Self {
                id,
                verts: verts_id,
                uvs: 0,
                vertex_count: verts.len() as i32,
                kind: RenderKind::Lines,
            }

        }

    }

    pub fn update(&mut self, verts: &[f32], uvs: &[f32]) {

        if verts.len() / 3 != uvs.len() / 2 {
            panic!("UVs - VERTs count mismatch ({} != {})", uvs.len() / 3, verts.len() / 2);
        }
        unsafe {
            gl::BindVertexArray(self.id);
            transfer_to_array_buffer(self.verts, verts);
            transfer_to_array_buffer(self.uvs, uvs);
            gl::BindVertexArray(0);
        }
        self.vertex_count = verts.len() as i32;
    }

    pub fn empty_textured() -> Self {

        unsafe {
            
            let mut id = 0;
    
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            
            let mut ids = [0,0];
            gl::GenBuffers(2, ids.as_mut_ptr());
            configure_float_vbo(ids[0], 0, 3);
            configure_float_vbo(ids[1], 1, 2);
    
            gl::BindVertexArray(0);
    
            Self {
                id,
                verts: ids[0],
                uvs: ids[1],
                vertex_count: 0,
                kind: RenderKind::Triangles,
            }

        }

    }

    pub fn textured(verts: &[f32], uvs: &[f32]) -> Self {
        unsafe {

            if verts.len() / 3 != uvs.len() / 2 {
                panic!("UVs - VERTs count mismatch ({} != {})", uvs.len() / 3, verts.len() / 2);
            }
            
            let mut id = 0;

            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            
            let mut verts_id = 0;
            gl::GenBuffers(1, &mut verts_id);
            configure_float_vbo(verts_id, 0, 3);
            transfer_to_array_buffer(verts_id, verts);

            let mut uvs_id = 0;
            gl::GenBuffers(1, &mut uvs_id);
            configure_float_vbo(uvs_id, 1, 2);
            transfer_to_array_buffer(uvs_id, uvs);

            gl::BindVertexArray(0);

            Self {
                id,
                verts: verts_id,
                uvs: uvs_id,
                vertex_count: verts.len() as i32,
                kind: RenderKind::Triangles,
            }

        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id)
        }
    }

    pub fn draw(&self) {
        let (gle, count) = match self.kind {
            RenderKind::Triangles => (gl::TRIANGLES, 3),
            RenderKind::Lines => (gl::LINES, 2),
        };
        unsafe {
            gl::DrawArrays(
                gle,
                0,
                self.vertex_count / count
            );
        }
    }

    pub fn triangle_count(&self) -> i32 {self.vertex_count / 3}

}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
            let vbos = [self.verts, self.uvs];
            gl::DeleteBuffers(vbos.len() as i32, vbos.as_ptr());
        }
    }

}

fn configure_float_vbo(vbo: uint, location: uint, vals_per_vertex: usize) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(location);
        gl::VertexAttribPointer(
            location,
            vals_per_vertex as int, // values per vertex
            gl::FLOAT,
            gl::FALSE,
            (vals_per_vertex * std::mem::size_of::<f32>()) as int, // stride
            std::ptr::null() // first offset
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

fn transfer_to_array_buffer<T>(vbo: uint, data: &[T]) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<f32>())
                as gl::types::GLsizeiptr,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

