
use crate::prelude::*;
use world::VoxelData;

#[derive(Clone, Copy, Default)]
pub struct Light(u8);
impl Light {
    #[inline(always)]
    pub fn block(&self) -> u8 {
        self.0 & 0xf
    }
    #[inline(always)]
    pub fn set_block(&mut self, val: u8) {
        self.0 &= 0xf0;
        self.0 |= val;
    }
    #[inline(always)]
    pub fn sky(&self) -> u8 {
        self.0 >> 4
    }
    #[inline(always)]
    pub fn set_sky(&mut self, val: u8) {
        self.0 &= 0xf;
        self.0 |= val << 4;
    }
}

pub fn calc_light(pos: ChunkPos, world: &mut VoxelData) {
    let Vector3 {x, y, z} = pos.into();
    let (mut removal_queue, mut queue) = world.chunk_at_mut(pos).map(|c| (
        std::mem::take(&mut c.light_remove_updates),
        std::mem::take(&mut c.light_updates)
    )).unwrap();

    // removal loop
    while let Some((pos,old_light)) = removal_queue.pop_front() {
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {
                let pos = Into::<BlockPos>::into(($x, $y, $z));
                let light = world.light_at_mut(&pos);
                let new_light = light.block();
                if new_light != 0 && new_light < old_light {
                    light.set_block(0);
                    removal_queue.push_back((($x,$y,$z).into(),new_light));
                } else if new_light >= old_light {
                    queue.push_back(($x,$y,$z).into());
                }
            };
        }
        let (x,y,z) = pos.as_tuple();
        prop!(x+1,y,z);
        prop!(x-1,y,z);
        prop!(x,y+1,z);
        prop!(x,y-1,z);
        prop!(x,y,z+1);
        prop!(x,y,z-1);
    }
    
    // propagation loop
    while let Some(pos) = queue.pop_front() {
        let pos_light = world.light_at_mut(&pos).block();
        if pos_light == 0 {continue}
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {
                let prop_pos = Into::<BlockPos>::into(($x, $y, $z));
                if world.block_at(&prop_pos).unwrap().transparent {
                    let l = world.light_at_mut(&prop_pos);
                    if l.block() + 2 <= pos_light {
                        l.set_block(pos_light - 1);
                        if pos_light > 2 {
                            queue.push_back(($x, $y, $z).into());
                        }
                    }
                }
            };
        }
        let (x,y,z) = pos.as_tuple();
        prop!(x+1,y,z);
        prop!(x-1,y,z);
        prop!(x,y+1,z);
        prop!(x,y-1,z);
        prop!(x,y,z+1);
        prop!(x,y,z-1);
    }
}
