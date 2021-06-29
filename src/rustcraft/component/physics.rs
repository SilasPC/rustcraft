
use crate::game_loop::block_at;
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
    no_clip: bool,
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
            no_clip: false
        }
    }

    pub fn set_flying(&mut self, flying: bool) {
        self.no_clip = flying;
        self.gravity = !flying;
    }

    pub fn set_size(&mut self, size: &Vector3<f32>) {
        self.size = *size;
    }

    pub fn try_jump(&mut self) {
        if self.grounded {
            self.vel.y += 5.;
        }
    }

    pub fn get_aabb(&self, pos: &Position) -> crate::util::AABB {
        const E: f32 = 0.;// 0.01; // 10. * std::f32::EPSILON;
        ((
            pos.pos.x+E,
            pos.pos.y+E,
            pos.pos.z+E,
        ),(
            pos.pos.x+self.size.x-E,
            pos.pos.y+self.size.y-E,
            pos.pos.z+self.size.z-E,
        )).into()
    }

    pub fn is_grounded(&self) -> bool {self.grounded}
    pub fn size(&self) -> &Vector3<f32> {&self.size}

    pub fn apply_force_once(&mut self, f: &Vector3<f32>) {
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
    /// returns true if position was updated
    pub fn update(&mut self, pos: &mut Position, delta: f32, block_map: &Vec<std::sync::Arc<Block>>, world: &WorldData) -> bool {

        self.vel += self.force;
        self.force = self.force.map(|_| 0.);

        if self.gravity {self.vel.y -= 10. * delta;}
        
        let mut new_pos = pos.pos + self.vel * delta;

        if !self.no_clip {
            macro_rules! test_it {
                ($x:expr, $y:expr, $z:expr) => {
                    if self.vel.x != 0. && check_hit(block_map, world, &Vector3 {
                        x: new_pos.x + $x * self.size.x,
                        ..pos.pos
                    }) {
                        new_pos.x = pos.pos.x;
                        self.vel.x = 0.;
                    }
                    if self.vel.y != 0. && check_hit(block_map, world, &Vector3 {
                        y: new_pos.y + $y * self.size.y,
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
                        z: new_pos.z + $z * self.size.z,
                        ..pos.pos
                    }) {
                        new_pos.z = pos.pos.z;
                        self.vel.z = 0.;
                    }
                };
            }
            test_it!(0.,0.,0.);/* 
            test_it!(1.,0.,0.);
            test_it!(0.,1.,0.);
            test_it!(0.,0.,1.);
            test_it!(1.,1.,0.);
            test_it!(1.,0.,1.);
            test_it!(0.,1.,1.);
            test_it!(1.,1.,1.); */
        }

        pos.pos = new_pos;

        // ?
        let fric = 0.06; // if self.grounded {0.06} else {0.01};
        self.vel.x -= self.vel.x * fric; // hmm
        self.vel.z -= self.vel.z * fric;

        if self.vel.y != 0. {
            self.grounded = false;
        }
        
        return true;

        fn check_hit(block_map: &Vec<std::sync::Arc<crate::rustcraft::block::Block>>, w: &crate::rustcraft::world::WorldData, pos: &Vector3<f32>) -> bool {
            block_at(w, pos)
                .map(|id| block_map[id].solid)
                .unwrap_or(true)
        }
    }

    pub fn system_update(data: &mut crate::Data) {
        for (ent, (pos, phys)) in data.ecs.query_mut::<(&mut Position, &mut Physics)>() {
            if phys.update(pos, data.delta, &data.block_map, &data.world) {
                data.ent_tree.update(ent, &phys.get_aabb(&pos));
            }
        }
    }

}