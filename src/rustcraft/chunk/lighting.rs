
use crate::prelude::*;
use world::VoxelData;

#[derive(Debug, Default)]
pub struct LightUpdates {
    block: VecDeque<BlockPos>,
    block_rem: VecDeque<(BlockPos, u8)>,
    sky: VecDeque<BlockPos>,
    sky_rem: VecDeque<(BlockPos, u8)>,
}

impl LightUpdates {
    pub fn reg_block(&mut self, pos: &BlockPos, prev: u8, new: u8) {
        // println!("block light {} => {} @ {:?}",prev,new,*pos);
        if new < prev {
            self.block_rem.push_back((*pos, prev));
        } else {
            self.block.push_back(*pos);
        }
        // just always do it for now
        self.sky_rem.push_back((*pos, 0)); // ! ?
    }
}

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

pub fn calc_light<'cnt: 'b, 'b>(pos: ChunkPos, world: &'b mut VoxelData<'cnt>) {
    let Vector3 {x, y, z} = pos.into();
    let (
        mut block_removal,
        mut block_prop,
        mut sky_removal,
        mut sky_prop,
    ) = world.chunk_at_mut(pos).map(|c| (
        std::mem::take(&mut c.light_updates.block_rem),
        std::mem::take(&mut c.light_updates.block),
        std::mem::take(&mut c.light_updates.sky_rem),
        std::mem::take(&mut c.light_updates.sky),
    )).unwrap();

    // block removal loop
    while let Some((pos,old_light)) = block_removal.pop_front() {
        for pos in Face::iter_all().map(|f| pos.shifted(f)) {
            let light = world.light_at_mut(&pos);
            let new_light = light.block();
            if new_light != 0 && new_light < old_light {
                light.set_block(0);
                block_removal.push_back((pos, new_light));
            } else if new_light >= old_light {
                block_prop.push_back(pos);
            }
        }
    }

    // sky removal loop
    while let Some((pos,old_light)) = sky_removal.pop_front() {
        for (down, pos) in Face::iter_all().map(|f| (f == Face::YNeg, pos.shifted(f))) {
            let light = world.light_at_mut(&pos);
            let new_light = light.sky();
            if new_light != 0 && new_light < old_light || old_light == 15 && down {
                light.set_sky(0);
                sky_removal.push_back((pos, new_light));
            } else if new_light >= old_light {
                sky_prop.push_back(pos);
            }
        }
    }
    
    // block propagation loop
    while let Some(pos) = block_prop.pop_front() {
        let pos_light = world.light_at_mut(&pos).block();
        if pos_light == 0 {continue}
        for prop_pos in Face::iter_all().map(|f| pos.shifted(f)) {
            if world.block_at(&prop_pos).unwrap().transparent {
                let l = world.light_at_mut(&prop_pos);
                if l.block() + 2 <= pos_light {
                    l.set_block(pos_light - 1);
                    if pos_light > 2 {
                        block_prop.push_back(prop_pos);
                    }
                }
            }
        }
    }

    // sky propagation loop
    while let Some(pos) = sky_prop.pop_front() {
        let pos_light = world.light_at_mut(&pos).sky();
        if pos_light == 0 {continue}
        for (down, prop_pos) in Face::iter_all().map(|f| (f == Face::YNeg, pos.shifted(f))) {
            if world.block_at(&prop_pos).unwrap().transparent {
                let l = world.light_at_mut(&prop_pos);
                if l.sky() + 2 <= pos_light {
                    let new_light = if down { pos_light } else { pos_light - 1 };
                    l.set_sky(new_light);
                    if pos_light > 2 {
                        sky_prop.push_back(prop_pos);
                    }
                }
            }
        }
    }

}
