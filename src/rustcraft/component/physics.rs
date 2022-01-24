
use crate::world::VoxelData;
use super::*;

#[derive(Clone, serde::Deserialize)]
pub struct Physics {
    #[serde(default = "util::bool_true")]
    pub gravity: bool,
    #[serde(skip, default = "util::vec_f32_zero")]
    force: Vector3<f32>,
    #[serde(skip, default = "util::vec_f32_zero")]
    vel: Vector3<f32>,
    #[serde(skip)]
    grounded: bool,
    #[serde(skip)]
    edge_stop: bool,
    #[serde(skip)]
    pub freecam: bool,
}

impl Physics {

    pub fn new() -> Self {
        let zero = Vector3 {x: 0., y: 0., z: 0.};
        Physics {
            gravity: true,
            grounded: true,
            force: zero,
            vel: zero,
            edge_stop: false,
            freecam: false
        }
    }

    pub fn try_jump(&mut self, delta: f32) {
        if self.grounded {
            self.vel.y += 5.;
        } else if self.freecam {
            let dir = if self.edge_stop {Face::YNeg} else {Face::YPos};
            self.apply_force_continuous(delta, &dir.to_dir());
        }
    }

    pub fn is_grounded(&self) -> bool {self.grounded}

    pub fn apply_force_once(&mut self, f: &Vector3<f32>) {
        self.vel += *f;
    }

    pub fn set_edge_stop(&mut self, val: bool) {
        self.edge_stop = val;
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
    pub fn update(&mut self, pos: &mut Position, delta: f32, world: &VoxelData) -> bool {

        let was_grounded = self.grounded;
        let old_y_vel = self.vel.y;

        self.vel += self.force;
        self.force = self.force.map(|_| 0.);

        if self.gravity {self.vel.y -= 10. * delta;}
        
        let mut new_pos = pos.pos.0;
        let mut new_vel = self.vel;
        let old_vel = self.vel;

        #[allow(non_snake_case)] {

            macro_rules! test {
                (x) => {{
                    if self.freecam {false} else {
                        let sE = 0.01;
                        let bE = pos.size.x - sE;
                        let byE = 0.99;
                        let offset = if new_vel.x > 0. {pos.size.x} else {0.};
                        world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+sE, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+byE, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+sE, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+byE, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true)
                    }
                }};
                (y) => {{
                    if self.freecam {false} else {
                        let sE = 0.01;
                        let bE = pos.size.x - sE;
                        let offset = if new_vel.y > 0. {pos.size.y} else {0.};
                        world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+offset, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+offset, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+offset, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+offset, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true)
                    }
                }};
                (z) => {{
                    if self.freecam {false} else {
                        let sE = 0.01;
                        let bE = pos.size.x - sE;
                        let byE = 0.99;
                        let offset = if new_vel.z > 0. {pos.size.z} else {0.};
                        world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+sE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+sE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+byE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                        world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+byE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true)
                    }
                }};
            }

            new_pos.x += new_vel.x * delta;
            if new_vel.x != 0. && test!(x) {
                if new_vel.x > 0. {
                    new_pos.x = new_pos.x.floor() + 1. - pos.size.x;
                } else {
                    new_pos.x = new_pos.x.ceil();
                }
				new_vel.x = 0.;
			}

            new_pos.z += new_vel.z * delta;
            if new_vel.z != 0. && test!(z) {
                if new_vel.z > 0. {
                    new_pos.z = new_pos.z.floor() + 1. - pos.size.z;
                } else {
                    new_pos.z = new_pos.z.ceil();
                }
				new_vel.z = 0.;
			}
            
            new_pos.y += new_vel.y * delta;
            if new_vel.y != 0. && test!(y) {
                if new_vel.y > 0. {
                    new_pos.y = new_pos.y.floor() + 1. - pos.size.y;
                } else {
                    new_pos.y = new_pos.y.ceil();
                }
				new_vel.y = 0.;
			} else if was_grounded && new_vel.y <= 0. && self.edge_stop {
                // not grounded, try cancel x
                
                new_vel.y = 0.;
                new_pos = pos.pos.0;
                compile_warning!(edge stop is a bit funky);
                /* new_pos.x = pos.pos.0.x;
                if !test!(y) {
                    // still not grounded, cancel x not enough, try cancel z instead
                    new_pos.x += old_vel.x * delta;
                    new_pos.z = pos.pos.0.z;
                    if !test!(y) {
                        // still not grounded, must cancel both
                        new_pos.x = pos.pos.0.x;
                    }
                } */
            }

        }

        self.vel = new_vel;
        pos.pos = new_pos.into();

        // ?
        let fric = 0.06; // if self.grounded {0.06} else {0.01};
        self.vel.x -= self.vel.x * fric; // hmm
        self.vel.z -= self.vel.z * fric;

        if self.vel.y != 0. {
            self.grounded = false;
        } else if self.vel.y == 0. && self.gravity && !self.freecam && !was_grounded {
            self.grounded = true;
        }
        
        return true;

        fn check_hit(w: &crate::rustcraft::world::VoxelData, pos: &Vector3<f32>) -> bool {
            w.block_at(&pos.as_coord())
                .map(|b| b.solid)
                .unwrap_or(true)
        }
    }

    pub fn system_update(data: &mut WorldData, delta: f32) {
        for (ent, (pos, phys)) in data.entities.ecs.query_mut::<(&mut Position, &mut Physics)>() {
            if phys.update(pos, delta, &data.blocks) {
                data.entities.tree.update(ent, &pos.get_aabb());
            }
        }
    }

}