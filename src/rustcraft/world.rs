
#[macro_export]
macro_rules! time_it {
    ($name:expr, $x:expr) => {{
        let now = Instant::now();
        let ret = $x;
        println!("{:?} took {:?} ms", $name, now.elapsed().as_millis());
        ret
    }};
    ($x:expr) => {{
        let now = Instant::now();
        let ret = $x;
        println!("{:?} took {:?} ms", stringify!($x), now.elapsed().as_millis());
        ret
    }}
}

use crate::prelude::*;

#[derive(Debug)]
pub struct TerrainGen {
    pub noise: crate::perlin::PerlinNoise,
    pub noise_basic: crate::perlin::PerlinNoise,
    pub palettes: [[usize; 3]; 2],
}

impl TerrainGen {
    pub fn is_cave(&self, x: isize, y: isize, z: isize) -> bool {
        let xf = x.abs() as f64 / 13.;
        let yf = y.abs() as f64 / 13.;
        let zf = z.abs() as f64 / 13.;
        let c = self.noise_basic.get3d([xf, yf, zf]);
        let c = (c+0.1).powf(1.5);
        c > 0.65
    }
    pub fn density(&self, x: isize, y: isize, z: isize) -> f64 {
        let xf = x.abs() as f64 / 70.;
        let yf = y.abs() as f64 / 40.;
        let zf = z.abs() as f64 / 70.;
        let d = self.noise.get3d([xf, yf, zf]);
        let d = d * 0.8 / yf;
        d
    }
    pub fn palette(&self, x: isize, z: isize) -> &[usize; 3] {
        let xf = (x.abs() as f64 + 0.5) / 10.;
        let zf = (z.abs() as f64 + 0.5) / 10.;
        let n = self.noise_basic.get2d([xf, zf]);
        
        &self.palettes[if n < 0.5 {0} else {1}]
    } 
}
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
    pub to_update: Vec<WorldPos<i32>>,
}

impl WorldData {
    
    pub fn new(seed: &str, air: Block) -> Self {
        let noise = crate::perlin::PerlinNoise::new(seed.to_owned(), 4, 0.5);
        let noise_basic = crate::perlin::PerlinNoise::new(seed.to_owned(), 1, 1.);
        let palettes = [
            [1,2,3],
            [1,5,5]
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

    pub fn light_at(&self, pos: &impl Coord) -> u8 {
        self.chunk_at(pos.as_chunk()).map(|c| c.light_at(pos)).unwrap_or(0)
    }
    pub fn set_light_at(&mut self, pos: &impl Coord, light: u8) {
        if let Some(c) = self.chunk_at_mut(pos.as_chunk()) {
            c.set_light_at(pos, light);
        }
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
                        let (verts, uvs, lights) = (meshing::make_mesh(p.into(), self, reg));
                        let c = self.chunks.get_mut(&p.into()).unwrap();
                        c.mesh.as_mut().unwrap().update_lit(&verts, &uvs, &lights);
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
                            let (verts, uvs, lights) = meshing::make_mesh(p, self, reg);
                            let c = self.chunks.get_mut(&p).unwrap();
                            if let Some(mesh) = &mut c.mesh {
                                mesh.update_lit(&verts, &uvs, &lights);
                            } else {
                                c.mesh = Some(VAO::textured_lit(&verts, &uvs, &lights));
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

}

fn gen_detail(pos: ChunkPos, world: &mut WorldData, reg: &Registry) {
    let (x,y,z) = pos.as_pos_i32().as_tuple();
    let dirt = &reg[2];
    let log = &reg[4];
    let leaves = &reg[6];
    for x in x..x+16 {
        for z in z..z+16 {
            for y in y..y+16 {
                let below: WorldPos<i32> = (x,y-1,z).into();
                if world.block_at_any_state(&below).unwrap().id == 3 {
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
                                    let here: WorldPos<i32> = (x,y,z).into();
                                    world.set_block_at_any_state(&here, log);
                                }
                                for dx in -2..=2i32 {
                                    for dz in -2..=2i32 {
                                        for dy in 0..=3i32 {
                                            if dx.abs()+dz.abs()+dy.abs() > 5 {continue}
                                            let y = y + h-3 + dy;
                                            let here: WorldPos<i32> = (x+dx,y+4,z+dz).into();
                                            world.replace_at_any_state(&here, leaves);
                                        }
                                    }
                                }
                                let here: WorldPos<i32> = (x,y+h,z).into();
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

