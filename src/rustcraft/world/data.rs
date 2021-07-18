
#[macro_use]
use crate::prelude::*;
use super::TerrainGen;

#[derive(Debug)]
pub enum Loading {
    Filling(i32, ChunkPos),
    Detailing(i32, ChunkPos),
    Meshing(i32, ChunkPos),
}

#[derive(Debug)]
pub struct WorldData {
    pub seed: String,
    pub air: Block,
    pub chunks: HashMap<ChunkPos, Box<Chunk>>,
    pub noise: TerrainGen,
    pub ticks: u64,
    pub to_load: VecDeque<Loading>,
    pub changed_chunks: HashSet<ChunkPos>,
    pub to_update: Vec<BlockPos>,
}

impl WorldData {
    
    pub fn new(seed: &str, air: Block) -> Self {
        let noise = crate::perlin::PerlinNoise::new(seed.to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed.to_owned(), 1, 1.);
        let palettes = [
            ["stone","dirt","grass"],
            ["stone","sand","sand"]
        ];
        let noise = TerrainGen {
            noise,
            noise_basic,
            palettes
        };
        WorldData { to_update: vec![], changed_chunks: HashSet::new(), to_load: VecDeque::new(), seed: seed.to_owned(), chunks: HashMap::new(), noise, air, ticks: 0 }
    }

    pub fn block_at(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunk_at(pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn block_at_any_state(&self, pos: &impl Coord) -> Option<&Block> {
        self.chunks.get(&pos.as_chunk()).map(|c| c.block_at(pos))
    }
    pub fn set_block_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunk_at_mut(pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.changed_chunks.insert(pos.as_chunk());
        }
        success
    }
    pub fn set_block_at_any_state(&mut self, pos: &impl Coord, block: &Block) -> bool {
        let success = self.chunks.get_mut(&pos.as_chunk())
            .map(|c| c.set_at(pos, block))
            .unwrap_or(false);
        if success {
            self.changed_chunks.insert(pos.as_chunk());
        }
        success
    }
    pub fn replace_at(&mut self, pos: &impl Coord, block: &Block) -> bool {
        if let Some(c) = self.chunk_at_mut(pos.as_chunk()) {
            if c.block_at(pos).replacable {
                c.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn replace_at_any_state(&mut self, pos: &impl Coord, block: &Block) -> bool {
        if let Some(c) = self.chunks.get_mut(&pos.as_chunk()) {
            if c.block_at(pos).replacable {
                c.set_at(pos, block)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn light_at(&self, pos: &impl Coord) -> &Light {
        self.chunk_at(pos.as_chunk()).unwrap().light_at(pos)
    }
    pub fn light_at_mut(&mut self, pos: &impl Coord) -> &mut Light {
        self.chunk_at_mut(pos.as_chunk()).unwrap().light_at_mut(pos)
    }

    pub fn chunk_at(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).filter(|c| c.chunk_state >= ChunkState::Detailed).map(Box::as_ref)
    }
    pub fn chunk_at_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos).filter(|c| c.chunk_state >= ChunkState::Detailed).map(Box::as_mut)
    }

    pub fn smooth_light_level(&self) -> f32 {
        (self.time_of_day() * std::f32::consts::TAU).sin() + 0.5
    } 
    pub fn time_of_day(&self) -> f32 {
        (self.ticks as f32 / 200.) % 1.
    }

    pub fn refresh(&mut self, reg: &Registry) {
        let mut cc = std::mem::take(&mut self.changed_chunks);
        let mut meshed = HashSet::new();
        cc.retain(|x| self.chunk_at(*x).map(Chunk::renderable).unwrap_or(false));
        // println!("refresh {:?}",cc);
        for cp in &cc {
            calc_light(*cp, self);
        }
        // println!("{}",cc.len());
        for cp in cc {
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let p = Vector3 {
                            x: x + cp.x,
                            y: y + cp.y,
                            z: z + cp.z,
                        };
                        // ! make_mesh is slow, old version faster
                        // ! need to make another solution here
                        // ! need to make a hybrid version as well
                        if !self.chunks.get(&p.into()).unwrap().renderable() || meshed.contains(&p) {continue}
                        let (m1,m2) = (meshing::make_mesh(p.into(), self, reg));
                        let c = self.chunks.get_mut(&p.into()).unwrap();
                        {
                            let m = c.mesh.as_mut().unwrap();
                            m.0.update_lit(&m1.0, &m1.1, &m1.2);
                            m.1.update_lit(&m2.0, &m2.1, &m2.2);
                        }
                        c.needs_refresh = false;
                        c.chunk_state = ChunkState::Rendered;
                        meshed.insert(p);
                    }
                }
            }
        }
    }

    pub fn load_around(&mut self, pos: &impl Coord) {
        let (x,y,z) = pos.as_chunk().as_tuple();
        let p = (x-5,y-5,z-5).into();
        println!("Filling from {:?}...", p);
        self.to_load.push_back(Loading::Filling(0, p))
    }

    pub fn load(&mut self, reg: &Registry, max_work: usize) {
        const DIAMETER: i32 = 10;
        if let Some(mut loading) = self.to_load.pop_front() {
            let mut work = 0;
            match loading {
                Loading::Filling(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        if let Some(c) = self.chunks.get_mut(&p) {
                            if c.chunk_state == ChunkState::Empty {
                                c.gen_terrain(&self.noise, reg);
                                work += 1;
                            }
                        } else {
                            let mut c = Box::new(Chunk::new(p, self.air.clone()));
                            c.gen_terrain(&self.noise, reg);
                            self.chunks.insert(p, c);
                            work += 1;
                        }
                        // println!("generated for {:?}",p);
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        let pos = pos + Vector3{x:1,y:1,z:1}.into();
                        println!("Detailing from {:?}...",pos);
                        loading = Loading::Detailing(0, pos);
                    }
                },
                Loading::Detailing(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER-2;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        if self.chunks.get(&p).unwrap().chunk_state == ChunkState::Filled {
                            gen_detail(p, self, reg);
                            work += 1;
                        }
                        // println!("detailed for {:?}",p);
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        let pos = pos + Vector3{x:1,y:1,z:1}.into();
                        println!("Meshing from {:?}...", pos);
                        loading = Loading::Meshing(0, pos);
                    }
                },
                Loading::Meshing(ref mut i, pos) => {
                    let (x,y,z) = pos.as_tuple();
                    const RAD: i32 = DIAMETER-4;
                    while *i < RAD*RAD*RAD && work < max_work {
                        let p = (
                            x + *i / (RAD*RAD),
                            y + (*i / RAD) % RAD,
                            z + *i % RAD
                        ).into();
                        {
                            let (m1, m2) = meshing::make_mesh(p, self, reg);
                            let c = self.chunks.get_mut(&p).unwrap();
                            if let Some(mesh) = &mut c.mesh {
                                mesh.0.update_lit(&m1.0, &m1.1, &m1.2);
                                mesh.1.update_lit(&m2.0, &m2.1, &m2.2);
                            } else {
                                c.mesh = Some((
                                    VAO::textured_lit(&m1.0, &m1.1, &m1.2),
                                    VAO::textured_lit(&m2.0, &m2.1, &m2.2)
                                ));
                            }
                            c.needs_refresh = false;
                            c.chunk_state = ChunkState::Rendered;
                        }
                        // self.chunks.get_mut(&p).unwrap().refresh(reg);
                        // println!("detailed for {:?}",p);
                        work += 1;
                        *i += 1;
                    }
                    if *i == RAD*RAD*RAD {
                        println!("Done loading");
                        return
                    }
                },
            }
            self.to_load.push_front(loading);
        }
    }

    pub fn raycast(&self, mut pos: WorldPos, heading: &Vector3<f32>, max_dist: f32) -> Option<RayCastHit> {
        
        let mut dist = 0.;
        while dist < max_dist && !self.check_hit(&pos) {
            dist += 0.1;
            pos.0 += 0.1 * heading;
        }

        if dist < max_dist {
            let hit = pos;
            let mut prev = pos;
            prev.0 -= 0.1 * heading;
            let mut prev = prev.as_block();
            compile_warning!(not always adjacent);
            /* if !prev.adjacent_to(&hit.as_block()) {
                dbg!((prev,hit));
            } */

            return Some(RayCastHit {
                hit,
                prev
            });
        } else {
            return None
        }

    }

    fn check_hit(&self, pos: &impl Coord) -> bool {
        self.block_at(pos)
            .map(|b| b.solid)
            .unwrap_or(false)
    }


}

