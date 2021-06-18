
use crate::world::WorldData;
use crate::block::Block;
use cgmath::*;
use super::*;

#[derive(Clone)]
pub struct Physics {
    size: Vector3<f32>,
    gravity: bool,
    force: Vector3<f32>,
    vel: Vector3<f32>,
    grounded: bool,
}

impl Physics {

    pub fn new(size: Vector3<f32>) -> Self {
        let zero = Vector3 {x: 0., y: 0., z: 0.};
        Physics {
            size,
            gravity: true,
            grounded: true,
            force: zero,
            vel: zero,
        }
    }

    pub fn set_size(&mut self, size: &Vector3<f32>) {
        self.size = *size;
    }

    pub fn try_jump(&mut self) {
        if self.grounded {
            self.vel.y += 5.;
        }
    }

    pub fn is_grounded(&self) -> bool {self.grounded}
    pub fn size(&self) -> &Vector3<f32> {&self.size}

    pub fn apply_force(&mut self, f: &Vector3<f32>) {
        self.vel += *f;
    }

    /* pub fn apply_force_movement(&mut self, delta: f32, f: &Vector3<f32>) {
        let vel_p = self.vel.project_on(*f);
        let mag = vel_p.magnitude();
        if mag < 3. {
            self.force += f * delta;
        }
    } */

    pub fn apply_force_continuous(&mut self, delta: f32, f: &Vector3<f32>) {
        self.force += f * delta;
    }
    pub fn update(&mut self, pos: &mut Position, delta: f32, block_map: &Vec<Block>, world: &WorldData) {

        self.vel += self.force;
        self.force = self.force.map(|_| 0.);

        if self.gravity {self.vel.y -= 10. * delta;}
        
        let mut new_pos = pos.pos + self.vel * delta;

        if self.vel.x != 0. && check_hit(block_map, world, &Vector3 {
            x: new_pos.x,
            ..pos.pos
        }) {
            new_pos.x = pos.pos.x;
            self.vel.x = 0.;
        }
        if self.vel.y != 0. && check_hit(block_map, world, &Vector3 {
            y: new_pos.y,
            ..pos.pos
        }) {
            new_pos.y = pos.pos.y;
            if self.vel.y < 0. {
                self.grounded = true;
                new_pos.y = new_pos.y.floor();
            } 
            self.vel.y = 0.;
        } else {
            self.grounded = false;
        }
        if self.vel.x != 0. && check_hit(block_map, world, &Vector3 {
            z: new_pos.z,
            ..pos.pos
        }) {
            new_pos.z = pos.pos.z;
            self.vel.z = 0.;
        }

        pos.pos = new_pos;

        // ?
        let fric = 0.06; // if self.grounded {0.06} else {0.01};
        self.vel.x -= self.vel.x * fric; // hmm
        self.vel.z -= self.vel.z * fric;

        if self.vel.y != 0. {
            self.grounded = false;
        }

        fn check_hit(block_map: &Vec<Block>, world: &WorldData, pos: &Vector3<f32>) -> bool {
            let cc = (pos / 16.).map(|c| c.floor() as isize);
            let mut sc = (pos % 16.).map(|c| c.floor() as isize);
            if cc.x < 0 || pos.y < 0. || cc.z < 0 {
                return false;
            }
            let chunk = &world.chunks[cc.x as usize][cc.z as usize];
            if sc.x < 0 {sc.x += 16}
            if sc.z < 0 {sc.z += 16}
            let id = chunk.data[sc.x as usize][sc.y as usize][sc.z as usize];
            block_map[id].solid
        }
    }

    pub fn system_update(data: &mut crate::Data) {
        for (_ent, (pos, phys)) in data.ecs.query_mut::<(&mut Position, &mut Physics)>() {
            phys.update(pos, data.delta, &data.block_map, &data.world);
        }
    }

}