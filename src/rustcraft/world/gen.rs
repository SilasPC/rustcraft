
use crate::prelude::*;

pub fn gen_detail(pos: ChunkPos, world: &mut WorldData, reg: &Registry) {
    let (x,y,z) = pos.as_block().as_tuple();
    let dirt = reg.get("dirt").as_block().unwrap();
    let log = reg.get("log").as_block().unwrap();
    let leaves = reg.get("leaves").as_block().unwrap();
    for x in x..x+16 {
        for z in z..z+16 {
            for y in y..y+16 {
                let below: BlockPos = (x,y-1,z).into();
                if world.blocks.block_at_any_state(&below).unwrap().id.as_ref() == "grass" {
                    let nx = x as f64 / 1.3;
                    let nz = z as f64 / 1.3;
                    let n = world.noise.noise_basic.get2d([nx,nz]);
                    const CUTOFF: f64 = 0.59;
                    if n > CUTOFF {
                        const INV: f64 = 1./1.3;
                        if world.noise.noise_basic.get2d([nx-INV,nz]) <= CUTOFF &&
                            world.noise.noise_basic.get2d([nx,nz-INV]) <= CUTOFF {
                                let h = 3 + (2. * world.noise.noise_basic.get2d([nx,nz])) as i32;
                                world.blocks.set_block_at_any_state(&below, dirt);
                                for y in y..=y+h {
                                    let here: BlockPos = (x,y,z).into();
                                    world.blocks.set_block_at_any_state(&here, log);
                                }
                                for dx in -2..=2i32 {
                                    for dz in -2..=2i32 {
                                        for dy in 0..=3i32 {
                                            if dx.abs()+dz.abs()+dy.abs() > 5 {continue}
                                            let y = y + h-3 + dy;
                                            let here: BlockPos = (x+dx,y+4,z+dz).into();
                                            world.blocks.replace_at_any_state(&here, leaves);
                                        }
                                    }
                                }
                                let here: BlockPos = (x,y+h,z).into();
                                world.blocks.replace_at_any_state(&here, leaves);
                            }
                    }
                    break;
                }
            }
        }    
    }
    let center = world.blocks.chunks.get_mut(&pos).unwrap();
    center.chunk_state = ChunkState::Detailed;
    center.needs_refresh = true;
}
