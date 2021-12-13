
use crate::prelude::*;

pub fn gen_detail<'cnt>(pos: ChunkPos, world: &mut WorldData<'cnt>, reg: &'cnt Content) {
    let (x,y,z) = pos.as_block().as_tuple();
    let dirt = reg.blocks.get("dirt").unwrap();
    let log = reg.blocks.get("log").unwrap();
    let leaves = reg.blocks.get("leaves").unwrap();
    for x in x..x+16 {
        for z in z..z+16 {
            'yloop: for y in y..y+16 {
                let below: BlockPos = (x,y-1,z).into();
                if world.blocks.block_at_any_state(&below).unwrap().id == "grass" {
                    let h = util::hash(&(x,z));
                    if h % 10 == 0 {
                        let h = 4 + h.rem_euclid(4) as i32; // 4..=7
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
                    break 'yloop;
                }
            }
        }    
    }
    let center = &mut world.blocks.chunks.get_mut(&pos).unwrap().chunk;
    center.chunk_state = ChunkState::Detailed;
    center.needs_refresh = true;
}
