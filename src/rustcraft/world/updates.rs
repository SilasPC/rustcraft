
use crate::component::*;
use cgmath::Vector3;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::coords::*;

#[derive(Default)]
pub struct Updates {
    pub current: usize,
    pub area: BinaryHeap<(Reverse<usize>, BlockPos)>,
    pub single: BinaryHeap<(Reverse<usize>, BlockPos)>,
}

impl Updates {
    pub fn add_area(&mut self, p: BlockPos) {
        self.area.push((Reverse(self.current+1),p));
    }
    pub fn add_single(&mut self, p: BlockPos) {
        self.single.push((Reverse(self.current+1),p));
    }
    pub fn add_area_immediate(&mut self, p: BlockPos) {
        self.area.push((Reverse(self.current),p));
    }
    pub fn add_single_immediate(&mut self, p: BlockPos) {
        self.single.push((Reverse(self.current),p));
    }
    pub fn update(data: &mut WorldData) {

        data.block_updates.current += 1;

        while let Some((Reverse(c),p)) = data.block_updates.area.pop() {
            if c > data.block_updates.current {
                data.block_updates.area.push((Reverse(c),p));
                break
            };
            for pos in Face::iter_all().map(|f| p.shifted(f)) {
                if let Some(on_update) = data.blocks.block_at(&pos)
                    .and_then(|b| b.behavior.as_ref())
                    .as_ref()
                    .and_then(|b| b.on_update)
                {
                    on_update(pos, data);
                }
            }
        }

        while let Some((Reverse(c),pos)) = data.block_updates.single.pop() {
            if c > data.block_updates.current {
                data.block_updates.single.push((Reverse(c),pos));
                break
            };
            if let Some(on_update) = data.blocks.block_at(&pos)
                .and_then(|b| b.behavior.as_ref())
                .as_ref()
                .and_then(|b| b.on_update)
            {
                on_update(pos, data);
            }
        }
        
    }
}
