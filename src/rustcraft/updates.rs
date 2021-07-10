
use crate::component::*;
use cgmath::Vector3;
use std::cmp::Reverse;
use crate::Data;
use std::collections::BinaryHeap;
use crate::coords::*;

#[derive(Default)]
pub struct Updates {
    pub current: usize,
    pub area: BinaryHeap<(Reverse<usize>, i32, i32, i32)>,
    pub single: BinaryHeap<(Reverse<usize>, i32, i32, i32)>,
}

impl Updates {
    pub fn add_area(&mut self, v: Vector3<f32>) {
        self.push_area(
            v.x.floor() as i32,
            v.y.floor() as i32,
            v.z.floor() as i32
        )
    }
    pub fn add_single(&mut self, v: Vector3<f32>) {
        self.push_single(
            v.x.floor() as i32,
            v.y.floor() as i32,
            v.z.floor() as i32
        )
    }
    pub fn push_area(&mut self, x: i32, y: i32, z: i32) {
        self.area.push((Reverse(self.current+1),x,y,z))
    }
    pub fn push_single(&mut self, x: i32, y: i32, z: i32) {
        self.single.push((Reverse(self.current+1),x,y,z))
    }
    pub fn update(&mut self, data: &mut Data) {
        macro_rules! update {
            ($x:expr,$y:expr,$z:expr) => {
                let here: WorldPos<i32> = ($x,$y,$z).into();
                if let Some(block) = data.world.block_at(&here) {
                    let block = block.clone();
                    if block.has_gravity {
                        let below: WorldPos<i32> = ($x,$y-1,$z).into();
                        if let Some(below) = data.world.block_at(&below) {
                            let below = below.as_ref();
                            if !below.solid {
                                data.world.set_block_at(&here, &data.registry[0]);
                                let fall_size = Vector3 {
                                    x: 1.,
                                    y: 1.,
                                    z: 1.,
                                };
                                let pos_comp = Position::from(here.as_pos_f32());
                                let phys = Physics::new(fall_size);
                                let aabb = phys.get_aabb(&pos_comp);
                                let falling_block = data.ecs.spawn((
                                    pos_comp, phys, FallingBlock::of(block)
                                ));
                                data.ent_tree.insert(falling_block, (), &aabb);
                                self.area.push((Reverse(self.current+1),$x,$y,$z));
                            }
                        }
                    }
                }
            };
        }
        while let Some((Reverse(c),x,y,z)) = self.area.pop() {
            if c > self.current {break};
            update!(x+1,y,z);
            update!(x-1,y,z);
            update!(x,y+1,z);
            update!(x,y-1,z);
            update!(x,y,z+1);
            update!(x,y,z-1);
        }
        while let Some((Reverse(c),x,y,z)) = self.single.pop() {
            if c > self.current {break};
            update!(x,y,z);
        }
        self.current += 1;
    }
}
