
use crate::prelude::*;

pub trait TerrainGenerator {
    //fn density(&self, x: isize, y: isize, z: isize) -> f64;
    //fn is_cave(&self, x: isize, y: isize, z: isize) -> bool;
    //fn palette(&self, x: isize, z: isize) -> &[&'static str; 3];
    fn gen_terrain<'cnt>(&self, chunk: &mut Chunk<'cnt>, reg: &'cnt Content);
    fn get_detailer(&self) -> Box<dyn ChunkDetailer>;
}

pub trait ChunkDetailer {
    fn detail(&mut self) -> bool;
}

#[derive(Debug)]
pub struct IslandGenerator {
    pub noise: crate::perlin::PerlinNoise,
    pub noise_basic: crate::perlin::PerlinNoise,
    pub palettes: [[&'static str; 3]; 2],
}

impl TerrainGenerator for IslandGenerator {
    fn gen_terrain<'cnt>(&self, chunk: &mut Chunk<'cnt>, reg: &'cnt Content) {

        for x in 0..16 {
            for z in 0..16 {
                let palette = self.palette(x,z);
                for y in 0..16 {
                    let pos = chunk.pos.map(|x| x as isize);

                    let ax = 16 * pos.x + x;
                    let ay = 16 * pos.y + y;
                    let az = 16 * pos.z + z;

                    if self.is_cave(ax, ay, az) {continue}

                    let d = self.density(ax,ay,az);
                    let da = self.density(ax,ay+1,az);
                    
                    chunk.data[x as usize][y as usize][z as usize] = 
                    &reg.blocks.get(
                        if d > 0.56 {
                            palette[0]
                        } else if d > 0.52 {
                            if da > 0.52 {
                                palette[1]
                            } else if ay >= 20 {
                                palette[2]
                            } else {
                                palette[1]
                            }
                        } else if ay > 20 {
                            "air"
                        } else {
                            "water"
                        }
                    ).unwrap();
                    /* if db > 0.52 && db < 0.56 && d < 0.51 && !(cb > 0.57) {
                        let t = noise.get2d([x as f64 / 1.5, z as f64 / 1.5]);
                        if t > 0.52 {
                            self.data[x as usize][y as usize][z as usize] = 4;
                        }
                    } */
                }
            }
        }
    }
    fn get_detailer(&self) -> Box<dyn ChunkDetailer> {box IslandDetailer}
}

impl IslandGenerator {

    pub fn new_dyn(seed: &str) -> Box<dyn TerrainGenerator> {
        let noise = crate::perlin::PerlinNoise::new(seed, 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed, 1, 1.);
        let palettes = [
            ["stone","dirt","grass"],
            ["stone","sand","sand"]
        ];
        box IslandGenerator {
            noise,
            noise_basic,
            palettes
        } as Box<dyn TerrainGenerator>
    }

    fn is_cave(&self, x: isize, y: isize, z: isize) -> bool {
        let xf = x.abs() as f64 / 13.;
        let yf = y.abs() as f64 / 13.;
        let zf = z.abs() as f64 / 13.;
        let c = self.noise_basic.get3d([xf, yf, zf]);
        c > 0.75
    }

    fn density(&self, x: isize, y: isize, z: isize) -> f64 {
    
        // ! GOOD RANDOM POINT-LIKE DISTRIBUTION:
        /* use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash,Hasher};
    
        let mut h = DefaultHasher::new();
        (x,y,z).hash(&mut h);
        if h.finish() % 4000 == 0 {
            return 1.0;
        } */
    
        let xf = x.abs() as f64 / 70.;
        let yf = y.abs() as f64 / 40.;
        let zf = z.abs() as f64 / 70.;
        let d = self.noise.get3d([xf, yf, zf]);
        let d = d * 0.8 / yf;
    
        let r = (xf*xf+zf*zf).sqrt().max(1.);
        d / r
    }

    fn palette(&self, x: isize, z: isize) -> &[&'static str; 3] {
        let xf = (x.abs() as f64 + 0.5) / 10.;
        let zf = (z.abs() as f64 + 0.5) / 10.;
        let n = self.noise_basic.get2d([xf, zf]);
        
        &self.palettes[if n < 0.5 {0} else {1}]
    } 

}

pub struct IslandDetailer;
impl ChunkDetailer for IslandDetailer {
    fn detail(&mut self) -> bool {false}
}
/* pub fn gen_detail<'cnt>(pos: ChunkPos, world: &mut WorldData<'cnt>, reg: &'cnt Content) {
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
} */