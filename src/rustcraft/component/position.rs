
use crate::lines::LineProgram;
use crate::Program;
use super::*;

#[derive(Clone)]
pub struct Position {
    pub size: Vector3<f32>,
    pub pos: WorldPos<f32>,
    pub rot: Euler<Deg<f32>>,
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("pos", &(self.pos.x as i32, self.pos.y as i32, self.pos.z as i32))
            .field("rot", &(self.rot.x, self.rot.y))
            .field("size", &(self.size.x as i32, self.size.y as i32, self.size.z as i32))
            .finish()
    }
}

impl Position {

    pub fn new(pos: WorldPos<f32>, size: Vector3<f32>) -> Self {
        Self {
            size,
            pos,
            rot: Euler::new(Deg(0.),Deg(0.),Deg(0.))
        }
    }

    pub fn get_aabb(&self) -> crate::util::AABB {
        const E: f32 = 0.;// 0.01; // 10. * std::f32::EPSILON;
        ((
            self.pos.x+E,
            self.pos.y+E,
            self.pos.z+E,
        ),(
            self.pos.x+self.size.x-E,
            self.pos.y+self.size.y-E,
            self.pos.z+self.size.z-E,
        )).into()
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

    pub fn system_draw_bounding_boxes(data: &mut crate::Data, program: &mut LineProgram) {
        program.enable();
        program.bind();
        program.load_color(&(0.8,0.8,0.8,1.0).into());
        for (ent, pos) in data.ecs.query_mut::<&Position>() {
            if ent == data.cam {continue};
            program.load_transform(&(Matrix4::from_translation(pos.pos.0)
            * Matrix4::from_nonuniform_scale(pos.size.x, pos.size.y, pos.size.z)));
            program.draw();
        }
    }

}
