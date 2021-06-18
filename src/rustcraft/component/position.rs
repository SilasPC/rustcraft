
use crate::Program;
use crate::vao::VAO;
use cgmath::*;
use super::*;

#[derive(Clone)]
pub struct Position {
    pub pos: Vector3<f32>,
    pub rot: Euler<Deg<f32>>,
}


impl From<Vector3<f32>> for Position {
    fn from(pos: Vector3<f32>) -> Self {
        Self {
            pos,
            rot: Euler::new(Deg(0.),Deg(0.),Deg(0.)),
        }
    }
}

impl Position {

    pub fn add(&mut self, p: &Vector3<f32>) {
        self.pos += *p;
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let p = &mut self.rot.x.0;
        let y = &mut self.rot.y.0;
        *p += pitch;
        *p = p.max(-90.).min(90.);
        *y += yaw;
        *y %= 360.;
    }

    pub fn pitch(&self) -> Deg<f32> {self.rot.x}
    pub fn yaw(&self) -> Deg<f32> {self.rot.y}

    pub fn heading(&self) -> Vector3<f32> {
        let yaw = Rad::from(self.yaw()).0;
        let pitch = Rad::from(self.pitch()).0;
        Vector3 {
            x: yaw.sin()*pitch.cos(),
            y: -pitch.sin(),
            z: -yaw.cos()*pitch.cos(),
        }.normalize()
    }

    pub fn system_draw_bounding_boxes(data: &mut crate::Data, program: &Program, cube: &VAO) {
        for (ent, (pos, phys)) in data.ecs.query_mut::<(&Position, &Physics)>() {
            if ent == data.cam {continue}
            let size = phys.size();
            program.load_mat4(2, 
                &(Matrix4::from_translation(pos.pos)
                * Matrix4::from_nonuniform_scale(size.x, size.y, size.z))
            );
            
            cube.draw();

        }
    }

}