#[derive(Copy, Clone, Debug)]
pub struct RayCastHit {
    pub hit: WorldPos,
    pub prev: BlockPos,
}

fn gen_detail(pos: ChunkPos, world: &mut WorldData, reg: &Registry) {
    let (x,y,z) = pos.as_block().as_tuple();
    let dirt = reg.get("dirt").as_block().unwrap();
    let log = reg.get("log").as_block().unwrap();
    let leaves = reg.get("leaves").as_block().unwrap();
    for x in x..x+16 {
        for z in z..z+16 {
            for y in y..y+16 {
                let below: BlockPos = (x,y-1,z).into();
                if world.block_at_any_state(&below).unwrap().id.as_ref() == "grass" {
                    let nx = x as f64 / 1.3;
                    let nz = z as f64 / 1.3;
                    let n = world.noise.noise_basic.get2d([nx,nz]);
                    const CUTOFF: f64 = 0.59;
                    if n > CUTOFF {
                        const INV: f64 = 1./1.3;
                        if world.noise.noise_basic.get2d([nx-INV,nz]) <= CUTOFF &&
                            world.noise.noise_basic.get2d([nx,nz-INV]) <= CUTOFF {
                                let h = 3 + (2. * world.noise.noise_basic.get2d([nx,nz])) as i32;
                                world.set_block_at_any_state(&below, dirt);
                                for y in y..=y+h {
                                    let here: BlockPos = (x,y,z).into();
                                    world.set_block_at_any_state(&here, log);
                                }
                                for dx in -2..=2i32 {
                                    for dz in -2..=2i32 {
                                        for dy in 0..=3i32 {
                                            if dx.abs()+dz.abs()+dy.abs() > 5 {continue}
                                            let y = y + h-3 + dy;
                                            let here: BlockPos = (x+dx,y+4,z+dz).into();
                                            world.replace_at_any_state(&here, leaves);
                                        }
                                    }
                                }
                                let here: BlockPos = (x,y+h,z).into();
                                world.replace_at_any_state(&here, leaves);
                            }
                    }
                    break;
                }
            }
        }    
    }
    let center = world.chunks.get_mut(&pos).unwrap();
    center.chunk_state = ChunkState::Detailed;
    center.needs_refresh = true;
}

