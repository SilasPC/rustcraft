
use super::*;

#[derive(Clone)]
pub struct Physics {
    gravity: bool,
    force: Vector3<f32>,
    vel: Vector3<f32>,
    grounded: bool,
}

impl Physics {

    pub fn new() -> Self {
        let zero = Vector3 {x: 0., y: 0., z: 0.};
        Physics {
            gravity: true,
            grounded: true,
            force: zero,
            vel: zero,
        }
    }

    pub fn try_jump(&mut self) {
        if self.grounded {
            self.vel.y += 5.;
        }
    }

    pub fn is_grounded(&self) -> bool {self.grounded}

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
    pub fn update(&mut self, pos: &mut Position, delta: f32, world: &WorldData) -> bool {

        let old_y_vel = self.vel.y;

        self.vel += self.force;
        self.force = self.force.map(|_| 0.);

        if self.gravity {self.vel.y -= 10. * delta;}
        
        let mut new_pos = pos.pos.0;

        #[allow(non_snake_case)] {

            macro_rules! test {
                (x) => {{
                    let sE = 0.01;
                    let bE = pos.size.x - sE;
                    let byE = 0.99;
                    let offset = if self.vel.x > 0. {pos.size.x} else {0.};
                    world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+sE, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+byE, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+sE, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+offset, new_pos.y+byE, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true)
                }};
                (y) => {{
                    let sE = 0.01;
                    let bE = pos.size.x - sE;
                    let offset = if self.vel.y > 0. {pos.size.y} else {0.};
                    world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+offset, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+offset, new_pos.z+sE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+offset, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+offset, new_pos.z+bE))).map(|b| b.solid).unwrap_or(true)
                }};
                (z) => {{
                    let sE = 0.01;
                    let bE = pos.size.x - sE;
                    let byE = 0.99;
                    let offset = if self.vel.z > 0. {pos.size.z} else {0.};
                    world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+sE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+sE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+sE, new_pos.y+byE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true) ||
                    world.block_at(&WorldPos::from((new_pos.x+bE, new_pos.y+byE, new_pos.z+offset))).map(|b| b.solid).unwrap_or(true)
                }};
            }

            new_pos.x += self.vel.x * delta;
            if self.vel.x != 0. && test!(x) {
                if self.vel.x > 0. {
                    new_pos.x = new_pos.x.floor() + 1. - pos.size.x;
                } else {
                    new_pos.x = new_pos.x.ceil();
                }
				self.vel.x = 0.;
			}

            
            new_pos.y += self.vel.y * delta;
            if self.vel.y != 0. && test!(y) {
                if self.vel.y > 0. {
                    new_pos.y = new_pos.y.floor() + 1. - pos.size.y;
                } else {
                    new_pos.y = new_pos.y.ceil();
                }
				self.vel.y = 0.;
			}

            new_pos.z += self.vel.z * delta;
            if self.vel.z != 0. && test!(z) {
                if self.vel.z > 0. {
                    new_pos.z = new_pos.z.floor() + 1. - pos.size.z;
                } else {
                    new_pos.z = new_pos.z.ceil();
                }
				self.vel.z = 0.;
			}

        }

        pos.pos = new_pos.into();

        // ?
        let fric = 0.06; // if self.grounded {0.06} else {0.01};
        self.vel.x -= self.vel.x * fric; // hmm
        self.vel.z -= self.vel.z * fric;

        if self.vel.y != 0. {
            self.grounded = false;
        } else if old_y_vel == 0. && self.gravity {
            self.grounded = true;
        }
        
        return true;

        fn check_hit(w: &crate::rustcraft::world::WorldData, pos: &Vector3<f32>) -> bool {
            w.block_at(&pos.as_coord())
                .map(|b| b.solid)
                .unwrap_or(true)
        }
    }

    pub fn system_update(data: &mut crate::Data) {
        for (ent, (pos, phys)) in data.ecs.query_mut::<(&mut Position, &mut Physics)>() {
            if phys.update(pos, data.delta, &data.world) {
                data.ent_tree.update(ent, &pos.get_aabb());
            }
        }
    }

}