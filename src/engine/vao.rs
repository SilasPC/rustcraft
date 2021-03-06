
use gl::types::{GLuint as uint, GLint as int, GLenum};

#[derive(Debug)]
pub enum RenderKind {
    Triangles,
    Lines,
}

#[derive(Debug)]
pub struct VAO {
    id: uint,
    verts: uint,
    uvs: uint,
    light: uint,
    vertex_count: i32,
    kind: RenderKind,
}

impl VAO {

    pub fn lines(verts: &[f32]) -> Self {
        
        unsafe {
            
            let mut id = 0;
    
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            
            assert!(id != 0);
            
            let mut verts_id = 0;
            gl::GenBuffers(1, &mut verts_id);
            configure_float_vbo(verts_id, 0, 3);
            transfer_to_array_buffer(verts_id, verts);
    
            gl::BindVertexArray(0);
    
            Self {
                id,
                verts: verts_id,
                uvs: 0,
                light: 0,
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

    pub fn update_lit(&mut self, verts: &[f32], uvs: &[f32], light: &[f32]) {

        if verts.len() / 3 != uvs.len() / 2 {
            panic!("UVs - VERTs count mismatch ({} != {})", uvs.len() / 3, verts.len() / 2);
        }
        if verts.len() / 3 != light.len() {
            panic!("VERTs - LIGHTs count mismatch ({} != {})", verts.len() / 3, light.len());
        }

        unsafe {
            gl::BindVertexArray(self.id);
            transfer_to_array_buffer(self.verts, verts);
            transfer_to_array_buffer(self.uvs, uvs);
            transfer_to_array_buffer(self.light, light);
            gl::BindVertexArray(0);
        }
        self.vertex_count = verts.len() as i32;
    }

    pub fn empty_textured() -> Self {

        unsafe {
            
            let mut id = 0;
    
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);

            assert!(id != 0);
            
            let mut ids = [0,0];
            gl::GenBuffers(2, ids.as_mut_ptr());
            configure_float_vbo(ids[0], 0, 3);
            configure_float_vbo(ids[1], 1, 2);
    
            gl::BindVertexArray(0);
    
            Self {
                id,
                verts: ids[0],
                uvs: ids[1],
                light: 0,
                vertex_count: 0,
                kind: RenderKind::Triangles,
            }

        }

    }

    pub fn textured(verts: &[f32], uvs: &[f32]) -> Self {
        unsafe {

            if verts.len() / 3 != uvs.len() / 2 {
                panic!("UVs - VERTs count mismatch ({} != {})", uvs.len() / 2, verts.len() / 3);
            }
            
            let mut id = 0;

            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);

            assert!(id != 0);
            
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
                light: 0,
                vertex_count: verts.len() as i32,
                kind: RenderKind::Triangles,
            }

        }
    }

    pub fn textured_lit(verts: &[f32], uvs: &[f32], light: &[f32]) -> Self {
        unsafe {

            if verts.len() / 3 != uvs.len() / 2 {
                panic!("UVs - VERTs count mismatch ({} != {})", uvs.len() / 3, verts.len() / 2);
            }
            if verts.len() / 3 != light.len() {
                panic!("VERTs - LIGHTs count mismatch ({} != {})", verts.len() / 3, light.len());
            }
            
            let mut id = 0;

            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);

            assert!(id != 0);
            
            let mut verts_id = 0;
            gl::GenBuffers(1, &mut verts_id);
            configure_float_vbo(verts_id, 0, 3);
            transfer_to_array_buffer(verts_id, verts);

            let mut uvs_id = 0;
            gl::GenBuffers(1, &mut uvs_id);
            configure_float_vbo(uvs_id, 1, 2);
            transfer_to_array_buffer(uvs_id, uvs);

            let mut lights_id = 0;
            gl::GenBuffers(1, &mut lights_id);
            configure_float_vbo(lights_id, 2, 1);
            transfer_to_array_buffer(lights_id, light);

            gl::BindVertexArray(0);

            Self {
                id,
                verts: verts_id,
                uvs: uvs_id,
                light: lights_id,
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

    pub fn draw_18(&self, offset: i32) {
        let (kind, count) = match self.kind {
            RenderKind::Triangles => (gl::TRIANGLES, 3),
            RenderKind::Lines => (gl::LINES, 2),
        };
        unsafe {
            gl::DrawArrays(
                kind,
                18 * offset, // number of verticies
                18 // number of verticies
            );
        }
    }

    pub fn draw_6(&self, offset: i32) {
        let (kind, count) = match self.kind {
            RenderKind::Triangles => (gl::TRIANGLES, 3),
            RenderKind::Lines => (gl::LINES, 2),
        };
        unsafe {
            gl::DrawArrays(
                kind,
                6 * offset, // number of verticies
                6 // number of verticies
            );
        }
    }

    pub fn draw_n(&self, n: i32, offset: i32) {
        let (kind, count) = match self.kind {
            RenderKind::Triangles => (gl::TRIANGLES, 3),
            RenderKind::Lines => (gl::LINES, 2),
        };
        unsafe {
            gl::DrawArrays(
                kind,
                n * offset, // number of verticies
                n // number of verticies
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

