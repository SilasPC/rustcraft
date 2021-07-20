
use rand::prelude::*;
use super::*;

pub struct WanderingAI {
    dir: Vector3<f32>,
    walk_time: f32,
}

impl WanderingAI {
    pub fn new() -> Self {
        Self { dir: Self::rnd(), walk_time: 0. }
    }
    pub fn update(&mut self, delta: f32) {
        self.walk_time += delta;
        if self.walk_time > 7. /*&& random::<f32>() < 0.005*/ {
            self.dir = Self::rnd();
            self.walk_time = 0.;
        }
        if self.walk_time > 5. {
            self.dir = Vector3 {x: 0., y: 0., z: 0.};
        }
    }
    pub fn heading(&self) -> &Vector3<f32> {&self.dir}
    fn rnd() -> Vector3<f32> {
        let rad: f32 = random::<f32>() * std::f32::consts::TAU;
        Vector3 {
            x: rad.sin(),
            y: 0.,
            z: rad.cos(),
        }
    }

    pub fn system_update(data: &mut crate::WorldData, delta: f32) {
        for (_ent, (phys, ai)) in data.entities.ecs.query_mut::<(&mut Physics, &mut WanderingAI)>() {
            ai.update(delta);
            phys.apply_force_continuous(delta * 40., ai.heading());
        }
    }

}
